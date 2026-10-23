// <luma-pool> — every pool window (decisions §22, §30 "12e": the Pool board of
// mockups/2026-10-13-12e, R1 Q34 c). The station sends the cells (the `pools`
// feed); the element draws them and hands every gesture back as a callback —
// the page turns each into a command line.
//
// It draws only the rows in view (a pool has up to 9999 numbers, R1-11),
// updates a cell in place when it changes and never redraws under a finger
// (R1-60), and keeps its scroll. Three designs, chosen by where it sits:
//   desk   cells with number, name, lock, link, badges, the fit edge and bar
//   phone  a tight matrix (1 px gaps, no padding): number and name
//   mini   inside a miniature pane: colour only; the name appears once a cell
//          is drawn at least 40 px wide
// A cell is its object's colour (NodeMeta) or the pool type's colour
// (`--lc-pool-<kind>`).
//
// Gestures (one hold through the pointer core): tap → onTap(no, cell);
// hold → onHold(no, cell, at) (the page opens the object sheet); a mouse drag
// from an object → onDrop(from, to, copy) (Ctrl / ⌘ copies; `drag` false
// turns it off); one finger scrolls. With `pressRelease()` true (an action like Top: the echo
// while held, decisions §32) a press calls onPress(no) and its release onRelease(no) — no hold
// sheet, no drag in between. `armPick(text, onTarget)` shows a banner
// and hands the next tapped number to onTarget (Move / Copy from the sheet).
//
//   const p = document.createElement('luma-pool');
//   p.setup({ kind: 'groups', settings, onTap, onHold, onDrop, onScrollRow, drag: () => true });
//   p.cells = [{ no, name, color, owner, locked, referenced, applicability, bar, badges }];
//   p.cursor = 3; p.scrollToRow(12);

import { hold } from './pointer.js';
import { icon } from './icon.js';

const MAX_NO = 9999;
/** Free numbers a Packed pool shows after its presets, to store into. */
const PACKED_FREE = 4;
const NAME_MIN = 40;   // px a miniature cell needs before it shows its name
const SLOP = 8;
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaPool extends HTMLElement {
  constructor() {
    super();
    this.byNo = new Map();     // no → cell data
    this.els = new Map();      // no → drawn element
    this.cfg = {};
    this.opts = {};
    this.geo = null;
    this._cursor = 0;
    this.pick = null;
  }

  connectedCallback() {
    if (this.scroller) return;
    this.dataset.kit = 'pool';
    this.scroller = document.createElement('div');
    this.scroller.className = 'lc-pool-scroll';
    this.sizer = document.createElement('div');
    this.sizer.className = 'lc-pool-sizer';
    this.scroller.appendChild(this.sizer);
    this.appendChild(this.scroller);
    this.scroller.addEventListener('scroll', () => { this.draw(); this.scrollEnded(); }, { passive: true });
    new ResizeObserver(() => this.layout()).observe(this);
    hold(this.scroller, {
      // macOS turns Ctrl + click into a context menu: that press is a Ctrl-drag
      // (Copy), not "show me more"; a real right-click still opens the sheet.
      filter: e => !!e.target.closest('.lc-pool-cell') && !(e.type === 'contextmenu' && e.ctrlKey && e.button === 0) && !this.opts.pressRelease?.(),
      onTap: ({ target }) => this.tapped(target.closest?.('.lc-pool-cell')),
      onHold: ({ target, x, y }) => {
        const el = target.closest?.('.lc-pool-cell');
        if (!el) return;
        const no = Number(el.dataset.no);
        this.opts.onHold?.(no, this.byNo.get(no), { x, y });
      },
    });
    this.mouseDrag();
    this.pressAndRelease();
  }

  /** A press-and-release action: its line at the press, its release line when the finger lifts. */
  pressAndRelease() {
    let p = null;
    this.scroller.addEventListener('pointerdown', e => {
      if (!this.opts.pressRelease?.() || this.pick) return;
      const el = e.target.closest('.lc-pool-cell');
      const no = el && Number(el.dataset.no);
      if (!el || !this.byNo.has(no)) return;
      try { this.scroller.setPointerCapture(e.pointerId); } catch (_) {}
      p = { id: e.pointerId, no, el };
      el.classList.add('held');
      this.opts.onPress?.(no);
    });
    const up = e => {
      if (!p || e.pointerId !== p.id) return;
      const { no, el } = p;
      p = null;
      el.classList.remove('held');
      this.opts.onRelease?.(no);
    };
    for (const t of ['pointerup', 'pointercancel', 'lostpointercapture']) this.scroller.addEventListener(t, up);
  }

  /** kind (window type), settings (the window's), callbacks. */
  setup({ kind = 'groups', settings = {}, ...opts } = {}) {
    this.kind = kind;
    this.cfg = { columns: 12, rows: 12, size: 80, first_index: 1, slots: 0, show_empty: true, font_size: 0,
      empty_paint: '', for_all_paint: '', for_some_paint: '', for_none_paint: '', ...settings };
    this.opts = opts;
    this.style.setProperty('--lc-pool-type', `var(--lc-pool-${kind}, var(--lc-pool-default))`);
    if (this.cfg.font_size) this.style.setProperty('--lc-pool-font', `${this.cfg.font_size}px`);
    this.layout();
  }

  set cells(list) {
    this.byNo = new Map((list || []).map(c => [c.no, c]));
    this.layout();
  }
  /** What the pool sends beside its cells (the Presets pool: every number in use, the ones the
   *  filter hid): Fixed slots draws the hidden ones as gaps, Packed leaves them out. */
  set head(h) {
    this.usedNos = new Set(h?.used || []);
    this.hiddenNos = new Set(h?.hidden || []);
  }
  get cells() { return [...this.byNo.values()]; }

  set cursor(no) {
    this._cursor = no;
    for (const [n, el] of this.els) el.classList.toggle('cur', n === no);
  }

  /** Which design this element draws now (see the top). */
  design() {
    if (this.closest('.lc-mini')) return 'mini';
    return matchMedia('(any-pointer: coarse)').matches && innerWidth <= 600 ? 'phone' : 'desk';
  }

  /** The numbers shown, in slot order. */
  numbers() {
    const first = Math.max(1, this.cfg.first_index || 1);
    // Packed (the Presets window, decisions 0288): the presets shown, then a few free numbers to
    // store into.
    if (this.cfg.packed) {
      const stored = [...this.byNo.keys()].filter(n => n >= first).sort((a, b) => a - b);
      const free = [];
      for (let n = first; free.length < PACKED_FREE && n <= MAX_NO; n++) if (!this.byNo.has(n) && !this.usedNos?.has(n)) free.push(n);
      return { first, list: [...stored, ...free] };
    }
    if (!this.cfg.show_empty) {
      const stored = [...this.byNo.keys()].filter(n => n >= first).sort((a, b) => a - b);
      return { first, list: this.cfg.slots > 0 ? stored.slice(0, this.cfg.slots) : stored };
    }
    return { first, list: null };
  }

  layout() {
    if (!this.scroller) return;
    const W = this.scroller.clientWidth, H = this.scroller.clientHeight;
    if (W <= 0 || H <= 0) return;
    const design = this.design();
    this.dataset.design = design;
    const gap = design === 'desk' ? 2 : 1;
    const pad = design === 'desk' ? 4 : 0;
    const coarse = matchMedia('(any-pointer: coarse)').matches;
    const min = parseFloat(getComputedStyle(this).getPropertyValue('--lc-pool-min')) || 0;
    const w = W - pad * 2;
    let cols;
    if (design === 'phone') cols = Math.max(1, Math.floor((w + gap) / (62 + gap)));
    else if (this.cfg.columns > 0) cols = Math.max(1, Math.min(this.cfg.columns, coarse ? Math.floor((w + gap) / (Math.max(min, 1) + gap)) || this.cfg.columns : this.cfg.columns));
    else cols = Math.max(1, Math.floor((w + gap) / (Math.max(this.cfg.size || 80, min) + gap)));
    const cellW = (w - gap * (cols - 1)) / cols;
    const fixedRows = design === 'desk' && !coarse && this.cfg.rows > 0;
    const cellH = design === 'phone' ? 52 : fixedRows ? (H - pad * 2 - gap * (this.cfg.rows - 1)) / this.cfg.rows : cellW;
    const pitch = cellH + gap;
    const { first, list } = this.numbers();
    const visRows = Math.ceil(H / pitch);
    let total;
    if (list) total = list.length;
    else if (this.cfg.slots > 0) total = this.cfg.slots;
    else {
      const maxNo = Math.max(first, ...this.byNo.keys());
      // Every stored number, and a screenful of room after the last one.
      total = Math.min(MAX_NO - first + 1, (maxNo - first + 1) + visRows * cols);
      total = Math.max(total, visRows * cols);
    }
    const rows = Math.ceil(total / cols);
    // The miniature: how big a cell is drawn (the pane is scaled).
    const pane = this.closest('.lc-mini');
    const scale = pane ? parseFloat(pane.style.getPropertyValue('--lc-mini')) || 1 : 1;
    const geo = { design, gap, pad, cols, cellW, cellH, pitch, first, list, total, rows, named: cellW * scale >= NAME_MIN };
    // New data alone keeps every drawn cell (repainted only where it changed);
    // a new size, design or numbering lays them out again.
    const key = JSON.stringify({ ...geo, list: list?.join(',') });
    if (key !== this.geoKey) {
      for (const el of this.els.values()) el.remove();
      this.els.clear();
      this.geoKey = key;
    }
    const was = this.geo;
    this.geo = geo;
    this.sizer.style.height = `${rows * pitch - gap + pad * 2}px`;
    // A new pitch (the pane settled or resized) keeps the scroll in rows, not
    // pixels: the row at the top stays the row at the top (R1 Q31).
    if (was && was.pitch !== pitch && this.pendingRow == null && this.scroller.scrollTop > 0) {
      this.scroller.scrollTop = this.scroller.scrollTop / was.pitch * pitch;
    }
    // A saved scroll waits until the pool is tall enough to reach it.
    if (this.pendingRow != null) {
      const top = this.pendingRow * pitch;
      this.scroller.scrollTop = top;
      if (this.byNo.size || this.scroller.scrollHeight - this.scroller.clientHeight >= top) this.pendingRow = null;
    }
    this.draw();
  }

  /** Draw the rows in view (and one more each side); reuse what's drawn. */
  draw() {
    const g = this.geo;
    if (!g) return;
    const top = this.scroller.scrollTop, H = this.scroller.clientHeight;
    const r0 = Math.max(0, Math.floor((top - g.pad) / g.pitch) - 1);
    const r1 = Math.min(g.rows - 1, Math.ceil((top + H) / g.pitch) + 1);
    const want = new Set();
    for (let i = r0 * g.cols; i <= Math.min(g.total - 1, (r1 + 1) * g.cols - 1); i++) {
      const no = g.list ? g.list[i] : g.first + i;
      if (no == null || no > MAX_NO) continue;
      want.add(no);
      let el = this.els.get(no);
      if (!el) {
        el = document.createElement('div');
        el.className = 'lc-pool-cell';
        el.dataset.no = no;
        el.style.width = `${g.cellW}px`;
        el.style.height = `${g.cellH}px`;
        this.sizer.appendChild(el);
        this.els.set(no, el);
      }
      const row = Math.floor(i / g.cols), col = i % g.cols;
      el.style.transform = `translate(${g.pad + col * (g.cellW + g.gap)}px, ${g.pad + row * g.pitch}px)`;
      this.paint(el, no);
    }
    for (const [no, el] of this.els) if (!want.has(no)) { el.remove(); this.els.delete(no); }
  }

  /** One cell's look; rewritten only when what it shows changed. */
  paint(el, no) {
    const c = this.byNo.get(no);
    const g = this.geo;
    const gap = !c && !!this.hiddenNos?.has(no);
    const sig = `${g.design}|${g.named}|${no === this._cursor}|${this.pick?.src === no}|${gap}|${c ? JSON.stringify(c) : ''}`;
    if (el.dataset.sig === sig) return;
    el.dataset.sig = sig;
    el.classList.toggle('empty', !c && !gap);
    // A preset the filter hid, in Fixed slots: its place stays, dim, and takes no tap.
    el.classList.toggle('gap', gap);
    el.classList.toggle('cur', no === this._cursor);
    el.classList.toggle('src', this.pick?.src === no);
    // Executor style (the Sequence pool): running = a blue mark, selected = a gold frame.
    el.classList.toggle('run', !!c?.running);
    el.classList.toggle('sel', !!c?.selected);
    el.style.setProperty('--lc-obj', c?.color || 'var(--lc-pool-type)');
    if (!c && this.cfg.empty_paint) el.style.background = this.cfg.empty_paint; else el.style.background = '';
    if (c?.extra?.groups && g.design !== 'mini') {
      el.classList.toggle('fit-none', c.applicability === 'none');
      el.classList.toggle('mine', !!c.extra.mine);
      el.innerHTML = presetHtml(no, c, g.design === 'phone');
      return;
    }
    if (gap) { el.innerHTML = `<span class="lc-pool-no">${no}</span>`; return; }
    if (g.design === 'mini') {
      el.innerHTML = c && g.named ? `<span class="lc-pool-name">${esc(c.name ?? '')}</span>` : '';
      return;
    }
    if (g.design === 'phone') {
      // A tight matrix: number and name — and an executor cell's active cue (the Sequence pool).
      const active = c?.cues?.find(q => q.state);
      el.innerHTML = `<span class="lc-pool-no">${no}${c?.owner || c?.locked ? '<i class="lc-pool-dot"></i>' : ''}</span>`
        + `<span class="lc-pool-name">${c ? esc(c.name ?? '') : ''}</span>`
        + (active ? `<span class="lc-pool-exe"><span class="${esc(active.state)}">${esc(active.text)}</span></span>` : '');
      return;
    }
    const fit = c?.applicability && { all: this.cfg.for_all_paint || 'var(--lc-ok)', some: this.cfg.for_some_paint || 'var(--lc-warn)',
      none: this.cfg.for_none_paint }[c.applicability];
    el.innerHTML =
      (fit ? `<i class="lc-pool-fit" style="background:${esc(fit)}"></i>` : '')
      + `<span class="lc-pool-no">${no}</span>`
      + (c?.owner ? `<span class="lc-pool-lock" title="Being edited by ${esc(c.owner)}">${icon('lock')} ${esc(c.owner)}</span>`
        : c?.locked ? `<span class="lc-pool-lock lk" title="Locked">${icon('lock')}</span>` : '')
      + (c?.referenced ? `<span class="lc-pool-ref" title="Something uses it">${icon('link')}</span>` : '')
      + (c?.badges?.length ? `<span class="lc-pool-badges">${c.badges.map(esc).join(' ')}</span>` : '')
      + `<span class="lc-pool-name">${c ? esc(c.name ?? '') : ''}</span>`
      + (c?.bar ? `<span class="lc-pool-bar"><i style="width:${Math.round(c.bar.value * 100)}%"></i><b>${esc(c.bar.text)}</b></span>` : '')
      + (c?.cues?.length ? `<span class="lc-pool-exe">${c.cues.map(q => `<span class="${esc(q.state)}">${q.text ? esc(q.text) : '&nbsp;'}</span>`).join('')}</span>` : '')
      + (c?.layout ? layoutHtml(c.layout) : '');
  }

  tapped(el) {
    if (!el || el.classList.contains('gap')) return;
    const no = Number(el.dataset.no);
    if (this.pick) {
      const p = this.pick;
      this.disarm();
      if (no !== p.src) p.onTarget(no);
      return;
    }
    this.opts.onTap?.(no, this.byNo.get(no));
  }

  /** Move / Copy from the sheet: a banner, then the next tap is the target. */
  armPick(text, src, onTarget) {
    this.disarm();
    const bn = document.createElement('div');
    bn.className = 'lc-pool-banner';
    bn.innerHTML = `<span>${esc(text)}</span><button type="button" class="lc-close" aria-label="Cancel">${icon('x')}</button>`;
    bn.querySelector('button').addEventListener('click', () => this.disarm());
    this.insertBefore(bn, this.scroller);
    this.pick = { src, onTarget, bn };
    for (const [n, el] of this.els) this.paint(el, n);
  }
  disarm() {
    if (!this.pick) return;
    this.pick.bn.remove();
    this.pick = null;
    for (const [n, el] of this.els) this.paint(el, n);
  }

  /** A mouse drags an object onto another number (Move, Ctrl / ⌘ = Copy). */
  mouseDrag() {
    let d = null;
    this.scroller.addEventListener('pointerdown', e => {
      if (e.pointerType !== 'mouse' || e.button !== 0 || this.pick || this.opts.pressRelease?.()) return;
      const el = e.target.closest('.lc-pool-cell');
      const no = el && Number(el.dataset.no);
      if (!el || !this.byNo.has(no) || !(this.opts.drag?.() ?? true)) return;
      d = { id: e.pointerId, x: e.clientX, y: e.clientY, no, ghost: null, over: null };
    });
    this.scroller.addEventListener('pointermove', e => {
      if (!d || e.pointerId !== d.id) return;
      if (!d.ghost) {
        if (Math.hypot(e.clientX - d.x, e.clientY - d.y) <= SLOP) return;
        try { this.scroller.setPointerCapture(e.pointerId); } catch (_) {}
        d.ghost = document.createElement('div');
        d.ghost.className = 'lc-pool-ghost';
        document.body.appendChild(d.ghost);
      }
      const copy = e.ctrlKey || e.metaKey;
      d.ghost.textContent = `${copy ? 'Copy' : 'Move'} ${this.opts.label?.() ?? ''} ${d.no}`.replace(/\s+/g, ' ');
      d.ghost.style.left = `${e.clientX + 12}px`;
      d.ghost.style.top = `${e.clientY + 12}px`;
      d.over?.classList.remove('drop');
      d.out?.el.classList.remove('drop');
      const under = document.elementFromPoint(e.clientX, e.clientY)?.closest('.lc-pool-cell');
      d.over = under && this.contains(under) && Number(under.dataset.no) !== d.no ? under : null;
      d.over?.classList.add('drop');
      // Outside the pool the host may take the object (the view bar: Assign, decisions 0287).
      d.out = d.over ? null : this.opts.dropOut?.(d.no, e.clientX, e.clientY) || null;
      if (d.out) { d.out.el.classList.add('drop'); d.ghost.textContent = d.out.label; }
    });
    const end = (e, cancelled) => {
      if (!d || e.pointerId !== d.id) return;
      const { ghost, over, out, no } = d;
      d = null;
      if (!ghost) return;
      ghost.remove();
      over?.classList.remove('drop');
      out?.el.classList.remove('drop');
      if (!cancelled && over) this.opts.onDrop?.(no, Number(over.dataset.no), e.ctrlKey || e.metaKey);
      else if (!cancelled && out) out.drop();
    };
    this.scroller.addEventListener('pointerup', e => end(e, false));
    this.scroller.addEventListener('pointercancel', e => end(e, true));
    this.scroller.addEventListener('lostpointercapture', e => end(e, false));
  }

  /** The first row in view (the window's saved scroll, R1 Q31). */
  firstRow() { return this.geo ? Math.floor(this.scroller.scrollTop / this.geo.pitch + 0.5) : 0; }
  scrollToRow(row) {
    this.pendingRow = row;
    if (this.geo) this.layout();
  }

  /** The slot index of a number, and the number at a slot. */
  indexOf(no) { const g = this.geo; return g.list ? g.list.indexOf(no) : no - g.first; }
  numberAt(i) { const g = this.geo; return g.list ? g.list[i] : g.first + i; }

  /** The keyboard cursor's next number (arrows, page, home / end). */
  step(cur, dir) {
    const g = this.geo;
    if (!g) return cur;
    const page = Math.max(1, Math.floor(this.scroller.clientHeight / g.pitch)) * g.cols;
    const i = Math.max(0, this.indexOf(cur));
    const last = g.list ? g.list.length - 1 : MAX_NO - g.first;
    const j = dir === 'home' ? 0 : dir === 'end' ? Math.min(last, g.total - 1)
      : i + ({ left: -1, right: 1, up: -g.cols, down: g.cols, page_up: -page, page_down: page }[dir] || 0);
    return this.numberAt(Math.max(0, Math.min(last, j))) ?? cur;
  }

  /** Scroll so that a number's cell is in view. */
  reveal(no) {
    const g = this.geo;
    if (!g) return;
    const i = this.indexOf(no);
    if (i < 0) return;
    if (i >= g.total) { this.cfg.slots = 0; this.layout(); }
    const top = g.pad + Math.floor(i / g.cols) * g.pitch, H = this.scroller.clientHeight;
    if (top < this.scroller.scrollTop) this.scroller.scrollTop = top;
    else if (top + g.cellH > this.scroller.scrollTop + H) this.scroller.scrollTop = top + g.cellH - H;
  }
  scrollEnded() {
    clearTimeout(this._scrollTimer);
    // Not while a saved scroll is still being put back.
    this._scrollTimer = setTimeout(() => { if (this.pendingRow == null) this.opts.onScrollRow?.(this.firstRow()); }, 250);
  }
}

/** A preset cell (decisions 0288): how it fits the selection along the top, its number, the
 *  fixtures using it, its mode letter, a preview of its values, a square per feature group. */
function presetHtml(no, c, phone) {
  const x = c.extra;
  const fit = { all: 'var(--lc-ok)', some: 'var(--lc-warn)', none: 'var(--lc-text-dim)' }[c.applicability] || 'var(--lc-pool-type)';
  const pv = (x.preview || []).map(p => {
    if (p.kind === 'color') return `<i class="sw" style="background:${esc(p.rgb)}"></i>`;
    if (p.kind === 'level') return `<i class="lvl"><b style="height:${Math.round(p.value * 100)}%"></b></i>`;
    if (p.kind === 'xy') return `<i class="xy"><b style="left:${Math.round(p.x * 100)}%;top:${Math.round(p.y * 100)}%"></b></i>`;
    return `<i class="tx">${esc(p.text)}</i>`;
  }).join('');
  const squares = (x.groups || []).map(gr => `<i title="${esc(gr.name)}" style="background:${gr.color ? esc(gr.color) : 'var(--lc-text-dim)'}"></i>`).join('');
  return `<i class="lc-pre-fit" style="background:${fit}"></i>`
    + `<span class="lc-pool-no">${no}${x.use ? `<b class="lc-pre-use" title="fixtures using it">${x.use}</b>` : ''}<b class="lc-pre-mode">${esc(x.mode)}</b></span>`
    + `<span class="lc-pool-name">${esc(c.name ?? '')}</span>`
    + (phone ? '' : `<span class="lc-pre-pv">${pv}</span>`)
    + `<span class="lc-pre-fg">${squares}</span>`;
}

/** A view's screens in miniature (the Views pool, decisions 0287): each pane a box where it sits. */
function layoutHtml(layout) {
  const pct = v => `${(Number(v) * 100).toFixed(2)}%`;
  const screens = (layout.screens || []).map(s => `<span class="lc-pool-scr" title="Screen ${esc(s.screen)}">`
    + (s.windows || []).map(w => `<i data-kind="${esc(w.kind)}" style="left:${pct(w.x)};top:${pct(w.y)};width:${pct(w.w)};height:${pct(w.h)}"></i>`).join('')
    + '</span>').join('');
  return `<span class="lc-pool-layout">${screens}</span>`;
}

customElements.define('luma-pool', LumaPool);
export { LumaPool };
