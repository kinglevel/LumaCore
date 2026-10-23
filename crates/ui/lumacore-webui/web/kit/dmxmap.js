// <luma-dmxmap> — the DMX map, zoomed out (decisions §34 round 3, 12, 18; the reviewed
// `dmx-map.html` v2): the Universes tab's Map view. A row per universe — its name, a usage bar
// and "84 / 512" (slots in use, a shared slot once) — and a bar of its 512 slots: each patched
// run a block (FID, range; coloured by fixture type, a multipatch copy hatched), the free runs
// dashed, a parked run (an overlap or past 512: it sends nothing until resolved) striped red.
// Below, the conflict list with its one-tap fixes. Tap a block selects its fixture (Shift /
// Meow adds); tap a free run with fixtures selected patches them there (one line). Every gesture
// is a command line; the data is the `patch` feed (`universes`, `runs`, `conflicts`).
// On a phone (decisions §37 5: "there can be a lot of universes") the map is a list of universes,
// a card each — number and name, slots in use of 512, how many fixtures, its parked runs in red
// and a thin overview strip (the runs by fixture type, parked red; colour only) — and a tap on a
// card zooms in (`zoom`: Edit Patch's slot grid). The conflict list stays under the cards. With
// fixtures selected (`selection`, the Patch list's) every card grows a "Patch here · u.aaa" cell
// — the first free run that fits them one after another, "No room" without one — and the tap
// is the one line (§34 "4d4 hardened" 2).
//
// Zoomed in (`zoom: 'slots'`, Edit Patch and Insert fixtures' sheet): one universe's 512 slots
// in rows of 20 (`u:aaa` labels), the FID on a run's first slot; the `current` fixture gold, the
// `picked` ones outlined, `flash` slots blinking (Identify / Probe / Find), `ghost` a block to
// come; a tap on a slot is an event (the host decides its line). On an upright phone (decisions
// §37 6) ten slots a row (`u:001`, `u:011`, …), each at least 44 px high and the full width; on
// its side 20 at that height.
//
// Drag and drop, both zooms (§34 round 3, 18; "4d4 hardened" 1, 3, 4; one engine, the pointer
// core's `lift`): a press on a block or run that moves lifts it (touch: the hold is the press —
// hold still, the ring fills, the run rises, a chip above the finger names it and where it lands;
// a plain finger scrolls); the ghost snaps to slots, gold where free, red where it would overlap
// (allowed: a drop there parks both); a selected run takes the whole selection — `layout`
// 'spacing' keeps its gaps (`/KeepSpacing`), 'bunch' packs it, 'one' moves the grabbed run
// alone; the drop is one `Assign Fixture … At DMX u.aaa` line (event `line`). A ghost that
// wraps is a ribbon per part (rounded ends, square at the row break, ⤶ ⤷), the FID again on
// each continuation; with several runs only the one that would overlap is red. A drag on empty
// ground lassos (Shift / Meow adds; hold-then-drag on touch); hold and let go is the context
// (event `context`); a lifted run held over one of `springs` (Edit Patch's ‹ ›) pages it.
//
//   el.update({ universes: [{ no, name, used }], runs: [{ no, copy, break, name, universe, start,
//               count, fixture_type, parked }], conflicts: [{ kind, a, b, fixes: [{ label, line,
//               primary }] }], level, zoomable, layout, selection: [{ no, footprint }], every })
//   el.update({ zoom: 'slots', universe, runs, current, picked: [no], flash: [slot], ghost: [from, to],
//               layout, springs: [{ el, fire }] })
//   el.selection() → [fixture no]   el.select([no])
//   event `line` → { line }   event `select` → { fixtures: [no], add }   event `zoom` → { universe }
//   event `slot` → { universe, slot, no }   event `context` → { fixtures: [no], x, y }

import { rowsWord, needs, dragSetting } from './grid.js';
import { lift } from './pointer.js';
import { toast } from './toast.js';
import { phoneMode, onPhoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const a3 = n => String(n).padStart(3, '0');
const pct = n => `${((n - 1) / 512) * 100}%`;
const width = n => `calc(${(n / 512) * 100}% - 2px)`;
const len = r => Math.max(1, r.count || 1);
/** Patching needs Setup (the patching level, §34 round 3, 18). */
const NEED = 'Setup';

class LumaDmxMap extends HTMLElement {
  constructor() {
    super();
    this.spec = { universes: [], runs: [], conflicts: [], level: '', zoom: 'map', zoomable: false, universe: 1, current: null, picked: [], flash: [], ghost: null, layout: 'spacing', selection: [], every: '', springs: [] };
    this.picked = new Set();
    this.drag = null;
  }
  connectedCallback() {
    this.dataset.kit = 'dmxmap';
    this.offPhone ||= onPhoneMode(() => this.render());
    this.offLift ||= this.wire();
    this.render();
  }
  disconnectedCallback() { this.offPhone?.(); this.offPhone = null; this.offLift?.(); this.offLift = null; this.chipEl?.remove(); this.chipEl = null; }
  update(spec) {
    this.spec = { ...this.spec, ...(spec || {}) };
    // A fixture that went leaves the selection.
    const there = new Set(this.spec.runs.map(r => r.no));
    for (const n of [...this.picked]) if (!there.has(n)) this.picked.delete(n);
    this.render();
  }
  selection() { return [...this.picked].sort((a, b) => a - b); }
  select(nos) { this.picked = new Set((nos || []).map(Number)); this.paint(); }

  render() {
    if (!this.isConnected) return;
    if (this.spec.zoom === 'slots') { this.renderSlots(); return; }
    const { universes, runs, conflicts } = this.spec;
    const why = needs(NEED, this.spec.level);
    // The block's colour: its fixture type, in the order the types first appear.
    const kinds = new Map();
    for (const r of runs) if (!kinds.has(r.fixture_type)) kinds.set(r.fixture_type, kinds.size % 4);
    this.kinds = kinds;
    if (phoneMode()) return this.renderCards(kinds, why);
    const rows = universes.map(u => {
      const on = runs.filter(r => r.universe === u.no).sort((a, b) => a.start - b.start || a.no - b.no);
      // An overlap shares the bar's height: the run that lands on another takes the lower half,
      // the one under it the upper half, so both stay in sight.
      const parts = [];
      let next = 1;
      let under = null;
      for (const r of on) {
        if (r.start > next) parts.push(this.freeHtml(u.no, next, r.start - next, why));
        const end = r.start + len(r) - 1;
        const lane = r.start < next ? 'lo' : '';
        if (lane && under) under.lane = 'hi';
        const word = r.copy ? `${r.no}·M${r.copy}` : String(r.no);
        const range = `${u.no}.${a3(r.start)}–${a3(Math.min(end, 512))}`;
        const title = `Fixture ${word} ${r.name}${r.break > 1 ? ` · break ${r.break}` : ''} · ${range}${r.parked ? ' · parked: sends nothing until resolved' : ''}`;
        const part = { lane, html: lane => `<button type="button" class="lc-dmap-blk k${kinds.get(r.fixture_type) ?? 0}${r.copy ? ' copy' : ''}${r.parked ? ' parked' : ''}${lane ? ` ${lane}` : ''}" data-no="${r.no}" data-copy="${r.copy || 0}" data-start="${r.start}" style="left:${pct(r.start)};width:${width(Math.min(end, 512) - r.start + 1)}" title="${esc(title)}" aria-label="${esc(title)}"><span>${esc(word)}</span><small>${esc(range)}</small></button>` };
        parts.push(part);
        if (end + 1 > next) under = part;
        next = Math.max(next, end + 1);
      }
      let blocks = parts.map(p => (typeof p === 'string' ? p : p.html(p.lane))).join('');
      if (next <= 512) blocks += this.freeHtml(u.no, next, 513 - next, why);
      const used = u.used || 0;
      return `<div class="lc-dmap-row">
        <div class="lc-dmap-name"><span>${esc(u.no)}${u.name ? ` · ${esc(u.name)}` : ''}</span>
          <span class="lc-dmap-usage" role="meter" aria-valuemin="0" aria-valuemax="512" aria-valuenow="${used}" aria-label="Slots in use"><i style="width:${(used / 512) * 100}%"></i></span>
          <small>${used} / 512</small>${this.spec.zoomable ? `<button type="button" class="lc-dmap-zoom" data-zoom="${esc(u.no)}" title="Edit Patch: universe ${esc(u.no)}'s slots"><luma-icon name="maximize-2"></luma-icon>Zoom in</button>` : ''}</div>
        <div class="lc-dmap-bar" data-universe="${esc(u.no)}">${blocks}</div></div>`;
    }).join('');
    const ruler = `<div class="lc-dmap-row lc-dmap-ruler" aria-hidden="true"><span></span><div>${[1, 128, 256, 384, 512].map(n => `<span>${n}</span>`).join('')}</div></div>`;
    this.innerHTML = `<div class="lc-dmap">
      <div class="lc-dmap-scroll">${universes.length ? rows + ruler : '<div class="lc-dmap-note">No universes yet.</div>'}<div class="lc-dmap-lasso" hidden data-lasso></div></div>
      ${this.conflictsHtml(conflicts, why)}
    </div>`;
    for (const b of this.querySelectorAll('[data-fix]')) b.onclick = () => { if (!b.hasAttribute('aria-disabled')) this.line(b.dataset.fix); };
    // Zoom in: Edit Patch on that universe (a button on the bar, or a double-tap on it).
    const zoom = u => this.dispatchEvent(new CustomEvent('zoom', { detail: { universe: Number(u) }, bubbles: true }));
    for (const b of this.querySelectorAll('[data-zoom]')) b.onclick = e => { e.stopPropagation(); zoom(b.dataset.zoom); };
    if (this.spec.zoomable) for (const bar of this.querySelectorAll('.lc-dmap-bar')) bar.ondblclick = () => zoom(bar.dataset.universe);
    this.paint();
    this.restoreDrag();
  }

  /** A phone: a card per universe; a tap zooms in; with a selection, Patch here. */
  renderCards(kinds, why) {
    const { universes, runs, conflicts, selection } = this.spec;
    const sel = (selection || []).map(s => ({ no: Number(s.no), n: Math.max(1, Number(s.footprint) || 1) }));
    const need = sel.reduce((a, s) => a + s.n, 0);
    const skip = new Set(sel.map(s => s.no));
    const cards = universes.map(u => {
      const on = runs.filter(r => r.universe === u.no);
      const fixtures = new Set(on.map(r => r.no)).size;
      const parked = on.filter(r => r.parked).length;
      const strip = on.map(r => `<i class="k${kinds.get(r.fixture_type) ?? 0}${r.parked ? ' parked' : ''}" style="left:${pct(r.start)};width:${width(Math.min(r.start + len(r) - 1, 512) - r.start + 1)}"></i>`).join('');
      const used = u.used || 0;
      let go = '';
      if (sel.length) {
        const at = this.firstFree(u.no, need, skip);
        go = at ? `<button type="button" class="lc-dmap-go" data-go="${esc(u.no)}" data-at="${at}"${why ? ` aria-disabled="true" title="${esc(why)}"` : ''}>Patch here<small>${esc(u.no)}.${a3(at)}</small></button>`
          : `<span class="lc-dmap-go no">No room</span>`;
      }
      return `<div class="lc-dmap-cardrow"><button type="button" class="lc-dmap-card" data-zoom="${esc(u.no)}" aria-label="Universe ${esc(u.no)}: ${used} of 512 slots in use — open its slots">
        <span class="lc-dmap-card-top"><b>${esc(u.no)}${u.name ? ` · ${esc(u.name)}` : ''}</b><small>${used} / 512</small></span>
        <span class="lc-dmap-card-sub"><span>${fixtures} fixture${fixtures === 1 ? '' : 's'}</span>${parked ? `<em>${parked} parked</em>` : ''}<luma-icon name="chevron-right"></luma-icon></span>
        <span class="lc-dmap-strip" aria-hidden="true">${strip}</span></button>${go}</div>`;
    }).join('');
    this.innerHTML = `<div class="lc-dmap lc-dmap-list">
      <div class="lc-dmap-scroll">${universes.length ? cards : '<div class="lc-dmap-note">No universes yet.</div>'}</div>
      ${this.conflictsHtml(conflicts, why)}
    </div>`;
    for (const b of this.querySelectorAll('[data-fix]')) b.onclick = () => { if (!b.hasAttribute('aria-disabled')) this.line(b.dataset.fix); };
    for (const b of this.querySelectorAll('[data-zoom]')) b.onclick = () => this.dispatchEvent(new CustomEvent('zoom', { detail: { universe: Number(b.dataset.zoom) }, bubbles: true }));
    for (const b of this.querySelectorAll('[data-go]')) b.onclick = () => {
      if (b.hasAttribute('aria-disabled')) return toast(why);
      this.line(`Assign Fixture ${rowsWord(sel.map(s => s.no))} At DMX ${b.dataset.go}.${a3(Number(b.dataset.at))}${this.spec.every || ''}`);
    };
  }

  /** Zoomed in: one universe's slots in rows of 20 (10 on an upright phone). */
  renderSlots() {
    const phone = phoneMode();
    const per = phone === 'up' ? 10 : 20;
    const { runs, universe, current } = this.spec;
    const picked = new Set((this.spec.picked || []).map(Number));
    const flash = new Set(this.spec.flash || []);
    const [g0, g1] = this.spec.ghost || [0, -1];
    const kinds = new Map();
    for (const r of runs) if (!kinds.has(r.fixture_type)) kinds.set(r.fixture_type, kinds.size % 4);
    const owner = new Array(513).fill(null);
    const shared = new Set();
    for (const r of runs.filter(r => r.universe === universe)) {
      for (let s = r.start; s < r.start + len(r) && s <= 512; s++) {
        if (owner[s]) shared.add(s);
        owner[s] = owner[s] || r;
      }
    }
    let html = '';
    for (let row = 0; row < Math.ceil(512 / per); row++) {
      html += `<div class="lc-dmap-srow"><span class="lc-dmap-lab">${universe}:${a3(row * per + 1)}</span>`;
      for (let c = 1; c <= per; c++) {
        const s = row * per + c;
        if (s > 512) break;
        const r = owner[s];
        const cls = ['lc-dmap-slot'];
        if (r) cls.push(r.no === current ? 'cur' : `k${kinds.get(r.fixture_type) ?? 0}`);
        if (r && picked.has(r.no) && r.no !== current) cls.push('picked');
        if (r?.parked || shared.has(s)) cls.push('parked');
        if (flash.has(s)) cls.push('flash');
        if (s >= g0 && s <= g1) cls.push('ghost');
        const first = r && r.start === s ? `<span class="fid">${esc(r.copy ? `${r.no}·M${r.copy}` : r.no)}</span>` : '';
        const label = `${universe}.${a3(s)}${r ? ` · Fixture ${r.no}${r.copy ? ` Multipatch ${r.copy}` : ''}` : ' · free'}`;
        html += `<button type="button" class="${cls.join(' ')}" data-slot="${s}"${r ? ` data-no="${r.no}" data-copy="${r.copy || 0}"` : ''} title="${esc(label)}" aria-label="${esc(label)}">${first}${s}</button>`;
      }
      html += '</div>';
    }
    this.innerHTML = `<div class="lc-dmap lc-dmap-slots${phone ? ` phone ${phone}` : ''}" style="--lc-dmap-per:${per}"><div class="lc-dmap-scroll">${html}<div class="lc-dmap-lasso" hidden data-lasso></div></div></div>`;
    this.per = per;
    this.restoreDrag();
  }
  /** A lift in progress while the host redrew (‹ › sprang, the feed moved): the ghost follows. */
  restoreDrag() {
    const d = this.drag;
    if (!d) return;
    this.querySelector('.lc-dmap')?.classList.add('lifting');
    for (const r of d.runs) for (const el of this.querySelectorAll(`[data-no="${r.no}"]`)) el.classList.add('drag');
    if (d.last) this.moveGhost(d.last);
  }
  freeHtml(u, start, count, why) {
    const label = `${u}.${a3(start)}–${a3(start + count - 1)} free`;
    return `<button type="button" class="lc-dmap-free" data-start="${start}" data-count="${count}" style="left:${pct(start)};width:${width(count)}"
      title="${esc(why || `${label} — tap to patch the selection here`)}" aria-label="${esc(label)}"${why ? ' aria-disabled="true"' : ''}></button>`;
  }
  conflictsHtml(conflicts, why) {
    if (!conflicts.length) return `<div class="lc-dmap-conflicts ok"><h3>No overlaps — every fixture sends</h3></div>`;
    const row = c => {
      const what = c.b ? `${esc(c.a.word)} ${esc(c.a.range)} overlaps ${esc(c.b.word)} ${esc(c.b.range)}` : `${esc(c.a.word)} ${esc(c.a.range)} runs past 512`;
      const fixes = c.fixes.map(f => `<button type="button" class="lc-dmap-fix${f.primary ? ' primary' : ''}" data-fix="${esc(f.line)}"${why ? ` aria-disabled="true" title="${esc(why)}"` : ` title="${esc(f.line)}"`}>${esc(f.label)}</button>`).join('');
      return `<div class="lc-dmap-crow"><span>${what}<small>${c.b ? 'Both are parked' : 'Parked'}: they keep their address and programming but send nothing until resolved.</small></span><span class="lc-dmap-fixes">${fixes}</span></div>`;
    };
    return `<div class="lc-dmap-conflicts"><h3>${conflicts.length} conflict${conflicts.length === 1 ? '' : 's'} · parked fixtures send nothing</h3>${conflicts.map(row).join('')}</div>`;
  }

  // ── The drag engine on the bars or the slot grid (pointer.js `lift`), wired once on the element ──
  wire() {
    const zoomOf = () => (this.spec.zoom === 'slots' ? 'slots' : 'map');
    const why = () => needs(NEED, this.spec.level);
    const item = e => {
      const t = e.target;
      if (phoneMode() && zoomOf() === 'map') return undefined; // the cards: buttons
      if (zoomOf() === 'map') {
        const blk = t.closest?.('.lc-dmap-blk');
        if (blk) return { no: Number(blk.dataset.no), copy: Number(blk.dataset.copy) || 0, el: blk };
        if (t.closest?.('.lc-dmap-free, .lc-dmap-bar')) return null;
        return t.closest?.('button, .lc-dmap-conflicts') ? undefined : null;
      }
      const slot = t.closest?.('.lc-dmap-slot');
      if (!slot) return t.closest?.('button') ? undefined : null;
      return slot.dataset.no ? { no: Number(slot.dataset.no), copy: Number(slot.dataset.copy) || 0, slot: Number(slot.dataset.slot), el: slot } : null;
    };
    const sc = () => this.querySelector('.lc-dmap-scroll');
    return lift(this, {
      item, scroller: sc,
      mode: () => String(dragSetting() || 'Auto').toLowerCase(), phone: () => !!phoneMode(),
      springs: () => this.spec.springs || [],
      onTap: (it, info) => this.tap(zoomOf(), it, info, why()),
      onLift: (it, info) => this.liftRun(zoomOf(), it, info, why()),
      onMove: info => this.moveGhost(info),
      onDrop: (info, cancelled) => this.drop(info, cancelled),
      onLasso: info => this.lassoDraw(sc(), info, info),
      onLassoMove: info => this.lassoDraw(sc(), this.lasso0, info),
      onLassoEnd: (info, cancelled) => this.lassoEnd(zoomOf(), sc(), cancelled, info),
      onContext: (it, at) => { if (!it) return; if (!this.picked.has(it.no)) { this.picked = new Set([it.no]); this.paint(); this.emitSel(false); } this.dispatchEvent(new CustomEvent('context', { detail: { fixtures: this.selection(), x: at.x, y: at.y }, bubbles: true })); },
    });
  }
  tap(zoom, it, info, why) {
    // The pointer was captured: what the tap released on is found by its point.
    const under = document.elementFromPoint(info.x, info.y);
    if (zoom === 'map') {
      if (it) return this.tapBlock(it.no, info);
      const free = under?.closest?.('.lc-dmap-free');
      if (free && this.contains(free)) this.tapFree(free, info.event, why);
      return;
    }
    if (it) return this.dispatchEvent(new CustomEvent('select', { detail: { fixtures: [it.no], add: info.add }, bubbles: true }));
    const slot = under?.closest?.('.lc-dmap-slot');
    if (slot) this.dispatchEvent(new CustomEvent('slot', { detail: { universe: this.spec.universe, slot: Number(slot.dataset.slot), no: null }, bubbles: true }));
  }
  /** The runs a lift moves: the grabbed fixture, or the whole selection when it is selected. */
  liftRun(zoom, it, info, why) {
    if (why) { toast(why); return; }
    if (it.copy) { toast('A multipatch copy moves from the Patch list'); return; }
    const picked = zoom === 'map' ? this.picked : new Set((this.spec.picked || []).map(Number));
    const layout = this.spec.layout || 'spacing';
    const nos = picked.has(it.no) && layout !== 'one' ? [...picked] : [it.no];
    const runs = this.spec.runs.filter(r => nos.includes(r.no) && !r.copy && (r.break || 1) <= 1);
    const grabbed = runs.find(r => r.no === it.no);
    if (!grabbed) return;
    const at = zoom === 'map' ? this.slotAtBar(it.el.closest('.lc-dmap-bar'), info.x) : it.slot;
    this.drag = { zoom, runs, grabbed, grab: Math.max(0, at - grabbed.start), layout, last: null, plan: null };
    this.querySelector('.lc-dmap')?.classList.add('lifting');
    for (const r of runs) for (const el of this.querySelectorAll(`[data-no="${r.no}"]`)) el.classList.add('drag');
    this.moveGhost(info);
  }
  /** Where the lifted runs would land, from the pointer: the plan, the ghost, the chip. */
  moveGhost(info) {
    const d = this.drag;
    if (!d) return;
    d.last = { x: info.x, y: info.y, event: info.event };
    const under = document.elementFromPoint(info.x, info.y);
    let u, at;
    if (d.zoom === 'map') {
      const bar = under?.closest?.('.lc-dmap-bar');
      if (!bar) return;
      u = Number(bar.dataset.universe); at = this.slotAtBar(bar, info.x);
    } else {
      const slot = under?.closest?.('.lc-dmap-slot');
      if (!slot || !this.contains(slot)) return;
      u = this.spec.universe; at = Number(slot.dataset.slot);
    }
    const g = d.grabbed;
    const here = d.runs.filter(r => r.universe === g.universe), away = d.runs.filter(r => r.universe !== g.universe);
    const place = [];
    if (d.layout === 'bunch' && d.runs.length > 1) {
      let s = at - d.grab;
      const order = [g, ...d.runs.filter(r => r !== g).sort((a, b) => a.universe - b.universe || a.start - b.start)];
      s = Math.max(1, Math.min(512 - order.reduce((n, r) => n + len(r), 0) + 1, s));
      for (const r of order) { place.push({ r, s }); s += len(r); }
    } else {
      let delta = at - d.grab - g.start;
      const lo = Math.min(...here.map(r => r.start)), hi = Math.max(...here.map(r => r.start + len(r) - 1));
      delta = Math.max(1 - lo, Math.min(512 - hi, delta));
      for (const r of here) place.push({ r, s: r.start + delta });
      let tail = Math.max(...here.map(r => r.start + len(r))) + delta;
      for (const r of away) { place.push({ r, s: tail }); tail += len(r); }
    }
    const moving = new Set(d.runs.map(r => r.no));
    const taken = this.taken(u, moving);
    const bad = new Set();
    for (const p of place) for (let s = p.s; s < p.s + len(p.r); s++) if (s > 512 || taken[s]) bad.add(p.r.no);
    const first = Math.min(...place.map(p => p.s));
    const moved = !(u === g.universe && place.every(p => p.s === p.r.start));
    const nos = d.runs.map(r => r.no);
    d.plan = { u, place, bad, first, moved, line: `Assign Fixture ${rowsWord(nos)} At DMX ${u}.${a3(first)}${d.layout === 'spacing' && nos.length > 1 ? ' /KeepSpacing' : ''}` };
    this.drawGhost(d);
    const parks = bad.size ? (nos.length === 1 ? ' · parks both' : ` · parks Fixture ${rowsWord([...bad])}`) : '';
    this.chip(info.x, info.y, moved ? `Fixture ${rowsWord(nos)} → ${u}.${a3(first)}${parks}` : `Fixture ${rowsWord(nos)} · ${g.universe}.${a3(g.start)}`, bad.size > 0);
  }
  drawGhost(d) {
    for (const el of this.querySelectorAll('.lc-dmap-ghost')) el.remove();
    for (const el of this.querySelectorAll('.lc-dmap-slot.ghost')) { el.classList.remove('ghost', 'bad', 'st', 'en', 'brk', 'cont'); el.querySelector('.fid.again')?.remove(); }
    const { u, place, bad } = d.plan;
    if (d.zoom === 'map') {
      const bar = this.querySelector(`.lc-dmap-bar[data-universe="${u}"]`);
      if (!bar) return;
      for (const p of place) {
        const n = Math.min(p.s + len(p.r) - 1, 512) - p.s + 1;
        if (n < 1 || p.s > 512) continue;
        bar.insertAdjacentHTML('beforeend', `<span class="lc-dmap-ghost${bad.has(p.r.no) ? ' bad' : ''}" style="left:${pct(p.s)};width:${width(n)}"><span class="fid">${esc(p.r.no)}</span></span>`);
      }
      return;
    }
    const per = this.per || 20;
    for (const p of place) {
      const last = p.s + len(p.r) - 1;
      for (let s = p.s; s <= Math.min(last, 512); s++) {
        const el = this.querySelector(`.lc-dmap-slot[data-slot="${s}"]`);
        if (!el) continue;
        el.classList.add('ghost');
        if (bad.has(p.r.no)) el.classList.add('bad');
        if (s === p.s) el.classList.add('st');
        if (s === last) el.classList.add('en');
        if (s % per === 0 && s !== last) el.classList.add('brk');
        if (s % per === 1 && s !== p.s) { el.classList.add('cont'); el.insertAdjacentHTML('afterbegin', `<span class="fid again">${esc(p.r.no)}</span>`); }
      }
    }
  }
  drop(info, cancelled) {
    const d = this.drag;
    this.drag = null;
    this.querySelector('.lc-dmap')?.classList.remove('lifting');
    for (const el of this.querySelectorAll('.drag')) el.classList.remove('drag');
    for (const el of this.querySelectorAll('.lc-dmap-ghost')) el.remove();
    for (const el of this.querySelectorAll('.lc-dmap-slot.ghost')) { el.classList.remove('ghost', 'bad', 'st', 'en', 'brk', 'cont'); el.querySelector('.fid.again')?.remove(); }
    this.chipEl?.remove(); this.chipEl = null;
    if (!d || cancelled || !d.plan?.moved) return;
    this.line(d.plan.line);
  }
  /** The chip above the pointer: the lifted run and where it lands (gold; red = parks). */
  chip(x, y, text, bad) {
    if (!this.chipEl) { this.chipEl = document.createElement('div'); this.chipEl.className = 'lc-dmap-chip'; this.chipEl.dataset.kit = 'chip'; document.body.append(this.chipEl); }
    const el = this.chipEl;
    el.textContent = text;
    el.classList.toggle('bad', !!bad);
    // Above the finger; below it near the top edge.
    const flip = y < 80;
    el.classList.toggle('below', flip);
    el.style.left = `${Math.max(8, Math.min(window.innerWidth - 8, x))}px`;
    el.style.top = `${flip ? y + 36 : y - 40}px`;
  }
  /** The slot under x on a bar (1–512). */
  slotAtBar(bar, x) {
    const r = bar.getBoundingClientRect();
    return Math.max(1, Math.min(512, Math.floor(((x - r.left) / r.width) * 512) + 1));
  }
  /** Slots of universe `u` in use by runs other than `skip` (a fixture number set). */
  taken(u, skip) {
    const t = new Array(514).fill(false);
    for (const r of this.spec.runs) if (r.universe === u && !skip.has(r.no)) for (let s = r.start; s < r.start + len(r) && s <= 512; s++) t[s] = true;
    return t;
  }
  /** The first run of `need` free slots in universe `u`, or null. */
  firstFree(u, need, skip) {
    const t = this.taken(u, skip);
    for (let s = 1; s + need - 1 <= 512; s++) { let ok = true; for (let i = 0; i < need && ok; i++) ok = !t[s + i]; if (ok) return s; }
    return null;
  }

  // ── The lasso: a box over blocks or runs ──
  lassoDraw(sc, a, b) {
    if (a === b) this.lasso0 = a;
    const box = sc.querySelector('[data-lasso]');
    const r = sc.getBoundingClientRect();
    const x = Math.min(a.x, b.x) - r.left + sc.scrollLeft, y = Math.min(a.y, b.y) - r.top + sc.scrollTop;
    Object.assign(box.style, { left: `${x}px`, top: `${y}px`, width: `${Math.abs(a.x - b.x)}px`, height: `${Math.abs(a.y - b.y)}px` });
    box.hidden = false;
    this.lassoRect = { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) };
  }
  lassoEnd(zoom, sc, cancelled, info) {
    const box = sc.querySelector('[data-lasso]');
    if (box) box.hidden = true;
    const rect = this.lassoRect;
    this.lassoRect = null;
    if (cancelled || !rect) return;
    const hits = q => q.left < rect.x + rect.w && q.right > rect.x && q.top < rect.y + rect.h && q.bottom > rect.y;
    const nos = new Set();
    for (const el of this.querySelectorAll(zoom === 'map' ? '.lc-dmap-blk' : '.lc-dmap-slot[data-no]')) if (hits(el.getBoundingClientRect())) nos.add(Number(el.dataset.no));
    if (zoom === 'map') {
      if (info.add) for (const n of nos) this.picked.add(n); else this.picked = nos;
      this.paint();
      return this.emitSel(info.add);
    }
    if (nos.size) this.dispatchEvent(new CustomEvent('select', { detail: { fixtures: [...nos].sort((a, b) => a - b), add: info.add }, bubbles: true }));
  }

  // ── Gestures: each one line ──
  tapBlock(no, info) {
    if (info.add) { if (this.picked.has(no)) this.picked.delete(no); else this.picked.add(no); } else this.picked = new Set([no]);
    this.paint();
    this.emitSel(info.add);
  }
  tapFree(b, e, why) {
    if (why) return toast(why);
    if (!this.picked.size) return toast('Select fixtures first, then tap a free run');
    const bar = b.closest('.lc-dmap-bar');
    const r = bar.getBoundingClientRect();
    const start = Number(b.dataset.start);
    const at = Math.min(start + Number(b.dataset.count) - 1, Math.max(start, Math.floor(((e.clientX - r.left) / r.width) * 512) + 1));
    this.line(`Assign Fixture ${rowsWord(this.selection())} At DMX ${bar.dataset.universe}.${a3(at)}`);
  }
  emitSel(add) { this.dispatchEvent(new CustomEvent('select', { detail: { fixtures: this.selection(), add: !!add }, bubbles: true })); }
  line(line) { this.dispatchEvent(new CustomEvent('line', { detail: { line }, bubbles: true })); }
  paint() {
    for (const b of this.querySelectorAll('.lc-dmap-blk')) b.classList.toggle('sel', this.picked.has(Number(b.dataset.no)));
  }
}
customElements.define('luma-dmxmap', LumaDmxMap);
