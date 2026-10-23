// <luma-status-strip> — the top bar (decisions top-bar-status-strip; the approved mockup
// 2026-10-22-top-bar, layout A, word indicators, the ring). One thin strip:
//
//   the mark      a ring and the station's name: full green standalone / master, half blue a
//                 member (its latency beside it), spinning amber busy, blinking amber reconnecting,
//                 red lost, hollow when the station is an idle master. A tap: event `mark`
//                 { x, y } (the host opens the details).
//   the Show tab  gold dot, the show's name, its unsaved dot; a tap runs `Menu "Backup"`.
//   the facts     User, Profile, DataPool, Sequence, LumaSession as `Key: Value` (folded away
//                 on a narrow strip).
//   indicators    Blind, Highlight, Freeze, Solo, Oops (with its step count), Meow: words,
//                 always there, lit in their mode's colour (`--lc-mode-*`). The first five are UI
//                 keys (`data-key`): the page's key handling runs a tap, a hold and the lit state.
//                 Meow: event `meow`. On a phone the words fold to letters.
//   the clock     this device's time.
//
//   el.update({ status, conn: 'up' | 'retry' | 'lost', busy, meow: '' | 'latched' | 'held' })
//     status = the station's `Status` for this client (show, user, modes, lumasession, strip)
//   events: line { line } · mark { x, y } · meow {}

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const MODES = [
  ['blind', 'Blind', 'BLIND', 'B', 'blind'],
  ['highlight', 'Highlight', 'HI', 'H', 'hl'],
  ['freeze', 'Freeze', 'FREEZE', 'F', 'freeze'],
  ['solo', 'Solo', 'SOLO', 'S', 'solo'],
];

/** The mark's state from the connection, the station and its LumaSession. */
export function markState({ conn = 'up', busy = false, ls = null } = {}) {
  if (conn === 'lost') return { state: 'lost', sub: 'lost', tip: 'Lost the station — reconnecting' };
  if (conn === 'retry') return { state: 'retry', sub: 'retry', tip: 'Reconnecting to the station…' };
  if (ls?.health === 'lost') return { state: 'lost', sub: 'master lost', tip: `LumaSession: the master is lost` };
  if (busy) return { state: 'busy', sub: 'busy', tip: 'The station is busy' };
  const role = ls?.status || 'Standalone';
  if (role === 'Member') {
    const lat = ls.latency_ms != null ? `${ls.latency_ms} ms` : '';
    return { state: ls.health === 'ok' ? 'member' : 'retry', sub: lat || 'member', tip: `LumaSession member of ${ls.master || 'the master'}${lat ? ` · ${lat}` : ''}` };
  }
  if (role === 'IdleMaster') return { state: 'idle', sub: 'idle', tip: 'An idle master: alone, no output' };
  if (role === 'Master') return { state: 'ok', sub: 'master', tip: 'LumaSession master' };
  return { state: 'ok', sub: '', tip: 'Connected · standalone' };
}

class LumaStatusStrip extends HTMLElement {
  constructor() {
    super();
    this.d = { status: null, conn: 'up', busy: false, meow: '' };
  }

  connectedCallback() {
    this.dataset.kit = 'status-strip';
    if (!this.built) this.build();
    this.ro = new ResizeObserver(() => this.fold());
    this.ro.observe(this);
    this.tick = setInterval(() => this.clock(), 1000);
    this.clock();
  }
  disconnectedCallback() { this.ro?.disconnect(); clearInterval(this.tick); }

  build() {
    this.built = true;
    this.innerHTML = `
      <button type="button" class="lc-ss-mark" data-act="mark"><span class="lc-ss-ring"></span><span class="lc-ss-st"></span><small class="lc-ss-sub"></small></button>
      <span class="lc-ss-sep"></span>
      <button type="button" class="lc-ss-show" data-act="show" title="The show: Menu &quot;Backup&quot;"><i></i><span class="lc-ss-name"></span><b class="lc-ss-dirty" hidden>•</b></button>
      <span class="lc-ss-facts"></span>
      <span class="lc-ss-fill"></span>
      <span class="lc-ss-ind">
        ${MODES.map(([key, name, word, letter, tok]) => `<button type="button" class="lc-ss-m" data-key="${key}" style="--c:var(--lc-mode-${tok})" title="${name} — tap: toggle · hold: its menu"><span class="w">${word}</span><span class="l">${letter}</span></button>`).join('')}
        <button type="button" class="lc-ss-m lc-ss-oops" data-key="oops" style="--c:var(--lc-mode-oops)" title="Oops — tap: undo one step · hold: the Oops window"><span class="w">OOPS</span><span class="l">O</span><span class="n"></span></button>
        <button type="button" class="lc-ss-m lc-ss-meow" data-act="meow" style="--c:var(--lc-mode-meow)" title="Meow (latching): the next key runs its Meow command lines. Or hold the Meow key (Set MeowKey, default Shift)."><span class="w">MEOW</span><span class="l">M</span></button>
      </span>
      <span class="lc-ss-sep lc-ss-csep"></span>
      <span class="lc-ss-clock"></span>`;
    this.addEventListener('click', e => {
      const b = e.target.closest('[data-act]');
      if (!b) return;
      e.stopPropagation(); // the page closes a menu on a click outside it: not the one this opens
      const r = b.getBoundingClientRect();
      if (b.dataset.act === 'mark') this.emit('mark', { x: r.left, y: r.bottom + 4 });
      else if (b.dataset.act === 'show') this.emit('line', { line: 'Menu "Backup"' });
      else if (b.dataset.act === 'meow') this.emit('meow', {});
    });
  }

  emit(name, detail) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  update(d) {
    this.d = { ...this.d, ...d };
    if (!this.built) this.build();
    const st = this.d.status || {};
    const strip = st.strip || {};
    const m = markState({ conn: this.d.conn, busy: this.d.busy, ls: st.lumasession });
    const mark = this.querySelector('.lc-ss-mark');
    mark.dataset.state = m.state;
    mark.title = m.tip;
    this.querySelector('.lc-ss-st').textContent = strip.station || 'LumaCore';
    this.querySelector('.lc-ss-sub').textContent = m.sub;
    const show = st.show || {};
    this.querySelector('.lc-ss-name').textContent = show.name || 'no show';
    this.querySelector('.lc-ss-dirty').hidden = !show.dirty;
    const ls = st.lumasession;
    const facts = [['User', st.user], ['Profile', strip.profile], ['DataPool', strip.datapool], ['Sequence', strip.sequence],
      ['LumaSession', ls && ls.status !== 'Standalone' ? `${ls.status}${ls.master ? ` of ${ls.master}` : ''}` : '']];
    this.querySelector('.lc-ss-facts').innerHTML = facts.filter(([, v]) => v).map(([k, v]) => `<span class="lc-ss-f">${esc(k)}: <b>${esc(v)}</b></span>`).join('');
    const on = new Set((st.modes || []).map(x => String(x).toLowerCase()));
    for (const [key, name] of MODES) this.querySelector(`[data-key="${key}"]`).classList.toggle('on', on.has(name.toLowerCase()));
    const oops = this.querySelector('.lc-ss-oops');
    oops.classList.toggle('on', (strip.oops || 0) > 0);
    oops.querySelector('.n').textContent = strip.oops ? String(strip.oops) : '';
    const meow = this.querySelector('.lc-ss-meow');
    meow.classList.toggle('on', !!this.d.meow);
    meow.classList.toggle('held', this.d.meow === 'held');
    this.fold();
  }

  /** Narrow: the facts fold away; a phone: the clock goes and the words become letters. */
  fold() {
    const w = this.clientWidth;
    if (!w) return;
    this.classList.toggle('mid', w < 980);
    this.classList.toggle('narrow', w < 600);
  }

  clock() {
    const c = this.querySelector('.lc-ss-clock');
    if (!c) return;
    const t = new Date();
    const p = n => String(n).padStart(2, '0');
    c.textContent = `${p(t.getHours())}:${p(t.getMinutes())}:${p(t.getSeconds())}`;
  }
}
customElements.define('luma-status-strip', LumaStatusStrip);
