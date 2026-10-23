// Insert fixtures… (decisions §34 round 3 4, the approved insert-fixtures.html v6): the usual two
// steps in our kit, every field one part of the one Store line, shown live; nothing happens until
// Create.
//
//   1 Choose — a type, its revision and mode. Two views of the same choice (the left part
//     changes; Mode and the channel map stay put): the No list (the show's fixture types by
//     number — a <luma-grid>: No, Name, Manufacturer, Revision, Modes, Used; it opens here) and
//     Manufacturer · Type · Revision columns, which reach the library's files too (Show /
//     Library / Both). Search covers manufacturer, type, revision and mode names. The channel
//     map (a read-only <luma-grid>) sorts by Structure (the geometry), Channels (DMX order) or
//     Feature group; Details shows the file's own facts. Upload puts a file from this device into
//     library/gdtf/ (Setup and up).
//   2 Place — Name, Quantity, First FID and Address (each with the free values that fit, the
//     value dialog's), Every (n slots apart; the patch's Offset cell fills it: footprint + n),
//     MVR group, Class. Create runs
//     `Store Fixture a Thru b At DMX u.aaa /FixtureType … /Mode n [/Revision n] [/Every n]`,
//     then `Label …`, `Assign … At MvrGroup n` and `Assign … At MvrClass n` — each statement
//     its own Oops step. A block
//     that would overlap or pass 512 is said here and turns red in the grid; Create still runs
//     it (decided: resolve it in the patch, no question). The DMX sheet joins this step with the
//     DMX map element (round 3 part 4d).
//
// On a phone (decisions §37 4) one step a screen: Choose splits in two — Type (the search, Show /
// Library / Both, ⚙ More for the other filters in a sheet — No list / Manufacturer · Type ·
// Revision, Used only, Details — and Upload; the list of types) then Mode (the type's modes with
// their slots, the channel map folded under them, "Channels ▸") — then Place, the form as on the
// desk. Each is full screen with ‹ Back and step dots in a one-line title; a tap on a type goes on
// to Mode, a tap on a mode to Place. The footer is the line on one line (a tap reads it whole)
// and the step's button. The lines are the desk's.
//
//   openInsert({ rows (the patch grid's fixture rows: no, address, footprint), groups ([{ at:
//     'MvrGroup 2', name }]), classes ([{ at: 'MvrClass 1', name }]), offset (spare slots), level,
//     run(line), closed() })  → close()

import './grid.js';
import { freeNumbers, freeAddresses } from './value.js';
import { openPick } from './popup.js';
import { toast } from './toast.js';
import { phoneMode, onPhoneMode } from './phone.js';
import { openSheet } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const a3 = n => String(n).padStart(3, '0');
const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];
const quoted = s => `"${String(s).replace(/[";]/g, '')}"`;
const lower = s => String(s || '').toLowerCase();

const TYPE_COLUMNS = [
  { id: 'no', label: 'No', field: 'no', width: 56, priority: 1, frozen: true, readonly: true },
  { id: 'name', label: 'Name', field: 'name', width: 200, priority: 1, readonly: true },
  { id: 'manufacturer', label: 'Manufacturer', field: 'manufacturer', width: 140, priority: 1, readonly: true },
  { id: 'revision', label: 'Revision', field: 'revision', width: 120, priority: 2, readonly: true },
  { id: 'modes', label: 'Modes', field: 'modes_text', width: 200, priority: 2, readonly: true },
  { id: 'used', label: 'Used', field: 'used', width: 70, priority: 1, readonly: true },
];
const MAP_COLUMNS = [
  { id: 'dmx', label: 'DMX', field: 'dmx', width: 64, priority: 1, frozen: true, readonly: true },
  { id: 'attribute', label: 'Attribute', field: 'attribute', width: 150, priority: 1, readonly: true },
  { id: 'pretty', label: 'Short', field: 'pretty', width: 80, priority: 2, readonly: true },
  { id: 'geometry', label: 'Geometry', field: 'geometry', width: 120, priority: 2, readonly: true },
  { id: 'feature_group', label: 'Feature', field: 'feature_group', width: 100, priority: 2, readonly: true },
];

let current = null;
export function closeInsert() { current?.(); }

export function openInsert({ rows = [], groups = [], classes = [], offset = 0, level = '', group = null, run, closed } = {}) {
  closeInsert();
  const st = {
    step: 1, view: 'list', where: 'show', used: false, details: false, q: '', sort: 'structure',
    data: { show: [], library: [] }, pick: null, map: null,
    man: null, type: null,
    // A phone: the Choose step's screen ('type' | 'mode'), the More sheet, the channel map, the line.
    sub: 'type', more: false, chan: false, lineOpen: false,
    place: { name: '', qty: 1, fid: '', addr: '', every: '', group, cls: null },
  };
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ins-scrim';
  scrim.dataset.kit = 'insert';
  document.body.append(scrim);
  const box = document.createElement('div');
  box.className = 'lc-box lc-ins';
  box.setAttribute('role', 'dialog');
  box.setAttribute('aria-label', 'Insert fixtures');
  scrim.append(box);

  // ── The data ──
  // The library's files come from the drive the Library list's rail picked (decisions §38 5).
  const load = async () => {
    const q = st.drive && st.drive !== 'local-library' ? `?drive=${encodeURIComponent(st.drive)}` : '';
    try { st.data = await (await fetch(`/api/fixture-types${q}`)).json(); } catch (_) { st.data = { show: [], library: [] }; }
    draw();
  };
  const types = () => {
    const show = st.where === 'library' ? [] : st.data.show.map(t => ({ ...t, key: `s:${t.no}`, revision: t.revisions.at(-1)?.name || '' }));
    const lib = st.where === 'show' ? [] : st.data.library.filter(f => !f.error).map(f => ({ ...f, key: `f:${f.file}`, no: '', used: f.in_show ? `type ${f.in_show}` : 'file' }));
    const q = lower(st.q);
    return [...show, ...lib]
      .filter(t => !st.used || (typeof t.used === 'number' && t.used > 0))
      .filter(t => !q || [t.name, t.manufacturer, t.revision, ...(t.modes || []).map(m => m.name)].some(x => lower(x).includes(q)))
      .map(t => ({ ...t, modes_text: (t.modes || []).map(m => `${m.name} ${m.footprint}`).join(' · ') }));
  };
  const chosen = () => {
    if (!st.pick) return null;
    return st.data[st.pick.source === 'show' ? 'show' : 'library'].find(t => (st.pick.source === 'show' ? t.no === st.pick.no : t.file === st.pick.file)) || null;
  };
  const choose = (t, mode) => {
    st.pick = { source: t.source, no: t.no, file: t.file, drive: t.drive || null, revision: 0, mode: mode || st.pick?.mode || 1 };
    if (!(t.modes || [])[st.pick.mode - 1]) st.pick.mode = 1;
    if (!st.place.name) st.place.name = t.name || '';
    loadMap();
  };
  const loadMap = async () => {
    const p = st.pick;
    if (!p) return;
    const q = p.source === 'show' ? `no=${p.no}` : `file=${encodeURIComponent(p.file)}${p.drive && p.drive !== 'local-library' ? `&drive=${encodeURIComponent(p.drive)}` : ''}`;
    try {
      const r = await fetch(`/api/fixture-type?${q}&revision=${p.revision}&mode=${p.mode}`);
      st.map = r.ok ? await r.json() : null;
    } catch (_) { st.map = null; }
    draw();
  };

  // ── The line ──
  const footprint = () => chosen()?.modes?.[st.pick.mode - 1]?.footprint || st.map?.footprint || 1;
  const taken = () => new Set(rows.map(r => Number(r.no)));
  const others = () => rows.map(r => {
    const m = String(r.address || '').match(/^(\d+)\.(\d+)$/);
    return m ? { id: Number(r.no), u: +m[1], s: +m[2], len: Number(r.footprint) || 1 } : null;
  }).filter(Boolean);
  const placeWhy = () => {
    const p = st.place, qty = Number(p.qty), fid = Number(p.fid);
    if (!chosen()) return 'Choose a fixture type first';
    if (!(qty >= 1)) return 'Quantity: how many fixtures';
    if (!(fid >= 1)) return 'First FID: the first new fixture number';
    for (let i = 0; i < qty; i++) if (taken().has(fid + i)) return `Fixture ${fid + i} exists: pick a free First FID`;
    if (p.addr && !/^\d+\.\d+$/.test(p.addr)) return 'Address: universe.address, e.g. 1.150';
    if (p.every && !(Number(p.every) >= 1)) return 'Every: slots from one fixture to the next';
    return '';
  };
  const lines = () => {
    const t = chosen();
    if (!t) return [];
    const p = st.place, qty = Math.max(1, Number(p.qty) || 1), fid = Number(p.fid) || 0;
    const ids = qty > 1 ? `${fid} Thru ${fid + qty - 1}` : `${fid}`;
    const type = t.source === 'show' ? t.no : quoted(t.file);
    const rev = st.pick.revision && st.pick.revision !== (t.revisions || []).length ? ` /Revision ${st.pick.revision}` : '';
    const every = everyOf();
    const drive = t.source !== 'show' && t.drive && t.drive !== 'local-library' ? ` /Drive ${quoted(t.drive)}` : '';
    const out = [`Store Fixture ${ids} At DMX${p.addr ? ` ${p.addr}` : ''} /FixtureType ${type}${drive} /Mode ${st.pick.mode}${rev}${every ? ` /Every ${every}` : ''}`];
    if (p.name.trim()) out.push(`Label Fixture ${ids} ${quoted(p.name.trim())}`);
    if (p.group) out.push(`Assign Fixture ${ids} At ${p.group.at}`);
    if (p.cls) out.push(`Assign Fixture ${ids} At ${p.cls.at}`);
    return out;
  };
  /** Every as typed, else the Offset cell's spare slots after the footprint ('' = packed). */
  const everyOf = () => st.place.every || (offset > 0 ? String(footprint() + offset) : '');
  /** What the block does to the rig: where it lands, overlaps, past 512. */
  const landing = () => {
    const p = st.place, qty = Math.max(1, Number(p.qty) || 1), len = footprint();
    const m = String(p.addr).match(/^(\d+)\.(\d+)$/);
    if (!m) return { text: p.addr ? '' : 'At DMX without an address: the first free run anywhere.', tone: 'dim' };
    const u = +m[1], step = Math.max(len, Number(everyOf()) || 0);
    const end = +m[2] + step * (qty - 1) + len - 1;
    if (end > 512) return { text: `Runs to ${u}.${a3(end)}: past 512 — the line is refused`, tone: 'warn' };
    const hit = new Set();
    for (let i = 0; i < qty; i++) {
      const s = +m[2] + step * i;
      for (const o of others()) if (o.u === u && s <= o.s + o.len - 1 && o.s <= s + len - 1) hit.add(o.id);
    }
    if (hit.size) return { text: `Overlaps fixture ${[...hit].sort((a, b) => a - b).join(', ')} — both turn red and are parked until resolved`, tone: 'warn' };
    return { text: `${u}.${m[2].padStart(3, '0')}–${u}.${a3(end)} · all free`, tone: 'ok' };
  };

  // ── Drawing ──
  let moreSheet = null;
  const offPhone = onPhoneMode(() => draw());
  const close = () => {
    if (!scrim.isConnected) return;
    moreSheet?.close();
    offPhone();
    scrim.remove();
    window.removeEventListener('keydown', onEsc, true);
    current = null;
    closed?.();
  };
  const onEsc = e => { if (e.key === 'Escape' && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, .lc-dlg-scrim')) { e.preventDefault(); e.stopPropagation(); close(); } };
  window.addEventListener('keydown', onEsc, true);
  current = close;

  const head = () => `<header class="lc-box-title lc-ins-title"><luma-icon name="plug"></luma-icon>
      <span class="lc-ins-crumb"><span>Patch</span><i>›</i><b>Insert fixtures</b></span>
      <span class="lc-ins-steps" role="group" aria-label="Steps">
        <button type="button" data-step="1" aria-pressed="${st.step === 1}"><b>1</b> Choose</button>
        <button type="button" data-step="2" aria-pressed="${st.step === 2}"${chosen() ? '' : ' disabled'}><b>2</b> Place</button></span>
      <button type="button" class="lc-close" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>`;
  const seg = (name, value, opts) => `<span class="lc-ins-seg" role="group">${opts.map(([v, l]) => `<button type="button" data-${name}="${v}" aria-pressed="${value === v}">${esc(l)}</button>`).join('')}</span>`;
  const list = (name, items, on) => `<div class="lc-ins-list" role="listbox">${items.map(([v, l, sub]) =>
    `<button type="button" role="option" data-${name}="${esc(v)}" aria-selected="${on === v}"><span>${esc(l)}</span>${sub ? `<small>${esc(sub)}</small>` : ''}</button>`).join('') || '<div class="lc-ins-empty">Nothing here</div>'}</div>`;

  const chooseHtml = () => {
    const all = types();
    const t = chosen();
    const mans = [...new Set(all.map(x => x.manufacturer || '—'))].sort((a, b) => a.localeCompare(b));
    const ofMan = all.filter(x => (x.manufacturer || '—') === st.man);
    const left = st.view === 'list'
      ? `<div class="lc-ins-col grow"><div class="lc-ins-colhead"><h3>${st.where === 'show' ? 'Fixture types in this show' : st.where === 'library' ? 'The library\'s GDTF files' : 'The show and the library'}</h3>${st.where === 'show' ? '<button type="button" class="lc-ins-chip" data-tolib>Library…</button>' : ''}</div>${st.where === 'library' ? '<luma-files class="lc-ins-files"></luma-files>' : '<luma-grid class="lc-ins-types"></luma-grid>'}</div>`
      : `<div class="lc-ins-col"><div class="lc-ins-colhead"><h3>Manufacturer</h3></div>${list('man', mans.map(m => [m, m]), st.man)}</div>
         <div class="lc-ins-col"><div class="lc-ins-colhead"><h3>Type</h3></div>${list('type', ofMan.map(x => [x.key, x.name, x.source === 'show' ? `show ${x.no}` : x.file]), t ? (t.source === 'show' ? `s:${t.no}` : `f:${t.file}`) : null)}</div>
         <div class="lc-ins-col"><div class="lc-ins-colhead"><h3>Revision</h3></div>${t ? list('rev', (t.source === 'show' ? t.revisions.map(r => [String(r.no), r.name || `Revision ${r.no}`]) : [['0', t.revision || 'the file']]), String(st.pick.revision || (t.revisions?.length ?? 0))) : list('rev', [], null)}</div>`;
    const modes = t ? (t.modes || []).map(m => [String(m.no), m.name, `${m.footprint} ch`]) : [];
    const map = st.map?.channels || [];
    return `<div class="lc-ins-search">
        <label class="lc-ins-find"><luma-icon name="search"></luma-icon><input data-q type="search" placeholder="Search name, manufacturer, mode…" value="${esc(st.q)}" aria-label="Search fixture types"></label>
        ${seg('view', st.view, [['list', 'No list'], ['cols', 'Manufacturer · Type · Revision']])}
        ${seg('where', st.where, [['show', 'Show'], ['library', 'Library'], ['both', 'Both']])}
        <button type="button" class="lc-ins-chip" data-used aria-pressed="${st.used}">Used only</button>
        <button type="button" class="lc-ins-chip" data-details aria-pressed="${st.details}">Details</button>
        <label class="lc-ins-chip lc-ins-upload"${LEVELS.indexOf(level) < LEVELS.indexOf('Setup') ? ' aria-disabled="true" title="Uploading needs Setup"' : ''}><luma-icon name="arrow-up-from-line"></luma-icon>Upload<input type="file" accept=".gdtf" data-upload hidden></label>
      </div>
      <div class="lc-ins-chooser">
        <div class="lc-ins-left">${left}</div>
        <div class="lc-ins-col"><div class="lc-ins-colhead"><h3>Mode</h3></div>${list('mode', modes, t ? String(st.pick.mode) : null)}</div>
        <div class="lc-ins-col grow"><div class="lc-ins-colhead"><h3>Channel map${st.map ? ` · ${st.map.footprint} ch` : ''}</h3>
          ${seg('sort', st.sort, [['structure', 'Structure'], ['channels', 'Channels'], ['feature', 'Feature group']])}</div>
          ${map.length || !t ? '<luma-grid class="lc-ins-map"></luma-grid>' : '<div class="lc-ins-empty">Reading the mode…</div>'}</div>
      </div>
      ${st.details && st.map ? detailsHtml(st.map) : ''}`;
  };
  const detailsHtml = m => {
    const d = m.details || {};
    const facts = [['Manufacturer', m.manufacturer], ['Long name', d.long_name], ['Short name', d.short_name], ['Description', d.description],
      ['FixtureTypeID', d.fixture_type_id], ['GDTF version', m.gdtf_version], ['Revision', m.revision],
      ...(d.revisions || []).slice(-3).reverse().map(r => ['File revision', [r.text, r.date, r.user_id ? `user ${r.user_id}` : ''].filter(Boolean).join(' · ')])];
    const p = st.pick;
    const src = `/api/fixture-type/thumbnail?${p.source === 'show' ? `no=${p.no}&revision=${p.revision}` : `file=${encodeURIComponent(p.file)}`}`;
    return `<div class="lc-ins-details lc-selectable"><img class="lc-ins-thumb" src="${esc(src)}" alt="" onerror="this.remove()">${facts.filter(([, v]) => v).map(([k, v]) => `<div><small>${esc(k)}</small><span>${esc(v)}</span></div>`).join('')}</div>`;
  };
  const placeHtml = () => {
    const t = chosen(), p = st.place, mode = t?.modes?.[st.pick.mode - 1];
    const qty = Math.max(1, Number(p.qty) || 1);
    const fids = freeNumbers(taken(), qty, Math.max(0, ...taken()) + 1);
    const addrs = freeAddresses(others(), Math.max(footprint(), Number(everyOf()) || 0) * (qty - 1) + footprint());
    const land = landing();
    const field = (k, label, value, attrs = '') => `<label class="lc-ins-k" for="ins-${k}">${esc(label)}</label><input class="lc-ins-v" id="ins-${k}" data-f="${k}" value="${esc(value)}" ${attrs}>`;
    const chips = (k, list) => `<span></span><div class="lc-ins-free">${list.length ? `<small>Free:</small>${list.map(([v, l]) => `<button type="button" class="lc-ins-chip" data-free="${k}" data-v="${esc(v)}" aria-pressed="${String(p[k]) === v}">${esc(l)}</button>`).join('')}` : ''}</div>`;
    return `<div class="lc-ins-place">
      <div class="lc-ins-fields">
        <span class="lc-ins-k">Type</span><span class="lc-ins-ro">${esc(t?.name || '')} <small>${esc(t?.manufacturer || '')}</small></span>
        <span class="lc-ins-k">Mode</span><span class="lc-ins-ro">${esc(mode?.name || '')} <small>${esc(mode ? `${mode.footprint} ch` : '')}</small></span>
        ${field('name', 'Name', p.name, 'autocomplete="off"')}
        ${field('qty', 'Quantity', p.qty, 'inputmode="numeric"')}
        ${field('fid', 'First FID', p.fid, 'inputmode="numeric" placeholder="the first new number"')}
        ${chips('fid', fids)}
        ${field('addr', 'Address', p.addr, 'inputmode="decimal" placeholder="u.aaa — empty: the first free run"')}
        ${chips('addr', addrs)}
        ${field('every', 'Every', p.every, `inputmode="numeric" placeholder="${offset ? `${footprint() + offset} (Offset ${offset})` : `packed (${footprint()} slots)`}"`)}
        <span class="lc-ins-k">MVR group</span><button type="button" class="lc-ins-v lc-ins-pickgroup" data-group>${esc(p.group ? `${p.group.at} · ${p.group.name}` : 'None')}</button>
        <span class="lc-ins-k">Class</span>${classes.length
          ? `<button type="button" class="lc-ins-v lc-ins-pickgroup" data-cls>${esc(p.cls ? `${p.cls.at} · ${p.cls.name || 'no name'}` : 'None')}</button>`
          : '<span class="lc-ins-ro"><small>No MVR classes in the show (they come with an MVR import)</small></span>'}
      </div>
      <div class="lc-ins-land ${land.tone}">${esc(land.text)}</div>
      <p class="lc-ins-note">The DMX sheet joins this step with the DMX map (part 4d).</p>
    </div>`;
  };
  // ── A phone: one step a screen ──
  const phoneScreen = () => (st.step === 2 ? 'place' : st.sub);
  const phoneHead = () => {
    const at = { type: 0, mode: 1, place: 2 }[phoneScreen()];
    const name = ['Type', 'Mode', 'Place'][at];
    return `<header class="lc-box-title lc-ins-title phone">${at ? '<button type="button" class="lc-ins-back" data-back aria-label="Back"><luma-icon name="chevron-left"></luma-icon>Back</button>' : '<luma-icon name="plug"></luma-icon>'}
      <span class="lc-ins-ptitle"><b>Insert fixtures</b><span class="lc-ins-dots" aria-hidden="true">${[0, 1, 2].map(i => `<i class="${i <= at ? 'on' : ''}"></i>`).join('')}</span><span>${name}</span></span>
      <button type="button" class="lc-close" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>`;
  };
  const phoneType = () => {
    const all = types();
    const t = chosen();
    const on = t ? (t.source === 'show' ? `s:${t.no}` : `f:${t.file}`) : null;
    // The No list by number; Manufacturer · Type · Revision groups the types by manufacturer.
    const sorted = st.view === 'cols' ? [...all].sort((a, b) => String(a.manufacturer || '').localeCompare(String(b.manufacturer || '')) || String(a.name).localeCompare(String(b.name))) : all;
    let lastMan = null;
    const items = sorted.map(x => {
      const head = st.view === 'cols' && (x.manufacturer || '—') !== lastMan ? `<div class="lc-ins-group">${esc((lastMan = x.manufacturer || '—'))}</div>` : '';
      return `${head}<button type="button" role="option" data-ptype="${esc(x.key)}" aria-selected="${on === x.key}"><span>${x.no !== '' ? `<b>${esc(x.no)}</b> ` : ''}${esc(x.name)}</span><small>${esc([st.view === 'cols' ? '' : x.manufacturer, x.source === 'show' ? `${(x.modes || []).length} modes · used ${x.used}` : x.file].filter(Boolean).join(' · '))}</small></button>`;
    }).join('');
    return `<div class="lc-ins-search phone">
        <label class="lc-ins-find"><luma-icon name="search"></luma-icon><input data-q type="search" placeholder="Search name, manufacturer, mode…" value="${esc(st.q)}" aria-label="Search fixture types"></label>
        <div class="lc-ins-chips">${seg('where', st.where, [['show', 'Show'], ['library', 'Library'], ['both', 'Both']])}
          <button type="button" class="lc-ins-chip" data-pmore aria-pressed="${st.view === 'cols' || st.used || st.details}"><luma-icon name="sliders-horizontal"></luma-icon>More</button>
          <label class="lc-ins-chip lc-ins-upload"${LEVELS.indexOf(level) < LEVELS.indexOf('Setup') ? ' aria-disabled="true" title="Uploading needs Setup"' : ''}><luma-icon name="arrow-up-from-line"></luma-icon>Upload<input type="file" accept=".gdtf" data-upload hidden></label></div>
      </div>
      <div class="lc-ins-list phone" role="listbox">${items || '<div class="lc-ins-empty">Nothing here</div>'}</div>`;
  };
  const phoneMode_ = () => {
    const t = chosen();
    if (!t) return '<div class="lc-ins-empty">Pick a fixture type first.</div>';
    const revs = t.source === 'show' && (t.revisions || []).length > 1 ? `<div class="lc-ins-colhead"><h3>Revision</h3>${seg('rev', String(st.pick.revision || t.revisions.length), t.revisions.map(r => [String(r.no), r.name || `Revision ${r.no}`]))}</div>` : '';
    const map = st.map?.channels || [];
    return `<div class="lc-ins-pscroll">
        <div class="lc-ins-picked"><b>${esc(t.name)}</b><small>${esc([t.manufacturer, t.source === 'show' ? `show ${t.no}` : t.file].filter(Boolean).join(' · '))}</small></div>
        ${revs}
        <div class="lc-ins-colhead"><h3>Mode</h3></div>
        <div class="lc-ins-list phone" role="listbox">${(t.modes || []).map(m => `<button type="button" role="option" data-pmode="${m.no}" aria-selected="${st.pick.mode === m.no}"><span>${esc(m.name)}</span><small>${esc(`${m.footprint} ch`)}</small></button>`).join('')}</div>
        <button type="button" class="lc-ins-fold" data-chan aria-expanded="${st.chan}"><luma-icon name="${st.chan ? 'chevron-down' : 'chevron-right'}"></luma-icon>Channels${st.map ? ` · ${st.map.footprint} ch` : ''}</button>
        ${st.chan ? `<div class="lc-ins-pmap">${seg('sort', st.sort, [['structure', 'Structure'], ['channels', 'Channels'], ['feature', 'Feature group']])}${map.length ? '<luma-grid class="lc-ins-map"></luma-grid>' : '<div class="lc-ins-empty">Reading the mode…</div>'}</div>` : ''}
        ${st.details && st.map ? detailsHtml(st.map) : ''}
      </div>`;
  };
  /** ⚙ More: the Type screen's other filters, in the kit's sheet. */
  const drawMore = () => {
    const want = st.more;
    moreSheet?.close();
    moreSheet = null;
    st.more = want;
    if (!st.more || !phoneMode() || phoneScreen() !== 'type') return;
    const sh = openSheet({
      title: 'Filters', side: phoneMode() === 'side', host: box, cls: 'lc-ins-moresheet',
      body: `<div class="lc-psheet-sec">View</div>${seg('view', st.view, [['list', 'No list'], ['cols', 'Manufacturer · Type · Revision']])}
        <div class="lc-ins-chips"><button type="button" class="lc-ins-chip" data-used aria-pressed="${st.used}">Used only</button>
        <button type="button" class="lc-ins-chip" data-details aria-pressed="${st.details}">Details</button></div>`,
      onClose: () => { if (moreSheet === sh) { moreSheet = null; st.more = false; } },
    });
    moreSheet = sh;
  };
  const drawPhone = () => {
    const screen = phoneScreen();
    const why = screen === 'place' ? placeWhy() : '';
    const ls = screen === 'place' ? lines() : [];
    const t = chosen();
    const line = screen === 'place' ? (why ? `<span class="warn">${esc(why)}</span>` : ls.map(l => `<code>${esc(l)}</code>`).join(''))
      : esc(t ? `${t.name} · ${t.modes?.[st.pick.mode - 1]?.name || ''}` : 'Pick a fixture type');
    const button = screen === 'type' ? `<button type="button" class="lc-btn primary" data-tomode${t ? '' : ' disabled'}>Next · Mode</button>`
      : screen === 'mode' ? `<button type="button" class="lc-btn primary" data-next${t ? '' : ' disabled'}>Next · Place</button>`
        : `<button type="button" class="lc-btn primary" data-create${why ? ' disabled' : ''}>Create</button>`;
    box.innerHTML = `${phoneHead()}<div class="lc-box-body lc-ins-body">${screen === 'type' ? phoneType() : screen === 'mode' ? phoneMode_() : placeHtml()}</div>
      <footer class="lc-ins-foot phone"><button type="button" class="lc-ins-line lc-selectable${st.lineOpen ? ' open' : ''}" data-lineopen title="The line">${line}</button>${button}</footer>`;
    const mg = box.querySelector('.lc-ins-map');
    if (mg) {
      mg.update({ key: 'insert.map', id: 'dmx', row_key: 'key', columns: MAP_COLUMNS, rows: mapRows(), state: {}, level, caption: '', bar: false });
      mg.addEventListener('line', e => e.stopPropagation());
    }
    drawMore();
  };

  const draw = () => {
    const phone = phoneMode();
    box.classList.toggle('phone', !!phone);
    scrim.classList.toggle('phone', !!phone);
    if (phone) {
      const focus = document.activeElement?.dataset?.f || (document.activeElement?.dataset?.q != null ? 'q' : null);
      drawPhone();
      if (focus) {
        const el = focus === 'q' ? box.querySelector('[data-q]') : box.querySelector(`[data-f="${focus}"]`);
        if (el) { el.focus(); el.setSelectionRange?.(el.value.length, el.value.length); }
      }
      return;
    }
    moreSheet?.close();
    const why = st.step === 2 ? placeWhy() : '';
    const ls = st.step === 2 ? lines() : [];
    const focus = document.activeElement?.dataset?.f || (document.activeElement?.dataset?.q != null ? 'q' : null);
    box.innerHTML = `${head()}<div class="lc-box-body lc-ins-body">${st.step === 1 ? chooseHtml() : placeHtml()}</div>
      <footer class="lc-ins-foot"><div class="lc-ins-line lc-selectable">${st.step === 1
        ? esc(chosen() ? `${chosen().name} · ${chosen().modes?.[st.pick.mode - 1]?.name || ''}` : 'Pick a fixture type and a mode')
        : why ? `<span class="warn">${esc(why)}</span>` : ls.map(l => `<code>${esc(l)}</code>`).join('')}</div>
        ${st.step === 1 ? `<button type="button" class="lc-btn primary" data-next${chosen() ? '' : ' disabled'}>Next · Place</button>`
          : `<button type="button" class="lc-btn" data-step="1">‹ Choose</button><button type="button" class="lc-btn primary" data-create${why ? ' disabled' : ''}>Create</button>`}</footer>`;
    if (st.step === 1) {
      // Library: the one files element (decisions §38 5) — the drives rail, the gdtf folder's
      // files with their facts; a row picked is the type, the rail's drive reloads the list.
      const lf = box.querySelector('.lc-ins-files');
      if (lf) {
        lf.update({ feeds: window.feeds?.() || null, kind: 'gdtf', drive: st.drive || 'local-library', folder: st.libFolder || null, level });
        const t = chosen();
        if (t && t.source !== 'show') lf.grid()?.select?.([t.file.split('/').pop()]);
        lf.addEventListener('where', e => { e.stopPropagation(); const w = e.detail; const changed = w.drive !== (st.drive || 'local-library'); st.drive = w.drive; st.libFolder = w.folder; if (changed) load(); });
        lf.addEventListener('select', e => {
          e.stopPropagation();
          const d = e.detail, row = d.rows?.length === 1 ? d.rows[0] : null;
          if (!row || row.dir) return;
          const rel = d.folder && d.folder !== 'gdtf' && d.folder.startsWith('gdtf/') ? `${d.folder.slice(5)}/${row.file}` : row.file;
          const found = types().find(x => x.source === 'library' && x.file === rel);
          if (found && (!chosen() || chosen().source === 'show' || chosen().file !== rel)) choose(found);
        });
        lf.addEventListener('line', e => e.stopPropagation());
      }
      const tg = box.querySelector('.lc-ins-types');
      if (tg) {
        tg.update({ key: 'insert.types', id: 'key', row_key: 'key', columns: TYPE_COLUMNS, rows: types(), state: {}, level, caption: '', bar: false });
        const t = chosen();
        if (t) tg.select?.([t.source === 'show' ? `s:${t.no}` : `f:${t.file}`]);
        tg.addEventListener('select', e => {
          const k = e.detail?.rows?.[0];
          const row = types().find(x => x.key === k);
          if (row && (!chosen() || row.key !== (chosen().source === 'show' ? `s:${chosen().no}` : `f:${chosen().file}`))) choose(row);
        });
        tg.addEventListener('line', e => e.stopPropagation());
      }
      const mg = box.querySelector('.lc-ins-map');
      if (mg) {
        mg.update({ key: 'insert.map', id: 'dmx', row_key: 'key', columns: MAP_COLUMNS, rows: mapRows(), state: {}, level, caption: '', bar: false });
        mg.addEventListener('line', e => e.stopPropagation());
      }
    }
    if (focus) {
      const el = focus === 'q' ? box.querySelector('[data-q]') : box.querySelector(`[data-f="${focus}"]`);
      if (el) { el.focus(); el.setSelectionRange?.(el.value.length, el.value.length); }
    }
  };
  const mapRows = () => {
    const ch = (st.map?.channels || []).map((c, i) => ({ ...c, key: String(i), dmx: c.virtual ? 'virtual' : `${c.break > 1 ? `${c.break}/` : ''}${c.offset + 1}`, feature_group: c.feature_group || '—' }));
    const by = { channels: (a, b) => a.break - b.break || a.offset - b.offset, feature: (a, b) => a.feature_group.localeCompare(b.feature_group) || a.offset - b.offset };
    if (st.sort === 'structure') {
      const order = [...new Set(ch.map(c => c.geometry))];
      return ch.sort((a, b) => order.indexOf(a.geometry) - order.indexOf(b.geometry) || a.offset - b.offset);
    }
    return ch.sort(by[st.sort]);
  };

  // ── Gestures: each a part of the line ──
  box.addEventListener('input', e => {
    if (e.target.dataset.q != null) { st.q = e.target.value; draw(); return; }
    const k = e.target.dataset.f;
    if (k) { st.place[k] = e.target.value.trim(); draw(); }
  });
  box.addEventListener('change', async e => {
    const input = e.target.closest('[data-upload]');
    if (!input?.files?.[0]) return;
    const file = input.files[0];
    const r = await fetch(`/api/gdtf?name=${encodeURIComponent(file.name)}`, { method: 'POST', body: file }).then(x => x.json()).catch(() => ({ ok: false, message: 'The upload failed' }));
    if (!r.ok) { toast(r.message || 'The upload failed'); return; }
    toast(`${r.name} is in library/gdtf/`);
    st.where = st.where === 'show' ? 'both' : st.where;
    await load();
    const t = st.data.library.find(f => f.file === r.name);
    if (t && !t.error) choose({ ...t, source: 'library' });
  });
  box.addEventListener('click', async e => {
    const b = e.target.closest('button, label');
    if (!b || b.disabled || b.getAttribute('aria-disabled') === 'true') return;
    const d = b.dataset;
    if (d.x != null) return close();
    // A phone's screens: Back, Type → Mode → Place, the More sheet, the channel map, the line.
    if (d.back != null) { if (st.step === 2) { st.step = 1; st.sub = 'mode'; } else st.sub = 'type'; return draw(); }
    if (d.ptype) { const t = types().find(x => x.key === d.ptype); if (t) { choose(t); st.sub = 'mode'; draw(); } return; }
    if (d.tomode != null) { st.sub = 'mode'; return draw(); }
    if (d.pmode) {
      if (!st.pick) return;
      st.pick.mode = +d.pmode;
      loadMap();
      st.step = 2;
      if (!st.place.fid) st.place.fid = freeNumbers(taken(), Math.max(1, Number(st.place.qty) || 1), Math.max(0, ...taken()) + 1)[0]?.[0] || '';
      return draw();
    }
    if (d.pmore != null) { st.more = true; return draw(); }
    if (d.chan != null) { st.chan = !st.chan; return draw(); }
    if (d.lineopen != null) { st.lineOpen = !st.lineOpen; return draw(); }
    if (d.step) { st.step = +d.step; return draw(); }
    if (d.next != null) {
      st.step = 2;
      if (!st.place.fid) st.place.fid = freeNumbers(taken(), Math.max(1, Number(st.place.qty) || 1), Math.max(0, ...taken()) + 1)[0]?.[0] || '';
      return draw();
    }
    if (d.view) { st.view = d.view; if (d.view === 'cols' && st.where === 'show') st.where = 'both'; return draw(); }
    if (d.where) { st.where = d.where; return draw(); }
    if (d.used != null) { st.used = !st.used; return draw(); }
    if (d.details != null) { st.details = !st.details; return draw(); }
    if (d.tolib != null) { st.view = 'cols'; st.where = 'library'; return draw(); }
    if (d.sort) { st.sort = d.sort; return draw(); }
    if (d.man) { st.man = d.man; return draw(); }
    if (d.type) { const t = types().find(x => x.key === d.type); if (t) choose(t); return; }
    if (d.rev) { if (st.pick) { st.pick.revision = +d.rev; loadMap(); } return; }
    if (d.mode) { if (st.pick) { st.pick.mode = +d.mode; loadMap(); } return; }
    if (d.free) { st.place[d.free] = d.v; return draw(); }
    if (d.group != null) {
      const items = [{ no: 1, name: 'None' }, ...groups.map((g, i) => ({ no: i + 2, name: `${g.at} · ${g.name || 'no name'}` }))];
      const got = await openPick({ title: 'MVR group', pick: { kind: 'MvrGroup', items, plain: true } });
      if (got?.no) { st.place.group = got.no === 1 ? null : groups[got.no - 2]; draw(); }
      return;
    }
    if (d.cls != null) {
      const items = [{ no: 1, name: 'None' }, ...classes.map((c, i) => ({ no: i + 2, name: `${c.at} · ${c.name || 'no name'}` }))];
      const got = await openPick({ title: 'MVR class', pick: { kind: 'MvrClass', items, plain: true } });
      if (got?.no) { st.place.cls = got.no === 1 ? null : classes[got.no - 2]; draw(); }
      return;
    }
    if (d.create != null) {
      if (placeWhy()) return;
      const all = lines();
      close();
      run(all.join('; '));
    }
  });
  draw();
  load();
  return close;
}
