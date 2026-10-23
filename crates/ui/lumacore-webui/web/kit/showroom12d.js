// The Showroom's 12d part (decisions §30 "12d hardening"): <luma-grid> with
// every gesture, the edit card, <luma-slab>, the power toggle's looks, the
// settings cells' kind badges and home marks, and <luma-setup> — the locked
// boards' own examples (SetupDmx, SetupNetwork, SetupSettings, PowerToggle).
// Nothing here runs on the station: a small stand-in applies each line to
// the demo rows, and the line shows under the element (› …).

import './grid.js';
import './slab.js';
import { openSetup } from './setup.js';
import { openEditCard } from './editcard.js';
import { toast } from './toast.js';

const LINE_COLS = [
  { id: 'name', label: 'Name', field: 'name', prop: 'Name', kind: 'text', width: 150, priority: 1, frozen: true, per_row: true, section: 'General', rights: 'Setup' },
  { id: 'direction', label: 'Direction', field: 'direction', prop: null, kind: 'text', width: 100, priority: 2 },
  { id: 'protocol', label: 'Protocol', field: 'protocol', prop: 'Protocol', kind: 'list', options: ['sACN', 'Art-Net'], width: 130, priority: 1, section: 'General', rights: 'Setup' },
  { id: 'universes', label: 'Universes', field: 'universes', prop: null, kind: 'text', width: 110, priority: 2 },
  { id: 'rate', label: 'Rate', field: 'rate', prop: 'Rate', kind: 'int', unit: 'Hz', min: 1, max: 60, width: 100, priority: 1, section: 'Timing', rights: 'Setup' },
  { id: 'send', label: 'Send', field: 'send', prop: 'Send', kind: 'list', options: ['Full', 'Changes'], width: 110, priority: 2, section: 'Timing', rights: 'Setup' },
  { id: 'sync', label: 'Sync', field: 'sync', prop: 'Sync', kind: 'bool', options: ['Yes', 'No'], width: 90, priority: 3, section: 'Timing', rights: 'Setup' },
  { id: 'gap', label: 'Packet gap', field: 'gap', prop: 'PacketGap', kind: 'int', unit: 'µs', min: 0, max: 10000, width: 120, priority: 3, section: 'Timing', rights: 'Setup' },
  { id: 'state', label: 'State', field: 'state', prop: null, kind: 'text', tone: 'tone', width: 190, priority: 1 },
];
const demoLines = () => [
  { guid: 'o1', no: 1, object: 'Sacn', name: 'Sacn 1', direction: 'out', protocol: 'sACN', universes: '1–4', rate: 30, send: 'Full', sync: true, gap: 0, state: 'sending', tone: 'ok' },
  { guid: 'o2', no: 2, object: 'Sacn', name: 'Sacn 2', direction: 'out', protocol: 'sACN', universes: '5–6', rate: 44, send: 'Full', sync: false, gap: 0, state: 'sending', tone: 'ok' },
  { guid: 'o3', no: 3, object: 'Sacn', name: 'Sacn 3', direction: 'out', protocol: 'sACN', universes: '7', rate: 30, send: 'Changes', sync: true, gap: 500, state: 'sending', tone: 'ok' },
  { guid: 'o4', no: 4, object: 'Sacn', name: 'Sacn 4', direction: 'out', protocol: 'sACN', universes: '8', rate: 25, send: 'Full', sync: false, gap: 0, state: 'no universe routed', tone: 'alert' },
  { guid: 'i1', no: 5, object: 'Sacn', name: 'Sacn 5', direction: 'in', protocol: 'sACN', universes: '20', state: 'receiving', tone: 'warn' },
];
const NIC_COLS = [
  { id: 'name', label: 'Interface', field: 'name', prop: null, kind: 'text', width: 140, priority: 1, frozen: true },
  { id: 'address', label: 'Address', field: 'address', prop: null, kind: 'text', width: 160, priority: 2 },
  { id: 'link', label: 'Link', field: 'link', prop: null, kind: 'text', tone: 'link_tone', width: 140, priority: 2 },
  { id: 'lumasession', label: 'LumaSession', field: 'lumasession', prop: 'LumaSession', kind: 'bool', options: ['Yes', 'No'], width: 130, priority: 1, section: 'Services', rights: 'Setup', home: 'station' },
  { id: 'shell', label: 'Shell', field: 'shell', prop: 'Shell', kind: 'bool', options: ['Yes', 'No'], width: 110, priority: 1, section: 'Services', rights: 'Setup', home: 'station' },
  { id: 'http', label: 'HTTP', field: 'http', prop: 'HTTP', kind: 'bool', options: ['Yes', 'No'], width: 110, priority: 1, section: 'Services', rights: 'Setup', home: 'station' },
];
const demoNics = () => [
  { name: 'en0', no: 1, object: 'Interface', address: '192.168.1.40/24', link: 'up · 1 Gb/s', link_tone: 'ok', lumasession: true, shell: true, http: true },
  { name: 'en1', no: 2, object: 'Interface', address: '10.0.0.40/16', link: 'up · 1 Gb/s', link_tone: 'ok', lumasession: true, shell: true, http: false },
  { name: 'en2', no: 3, object: 'Interface', address: '10.10.0.40/16', link: 'up · 100 Mb/s', link_tone: 'ok', lumasession: false, shell: true, http: true },
  { name: 'en5', no: 4, object: 'Interface', address: '—', link: 'no link', link_tone: 'warn', lumasession: false, shell: false, http: false },
];
const SLAB_EDIT = {
  select: { line: null }, new: { line: 'Store {object}', rights: 'Setup' }, copy: { line: 'Copy {object} {rows} At {at}', rights: 'Setup' },
  cut: { line: 'Move {object} {rows} At {at}', rights: 'Setup' }, paste: { line: null, rights: 'Setup' }, delete: { line: 'Delete {object} {rows}', rights: 'Setup' },
  oops: { line: 'Oops' }, import: { line: 'Import {object}', rights: 'Setup' }, export: { line: 'Export {object} {rows}' }, edit: { line: null, rights: 'Setup' },
};
const HW_EDIT = { ...SLAB_EDIT, new: { line: null, off: 'Interfaces come from the hardware' }, cut: { line: null, off: 'Interfaces come from the hardware' },
  paste: { line: null, off: 'Interfaces come from the hardware' }, delete: { line: null, off: 'Interfaces come from the hardware' },
  copy: { line: null, off: 'Copy is planned for interfaces' }, import: { line: null, off: 'Import is planned (part 5)' }, export: { line: null, off: 'Export is planned (part 5)' } };
const DMX_DEF = {
  name: 'DMXProtocols', label: 'DMX Protocols', icon: 'radio-tower', kind: 'setup', home: 'Lines: in the show · NIC map: this station',
  power: { label: 'DMX output', on: { word: 'SENDING', press: 'hold output (asks)', line: 'Set Station "HoldOutput" On' }, off: { word: 'OUTPUT HELD', press: 'send again', line: 'Set Station HoldOutput Off' }, failing: { word: 'SENDING', press: '' }, rights: 'Setup' },
  tabs: [
    { name: 'Lines', label: 'Main', sub: 'in/out · 4 out · 1 in', home: 'show', phone: [],
      content: { kind: 'grid', grid: { key: 'showroom.lines', object: 'Sacn', id: 'no', row_key: 'guid', new_line: 'New line…', columns: LINE_COLS } },
      slab: SLAB_EDIT,
      actions: [{ id: 'assign', label: 'Assign…', line: 'Assign Universe 1 At {object} {rows}', rights: 'Setup', needs: 'one' }, { id: 'info', label: 'Info', line: 'Info {object} {rows}', needs: 'one' },
        { id: 'fill', label: 'Fill series', line: 'Set {object} {rows} Rate 21 Thru 30', rights: 'Setup', needs: 'selection' }, { id: 'tab', label: 'Settings tab', line: 'Menu "DMXProtocols" /Tab Routing' }],
      bound: [{ object: 'Sacn', prop: 'Rate', label: 'Rate' }, { object: 'Sacn', prop: 'Send', label: 'Send' }, { object: 'Sacn', prop: 'Sync', label: 'Sync' }, { object: 'Sacn', prop: 'PacketGap', label: 'Packet gap' }] },
    { name: 'Interfaces', label: 'Video', sub: 'not mapped here', home: 'station',
      content: { kind: 'grid', grid: { key: 'showroom.nics', object: 'Interface', id: 'name', row_key: 'name', hardware: 'Interfaces come from the hardware', columns: NIC_COLS } },
      slab: HW_EDIT, actions: [{ id: 'all', label: 'All services', line: 'Set Interface {rows} "Shell" Yes', rights: 'Setup', needs: 'selection' }] },
    { name: 'Discovery', label: 'Discovery', sub: 'part 4', content: { kind: 'empty', note: 'Discovery is drawn in part 4 (Art-Net nodes, sACN sources).' }, slab: SLAB_EDIT },
  ],
};

/** The stand-in station: `Set <Object> <rows> <Prop> <value>` on demo rows. */
export function applyDemo(rows, cols, line) {
  const out = rows.map(r => ({ ...r }));
  for (const st of line.split(/\s*;\s*/)) {
    const t = st.match(/"[^"]*"|\S+/g) || [];
    if (t[0] !== 'Set' || t.length < 5) continue;
    const obj = t[1];
    let i = 2;
    const ids = [];
    const take = () => ids.push(t[i++].replace(/"/g, ''));
    take();
    while (t[i] === '+' || /^thru$/i.test(t[i] || '')) {
      if (/^thru$/i.test(t[i])) { const a = Number(ids[ids.length - 1]); const b = Number(t[i + 1]); for (let n = a + 1; n <= b; n++) ids.push(String(n)); i += 2; } else { i++; take(); }
    }
    const prop = t[i++]?.replace(/"/g, '');
    const raw = t.slice(i).join(' ').replace(/^"|"$/g, '');
    const col = cols.find(c => c.prop?.toLowerCase() === prop?.toLowerCase());
    if (!col) continue;
    const hit = out.filter(r => (r.object || obj) === obj && ids.includes(String(r.no)) || (r.object === obj && ids.includes(String(r.name))));
    const spread = /^(-?\d+)\s+thru\s+(-?\d+)$/i.exec(raw);
    hit.forEach((r, k) => {
      const key = col.field ?? col.id;
      if (r[key] === undefined) return;
      if (col.kind === 'bool') r[key] = /^(yes|on|true)$/i.test(raw);
      else if (col.kind === 'int') {
        const now = Number(r[key]) || 0;
        const rel = /^([+\-*/])\s+(\d+(\.\d+)?)$/.exec(raw);
        let v = spread ? Math.round(Number(spread[1]) + (hit.length > 1 ? k * (Number(spread[2]) - Number(spread[1])) / (hit.length - 1) : 0))
          : rel ? Math.round({ '+': now + +rel[2], '-': now - +rel[2], '*': now * +rel[2], '/': now / +rel[2] }[rel[1]]) : Number(raw);
        if (!Number.isFinite(v)) return;
        if (col.min != null) v = Math.max(col.min, v);
        if (col.max != null) v = Math.min(col.max, v);
        r[key] = v;
      } else r[key] = raw;
    });
  }
  return out;
}

export function build12d(root, { section, div, btn }) {
  // ── Settings cells: kinds, home marks, rights ──
  let level = 'Admin';
  const fields = () => [
    { name: 'Station name', section: 'Station', input: { type: 'text' }, value: 'FOH', line: 'Set Station "Name" {value}', home: 'station', rights: 'Setup' },
    { name: 'Location', section: 'Station', input: { type: 'text' }, value: 'Front of house', line: 'Set Station "Location" {value}', home: 'station', rights: 'Setup' },
    { name: 'Priority', section: 'Station', input: { type: 'choice', options: ['Never', 'VeryLow', 'Low', 'Normal', 'High'] }, value: 'Normal', line: 'Set Station "Priority" {value}', home: 'station', rights: 'Setup' },
    { name: 'Recovery cap', section: 'Station', input: { type: 'number', min: 50, max: 5000 }, unit: 'MB', value: '500', line: 'Set Station "RecoveryCap" {value}', home: 'station', rights: 'Setup' },
    { name: 'Online', section: 'Station', input: { type: 'toggle' }, value: 'On', line: 'Set Station "Online" {value}', home: 'station', rights: 'Setup' },
    { name: 'Build', section: 'Station', input: { type: 'text' }, value: 'e847dff', line: '', readonly: 'The build is read-only' },
    { name: 'Autosave', section: 'Show', input: { type: 'number', min: 0, max: 1440, named: ['Off'] }, unit: 'min', value: '5', line: 'Set Show "Autosave" {value}', home: 'show', rights: 'Setup' },
    { name: 'Description', section: 'Show', input: { type: 'text' }, value: 'Festival 2026', line: 'Set Show "Description" {value}', home: 'show', rights: 'Program' },
    { name: 'Suggestions', section: 'Show', input: { type: 'list', options: ['Auto', 'TabOnly', 'Off'] }, value: 'Auto', line: 'Set UserProfile "Suggestions" {value}', home: 'user', rights: 'Guest' },
  ];
  let values = fields();
  const kinds = document.createElement('luma-settings');
  kinds.setAttribute('embedded', '');
  const kindsNote = div('lc-sr-note lc-selectable', 'tap a cell');
  const drawKinds = () => kinds.update({ title: 'Kinds', fields: values, level, note: 'The home icon (§39): a box = show file, every station; a person = your profile, in the show; a server = this station only, never in the show. Gold where a cell differs from its tab (here: Show).', home: 'show' });
  kinds.addEventListener('run', e => {
    const line = e.detail.line;
    kindsNote.textContent = `› ${line}`;
    const f = values.find(x => x.line && line.startsWith(x.line.replace('{value}', '').trim()));
    if (!f) return;
    let v = line.slice(f.line.replace('{value}', '').trim().length).trim().replace(/^"|"$/g, '');
    const rel = /^([+-]) 1$/.exec(v);
    if (rel) v = String(Math.max(f.input.min ?? 0, Math.min(f.input.max ?? 1e9, (Number(f.value) || 0) + (rel[1] === '+' ? 1 : -1))));
    f.value = v;
    drawKinds();
  });
  const lv = div('lc-sr-row');
  for (const l of ['Admin', 'Setup', 'Program', 'Guest']) lv.append(btn(`as ${l}`, () => { level = l; drawKinds(); kindsNote.textContent = `› viewing as ${l}`; }));
  const kindsBox = div('lc-sr-wide');
  kindsBox.append(lv, kinds, kindsNote);
  root.append(section('Settings cells show their kind — Aa text · 123 number (− / +, keypad) · I/O switch · ≡ list; home marks; greyed below your level', kindsBox));
  drawKinds();

  // ── luma-grid ──
  let rows = demoLines();
  const gridNote = div('lc-sr-note lc-selectable', 'every gesture ends as one line');
  const grid = document.createElement('luma-grid');
  grid.id = 'sr-grid';
  const gstate = { columns: [], widths: {}, sort: null, set: 'Full', new_line: true };
  const spec = () => ({ key: 'showroom.lines', object: 'Sacn', id: 'no', row_key: 'guid', columns: LINE_COLS, rows, new_line: 'New line…', state: gstate, level: 'Admin', caption: 'Lines' });
  grid.addEventListener('line', e => {
    const line = e.detail.line;
    gridNote.textContent = `› ${line}`;
    if (line.startsWith('Set GridColumns')) return;
    rows = applyDemo(rows, LINE_COLS, line);
    grid.update(spec());
  });
  grid.addEventListener('new', () => { gridNote.textContent = '› Store Sacn'; });
  const gridBox = div('lc-sr-gridbox');
  gridBox.append(grid);
  const gridRow = div('lc-sr-row');
  gridRow.append(
    btn('250 rows (virtual)', () => {
      rows = Array.from({ length: 250 }, (_, i) => ({ guid: `v${i + 1}`, no: i + 1, object: 'Sacn', name: `Sacn ${i + 1}`, direction: 'out', protocol: 'sACN', universes: String(i + 1), rate: 20 + (i % 25), send: i % 3 ? 'Full' : 'Changes', sync: !!(i % 2), gap: 0, state: 'sending', tone: 'ok' }));
      grid.update(spec());
    }),
    btn('Back to 5 rows', () => { rows = demoLines(); grid.update(spec()); }),
    btn('Mark Sacn 4 invalid', () => grid.update({ ...spec(), marks: { o4: { invalid: 'no universe routed' }, o2: { warn: 'late' }, o1: { flash: true } } })),
    // Another station's value sent and not yet reported back / refused by
    // it (the Stations grid, decisions §30 "Remote station edits").
    btn('Pending + refused cells', () => {
      rows = rows.map((r, i) => i === 0 ? { ...r, pending: { rate: 44 } } : i === 1 ? { ...r, refused: { protocol: "'Set Station Protocol' needs Setup — you are Program. [E-RIGHTS-001]" } } : r);
      grid.update(spec());
    }),
  );
  const gridWrap = div('lc-sr-wide');
  gridWrap.append(gridRow, gridBox, gridNote);
  root.append(section('luma-grid — tap, row numbers, headers (twice = the whole column), + / − latch, Thru, lasso, two-finger tap or the Edit chip = the edit card; one line per edit', gridWrap));
  grid.update(spec());

  // A phone-width grid (folding) and the cards view, same rows.
  let nics = demoNics();
  const nicNote = div('lc-sr-note lc-selectable', 'e.g. tap the Shell header, − on en5, Edit → Shell No');
  const nicGrid = document.createElement('luma-grid');
  nicGrid.id = 'sr-nic-grid';
  const nicSpec = (view = 'grid') => ({ key: 'showroom.nics', object: 'Interface', id: 'name', row_key: 'name', columns: NIC_COLS, rows: nics, state: {}, level: 'Admin', view, hardware_note: 'No new-object line: interfaces come from the hardware' });
  nicGrid.addEventListener('line', e => {
    nicNote.textContent = `› ${e.detail.line}`;
    if (!e.detail.line.startsWith('Set GridColumns')) { nics = applyDemo(nics, NIC_COLS, e.detail.line); nicGrid.update({ ...nicSpec(nicGrid.spec.view) }); }
  });
  const nicBox = div('lc-sr-gridbox phone');
  nicBox.append(nicGrid);
  const nicRow = div('lc-sr-row');
  nicRow.append(btn('Grid', () => nicGrid.update(nicSpec('grid'))), btn('Cards', () => nicGrid.update(nicSpec('cards'))));
  const nicWrap = div('lc-sr-wide');
  nicWrap.append(nicRow, nicBox, nicNote);
  root.append(section('luma-grid at phone width — low-priority columns fold into the row, the rest scroll sideways; Grid | Cards share the selection', nicWrap));
  nicGrid.update(nicSpec());

  // ── The edit card ──
  const cardNote = div('lc-sr-note lc-selectable', 'one change = one line');
  const cardRow = div('lc-sr-row');
  cardRow.append(btn('Edit card: Sacn 1 Thru 3 + 5', () => {
    const sel = rows.filter(r => ['o1', 'o2', 'o3', 'i1'].includes(r.guid));
    const card = openEditCard({ object: 'Sacn', columns: LINE_COLS, rows: sel, level: 'Admin', id: 'no', row_key: 'guid' }, {
      run: line => { cardNote.textContent = `› ${line}`; rows = applyDemo(rows, LINE_COLS, line); grid.update(spec()); card.update(rows.filter(r => sel.some(s => s.guid === r.guid))); },
    });
  }), btn('Edit card as Program (greyed)', () => {
    openEditCard({ object: 'Sacn', columns: LINE_COLS, rows: rows.slice(0, 2), level: 'Program', id: 'no', row_key: 'guid' }, { run: line => { cardNote.textContent = `› ${line}`; } });
  }));
  const cardWrap = div('lc-sr-wide');
  cardWrap.append(cardRow, cardNote);
  root.append(section('The edit card — tabs from sections, ‹different values› with the range, per-row fields read-only, rows without a field skipped', cardWrap));

  // ── luma-slab + luma-power ──
  const slabNote = div('lc-sr-note lc-selectable', 'greyed actions say why on a tap');
  let count = 1, clip = null, power = 'on';
  const slabs = [document.createElement('luma-slab'), document.createElement('luma-slab')];
  const showSlab = document.createElement('luma-slab');
  const drawSlabs = () => {
    const pw = { state: power, word: power === 'off' ? 'OFFLINE' : 'ONLINE', press: power === 'off' ? 'go online' : 'go offline (asks)', reason: power === 'failing' ? 'en1 has no link — DMX not sent' : '', rights: 'Setup', level: 'Admin' };
    for (const s of slabs) s.update({ edit: SLAB_EDIT, actions: DMX_DEF.tabs[0].actions, phone: ['assign'], level: 'Admin', count, clip,
      bound: [{ label: 'Rate', value: '30 Hz' }, { label: 'Send', value: 'Changes' }, { label: 'Sync', value: 'E1.31' }, { label: 'Packet gap', value: '0 µs' }], power: pw });
    showSlab.update({ edit: { ...SLAB_EDIT, new: { line: null, off: 'New show is on the right' } }, count, level: 'Admin', phone: ['load', 'save', 'newshow'],
      actions: [{ id: 'load', label: 'Load show…', tone: 'load', line: 'LoadShow' }, { id: 'save', label: 'Save show', tone: 'save', line: 'SaveShow' }, { id: 'saveas', label: 'Save as…', tone: 'saveas', line: 'SaveShow /As' },
        { id: 'newshow', label: 'New show…', tone: 'newshow', line: 'NewShow' }, { id: 'info', label: 'Info', tone: 'info', line: 'Info Show' }, { id: 'history', label: 'History', tone: 'history', line: 'List Show /History' }] });
  };
  for (const s of [...slabs, showSlab]) {
    s.addEventListener('act', e => { slabNote.textContent = `› ${e.detail.action?.line || e.detail.id}`; if (e.detail.id === 'copy' || e.detail.id === 'cut') { clip = { op: e.detail.id, count }; drawSlabs(); } });
    s.addEventListener('power', () => { power = power === 'on' ? 'off' : 'on'; slabNote.textContent = `› Set Station "Online" ${power === 'on' ? 'On' : 'Off'}`; drawSlabs(); });
  }
  const slabRow = div('lc-sr-row');
  slabRow.append(btn('Nothing selected', () => { count = 0; drawSlabs(); }), btn('1 selected', () => { count = 1; drawSlabs(); }), btn('3 selected', () => { count = 3; drawSlabs(); }),
    btn('Power: on', () => { power = 'on'; drawSlabs(); }), btn('off', () => { power = 'off'; drawSlabs(); }), btn('on but failing', () => { power = 'failing'; drawSlabs(); }));
  const desk = div('lc-sr-slabbox'); desk.append(slabs[0]);
  const phone = div('lc-sr-slabbox phone'); phone.append(slabs[1]);
  const shows = div('lc-sr-slabbox'); shows.append(showSlab);
  const slabWrap = div('lc-sr-wide');
  slabWrap.append(slabRow, desk, phone, shows, slabNote);
  root.append(section('luma-slab — Select · New · Copy · Cut · Paste · Delete · Oops · Import · Export · Edit (icons + standard colours), the tab’s actions, bound cells, the power toggle; a 2-column block on a phone', slabWrap));
  drawSlabs();

  const powers = div('lc-sr-powers');
  for (const [state, word, press, reason] of [['on', 'ONLINE', 'go offline (asks)'], ['off', 'OFFLINE', 'go online'], ['failing', 'SENDING', '', 'en1 has no link — DMX not sent'], ['on', 'SENDING', 'hold output (asks)']]) {
    const box = div('');
    const p = document.createElement('luma-power');
    box.append(p);
    powers.append(box);
    p.update({ state, word, press, reason });
    p.addEventListener('press', () => toast(`Pressed: ${word}`));
  }
  const compact = document.createElement('luma-power');
  compact.setAttribute('compact', '');
  const cbox = div('lc-sr-slabbox phone');
  cbox.append(compact);
  compact.update({ state: 'on', word: 'JOINED', press: 'leave (asks)' });
  const pwWrap = div('lc-sr-wide');
  pwWrap.append(powers, cbox);
  root.append(section('luma-power — look A: big icon, the state word, what a press does, a 5 px state bar; failing fills dark alert with the reason', pwWrap));

  // ── luma-setup ──
  const setupRow = div('lc-sr-row');
  const setupNote = div('lc-sr-note lc-selectable', 'the frame runs nothing here');
  const openDemo = (def, tab) => {
    let lines = demoLines(), nicRows = demoNics();
    const ctl = openSetup(def, {
      tab, level: 'Admin', grids: {},
      chips: [{ tone: 'ok', text: 'master · sending' }],
      power: { state: 'on' },
      run: line => {
        setupNote.textContent = `› ${line}`;
        if (/^Menu /.test(line)) { const m = /\/Tab (\S+)/.exec(line); if (m) ctl.setTab(m[1]); return; }
        if (/^Set Station "HoldOutput" (On|Off)/.test(line)) { ctl.setPower({ state: /On$/.test(line) ? 'off' : 'on' }); return; }
        lines = applyDemo(lines, LINE_COLS, line); nicRows = applyDemo(nicRows, NIC_COLS, line);
        ctl.setRows('Lines', lines); ctl.setRows('Interfaces', nicRows);
        ctl.setBound({ 'Sacn.Rate': { value: `${lines[0].rate} Hz` } });
      },
      onTab: t => { setupNote.textContent = `› Menu "${def.name}" /Tab ${t}`; },
    });
    ctl.setRows('Lines', lines, { caption: '' });
    ctl.setRows('Interfaces', nicRows, { caption: 'Interfaces of FOH' });
    ctl.setBound({ 'Sacn.Rate': { value: '30 Hz' }, 'Sacn.Send': { value: 'Changes' }, 'Sacn.Sync': { value: 'E1.31' }, 'Sacn.PacketGap': { value: '0 µs' } });
    for (const t of def.tabs) if (t.content?.kind === 'settings') ctl.setFields(t.content.settings, values);
  };
  setupRow.append(btn('DMX Protocols (the SetupDmx board)', () => openDemo(DMX_DEF)));
  // Every menu the station declares (GET /api/menu-types), as skeletons.
  fetch('/api/menu-types').then(r => (r.ok ? r.json() : [])).then(defs => {
    for (const d of defs.filter(x => x.kind === 'setup')) setupRow.append(btn(`${d.label || d.name} (registry)`, () => openDemo(d)));
    // The Patch dialog with sample rows (34g4 part 3b, the locked round 2 + patch-lock.html): its
    // claim button switches between Live and the draft here.
    const patch = defs.find(x => x.area);
    if (patch) setupRow.append(btn(`${patch.label} (sample: MVR tree, Live / Draft)`, () => openPatchDemo(patch)));
  }).catch(() => {});
  const openPatchDemo = def => {
    const L = 'a1', T1 = 'g1', T2 = 'g2', DS = 'g3', TOUR = 'a2';
    // A node's ≡ menu lines, as the patch feed sends them (lumacore-views `node_lines`).
    const nodeLines = (word, no) => ({ select: `${word} ${no}`, at: `${word} ${no}`, label: `Label ${word} ${no} {name}`,
      child: `Store MvrGroup 4 {name}; Move MvrGroup 4 At ${word} ${no} /NoConfirmation`, copy: `Copy ${word} ${no} At ${word === 'MvrLayer' ? 3 : 4}`,
      move: word === 'MvrGroup' ? `Move ${word} ${no} At {to}` : null, delete: `Delete ${word} ${no}` });
    const fx = (no, name, addr, parent, tags) => ({ no, guid: `f${no}`, name, note: '', fixture_type: 'Robe MegaPointe', mode: 'Mode 1', address: addr,
      pos_x: '-6', pos_y: '2.1', pos_z: '7.2', rot_x: '0', rot_y: '0', rot_z: '0', master_react: 'Grand', tags: tags.join(' · '), tag_guids: tags.map(t => `t:${t}`), group: '', parent, class: null });
    const fixtures = [fx(101, 'MegaPointe SL 1', '1.001', DS, ['FOH']), fx(102, 'MegaPointe SL 2', '1.035', DS, ['FOH']), fx(103, 'MegaPointe SR 3', '1.069', T1, ['Song 3']), fx(201, 'X5 Wash 1', '2.001', T2, []), fx(905, 'Floor spot 5', '9.001', TOUR, []), fx(999, 'Spare', '–', null, [])];
    const sample = (mode, holder = null) => ({ mode, holder, fixtures,
      tree: [{ kind: 'layer', guid: L, name: 'House', parent: null, count: 4, lines: nodeLines('MvrLayer', 1) }, { kind: 'group', guid: T1, name: 'Truss 1', parent: L, count: 3, lines: nodeLines('MvrGroup', 1) }, { kind: 'group', guid: DS, name: 'Downstage', parent: T1, count: 2, lines: nodeLines('MvrGroup', 2) }, { kind: 'group', guid: T2, name: 'Truss 2', parent: L, count: 1, lines: nodeLines('MvrGroup', 3) }, { kind: 'layer', guid: TOUR, name: 'Touring', parent: null, count: 1, lines: nodeLines('MvrLayer', 2) }],
      classes: [], tag_nodes: [{ kind: 'tag', guid: 't:FOH', name: 'FOH', parent: null, count: 2 }, { kind: 'tag', guid: 't:Song 3', name: 'Song 3', parent: null, count: 1 }], universes: [], fixture_types: [] });
    const ctl = openSetup(def, {
      level: 'Admin', grids: {},
      run: line => {
        setupNote.textContent = `› ${line}`;
        // The claim button (34g4 entry): a hold claims, a press (or the ✕) leaves the draft.
        if (/^Claim /.test(line)) ctl.setArea(sample('draft', 'Admin'));
        if (/^Unclaim /.test(line)) ctl.setArea(sample('live'));
      },
    });
    ctl.setArea(sample('live'));
    ctl.setRows('Patch', fixtures);
  };
  const setupWrap = div('lc-sr-wide');
  setupWrap.append(setupRow, setupNote);
  root.append(section('luma-setup — title row with chips and the home mark, left tabs (a top strip on a phone), content, the slab; ≤ screen − 32 px, full screen on a phone', setupWrap));
}
