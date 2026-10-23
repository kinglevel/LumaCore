// Edit Patch — the DMX map zoomed in (decisions §34 round 3, 12, 18, "the ways into Edit Patch";
// `dmx-map.html` v2, `edit-patch-entry.html` v1): the selected fixtures on the left (a
// <luma-grid>: FID, Name, Address), one universe's slot grid on the right (<luma-dmxmap
// zoom="slots">, ‹ › universe), the function row below. Opening runs nothing; every tool is one
// line:
//   Patch (a tap on a slot)  Assign Fixture <current> At DMX u.aaa — then the next fixture (Skip
//                            patched: the next without an address); Layout Keep spacing / Bunch
//                            up moves the whole selection from the slot instead
//   Probe (a tap)            Identify DMX u.aaa
//   Find                     Identify DMX <the current fixture's footprint> /As Fixture n, ‹ › a
//                            footprint along, Patch here = Assign … At DMX u.aaa
//   Identify                 Identify Fixture <all> / Off Identify
//   Unpatch · Next free address · Next free universe · Move to this universe — Assign / Unassign
// A run drags (§34 "4d4 hardened" 1, 3; the map's engine): on the desk a press that moves lifts
// it, on touch the hold is the press ("hold a run to move it"); the drop is the map's one line;
// a lifted run held over ‹ › springs the grid to that universe; a tap on a run makes its fixture
// current (added to the list when it is not in it), a lasso over runs selects them.
// While anything flashes a red strip says so (Stop = Off Identify). Closing (✕, Esc) runs `Off
// Identify Fixture Thru; Off Identify DMX Thru` when the dialog started one. Setup and up changes;
// anyone looks. On a phone (decisions §37 6) it is full screen, the fixtures fold to one line
// (the current fixture, ‹ › to the next of the selection) and the slots are rows of 10.
//
//   openEditPatch({ fixtures: [{ no, name, address, footprint }], universes: [{ no, name, used }],
//     runs, universe, typed ('2.150', the value dialog's), offset, level, run(line), closed,
//     context(nos, { x, y }) (the hold on a run: the host's context menu) })
//     → { update({ fixtures, universes, runs }), close() }

import { needs } from './grid.js';
import { inOrder } from './value.js';
import { toast } from './toast.js';
import { phoneMode, onPhoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const a3 = n => String(n).padStart(3, '0');
const NEED = 'Setup';
const COLUMNS = [
  { id: 'no', label: 'FID', field: 'no', width: 64, priority: 1, frozen: true, readonly: true },
  { id: 'name', label: 'Name', field: 'name', width: 140, priority: 1, readonly: true },
  { id: 'address', label: 'Address', field: 'address', width: 90, priority: 1, readonly: true },
];
/** `1.057` → { u, s } (null without an address). */
const parse = a => { const m = String(a || '').match(/^(\d+)[.:](\d+)$/); return m ? { u: +m[1], s: +m[2] } : null; };

let current = null;
export function closeEditPatch() { current?.(); }

export function openEditPatch({ fixtures = [], universes = [], runs = [], universe, typed, offset = 0, level = '', run, closed, context } = {}) {
  closeEditPatch();
  const t = parse(typed);
  const st = {
    fx: fixtures, universes, runs, cur: 0, mode: 'patch', layout: 'one', skip: true,
    ident: false, probe: null, find: null, started: false,
    u: t?.u || universe || parse(fixtures[0]?.address)?.u || 1,
  };
  const why = () => needs(NEED, level);
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ep-scrim';
  scrim.dataset.kit = 'editpatch';
  document.body.append(scrim);
  const box = document.createElement('div');
  box.className = 'lc-box lc-ep';
  box.setAttribute('role', 'dialog');
  box.setAttribute('aria-label', 'Edit Patch');
  scrim.append(box);
  box.innerHTML = `<header class="lc-box-title lc-ep-title"><luma-icon name="grid-3x3"></luma-icon>
      <span class="lc-ep-crumb"><span>Patch</span> <i>›</i> <b>Edit Patch</b></span><span class="lc-ep-chip" data-chip></span>
      <button type="button" class="lc-close" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
    <div class="lc-ep-warn" data-warn hidden><luma-icon name="triangle-alert"></luma-icon><span data-warntext></span><button type="button" class="lc-btn" data-stop>Stop</button></div>
    <div class="lc-ep-body">
      <div class="lc-ep-left"><luma-grid class="lc-ep-list"></luma-grid>
        <div class="lc-ep-one" data-one><button type="button" class="lc-btn" data-cur="-1" aria-label="Previous fixture"><luma-icon name="chevron-left"></luma-icon></button>
          <span data-onetext></span><button type="button" class="lc-btn" data-cur="1" aria-label="Next fixture"><luma-icon name="chevron-right"></luma-icon></button></div></div>
      <div class="lc-ep-right">
        <div class="lc-ep-bar"><button type="button" class="lc-btn" data-u="-1" aria-label="Previous universe"><luma-icon name="chevron-left"></luma-icon></button>
          <span class="lc-ep-u" data-uname></span><button type="button" class="lc-btn" data-u="1" aria-label="Next universe"><luma-icon name="chevron-right"></luma-icon></button>
          <span class="lc-ep-find" data-findbar hidden><button type="button" class="lc-btn" data-step="-1">‹ Footprint</button><span data-findat></span><button type="button" class="lc-btn" data-step="1">Footprint ›</button><button type="button" class="lc-btn primary" data-here>Patch here</button></span>
          <span class="lc-ep-hint" data-hint></span></div>
        <luma-dmxmap class="lc-ep-sheet"></luma-dmxmap>
      </div>
    </div>
    <footer class="lc-ep-fn" data-fn></footer>
    <div class="lc-ep-last lc-selectable" data-last></div>`;
  const list = box.querySelector('.lc-ep-list');
  const phone = () => { const m = phoneMode(); box.classList.toggle('phone', !!m); box.classList.toggle('side', m === 'side'); scrim.classList.toggle('phone', !!m); };
  phone();
  const offPhone = onPhoneMode(phone);
  const map = box.querySelector('luma-dmxmap');

  // ── The lines ──
  const cur = () => st.fx[st.cur];
  const all = () => inOrder(st.fx.map(f => Number(f.no)));
  const every = () => (offset > 0 && st.fx.length > 1 ? ` /Every ${Math.max(...st.fx.map(f => Number(f.footprint) || 1)) + offset}` : '');
  const send = line => {
    if (why()) { toast(why()); return false; }
    box.querySelector('[data-last]').textContent = `› ${line}`;
    run(line);
    return true;
  };
  const identify = line => { if (send(line)) st.started = true; };
  const stopAll = () => { if (st.ident || st.probe || st.find) send('Off Identify'); st.ident = false; st.probe = null; st.find = null; };
  const nextFree = () => {
    const used = new Set(st.universes.filter(u => (u.used || 0) > 0).map(u => Number(u.no)));
    const empty = st.universes.find(u => !(u.used > 0));
    if (empty) return Number(empty.no);
    let n = 1;
    while (used.has(n)) n++;
    return n;
  };
  const findLine = () => {
    const f = cur(), fp = Number(f.footprint) || 1;
    return `Identify DMX ${st.find.u}.${a3(st.find.s)} Thru ${st.find.u}.${a3(Math.min(512, st.find.s + fp - 1))} /As Fixture ${f.no}`;
  };
  const tapSlot = ({ slot }) => {
    if (st.mode === 'probe') { st.probe = slot; identify(`Identify DMX ${st.u}.${a3(slot)}`); return draw(); }
    if (st.mode === 'find') { st.find = { u: st.u, s: slot }; identify(findLine()); return draw(); }
    if (!cur()) return;
    const at = `${st.u}.${a3(slot)}`;
    const line = st.layout === 'one' ? `Assign Fixture ${cur().no} At DMX ${at}`
      : `Assign Fixture ${all()} At DMX ${at}${st.layout === 'spacing' ? ' /KeepSpacing' : every()}`;
    if (!send(line)) return;
    if (st.layout === 'one') advance();
    draw();
  };
  const advance = () => {
    for (let i = st.cur + 1; i < st.fx.length; i++) {
      if (!st.skip || !parse(st.fx[i].address)) { st.cur = i; return; }
    }
  };
  const fn = {
    patch: () => { stopAll(); st.mode = 'patch'; },
    probe: () => { if (st.mode === 'probe') { stopAll(); st.mode = 'patch'; } else { stopAll(); st.mode = 'probe'; } },
    find: () => {
      if (st.mode === 'find') { stopAll(); st.mode = 'patch'; return; }
      if (!cur()) return;
      stopAll();
      const a = parse(cur().address) || { u: st.u, s: 1 };
      st.mode = 'find'; st.u = a.u; st.find = { u: a.u, s: a.s };
      identify(findLine());
    },
    identify: () => { if (st.ident) { send('Off Identify'); st.ident = false; } else { stopAll(); st.ident = true; identify(`Identify Fixture ${all()}`); } },
    unpatch: () => send(`Unassign Fixture ${all()} At DMX`),
    next_free: () => send(`Assign Fixture ${all()} At DMX${every()}`),
    next_universe: () => { const n = nextFree(); if (send(`Assign Fixture ${all()} At DMX ${n}${every()}`)) st.u = n; },
    this_universe: () => send(`Assign Fixture ${all()} At DMX ${st.u}${every()}`),
    layout: () => { st.layout = { one: 'spacing', spacing: 'bunch', bunch: 'one' }[st.layout]; },
    skip: () => { st.skip = !st.skip; },
  };

  // ── Drawing ──
  const draw = () => {
    const f = cur();
    box.querySelector('[data-chip]').textContent = `${st.fx.length} fixture${st.fx.length === 1 ? '' : 's'} · ${all() || 'none'}`;
    const u = st.universes.find(x => Number(x.no) === st.u);
    box.querySelector('[data-uname]').textContent = `Universe ${st.u}${u?.name ? ` · ${u.name}` : ''} · ${u?.used || 0} / 512${u ? '' : ' · new'}`;
    const flashing = st.ident ? `Identify Fixture ${all()}` : st.probe ? `Probe ${st.u}.${a3(st.probe)}` : st.find ? `Find: Fixture ${f?.no} at ${st.find.u}.${a3(st.find.s)}` : '';
    box.querySelector('[data-warn]').hidden = !flashing;
    box.querySelector('[data-warntext]').textContent = flashing ? `${flashing} — flashing on the live rig` : '';
    const findbar = box.querySelector('[data-findbar]');
    findbar.hidden = st.mode !== 'find';
    if (st.find) box.querySelector('[data-findat]').textContent = `${st.find.u}.${a3(st.find.s)}`;
    const move = matchMedia('(any-pointer: coarse)').matches ? 'hold a run to move it' : 'drag a run to move it';
    box.querySelector('[data-hint]').textContent = why() || { patch: f ? (st.layout === 'one' ? `Tap a slot: Fixture ${f.no} goes there · ${move}` : `Tap a slot: the selection goes there (${st.layout === 'spacing' ? 'keeping its spacing' : 'bunched up'}) · ${move}`) : '', probe: 'Tap any slot to flash it', find: 'Step until the unit reacts, then Patch here' }[st.mode];
    // The slots that flash: the identified fixtures', the probe, the found run.
    const flash = [];
    if (st.ident) for (const r of st.runs.filter(r => r.universe === st.u && st.fx.some(x => Number(x.no) === r.no))) for (let s = r.start; s < r.start + r.count; s++) flash.push(s);
    if (st.probe) flash.push(st.probe);
    if (st.find && st.find.u === st.u && f) for (let s = st.find.s; s < st.find.s + (Number(f.footprint) || 1); s++) flash.push(s);
    box.querySelector('[data-onetext]').innerHTML = f ? `<b>Fixture ${esc(f.no)}</b> ${esc(f.name || '')} · ${esc(f.address || 'no address')} <small>${st.cur + 1} of ${st.fx.length}</small>` : 'No fixtures';
    map.update({ zoom: 'slots', universe: st.u, runs: st.runs, current: f ? Number(f.no) : null, picked: st.fx.map(x => Number(x.no)), flash, layout: st.layout,
      springs: [...box.querySelectorAll('[data-u]')].map(el => ({ el, fire: () => { st.u = Math.max(1, st.u + Number(el.dataset.u)); draw(); } })) });
    list.update({ key: 'editpatch.list', id: 'no', row_key: 'key', columns: COLUMNS, rows: st.fx.map(x => ({ ...x, key: String(x.no) })), state: {}, level, caption: '', bar: false });
    if (f) list.select?.([String(f.no)]);
    const mode = id => (st.mode === id ? ' on' : '');
    const off = why() ? ` aria-disabled="true" title="${esc(why())}"` : '';
    const b = (id, icon, label, sub = '', cls = '') => `<button type="button" class="lc-ep-btn${cls}" data-fn="${id}"${off}><luma-icon name="${icon}"></luma-icon><span>${esc(label)}</span>${sub ? `<small>${esc(sub)}</small>` : ''}</button>`;
    box.querySelector('[data-fn]').innerHTML = [
      b('patch', 'plug', 'Patch', 'tap a slot', mode('patch')),
      b('probe', 'crosshair', 'Probe', 'tap any slot', mode('probe')),
      b('find', 'search', 'Find', 'step by footprint', mode('find')),
      b('identify', 'focus', 'Identify', st.ident ? 'flashing' : 'the selection', st.ident ? ' on' : ''),
      b('unpatch', 'unplug', 'Unpatch'),
      b('next_free', 'plug', 'Next free address'),
      b('next_universe', 'plug', 'Next free universe', `universe ${nextFree()}`),
      b('this_universe', 'plug', 'Move to this universe', `universe ${st.u}`),
      b('layout', 'columns-3', { one: 'One at a time', spacing: 'Keep spacing', bunch: 'Bunch up' }[st.layout], 'a tap moves'),
      b('skip', 'list-start', 'Skip patched', st.skip ? 'On' : 'Off', st.skip ? ' on' : ''),
      offset ? b('offset', 'columns-2', `Offset · ${offset}`, 'from the patch window') : '',
    ].join('');
  };

  // ── Wiring ──
  list.addEventListener('select', e => {
    const k = e.detail?.rows?.[0];
    const i = st.fx.findIndex(x => String(x.no) === String(k));
    if (i >= 0 && i !== st.cur) { st.cur = i; const a = parse(st.fx[i].address); if (a && st.mode === 'patch') st.u = a.u; draw(); }
  });
  list.addEventListener('line', e => e.stopPropagation());
  map.addEventListener('slot', e => { e.stopPropagation(); tapSlot(e.detail); });
  // The drop (4d4): the map's one line.
  map.addEventListener('line', e => { e.stopPropagation(); send(e.detail.line); });
  // A tap on a run makes its fixture current (added when it is not in the list); a lasso
  // selects the runs it covers (Shift / Meow adds).
  map.addEventListener('select', e => {
    e.stopPropagation();
    const nos = (e.detail.fixtures || []).map(Number);
    if (!nos.length) return;
    const known = new Map(st.fx.map(x => [Number(x.no), x]));
    const fresh = nos.filter(n => !known.has(n)).map(n => { const r = st.runs.find(x => x.no === n && !x.copy); return r && { no: n, name: r.name, address: `${r.universe}.${a3(r.start)}`, footprint: r.count }; }).filter(Boolean);
    if (nos.length === 1 && !e.detail.add) { if (fresh.length) st.fx = [...st.fx, ...fresh]; }
    else if (e.detail.add) st.fx = [...st.fx, ...fresh];
    else st.fx = [...st.fx.filter(x => nos.includes(Number(x.no))), ...fresh];
    const i = st.fx.findIndex(x => Number(x.no) === nos[0]);
    if (i >= 0) st.cur = i;
    draw();
  });
  map.addEventListener('context', e => { e.stopPropagation(); context?.(e.detail.fixtures, { x: e.detail.x, y: e.detail.y }); });
  box.addEventListener('click', e => {
    const btn = e.target.closest('button');
    if (!btn) return;
    if (btn.dataset.x != null) return close();
    if (btn.dataset.stop != null) { stopAll(); st.mode = 'patch'; return draw(); }
    if (btn.dataset.u) { st.u = Math.max(1, st.u + Number(btn.dataset.u)); return draw(); }
    // A phone's one line: the next or previous fixture of the selection (its universe shown).
    if (btn.dataset.cur) {
      st.cur = Math.max(0, Math.min(st.fx.length - 1, st.cur + Number(btn.dataset.cur)));
      const a = parse(cur()?.address);
      if (a && st.mode === 'patch') st.u = a.u;
      return draw();
    }
    if (btn.dataset.step && st.find) {
      const fp = Number(cur()?.footprint) || 1;
      st.find.s = Math.max(1, Math.min(512 - fp + 1, st.find.s + Number(btn.dataset.step) * fp));
      identify(findLine());
      return draw();
    }
    if (btn.dataset.here != null && st.find && cur()) {
      if (send(`Assign Fixture ${cur().no} At DMX ${st.find.u}.${a3(st.find.s)}`)) { stopAll(); st.mode = 'patch'; advance(); }
      return draw();
    }
    const f = btn.dataset.fn && fn[btn.dataset.fn];
    if (!f) return;
    if (btn.getAttribute('aria-disabled') && !['layout', 'skip'].includes(btn.dataset.fn)) { toast(why()); return; }
    f();
    draw();
  });
  const onEsc = e => { if (e.key === 'Escape' && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, .lc-dlg-scrim, .lc-vd')) { e.preventDefault(); e.stopPropagation(); close(); } };
  window.addEventListener('keydown', onEsc, true);
  const close = () => {
    if (!scrim.isConnected) return;
    // Leaving ends what this dialog made flash (§34 round 3, "Identify, Probe and Find").
    if (st.started && !why()) run('Off Identify Fixture Thru; Off Identify DMX Thru');
    scrim.remove();
    offPhone();
    window.removeEventListener('keydown', onEsc, true);
    current = null;
    closed?.();
  };
  current = close;
  draw();
  return {
    update({ fixtures: fx, universes: us, runs: rs } = {}) {
      if (!scrim.isConnected) return;
      const no = cur()?.no;
      // The list keeps the order it opened with (the feed's rows come sorted by the grid's
      // column — a fixture just patched would jump, and "the next one" with it); each row's
      // facts follow the feed, a fixture the feed lost stays as it was.
      if (fx) {
        const fresh = new Map(fx.map(x => [Number(x.no), x]));
        st.fx = [...st.fx.map(x => fresh.get(Number(x.no)) || x), ...fx.filter(x => !st.fx.some(y => Number(y.no) === Number(x.no)))];
      }
      if (us) st.universes = us;
      if (rs) st.runs = rs;
      const i = st.fx.findIndex(x => x.no === no);
      st.cur = i >= 0 ? i : Math.min(st.cur, Math.max(0, st.fx.length - 1));
      draw();
    },
    close,
  };
}
