// <luma-cue-sheet> — sequence window A (decisions §30 "12f", the approved round 7 board): one
// sequence's cue sheet, drawn from the `sequence` feed. CueZero first, OffCue last; the running cue
// marked. `+` opens a cue: its feature groups' own Fade · Delay · Out fade · Out delay · Path (an
// unset one shows the cue's in brackets) and the curves — every feature group's line dimmed, the
// active one bright with two handles (the end of its delay, the end of its fade) dragged sideways.
// Every timing field works the same: a tap opens the slider panel (− / +, Keypad), press and drag
// edits at once (a gauge under the finger); the values and the curve redraw live, never the sheet
// under the finger; the graph's scale holds still while dragging (≥ 2 s, +20 %, rounded) and grows
// only past its end. A tap on a feature group's name (or chip) makes it the active curve; a Path
// cell picks the shape. Every edit is a `Set Sequence … Cue … "<Timing>"` line (a drag on the gesture channel: one Oops
// step at its end).
//
// Sequence window B (`layout = 'Timeline'`, the approved round 7 board): the same cues side by side,
// a column each — its head (a tap = the cue's options), a ruler in seconds, a row per timing with
// its span over the seconds (a feature group's fade drawn in its Path's shape), "+ add timing"
// (feature group × type → the slider panel). Fields edit the same way; the spans redraw live; the
// scale is the window's Zoom (100 % = 60 px a second) and never moves while dragging.
//
//   el.layout = 'Sheet' | 'Timeline'  ·  el.zoom = percent
//   el.update(feedData)  ·  events: run { line } · gesture { id, line, phase, label?, from?, value } (bubbles)

import { openMenu } from './menu.js';
import { openSliderPanel } from './sliderpanel.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const fmt = v => String(Math.round(v * 100) / 100);
const SECONDS = 10;   // a drag's travel spans 0–10 s (more by the keypad)
const TRAVEL = 220;   // px for that travel
const EVERY_MS = 33;
const KEYS = [['fade', 'Fade', 'Fade'], ['delay', 'Delay', 'Delay'], ['outfade', 'Out fade', 'OutFade'], ['outdelay', 'Out delay', 'OutDelay']];
const WORD = Object.fromEntries(KEYS.map(([k, , w]) => [k, w]));
const LABEL = Object.fromEntries(KEYS.map(([k, l]) => [k, l]));
const SHAPES = {
  Linear: t => t,
  SCurve: t => t * t * (3 - 2 * t),
  EaseIn: t => t * t,
  EaseOut: t => 1 - (1 - t) * (1 - t),
  Snap: () => 1,
};
let gestures = 0;
const newId = () => `q${Date.now().toString(36)}.${++gestures}`;

/** The graph's time axis: at least 2 s, a fifth of headroom, rounded to a whole step. */
export function niceTotal(t) {
  const v = Math.max(2, t * 1.2);
  const step = v <= 10 ? 1 : v <= 30 ? 2 : 5;
  return Math.ceil(v / step) * step;
}

class LumaCueSheet extends HTMLElement {
  constructor() {
    super();
    this.d = null;
    this.open = null;   // the open cue's label
    this.fg = 0;        // the active feature group (index)
    this.drag = null;
    this._layout = 'Sheet';
    this._zoom = 100;
  }

  get layout() { return this._layout; }
  set layout(v) {
    v = v === 'Timeline' ? 'Timeline' : 'Sheet';
    if (v === this._layout) return;
    this._layout = v;
    this.scroller = null;
    if (this.d && !this.drag) this.render();
  }
  get zoom() { return this._zoom; }
  set zoom(v) {
    v = Math.max(20, Math.min(400, Number(v) || 100));
    if (v === this._zoom) return;
    this._zoom = v;
    if (this._layout === 'Timeline' && this.d && !this.drag) this.render();
  }

  connectedCallback() {
    this.dataset.kit = 'cue-sheet';
    this.ro = new ResizeObserver(() => this.drawCurves());
    this.ro.observe(this);
  }
  disconnectedCallback() { this.ro?.disconnect(); }

  run(line) { this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }
  gesture(detail) { this.dispatchEvent(new CustomEvent('gesture', { detail, bubbles: true })); }

  update(d) {
    this.d = d;
    // A drag keeps its field and curve; the rest redraws when it ends.
    if (this.drag) { this.pending = true; return; }
    this.render();
  }

  cue(label) { return this.d?.cues?.find(c => c.cue === label); }

  /** A field's value: { v, inh } — a feature group's own, else the cue's (in brackets). */
  value(c, g, k) {
    if (g == null) return { v: c[k] ?? 0, inh: false };
    const own = c.groups?.[g]?.[k];
    return own != null ? { v: own, inh: false } : { v: c[k] ?? 0, inh: true };
  }
  path(c, g) {
    const own = c.groups?.[g]?.path;
    return own ? { v: own, inh: false } : { v: 'Linear', inh: true };
  }
  line(c, g, k, v) {
    const fg = g == null ? '' : ` ${this.d.feature_groups[g].word}`;
    return `${c.line}${fg} "${WORD[k]}" ${v}`;
  }
  label(c, g, k) {
    return `Cue ${c.cue}${g == null ? '' : ` ${this.d.feature_groups[g].name}`} ${LABEL[k]}`;
  }

  render() {
    const d = this.d;
    const keep = this.scroller ? { t: this.scroller.scrollTop, l: this.scroller.scrollLeft } : null;
    if (!d || d.missing) {
      this.innerHTML = `<div class="lc-cs-note">${esc(d?.note || 'No sequence.')}</div>`;
      this.scroller = null;
      return;
    }
    const fgs = d.feature_groups || [];
    if (this.fg >= fgs.length) this.fg = 0;
    if (this._layout === 'Timeline') return this.renderTimeline(keep);
    const t = (c, k) => {
      const x = this.value(c, null, k);
      return `<div class="lc-cs-c n lc-cs-t" data-cue="${esc(c.cue)}" data-k="${k}">${fmt(x.v)}</div>`;
    };
    let html = `<div class="lc-cs-grid"><div class="lc-cs-row lc-cs-head"><div class="lc-cs-c"></div><div class="lc-cs-c">No</div><div class="lc-cs-c">Name</div><div class="lc-cs-c">Trig</div>`
      + KEYS.map(([, l]) => `<div class="lc-cs-c">${l}</div>`).join('') + `<div class="lc-cs-c">Info</div></div>`;
    d.cues.forEach((c, i) => {
      const open = this.open === c.cue;
      const cls = `${c.run ? ' run' : c.current ? ' cur' : ''}${c.fixed ? ' fixed' : ''}${open ? ' open' : ''}${i % 2 ? ' alt' : ''}`;
      const info = [`${c.values} value${c.values === 1 ? '' : 's'}`, c.parts > 1 ? `${c.parts} parts` : ''].filter(Boolean).join(' · ');
      html += `<div class="lc-cs-row${cls}" data-row="${esc(c.cue)}">`
        + `<div class="lc-cs-c"><button type="button" class="lc-cs-exp" data-exp="${esc(c.cue)}" aria-expanded="${open}" aria-label="${open ? 'Fold' : 'Open'} cue ${esc(c.cue)}">${open ? '−' : '+'}</button></div>`
        // CueZero is Cue 0, OffCue "Off"; their names say what they are.
        + `<div class="lc-cs-c n">${esc(c.fixed === 'OffCue' ? 'Off' : c.cue)}</div><div class="lc-cs-c name">${esc(c.fixed || c.name)}</div><div class="lc-cs-c">${esc(c.fixed ? '' : c.trig)}</div>`
        + KEYS.map(([k]) => t(c, k)).join('') + `<div class="lc-cs-c dim">${esc(info)}</div></div>`;
      if (open) html += `<div class="lc-cs-detail" data-detail="${esc(c.cue)}">${this.detailHtml(c, fgs)}</div>`;
    });
    this.innerHTML = `${html}</div>`;
    this.scroller = this.querySelector('.lc-cs-grid');
    if (keep) { this.scroller.scrollTop = keep.t; this.scroller.scrollLeft = keep.l; }
    this.wire();
    this.drawCurves();
  }

  detailHtml(c, fgs) {
    if (!fgs.length) return '<div class="lc-cs-note">The show has no feature groups.</div>';
    const on = g => (g === this.fg ? ' on' : '');
    let h = `<div class="lc-cs-chips">${fgs.map((f, g) => `<button type="button" data-fg="${g}" aria-pressed="${g === this.fg}">${esc(f.name)}</button>`).join('')}</div>`;
    h += `<div class="lc-cs-tim" style="--n:${fgs.length}"><div class="h">Timing</div>`
      + fgs.map((f, g) => `<div class="h col${on(g)}" data-col="${g}">${f.color ? `<i style="background:${esc(f.color)}"></i>` : ''}${esc(f.name)}</div>`).join('');
    for (const [k, l] of KEYS) {
      h += `<div class="h">${l}</div>` + fgs.map((_, g) => {
        const x = this.value(c, g, k);
        return `<div class="lc-cs-t${x.inh ? ' inh' : ''}${on(g)}" data-cue="${esc(c.cue)}" data-g="${g}" data-k="${k}">${x.inh ? `(${fmt(x.v)})` : fmt(x.v)}</div>`;
      }).join('');
    }
    h += '<div class="h">Path</div>' + fgs.map((_, g) => {
      const p = this.path(c, g);
      return `<div class="lc-cs-p${p.inh ? ' inh' : ''}${on(g)}" data-cue="${esc(c.cue)}" data-g="${g}">${esc(p.v)}</div>`;
    }).join('');
    return `${h}</div><canvas class="lc-cs-graph" data-cue="${esc(c.cue)}"></canvas>`;
  }

  wire() {
    for (const b of this.querySelectorAll('[data-exp]')) b.onclick = () => { this.open = this.open === b.dataset.exp ? null : b.dataset.exp; this.render(); };
    // Switching the feature group repaints only the columns, chips and curve (no jump).
    const pick = g => {
      this.fg = g;
      for (const x of this.querySelectorAll('[data-fg]')) x.setAttribute('aria-pressed', Number(x.dataset.fg) === g);
      for (const x of this.querySelectorAll('.lc-cs-tim [data-g], .lc-cs-tim [data-col]')) x.classList.toggle('on', Number(x.dataset.g ?? x.dataset.col) === g);
      this.drawCurves();
    };
    for (const b of this.querySelectorAll('[data-fg]')) b.onclick = () => pick(Number(b.dataset.fg));
    for (const b of this.querySelectorAll('[data-col]')) b.onclick = () => pick(Number(b.dataset.col));
    for (const el of this.querySelectorAll('.lc-cs-t')) this.wireField(el);
    for (const el of this.querySelectorAll('.lc-cs-p')) el.onclick = e => this.pickPath(el, e);
    const cv = this.querySelector('.lc-cs-graph');
    if (cv) this.wireGraph(cv);
  }

  /** What a field edits: { c, g, k, v, line(v), label }. */
  target(el) {
    const c = this.cue(el.dataset.cue);
    if (!c) return null;
    const g = el.dataset.g == null ? null : Number(el.dataset.g);
    const k = el.dataset.k;
    return { c, g, k, v: this.value(c, g, k).v, line: v => this.line(c, g, k, v), label: this.label(c, g, k) };
  }

  /** Set a value in the drawn data (the feed's next build replaces it) and repaint its fields. */
  setLive(c, g, k, v) {
    if (g == null) c[k] = v;
    else (c.groups[g] ||= {})[k] = v;
    for (const el of this.querySelectorAll(`.lc-cs-t[data-cue="${CSS.escape(c.cue)}"]`)) {
      // This field and every group field inheriting it.
      if (el.dataset.k !== k) continue;
      const x = this.value(c, el.dataset.g == null ? null : Number(el.dataset.g), k);
      el.textContent = x.inh ? `(${fmt(x.v)})` : fmt(x.v);
      el.classList.toggle('inh', x.inh);
    }
    if (this._layout === 'Timeline') this.liveColumn(c);
    else this.drawCurves();
  }

  send(t, id, phase, v, first) {
    const detail = { id, line: t.line(fmt(v)), phase, value: fmt(v) };
    if (first) Object.assign(detail, { label: t.label, from: fmt(t.v) });
    this.gesture(detail);
  }

  wireField(el) {
    let g = null;
    el.addEventListener('pointerdown', e => {
      const t = this.target(el);
      if (!t) return;
      e.preventDefault();
      try { el.setPointerCapture(e.pointerId); } catch { /* synthetic */ }
      g = { t, id: newId(), y0: e.clientY, x: e.clientX, y: e.clientY, v: t.v, moved: false, last: 0, sent: false };
    });
    el.addEventListener('pointermove', e => {
      if (!g) return;
      const dy = e.clientY - g.y0;
      if (!g.moved && Math.abs(dy) < 4) return;
      if (!g.moved) {
        g.moved = true;
        this.drag = g;
        el.classList.add('active');
        g.gauge = this.gauge(e.clientX, e.clientY);
      }
      g.v = Math.max(0, Math.round((g.t.v - (dy / TRAVEL) * SECONDS) * 10) / 10);
      this.setLive(g.t.c, g.t.g, g.t.k, g.v);
      g.gauge.set(g.v);
      const now = performance.now();
      if (now - g.last >= EVERY_MS) {
        g.last = now;
        this.send(g.t, g.id, 'move', g.v, !g.sent);
        g.sent = true;
      }
    });
    const up = () => {
      if (!g) return;
      const was = g;
      g = null;
      el.classList.remove('active');
      was.gauge?.remove();
      this.drag = null;
      if (was.moved) this.send(was.t, was.id, 'end', was.v, !was.sent);
      else this.panel(was.t, was.x, was.y);
      if (this.pending) { this.pending = false; this.render(); }
    };
    el.addEventListener('pointerup', up);
    el.addEventListener('pointercancel', up);
  }

  /** The gauge under the finger while a field is dragged. */
  gauge(x, y) {
    const el = document.createElement('div');
    el.className = 'lc-cs-gauge';
    el.innerHTML = '<b></b><div class="tr"><i></i></div>';
    Object.assign(el.style, { left: `${Math.max(8, Math.min(x + 28, innerWidth - 72))}px`, top: `${Math.max(8, Math.min(y - 110, innerHeight - 228))}px` });
    document.body.append(el);
    return {
      set: v => {
        el.querySelector('b').textContent = `${fmt(v)} s`;
        el.querySelector('i').style.height = `${Math.min(100, (v / SECONDS) * 100)}%`;
      },
      remove: () => el.remove(),
    };
  }

  panel(t, x, y) {
    let id = newId(), first = true;
    openSliderPanel({
      at: { x, y: y - 140 },
      title: t.label,
      value: t.v,
      min: 0,
      max: SECONDS,
      step: 0.1,
      unit: ' s',
      dim: this.d?.popup_dim,
      onMove: v => { this.setLive(t.c, t.g, t.k, v); this.send(t, id, 'move', v, first); first = false; },
      onEnd: v => { this.setLive(t.c, t.g, t.k, v); this.send(t, id, 'end', v, first); id = newId(); first = true; t.v = v; },
      onKeypad: text => this.run(t.line(text)),
    });
  }

  pickPath(el, e) {
    e.stopPropagation();
    const c = this.cue(el.dataset.cue);
    if (!c) return;
    const r = el.getBoundingClientRect();
    this.pathMenu(c, Number(el.dataset.g), { x: r.left, y: r.bottom + 4 });
  }
  pathMenu(c, g, at) {
    const cur = this.path(c, g);
    const base = `${c.line} ${this.d.feature_groups[g].word} "Path"`;
    // The cue's own = Reset (decisions §33 61, 87).
    const reset = `Reset${c.line.slice(3)} ${this.d.feature_groups[g].word} "Path"`;
    openMenu(at, [
      { header: `Cue ${c.cue} · ${this.d.feature_groups[g].name} path` },
      ...(this.d.paths || Object.keys(SHAPES)).map(p => ({ label: p, check: !cur.inh && cur.v === p, fn: () => { this.fg = g; this.run(`${base} ${p}`); } })),
      { label: 'The cue’s (Linear)', check: cur.inh, fn: () => this.run(reset) },
    ]);
  }

  // ── The graph ─────────────────────────────────────────────────────────
  times(c, g) {
    return { dl: this.value(c, g, 'delay').v, fd: this.value(c, g, 'fade').v, shape: SHAPES[this.path(c, g).v] || SHAPES.Linear };
  }

  drawCurves() {
    const cv = this.querySelector('.lc-cs-graph');
    const c = cv && this.cue(cv.dataset.cue);
    if (!c) return;
    const fgs = this.d.feature_groups;
    const css = getComputedStyle(this);
    const tok = n => css.getPropertyValue(`--lc-${n}`).trim();
    const W = Math.max(320, cv.clientWidth), H = 220, L = 52, B = 26, T = 16, R = Math.min(170, W * 0.3);
    const dpr = devicePixelRatio || 1;
    cv.width = W * dpr; cv.height = H * dpr;
    const ctx = cv.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, W, H);
    const font = `${tok('fs-small') || '12px'} ${tok('font') || 'sans-serif'}`;
    ctx.font = font;
    ctx.strokeStyle = tok('line'); ctx.lineWidth = 1;
    for (let i = 0; i <= 10; i++) { const x = L + (W - L - R) * i / 10; ctx.beginPath(); ctx.moveTo(x, T); ctx.lineTo(x, H - B); ctx.stroke(); }
    for (let i = 0; i <= 5; i++) { const y = T + (H - T - B) * i / 5; ctx.beginPath(); ctx.moveTo(L, y); ctx.lineTo(W - R, y); ctx.stroke(); }
    ctx.fillStyle = tok('text-dim');
    for (let i = 0; i <= 5; i++) ctx.fillText(`${100 - i * 20}%`, 8, T + (H - T - B) * i / 5 + 4);
    const total = cv.freeze ?? niceTotal(Math.max(0, ...fgs.map((_, g) => { const x = this.times(c, g); return x.dl + x.fd; })));
    const X = t => L + (W - L - R) * (t / total), Y = v => H - B - (H - T - B) * v;
    const curve = (g, colour, w) => {
      const { dl, fd, shape } = this.times(c, g);
      ctx.strokeStyle = colour; ctx.lineWidth = w;
      ctx.beginPath(); ctx.moveTo(L, Y(0)); ctx.lineTo(X(dl), Y(0));
      for (let i = 1; i <= 48; i++) { const p = i / 48; ctx.lineTo(X(dl + p * fd), Y(fd ? shape(p) : 1)); }
      ctx.lineTo(W - R, Y(1)); ctx.stroke();
    };
    ctx.globalAlpha = 0.28;
    fgs.forEach((_, g) => { if (g !== this.fg) curve(g, tok('text'), 1.5); });
    ctx.globalAlpha = 1;
    curve(this.fg, tok('accent'), 3);
    // The active group's handles: the end of its delay (the curve starts), the end of its fade.
    const a = this.times(c, this.fg);
    const handles = [{ k: 'delay', x: X(a.dl), y: Y(0) }, { k: 'fade', x: X(a.dl + a.fd), y: Y(1) }];
    for (const h of handles) {
      ctx.fillStyle = cv.dragging === h.k ? tok('strong') : tok('accent');
      ctx.strokeStyle = tok('bg'); ctx.lineWidth = 2;
      ctx.beginPath(); ctx.rect(h.x - 8, h.y - 8, 16, 16); ctx.fill(); ctx.stroke();
      const txt = h.k === 'delay' ? `delay ${fmt(a.dl)}s` : `fade ${fmt(a.fd)}s`;
      const tw = ctx.measureText(txt).width;
      ctx.fillStyle = tok('text');
      ctx.fillText(txt, h.x + 12 + tw > W - 4 ? h.x - 12 - tw : h.x + 12, h.k === 'delay' ? h.y - 10 : h.y + 22);
    }
    ctx.fillStyle = tok('text-dim');
    for (let i = 0; i <= 4; i++) ctx.fillText(`${fmt(total * i / 4)}s`, X(total * i / 4) - 8, H - 8);
    // The legend sits low on the right, clear of the fade handle at the top.
    ctx.fillStyle = tok('accent');
    ctx.fillText(`${fgs[this.fg].name}: ${this.path(c, this.fg).v}`, W - R + 14, H - B - 44);
    ctx.fillText(`delay ${fmt(a.dl)}s · fade ${fmt(a.fd)}s`, W - R + 14, H - B - 28);
    ctx.fillStyle = tok('text-dim');
    ctx.fillText('the others, dimmed', W - R + 14, H - B - 10);
    cv.geo = { L, R, W, total, handles };
  }

  /** Drag a handle sideways: the delay's end or the fade's end; the scale holds still (it grows
   *  only past its end), the grab keeps its offset, the table follows live; one gesture. */
  wireGraph(cv) {
    cv.addEventListener('pointerdown', e => {
      const geo = cv.geo;
      const c = this.cue(cv.dataset.cue);
      if (!geo || !c) return;
      const r = cv.getBoundingClientRect(), x = e.clientX - r.left, y = e.clientY - r.top;
      const hit = geo.handles.map(h => ({ ...h, d: Math.hypot(h.x - x, h.y - y) })).sort((a, b) => a.d - b.d)[0];
      if (!hit || hit.d > 24) return;
      e.preventDefault();
      try { cv.setPointerCapture(e.pointerId); } catch { /* synthetic */ }
      const g = this.fg, k = hit.k;
      const from = this.times(c, g);
      cv.freeze = geo.total;
      cv.dragging = k;
      const raw = ev => ((ev.clientX - r.left - geo.L) / (geo.W - geo.L - geo.R)) * cv.freeze;
      const off = (k === 'delay' ? from.dl : from.dl + from.fd) - raw(e);
      const t = { c, g, k, v: k === 'delay' ? from.dl : from.fd, line: v => this.line(c, g, k, v), label: this.label(c, g, k) };
      const d = { id: newId(), last: 0, sent: false, moved: false, v: t.v };
      this.drag = d;
      const move = ev => {
        let at = Math.max(0, Math.min(3600, Math.round((raw(ev) + off) * 10) / 10));
        if (at > cv.freeze * 0.95) cv.freeze = Math.max(cv.freeze, at * 1.08);
        d.v = k === 'delay' ? at : Math.max(0, Math.round((at - this.times(c, g).dl) * 10) / 10);
        d.moved = true;
        this.setLive(c, g, k, d.v);
        const now = performance.now();
        if (now - d.last >= EVERY_MS) { d.last = now; this.send(t, d.id, 'move', d.v, !d.sent); d.sent = true; }
      };
      const up = () => {
        cv.removeEventListener('pointermove', move);
        cv.removeEventListener('pointerup', up);
        cv.removeEventListener('pointercancel', up);
        cv.freeze = null;
        cv.dragging = null;
        this.drag = null;
        if (d.moved) this.send(t, d.id, 'end', d.v, !d.sent);
        if (this.pending) { this.pending = false; this.render(); } else this.drawCurves();
      };
      cv.addEventListener('pointermove', move);
      cv.addEventListener('pointerup', up);
      cv.addEventListener('pointercancel', up);
    });
  }

  // ── Sequence window B: the timeline ───────────────────────────────────
  /** Pixels a second at the window's Zoom. */
  pxs() { return (60 * this._zoom) / 100; }

  /** Where a timing's span lies: { from, len } in seconds (Fade after Delay, Out fade after Out delay). */
  spanOf(c, g, k) {
    const v = kk => this.value(c, g, kk).v;
    if (k === 'fade') return { from: v('delay'), len: v('fade') };
    if (k === 'outfade') return { from: v('outdelay'), len: v('outfade') };
    return { from: 0, len: v(k) };
  }

  /** A column's seconds: past every timing's end, at least 2 s, half a second of room. */
  seconds(c) {
    const ends = [null, ...(this.d.feature_groups || []).map((_, g) => g)]
      .flatMap(g => ['fade', 'outfade'].map(k => { const s = this.spanOf(c, g, k); return s.from + s.len; }));
    return Math.max(2, ...ends) + 0.5;
  }

  /** The rows of a cue: its own Fade and Delay (Out ones when set), then each feature group that
   *  has timing of its own — the keys it sets and its Path. */
  rowsOf(c) {
    const rows = [['fade'], ['delay'], ...['outfade', 'outdelay'].filter(k => (c[k] ?? 0) > 0).map(k => [k])].map(([k]) => ({ g: null, k }));
    (this.d.feature_groups || []).forEach((_, g) => {
      const own = c.groups?.[g] || {};
      for (const [k] of KEYS) if (own[k] != null) rows.push({ g, k });
      if (own.path) rows.push({ g, k: 'path' });
    });
    return rows;
  }

  laneInner(c, g, k) {
    if (k === 'path') return '';
    const pxs = this.pxs();
    const { from, len } = this.spanOf(c, g, k);
    if (len <= 0 && k !== 'fade') return '';
    const left = 4 + from * pxs, w = Math.max(2, len * pxs);
    // A fade is drawn in its shape (the Path), rising across its span; a delay / snap is a line.
    if ((k === 'fade' || k === 'outfade') && len > 0) {
      const shape = SHAPES[g == null ? 'Linear' : this.path(c, g).v] || SHAPES.Linear;
      const out = k === 'outfade';
      const pts = Array.from({ length: 25 }, (_, i) => { const p = i / 24; const y = shape(p); return `${(p * w).toFixed(1)},${(2 + (out ? y : 1 - y) * 16).toFixed(1)}`; }).join(' ');
      return `<svg class="lc-tl-shape" style="left:${left}px;width:${w}px" viewBox="0 0 ${w} 20" preserveAspectRatio="none"><polyline points="${pts}"/></svg><i class="lc-tl-ends" style="left:${left}px;width:${w}px"></i>`;
    }
    return `<i class="lc-tl-span" style="left:${left}px;width:${w}px"></i>`;
  }

  rulerInner(secs) {
    const pxs = this.pxs();
    const step = [0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300].find(s => s * pxs >= 32) || 600;
    let h = '';
    for (let t = 0; t <= secs + 1e-9; t += step) h += `<span style="left:${4 + t * pxs}px">${fmt(t)}</span>`;
    return h;
  }

  columnHtml(c, i) {
    const fgs = this.d.feature_groups || [];
    const lane = Math.max(60, this.seconds(c) * this.pxs());
    const cls = `${c.run ? ' run' : c.current ? ' cur' : ''}${c.fixed ? ' fixed' : ''}${i % 2 ? ' alt' : ''}`;
    const q = esc(c.cue);
    let h = `<div class="lc-tl-col${cls}" data-tl="${q}" style="--lane:${lane}px">`
      + `<button type="button" class="lc-tl-head" data-head="${q}" aria-label="Cue ${q} options"><small>${esc(c.fixed === 'OffCue' ? 'Off' : c.cue)}${c.fixed ? '' : ` · ${esc(c.trig)}`}</small><b>${esc(c.fixed || c.name || ' ')}</b></button>`
      + `<div class="lc-tl-ruler"><span class="k"></span><span class="lc-tl-lane" data-ruler>${this.rulerInner(lane / this.pxs())}</span></div>`;
    for (const { g, k } of this.rowsOf(c)) {
      const name = g == null ? LABEL[k] : `${fgs[g].name} ${k === 'path' ? 'Path' : LABEL[k]}`;
      const dot = g != null && fgs[g].color ? `<i style="background:${esc(fgs[g].color)}"></i>` : '';
      let v;
      if (k === 'path') v = `<span class="lc-tl-v lc-cs-p" data-cue="${q}" data-g="${g}">${esc(this.path(c, g).v)}</span>`;
      else {
        const x = this.value(c, g, k);
        v = `<span class="lc-tl-v lc-cs-t${x.inh ? ' inh' : ''}" data-cue="${q}"${g == null ? '' : ` data-g="${g}"`} data-k="${k}">${x.inh ? `(${fmt(x.v)})` : fmt(x.v)}</span>`;
      }
      h += `<div class="lc-tl-row${g == null ? '' : ' fg'}"><span class="k">${dot}${esc(name)}</span>${v}<span class="lc-tl-lane" data-lane="${g ?? ''}|${k}">${this.laneInner(c, g, k)}</span></div>`;
    }
    return `${h}<button type="button" class="lc-tl-add" data-add="${q}">+ add timing</button></div>`;
  }

  renderTimeline(keep) {
    this.innerHTML = `<div class="lc-tl"><div class="lc-tl-track">${this.d.cues.map((c, i) => this.columnHtml(c, i)).join('')}</div></div>`;
    this.scroller = this.querySelector('.lc-tl');
    if (keep) { this.scroller.scrollTop = keep.t; this.scroller.scrollLeft = keep.l; }
    for (const el of this.querySelectorAll('.lc-cs-t')) this.wireField(el);
    for (const el of this.querySelectorAll('.lc-cs-p')) el.onclick = e => this.pickPath(el, e);
    // (The page closes a menu on a click outside it: these open one, so the click stops here.)
    for (const b of this.querySelectorAll('[data-head]')) b.onclick = e => { e.stopPropagation(); this.cueOptions(this.cue(b.dataset.head), b); };
    for (const b of this.querySelectorAll('[data-add]')) b.onclick = e => { e.stopPropagation(); this.addTiming(this.cue(b.dataset.add), b); };
  }

  /** While a field drags: that column's spans and ruler, in place (never rebuilt under the finger).
   *  The column only grows during a drag, so the cues right of it hold still until it ends. */
  liveColumn(c) {
    const col = this.querySelector(`.lc-tl-col[data-tl="${CSS.escape(c.cue)}"]`);
    if (!col) return;
    const was = parseFloat(col.style.getPropertyValue('--lane')) || 0;
    const lane = Math.max(60, this.seconds(c) * this.pxs(), this.drag ? was : 0);
    col.style.setProperty('--lane', `${lane}px`);
    col.querySelector('[data-ruler]').innerHTML = this.rulerInner(lane / this.pxs());
    for (const el of col.querySelectorAll('[data-lane]')) {
      const [g, k] = el.dataset.lane.split('|');
      el.innerHTML = this.laneInner(c, g === '' ? null : Number(g), k);
    }
  }

  /** A tap on a cue's head: its options, each a line. */
  cueOptions(c, el) {
    if (!c) return;
    const r = el.getBoundingClientRect();
    const fgs = this.d.feature_groups || [];
    const items = [{ header: `Cue ${c.fixed === 'OffCue' ? 'Off' : c.cue} · ${c.fixed || c.name || 'unnamed'}` }];
    if (!c.fixed) items.push({ icon: 'play', label: 'Go to this cue', fn: () => this.run(c.line.replace(/^Set /, 'Goto ')) });
    items.push({ icon: 'plus', label: 'Add timing…', fn: () => this.addTiming(c, el) });
    const own = fgs.map((f, g) => ({ f, g, keys: [...KEYS.map(([k]) => k).filter(k => c.groups?.[g]?.[k] != null), ...(c.groups?.[g]?.path ? ['path'] : [])] })).filter(x => x.keys.length);
    if (own.length) items.push(null);
    for (const { f, g, keys } of own) {
      items.push({ icon: 'undo-2', label: `${f.name}: the cue's timing`, fn: () => this.run(keys.map(k => `Reset${c.line.slice(3)} ${f.word} "${k === 'path' ? 'Path' : WORD[k]}"`).join('; ')) });
    }
    openMenu({ x: r.left, y: r.bottom + 4 }, items);
  }

  /** "+ add timing": a feature group (or the cue's own), then the type — a time opens the slider
   *  panel on it (its first move is the line that adds the row), Path the shape list. */
  addTiming(c, el) {
    if (!c) return;
    const r = el.getBoundingClientRect();
    const at = { x: r.left, y: r.bottom + 4 };
    const fgs = this.d.feature_groups || [];
    const types = g => {
      const items = [{ header: `Cue ${c.cue} · ${g == null ? 'the cue' : fgs[g].name}` },
        ...KEYS.map(([k, l]) => ({ label: l, fn: () => this.panel({ c, g, k, v: this.value(c, g, k).v, line: v => this.line(c, g, k, v), label: this.label(c, g, k) }, r.left + r.width / 2, r.top) }))];
      if (g != null) items.push({ label: 'Path…', fn: () => this.pathMenu(c, g, at) });
      openMenu(at, items);
    };
    openMenu(at, [
      { header: `Add timing · Cue ${c.fixed === 'OffCue' ? 'Off' : c.cue}` },
      { label: 'The cue (every feature group)', fn: () => types(null) },
      ...fgs.map((f, g) => ({ label: f.name, fn: () => types(g) })),
    ]);
  }
}
customElements.define('luma-cue-sheet', LumaCueSheet);

/** The sheet as a pop-up (`Edit Sequence n`): head with the sequence's name, × and a tap outside
 *  close it.  openCueSheet({ title, onClose }) → { el, setTitle(text), close() } */
export function openCueSheet({ title, dim, onClose }) {
  const scrim = document.createElement('div');
  scrim.className = `lc-cs-scrim${dim ? ' dim' : ''}`;
  scrim.dataset.kit = 'cue-sheet-popup';
  scrim.innerHTML = `<div class="lc-cs-pop lc-box" role="dialog" aria-label="Sequence">
      <header class="lc-cs-pop-head"><b></b>
        <button type="button" class="lc-close" data-kit="close" aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
      <luma-cue-sheet class="lc-box-body"></luma-cue-sheet></div>`;
  document.body.append(scrim);
  const el = scrim.querySelector('luma-cue-sheet');
  const head = scrim.querySelector('.lc-cs-pop-head b');
  head.textContent = title || 'Sequence';
  const close = () => {
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    onClose?.();
  };
  const onKey = e => { if (e.key === 'Escape' && !document.querySelector('[data-kit="slider-panel"]')) { e.stopPropagation(); close(); } };
  document.addEventListener('keydown', onKey, true);
  scrim.querySelector('[data-kit="close"]').addEventListener('click', close);
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  return { el, setTitle: t => { head.textContent = t; }, close, get open() { return scrim.isConnected; } };
}
