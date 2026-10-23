// FeedClient — one /ws/feeds socket per page: everything live in the page
// (decisions §30 "12b2 hardening": "Client feeds — one rule"; "12d
// hardening": "Everything live on /ws/feeds").
//
//   const sub = client.sub('health.render', { period_ms: 16 }, {
//     batch(msg) {},   // a JSON batch: {feed, kind, id, epoch, from_seq, to_seq, gap, rev?, cols|items|data}
//     binary(view) {}, // a binary frame (DMX): DataView over the message
//     reset() {},      // start over: another server run (epoch) or `since` too old
//     subbed(msg) {},  // the station took the subscription
//     error(message, code) {}, // the station refused it (`nosub`, E-FEED-…)
//   });
//   sub.set({ universes: [2] });  // new parameters (re-subscribes)
//   sub.close();                  // the window is gone
//   client.auth(token);           // log the socket in (null: out)
//
// The socket opens without a login: the station says `hello` (its client
// protocol and feed list); feeds marked open (the station view: the lock
// screen) work at once, the others answer `nosub` E-FEED-002 and wait here
// until `auth` → `authed`, then go again. Every subscription stays across a
// login, a logout and a reconnect (jittered backoff, so 30 tablets don't
// come back at the same instant). Samples resume with `since` + `epoch`;
// States with the `rev` they have (`unchanged` when the same) and keyed
// lists arrive as diffs (`set` / `del` against `base`), put together here:
// `batch` always gets the whole `data`. A page that is hidden drops fast
// feeds to 250 ms (the station still sends every sample, just in fewer
// messages). Network and drawing stay apart: handlers only store (a
// SampleRing) and mark their window dirty; each window draws in its own
// drawLoop.

const HIDDEN_PERIOD = 250;
const BACKOFF_MIN = 500;
const BACKOFF_MAX = 8000;
const LOGIN_REQUIRED = 'E-FEED-002';

/** A keyed list's row key as the station writes it (`3` and `"3"` match). */
const rowKey = (item, key) => String(item?.[key]);

/**
 * Put a keyed diff onto the data the page has: `{list, key, fields, set,
 * del, order?}` (the station's `hub::diff`): `list` is the array field,
 * `key` the field naming its rows.
 */
export function applyDiff(before, diff) {
  const { list, key } = diff;
  const out = { ...before, ...(diff.fields || {}) };
  const rows = (before[list] || []).map(item => [rowKey(item, key), item]);
  const del = new Set(diff.del || []);
  const kept = rows.filter(([k]) => !del.has(k));
  const at = new Map(kept.map(([k], i) => [k, i]));
  for (const [k, item] of Object.entries(diff.set || {})) {
    if (at.has(k)) kept[at.get(k)][1] = item;
    else { at.set(k, kept.length); kept.push([k, item]); }
  }
  let ordered = kept;
  if (diff.order) {
    const by = new Map(kept);
    ordered = diff.order.map(k => [k, by.get(k)]);
  }
  out[list] = ordered.map(([, item]) => item);
  return out;
}

export class FeedClient {
  /**
   * `onProtocol(n)`: the station speaks another client protocol than when
   * this page was loaded (it was updated) — the page reloads.
   * `onAuth(user)`: the socket's login changed (`authed`).
   */
  constructor({ url, onProtocol, onAuth } = {}) {
    this.url = url || `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/ws/feeds`;
    this.onProtocol = onProtocol || null;
    this.onAuth = onAuth || null;
    this.subs = new Map();
    this.nextId = 1;
    this.ws = null;
    this.retry = null;
    this.backoff = BACKOFF_MIN;
    this.epoch = null;
    this.protocol = null; // the first station hello's client protocol
    this.token = null;
    this.user = null;
    this.hidden = typeof document !== 'undefined' && document.visibilityState === 'hidden';
    document.addEventListener?.('visibilitychange', () => {
      const hidden = document.visibilityState === 'hidden';
      if (hidden === this.hidden) return;
      this.hidden = hidden;
      for (const s of this.subs.values()) if ((s.opts.period_ms ?? 1000) < HIDDEN_PERIOD) this.send(s);
    });
  }

  /** True while the socket is open. */
  get open() { return this.ws?.readyState === 1; }

  sub(feed, opts = {}, handlers = {}) {
    const s = { id: this.nextId++, feed, opts: { ...opts }, handlers, since: null, epoch: null, rev: null, data: null, waiting: false, client: this };
    s.set = o => { s.opts = { ...s.opts, ...o }; s.rev = null; s.data = null; this.send(s); };
    s.close = () => {
      if (!this.subs.delete(s.id)) return;
      if (this.open) this.ws.send(JSON.stringify({ op: 'unsub', id: s.id }));
    };
    this.subs.set(s.id, s);
    if (this.open) this.send(s); else this.connect();
    return s;
  }

  /** Log the socket in with a login token (null: log out). Subscriptions stay. */
  auth(token) {
    this.token = token || null;
    if (this.open) this.ws.send(JSON.stringify({ op: 'auth', token: this.token }));
    else this.connect();
  }

  /** Close the socket; subscriptions stay for the next connect. */
  disconnect() {
    clearTimeout(this.retry);
    const ws = this.ws;
    this.ws = null;
    if (ws) { ws.onclose = null; ws.close(); }
  }

  send(s) {
    if (!this.open) return this.connect();
    s.waiting = false;
    const period = s.opts.period_ms;
    const msg = { op: 'sub', id: s.id, feed: s.feed, ...s.opts };
    if (period != null && this.hidden) msg.period_ms = Math.max(period, HIDDEN_PERIOD);
    if (s.since != null && s.epoch) { msg.since = s.since; msg.epoch = s.epoch; }
    if (s.rev != null && s.data != null) msg.rev = s.rev;
    this.ws.send(JSON.stringify(msg));
  }

  connect() {
    if (this.ws && this.ws.readyState <= 1) return;
    clearTimeout(this.retry);
    if (!this.subs.size) return;
    const ws = new WebSocket(this.url);
    ws.binaryType = 'arraybuffer';
    this.ws = ws;
    ws.onopen = () => {
      this.backoff = BACKOFF_MIN;
      // In order: the login first, so the subscriptions after it are the user's.
      if (this.token) ws.send(JSON.stringify({ op: 'auth', token: this.token }));
      for (const s of this.subs.values()) this.send(s);
    };
    ws.onmessage = e => this.receive(e.data);
    ws.onclose = () => {
      if (this.ws !== ws) return;
      this.ws = null;
      if (this.subs.size) {
        // Jittered: many pages coming back after a restart don't all ask at once.
        this.retry = setTimeout(() => this.connect(), this.backoff * (0.5 + Math.random()));
        this.backoff = Math.min(this.backoff * 2, BACKOFF_MAX);
      }
    };
  }

  receive(data) {
    if (data instanceof ArrayBuffer) {
      const view = new DataView(data);
      const s = this.subs.get(view.getUint32(0, true));
      s?.handlers.binary?.(view);
      return;
    }
    let m;
    try { m = JSON.parse(data); } catch (_) { return; }
    // Batches due in one tick come as one frame.
    if (Array.isArray(m)) { for (const one of m) this.message(one); return; }
    this.message(m);
  }

  message(m) {
    switch (m.op) {
      case 'hello':
        if (this.protocol == null) this.protocol = m.client_protocol;
        else if (m.client_protocol !== this.protocol) this.onProtocol?.(m.client_protocol);
        return;
      case 'authed':
        this.user = m.user;
        this.onAuth?.(m.user);
        // What waited for a login goes again.
        if (m.user) for (const s of this.subs.values()) if (s.waiting) this.send(s);
        return;
      case 'subbed': { const s = this.subs.get(m.id); s?.handlers.subbed?.(m); return; }
      case 'nosub': {
        const s = this.subs.get(m.id);
        if (!s) return;
        if (m.code === LOGIN_REQUIRED) { s.waiting = true; s.rev = null; s.data = null; return; }
        s.handlers.error?.(m.message, m.code);
        return;
      }
      case 'error': { const s = this.subs.get(m.id); s?.handlers.error?.(m.message, m.code); return; }
      default: break;
    }
    const s = this.subs.get(m.id);
    if (!s) return;
    this.epoch = m.epoch;
    // Another server run, or the station couldn't serve `since`: start over.
    if ((s.epoch && m.epoch !== s.epoch) || m.recovered === false) {
      s.since = null;
      s.handlers.reset?.();
    }
    s.epoch = m.epoch;
    if (m.kind === 'samples') { s.since = m.to_seq; s.handlers.batch?.(m); return; }
    // State: the page has it already, a diff onto what it has, or all of it.
    if (m.unchanged) { s.rev = m.rev; return; }
    if (m.diff) {
      // Not onto what this page has: ask for the whole state again.
      if (s.data == null || s.rev !== m.base) { s.rev = null; s.data = null; this.send(s); return; }
      m.data = applyDiff(s.data, m.diff);
    }
    if (m.rev != null) { s.rev = m.rev; s.data = m.data; }
    s.handlers.batch?.(m);
  }
}

/**
 * Typed-array columns of the last `capacity` samples (oldest first). A gap
 * is stored as NaN rows (one per lost sample, at most the capacity) so a
 * graph keeps one point per sample and breaks its line there.
 */
export class SampleRing {
  constructor(capacity, columns) {
    this.capacity = capacity;
    this.columns = columns;
    this.cols = Object.fromEntries(columns.map(c => [c, new Float64Array(capacity)]));
    this.start = 0;
    this.length = 0;
  }
  clear() { this.start = 0; this.length = 0; }
  /** Index `i` (0 = oldest) → slot. */
  slot(i) { return (this.start + i) % this.capacity; }
  at(col, i) { return this.cols[col][this.slot(i)]; }
  last(col) { return this.length ? this.at(col, this.length - 1) : NaN; }
  pushRow(row) {
    const at = this.length < this.capacity ? this.slot(this.length) : this.start;
    for (const c of this.columns) this.cols[c][at] = row[c] ?? NaN;
    if (this.length < this.capacity) this.length++;
    else this.start = (this.start + 1) % this.capacity;
  }
  /** `n` lost samples. */
  gap(n) {
    for (let i = 0; i < Math.min(n, this.capacity); i++) this.pushRow({});
  }
  /** A columnar batch: `{name: [values…]}` (missing columns are NaN). */
  pushCols(cols) {
    const n = Object.values(cols)[0]?.length ?? 0;
    const row = {};
    for (let i = 0; i < n; i++) {
      for (const c of this.columns) row[c] = cols[c] ? cols[c][i] : NaN;
      this.pushRow(row);
    }
  }
  /** Max of a column over the ring (NaN rows skipped). */
  max(col) {
    let m = -Infinity;
    for (let i = 0; i < this.length; i++) { const v = this.at(col, i); if (v > m) m = v; }
    return m === -Infinity ? NaN : m;
  }
}

/**
 * One requestAnimationFrame draw loop for a window: `dirty()` asks for a
 * frame; `draw()` runs at most once per frame, only when something changed.
 */
export function drawLoop(draw) {
  let raf = 0;
  return {
    dirty() { if (!raf) raf = requestAnimationFrame(() => { raf = 0; draw(); }); },
    stop() { if (raf) cancelAnimationFrame(raf); raf = 0; },
  };
}
