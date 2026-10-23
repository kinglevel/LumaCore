// <luma-grid> — the one grid of the whole system (decisions §30 "12d
// hardening", "One grid for the whole system"; the SetupDmx board). A common
// object-grid model, our gestures:
//
//   - Rows are keyed by `row_key` (a GUID; a NIC's name for hardware rows);
//     columns come from the server's descriptors (MenuDef: id, label, field,
//     prop, kind text | int | bool | list, unit, min, max, options, width,
//     priority, frozen, rights, section, per_row, live; a host may add readonly and lock — the
//     reason a column can't be changed now, shown on its lock: the Patch dialog's Live columns).
//   - Selection = a set of row keys plus cells (key + column), so it survives
//     scrolling, sorting and live updates. A tap or a lasso replaces it; the
//     row-number gutter selects rows; a header tap selects the column on the
//     selected rows, twice (or with nothing selected) the whole column. `+`
//     and `−` latch until pressed again (or the other is); `Thru` is
//     one-shot (tap A, Thru, tap B). Mouse: Ctrl = +, Alt = −, Shift = Thru.
//     A bar shows "n selected (k hidden) · All · None · Invert".
//   - One-finger drag follows the device setting `Drag: Auto | Lasso |
//     Scroll` (local storage): Auto lassos with a mouse / pen or on a box ≥
//     600 px (two fingers scroll, scroll bars show) and scrolls with one
//     finger on a phone. Hold-then-drag lassos in every mode. A lasso starts
//     after 6 px or ~150 ms; a second finger before that scrolls (nothing was
//     selected yet), after it is ignored; the selection lands on finger-up;
//     the edges auto-scroll. Pinch does nothing.
//   - Edit: a two-finger tap (fingers still), the slab's Edit cell, the Edit
//     chip that shows beside the selection after a pause, double-click,
//     Enter / F2 / Ctrl+Enter, right-click or typing. Cells in one column
//     open the edit card on that field (its editor at once); rows or several
//     columns open the card (kit/editcard.js).
//   - Every change is ONE command line naming its rows, built here from the
//     descriptors: `Set Output 1 Thru 3 "Rate" 44`, `Set Interface "en0" + "en1"
//     "Shell" No` (event `line`). The column state (order, hidden, widths,
//     sort, column set, new-object line) is the user's, on their profile:
//     `Set GridColumns "<key>" …` lines, also through `line`.
//   - Columns: a header menu (⋮ or hold): sort, hide, move, width, the
//     filter row (yellow, AND, a funnel on filtered headers), column sets
//     Condensed / Full / Extended. Leading columns (frozen) stay; on a phone
//     (box < 600 px) low-priority columns fold into an expandable row and the
//     rest scroll sideways.
//   - Looks: editable cells boxed, read-only on the background, selected
//     cells the selection fill, selected rows an accent gutter, focus a 2 px
//     frame, invalid rows alert, warnings warn, cut rows an alert tint, a
//     flash on the name. Rows 48 px on touch, 40 compact. Virtualized beyond
//     200 rows. Nothing re-sorts under an open editor or a finger.
//
//   el.update({ key, object, id, row_key, columns, rows, new_line, state,
//     level, view: 'grid' | 'cards', caption, marks: { [key]: { invalid,
//     warn, flash } }, bar: false (no selection bar: a grid that picks one row) | 'head' (a
//     phone, decisions §37 1: the bar folds into the header's No cell — the count, a tap opens
//     + − Thru All None Invert and the lasso setting as a menu) })
//   events: line { line } · select { rows, cells, count, hidden, columns }
//     · new {} · edit { rows, column }
//   view 'mini' (§30 "12e round 5"): the whole list drawn small to fill a
//   miniature pane — see renderMini.
//   reorder (§30 "12e", the group editor): `spec.reorder` gives every row a
//   grip (44 px, drags on the pointer core; the rows still scroll) and ↑ ↓;
//   a move is one event `reorder { key, from, to }` (the host's one line).
//   open (§30 "Users & Profiles follows storage"): `spec.open` ends every row
//   with › (44 px) — event `open { key }`; the host drills into the row.
//   methods: sortBy(id, dir), setState(prop, value), selection(), setMode('+' | '-' | null), thru(), all(), none(),
//     invert(), edit(column?), rowsWord(keys), setCut(keys), setView(v),
//     toggleFilter(), setColumnSet(name), columnSet(), select(keys)
//   extending it (`LumaGrid` is exported; decisions device-grid-and-value-marks): a subclass
//   draws its own cells (`cellHtml`) and rows above the header (`headTop`) and keeps the rest.

import { hold, holdTime, drag } from './pointer.js';
import { openMenu } from './menu.js';
import { openKeypad } from './keypad.js';
import { toast } from './toast.js';
import { openEditCard } from './editcard.js';

const COARSE = typeof matchMedia === 'function' && matchMedia('(any-pointer: coarse)').matches;
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

/** The rights ladder, lowest first (lumacore_tree::UserRights). */
export const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];
/** '' when `have` reaches `need`, else "Needs Setup — you are Program". */
export function needs(need, have) {
  if (!need || !have) return '';
  return LEVELS.indexOf(have) >= LEVELS.indexOf(need) ? '' : `Needs ${need} — you are ${have}`;
}

/** A row id as a command-line word (names with spaces or symbols quoted). */
// An id is a number or quoted text (lum.md "Targets"): `3`, `1.5`, `"en0"`, `"Tower"`.
export const idWord = v => (/^\d+(\.\d+)*$/.test(String(v)) ? String(v) : `"${String(v).replace(/[";]/g, '')}"`);

/** Row ids as the station reads them: numbers sorted with runs as `Thru`
 *  (`1 Thru 3 + 5`), names joined with `+` (`en0 + en1`). */
export function rowsWord(ids) {
  const list = [...ids];
  if (!list.length) return '';
  if (list.every(v => /^\d+$/.test(String(v)))) {
    const n = [...new Set(list.map(Number))].sort((a, b) => a - b);
    const parts = [];
    for (let i = 0; i < n.length;) {
      let j = i;
      while (j + 1 < n.length && n[j + 1] === n[j] + 1) j++;
      parts.push(j - i >= 2 ? `${n[i]} Thru ${n[j]}` : j > i ? `${n[i]} + ${n[j]}` : `${n[i]}`);
      i = j + 1;
    }
    return parts.join(' + ');
  }
  return list.map(idWord).join(' + ');
}

/** A value as the line takes it: a text always quoted, a number as typed
 *  (relative `+ 10`, a spread `21 Thru 30`), a word as it is. */
export function valueText(col, v) {
  const s = String(v ?? '');
  if (col.kind === 'text') return `"${s.replace(/[";]/g, '')}"`;
  if (col.kind === 'int') return s.replace(/[^0-9A-Za-z.+\-*/% ]/g, '').trim() || '0';
  return /^[A-Za-z0-9.+\-*/]+$/.test(s) ? s : `"${s.replace(/[";]/g, '')}"`;
}

/** The device's one-finger drag (decisions §30 "12d": a device setting). */
const DRAG_KEY = 'lc.grid.drag';
export const DRAG_MODES = ['Auto', 'Lasso', 'Scroll'];
export function dragSetting() {
  try { const v = localStorage.getItem(DRAG_KEY); return DRAG_MODES.includes(v) ? v : 'Auto'; } catch { return 'Auto'; }
}
export function setDragSetting(v) {
  try { localStorage.setItem(DRAG_KEY, v); } catch {}
  for (const g of document.querySelectorAll('luma-grid')) g.renderAll?.();
}

const GUTTER = 56;
const START_PX = 6;
const START_MS = 150;
const SLOP = 10;
const TWO_TAP_MS = 450;
const CHIP_MS = 700;
const VIRTUAL = 200;
const SETS = ['Condensed', 'Full', 'Extended'];
const SET_PRIORITY = { Condensed: 1, Full: 2, Extended: 9 };
const cellKey = (k, c) => `${k}\u0001${c}`;

/** A cell's text: words for on/off, the unit after a number. */
export function cellText(col, v) {
  if (v === undefined || v === null || v === '') return '—';
  if (col.kind === 'bool') {
    const [on, off] = col.options?.length === 2 ? col.options : ['Yes', 'No'];
    return v === true || /^(yes|on|true)$/i.test(String(v)) ? on : off;
  }
  if (col.kind === 'int' && col.unit && /^-?\d+(\.\d+)?$/.test(String(v))) return `${v} ${col.unit}`;
  return String(v);
}
const isOn = v => v === true || /^(yes|on|true)$/i.test(String(v));

export class LumaGrid extends HTMLElement {
  constructor() {
    super();
    this.spec = { columns: [], rows: [], state: {} };
    this.selRows = new Set();
    this.selCells = new Map(); // key → Set(col)
    this.mode = null;          // '+' | '-' (latched)
    this.thruNext = false;     // one-shot
    this.anchor = null;        // { key, col }
    this.focus = null;         // { key, col }
    this.filters = {};         // col → text
    this.showFilter = false;
    this.expanded = new Set();
    this.cut = new Set();
    this.order = null;         // frozen row order while a finger is down / an editor is open
    this.busy = 0;             // editors open
    this.g = null;             // the gesture in progress
    this.touches = new Map();
    this.lastHeader = { col: null, t: 0 };
  }

  connectedCallback() {
    this.dataset.kit = 'grid';
    if (!this.shell) this.build();
    this.ro = new ResizeObserver(() => { this.measure(); this.renderAll(); });
    this.ro.observe(this);
  }
  disconnectedCallback() { this.ro?.disconnect(); clearTimeout(this.chipTimer); }

  // ── Spec ──
  update(spec) {
    const prevKey = this.spec.key;
    this.spec = { columns: [], rows: [], state: {}, ...spec };
    this.spec.state = { columns: [], widths: {}, sort: null, set: 'Full', new_line: true, ...(spec.state || {}) };
    if (prevKey && prevKey !== this.spec.key) this.clearSel();
    // A row that went away leaves the selection.
    const keys = new Set(this.spec.rows.map(r => this.keyOf(r)));
    for (const k of [...this.selRows]) if (!keys.has(k)) this.selRows.delete(k);
    for (const k of [...this.selCells.keys()]) if (!keys.has(k)) this.selCells.delete(k);
    if (!this.shell) this.build();
    this.renderAll();
    // An open edit card follows the rows (the station's answer to a change).
    if (this.card?.isOpen?.()) this.card.update(this.selection().rows.map(k => this.rowByKey(k)));
  }

  keyOf(r) { return String(r[this.spec.row_key || 'guid'] ?? r[this.spec.id || 'no']); }
  idOf(r) { return r[this.spec.id || 'no']; }
  rowByKey(k) { return this.spec.rows.find(r => this.keyOf(r) === k); }
  level() { return this.spec.level || ''; }
  narrow() { return (this.clientWidth || window.innerWidth) < 600; }

  /** The columns shown: the user's list or the column set; on a phone the
   *  low-priority ones fold into the row. */
  columns() {
    // A declared `no` column is the row-number gutter.
    const all = (this.spec.columns || []).filter(c => c.id !== 'no');
    const st = this.spec.state;
    let cols;
    if (st.columns?.length) {
      const by = new Map(all.map(c => [c.id, c]));
      cols = [...all.filter(c => c.frozen && !st.columns.includes(c.id)), ...st.columns.map(id => by.get(id)).filter(Boolean)];
    } else {
      const p = SET_PRIORITY[st.set] ?? 2;
      cols = all.filter(c => c.frozen || (c.priority ?? 1) <= p);
    }
    if (this.narrow()) return cols.filter(c => c.frozen || (c.priority ?? 1) <= 1);
    return cols;
  }
  folded() {
    if (!this.narrow()) return [];
    const shown = new Set(this.columns().map(c => c.id));
    const st = this.spec.state;
    const p = st.columns?.length ? 99 : (SET_PRIORITY[st.set] ?? 2);
    return (this.spec.columns || []).filter(c => c.id !== 'no' && !shown.has(c.id) && (st.columns?.length ? st.columns.includes(c.id) : (c.priority ?? 1) <= p));
  }
  // A column is at least 44 px on a touch screen: its header is a tap target
  // (decisions §30 "12c2", the floor), a narrow X / Y / Z stays narrow with a mouse.
  width(c) { const w = this.spec.state.widths?.[c.id] || c.width || 120; return COARSE ? Math.max(w, 44) : w; }

  /** Rows in display order: filtered, sorted (kept still while busy). */
  shownRows() {
    let rows = this.spec.rows.filter(r => this.passes(r));
    const s = this.spec.state.sort;
    if (s?.col) {
      const col = (this.spec.columns || []).find(c => c.id === s.col);
      if (col) {
        const dir = s.dir === 'desc' ? -1 : 1;
        const val = r => r[col.field ?? col.id];
        rows = rows.slice().sort((a, b) => {
          const x = val(a), y = val(b);
          if (typeof x === 'number' && typeof y === 'number') return (x - y) * dir;
          return String(x ?? '').localeCompare(String(y ?? ''), undefined, { numeric: true }) * dir;
        });
      }
    }
    if (this.order && (this.busy || this.g)) {
      const at = new Map(this.order.map((k, i) => [k, i]));
      rows = rows.slice().sort((a, b) => (at.get(this.keyOf(a)) ?? 1e9) - (at.get(this.keyOf(b)) ?? 1e9));
    } else {
      this.order = rows.map(r => this.keyOf(r));
    }
    return rows;
  }
  passes(r) {
    for (const [id, f] of Object.entries(this.filters)) {
      if (!f) continue;
      const col = (this.spec.columns || []).find(c => c.id === id);
      if (!col) continue;
      if (!cellText(col, r[col.field ?? col.id]).toLowerCase().includes(f.toLowerCase())) return false;
    }
    return true;
  }

  // ── Selection ──
  touched() {
    const out = new Set(this.selRows);
    for (const k of this.selCells.keys()) out.add(k);
    return out;
  }
  cellOn(k, c) { return this.selRows.has(k) || !!this.selCells.get(k)?.has(c); }
  clearSel() { this.selRows.clear(); this.selCells.clear(); }
  /** What is selected: rows (keys, display order), ids, cells, the column
   *  when every cell is in one, hidden by the filter. */
  /** The selected rows' keys in the order they were picked (a value dialog's Selection order). */
  pickOrder() {
    const keys = [...new Set([...this.selRows, ...this.selCells.keys()])];
    return keys.filter(k => this.rowByKey(k));
  }
  selection() {
    const touched = this.touched();
    const shown = new Set(this.shownRows().map(r => this.keyOf(r)));
    const rows = this.spec.rows.filter(r => touched.has(this.keyOf(r)));
    const cols = new Set();
    let cells = 0;
    for (const [, set] of this.selCells) { cells += set.size; for (const c of set) cols.add(c); }
    const column = !this.selRows.size && cols.size === 1 ? [...cols][0] : null;
    return {
      rows: rows.map(r => this.keyOf(r)), ids: rows.map(r => this.idOf(r)),
      cells, column, columns: [...cols], count: rows.length,
      hidden: rows.filter(r => !shown.has(this.keyOf(r))).length,
      whole: this.selRows.size,
    };
  }
  emitSel() {
    this.dispatchEvent(new CustomEvent('select', { detail: this.selection(), bubbles: true }));
    this.armChip();
  }
  /** Rows by key: op replace | add | remove. */
  applyRows(keys, op) {
    if (op === 'replace') this.clearSel();
    for (const k of keys) {
      if (op === 'remove') { this.selRows.delete(k); this.selCells.delete(k); } else { this.selRows.add(k); this.selCells.delete(k); }
    }
  }
  /** Cells [key, col]: op replace | add | remove. */
  applyCells(pairs, op) {
    if (op === 'replace') this.clearSel();
    const visible = this.columns().map(c => c.id).concat(this.folded().map(c => c.id));
    for (const [k, c] of pairs) {
      if (op === 'remove') {
        if (this.selRows.has(k)) { this.selRows.delete(k); this.selCells.set(k, new Set(visible)); }
        this.selCells.get(k)?.delete(c);
        if (this.selCells.get(k)?.size === 0) this.selCells.delete(k);
      } else if (!this.selRows.has(k)) {
        if (!this.selCells.has(k)) this.selCells.set(k, new Set());
        this.selCells.get(k).add(c);
      }
    }
  }
  /** The op a tap or lasso does: the latched mode or the mouse's modifier. */
  opFor(e) {
    if (e?.ctrlKey || e?.metaKey) return 'add';
    if (e?.altKey) return 'remove';
    return this.mode === '+' ? 'add' : this.mode === '-' ? 'remove' : 'replace';
  }
  setMode(m) { this.mode = this.mode === m ? null : m; this.renderBar(); }
  thru() { this.thruNext = !this.thruNext; this.renderBar(); }
  all() { this.applyRows(this.shownRows().map(r => this.keyOf(r)), 'replace'); this.paint(); this.emitSel(); }
  none() { this.clearSel(); this.anchor = null; this.paint(); this.emitSel(); }
  invert() {
    const t = this.touched();
    this.applyRows(this.shownRows().map(r => this.keyOf(r)).filter(k => !t.has(k)), 'replace');
    this.paint(); this.emitSel();
  }
  /** Select rows by key (a host's own pick). */
  select(keys) { this.applyRows(keys, 'replace'); this.paint(); this.emitSel(); }
  setCut(keys) { this.cut = new Set(keys || []); this.paint(); }

  /** A tap on a cell (col) or a row number (col null). */
  tapAt(k, col, e) {
    const op = this.opFor(e);
    const thru = this.thruNext || e?.shiftKey;
    if (thru && this.anchor) {
      this.range(this.anchor, { key: k, col }, op === 'remove' ? 'remove' : 'add');
      this.thruNext = false;
    } else if (col == null) {
      const on = this.selRows.has(k);
      this.applyRows([k], op === 'replace' && on && this.touched().size === 1 ? 'remove' : op);
    } else {
      this.applyCells([[k, col]], op);
    }
    this.anchor = { key: k, col };
    this.focus = { key: k, col: col ?? this.columns()[0]?.id };
    this.paint(); this.renderBar(); this.emitSel();
  }
  range(a, b, op) {
    const keys = this.shownRows().map(r => this.keyOf(r));
    const i = keys.indexOf(a.key), j = keys.indexOf(b.key);
    if (i < 0 || j < 0) return;
    const rows = keys.slice(Math.min(i, j), Math.max(i, j) + 1);
    if (a.col == null || b.col == null) return this.applyRows(rows, op);
    const ids = this.columns().map(c => c.id);
    const x = ids.indexOf(a.col), y = ids.indexOf(b.col);
    const cols = x < 0 || y < 0 ? [b.col] : ids.slice(Math.min(x, y), Math.max(x, y) + 1);
    this.applyCells(rows.flatMap(k => cols.map(c => [k, c])), op);
  }
  /** Header tap: the column on the selected rows; twice, or with nothing
   *  selected, the whole column. */
  tapHeader(col, e) {
    const now = performance.now();
    const touched = [...this.touched()];
    // Twice = the same header again soon, or while its column is already on
    // for every touched row (the first tap's result): then the whole column.
    const twice = this.lastHeader.col === col
      && (now - this.lastHeader.t < 1500 || (touched.length > 0 && touched.every(k => this.cellOn(k, col))));
    this.lastHeader = { col, t: now };
    const keys = !touched.length || twice ? this.shownRows().map(r => this.keyOf(r)) : touched;
    const op = this.opFor(e);
    this.applyCells(keys.map(k => [k, col]), op === 'remove' ? 'remove' : op === 'add' ? 'add' : 'replace');
    this.anchor = keys.length ? { key: keys[0], col } : null;
    this.paint(); this.emitSel();
  }

  // ── Lines ──
  line(text) { this.dispatchEvent(new CustomEvent('line', { detail: { line: text }, bubbles: true })); }
  rowsWordFor(keys) { return rowsWord(keys.map(k => this.idOf(this.rowByKey(k))).filter(v => v != null)); }
  stateLine(prop, value) {
    const key = String(this.spec.key || '').replace(/["; ]/g, '');
    // Default = the factory state: Reset, never a value (decisions §33 61).
    if (prop === 'Default') this.line(`Reset GridColumns "${key}"`);
    else if (value === 'Default') this.line(`Reset GridColumns "${key}" "${prop}"`);
    else this.line(`Set GridColumns "${key}" "${prop}" ${value}`.trim());
  }

  // ── Edit ──
  /** Open the edit card for the selection (cells in one column: that field,
   *  its editor at once). Nothing selected: the focused cell first. */
  edit(column) {
    let sel = this.selection();
    if (!sel.count && this.focus) { this.applyCells([[this.focus.key, this.focus.col]], 'replace'); this.paint(); sel = this.selection(); }
    if (!sel.count) { toast('Select rows or cells first'); return null; }
    const rows = sel.rows.map(k => this.rowByKey(k));
    const focus = column ?? sel.column;
    this.busy++;
    this.hideChip();
    const card = openEditCard({
      object: this.spec.object, columns: this.spec.columns, rows, focus, level: this.level(), home: this.spec.home || '',
      id: this.spec.id || 'no', row_key: this.spec.row_key || 'guid', hidden: sel.hidden, open: !!focus,
    }, {
      run: l => this.line(l),
      closed: () => { this.busy = Math.max(0, this.busy - 1); this.renderAll(); },
    });
    this.card = card;
    this.dispatchEvent(new CustomEvent('edit', { detail: { rows: sel.rows, column: focus }, bubbles: true }));
    return card;
  }

  /** Sort by column `id` (`asc` / `desc`; null = unsorted): the user's grid state, one line. */
  sortBy(id, dir = 'asc') {
    const st = this.spec.state;
    st.sort = id ? { col: id, dir } : null;
    this.renderAll();
    this.stateLine('Sort', id ? `"${id} ${dir === 'asc' ? 'Asc' : 'Desc'}"` : 'None');
  }
  /** One of the user's grid state properties by line (the patch's Offset cell). */
  setState(prop, value) { this.spec.state[prop.toLowerCase()] = value; this.stateLine(prop, String(value)); }

  // ── Header menu ──
  headerMenu(col, at) {
    const st = this.spec.state;
    const ids = this.columns().map(c => c.id);
    const i = ids.indexOf(col.id);
    const setCols = next => { st.columns = next; this.renderAll(); this.stateLine('Columns', `"${next.join(',')}"`); };
    const sort = dir => this.sortBy(dir ? col.id : null, dir);
    openMenu(at, [
      { header: col.label },
      { icon: 'arrow-up', label: 'Sort ascending', check: st.sort?.col === col.id && st.sort.dir === 'asc', fn: () => sort('asc') },
      { icon: 'arrow-down', label: 'Sort descending', check: st.sort?.col === col.id && st.sort.dir === 'desc', fn: () => sort('desc') },
      { icon: 'arrow-up-down', label: 'No sort', off: st.sort?.col !== col.id, fn: () => sort(null) },
      null,
      { icon: 'eye-off', label: 'Hide column', off: col.frozen || ids.length <= 1, fn: () => setCols(ids.filter(x => x !== col.id)) },
      { icon: 'chevron-left', label: 'Move left', off: col.frozen || i <= 0 || this.columns()[i - 1]?.frozen, fn: () => { const n = ids.slice(); [n[i - 1], n[i]] = [n[i], n[i - 1]]; setCols(n.filter(x => !this.colById(x)?.frozen)); } },
      { icon: 'chevron-right', label: 'Move right', off: col.frozen || i >= ids.length - 1, fn: () => { const n = ids.slice(); [n[i + 1], n[i]] = [n[i], n[i + 1]]; setCols(n.filter(x => !this.colById(x)?.frozen)); } },
      { icon: 'columns-3', label: 'Width…', hint: `${this.width(col)} px`, fn: () => this.askWidth(col) },
      null,
      { icon: 'funnel', label: this.showFilter ? 'Hide the filter row' : 'Filter row', fn: () => this.toggleFilter() },
      ...SETS.map(s => ({ label: `Columns: ${s}`, check: !st.columns?.length && st.set === s, fn: () => this.setColumnSet(s) })),
      ...(this.spec.new_line ? [{ icon: 'plus', label: 'Show the new-object line', check: st.new_line !== false, fn: () => this.toggleNewLine() }] : []),
      { icon: 'rotate-ccw', label: 'Reset columns', fn: () => { this.spec.state = { ...st, columns: [], widths: {}, sort: null, set: 'Full' }; this.renderAll(); this.stateLine('Default', ''); } },
    ]);
  }
  colById(id) { return (this.spec.columns || []).find(c => c.id === id); }
  async askWidth(col) {
    const got = await openKeypad({ title: `${col.label} · width (px)`, value: String(this.width(col)), min: 40, max: 600 });
    const n = Math.max(40, Math.min(600, parseInt(got?.text, 10) || 0));
    if (!got || !n) return;
    const w = { ...(this.spec.state.widths || {}), [col.id]: n };
    this.spec.state.widths = w;
    this.renderAll();
    this.stateLine('Widths', `"${Object.entries(w).map(([k, v]) => `${k}=${v}`).join(',')}"`);
  }
  setColumnSet(s) {
    const st = this.spec.state;
    if (st.columns?.length) { st.columns = []; this.stateLine('Columns', 'Default'); }
    st.set = s;
    this.renderAll();
    this.stateLine('ColumnSet', s);
  }
  columnSet() { return this.spec.state.columns?.length ? 'Custom' : this.spec.state.set; }
  toggleFilter() { this.showFilter = !this.showFilter; if (!this.showFilter) this.filters = {}; this.renderAll(); this.emitSel(); }
  setView(v) { this.spec.view = v; this.renderAll(); }
  toggleNewLine() { const on = this.spec.state.new_line === false; this.spec.state.new_line = on; this.renderAll(); this.stateLine('NewLine', on ? 'On' : 'Off'); }

  // ── Drawing ──
  build() {
    this.innerHTML = `<div class="lc-grid">
      <div class="lc-grid-bar" data-bar></div>
      <div class="lc-grid-scroll" tabindex="0" data-scroll>
        <div class="lc-grid-head" data-head></div>
        <div class="lc-grid-body" data-body></div>
        <div class="lc-grid-lasso" hidden data-lasso></div>
      </div>
      <button type="button" class="lc-grid-chip" hidden data-chip><luma-icon name="pencil"></luma-icon>Edit</button>
      <button type="button" class="lc-grid-chip more" hidden data-chip-more aria-label="Menu for the selection"><luma-icon name="ellipsis"></luma-icon></button>
    </div>`;
    this.shell = this.firstElementChild;
    this.scroller = this.querySelector('[data-scroll]');
    this.headEl = this.querySelector('[data-head]');
    this.bodyEl = this.querySelector('[data-body]');
    this.lassoEl = this.querySelector('[data-lasso]');
    this.chipEl = this.querySelector('[data-chip]');
    this.chipEl.addEventListener('click', e => { e.stopPropagation(); this.edit(); });
    // The selection chip's ⋯: the one context menu (§34 round 3, 18).
    this.moreEl = this.querySelector('[data-chip-more]');
    this.moreEl.addEventListener('click', e => {
      e.stopPropagation();
      const r = this.moreEl.getBoundingClientRect();
      this.context(r.left, r.bottom + 4);
    });
    this.scroller.addEventListener('scroll', () => {
      this.hideChip();
      // The scroll a paint put back itself (below): not another paint.
      if (this.restoring) { this.restoring = false; return; }
      if (this.virtual) this.renderRows();
    }, { passive: true });
    this.wire();
    this.measure();
  }
  measure() {
    const v = parseFloat(getComputedStyle(this).getPropertyValue('--lc-grid-row')) || 40;
    this.rowH = v;
  }

  renderAll() {
    if (!this.shell) return;
    if (this.spec.view === 'mini') return this.renderMini();
    this.mini?.remove();
    this.mini = null;
    this.shell.hidden = false;
    this.renderBar();
    const cols = this.columns();
    const cards = this.spec.view === 'cards';
    this.shell.classList.toggle('cards', cards);
    this.shell.classList.toggle('lasso-one', this.lassoOne('touch'));
    this.shell.classList.toggle('narrow', this.narrow());
    // Reorder: a 44 px grip before the gutter, ↑ ↓ (2 × 44 px) before the fill.
    const ro = this.spec.reorder;
    const tracks = `${ro ? '48px ' : ''}${GUTTER}px ${cols.map(c => `${this.width(c)}px`).join(' ')} ${ro ? '96px ' : ''}${this.spec.open ? '48px ' : ''}minmax(0, 1fr)`;
    this.style.setProperty('--lc-grid-cols', tracks);
    // Frozen columns stick at the left after the gutter.
    let left = GUTTER + (ro ? 48 : 0);
    this.sticky = new Map();
    for (const c of cols) { if (!c.frozen) break; this.sticky.set(c.id, left); left += this.width(c); }
    this.headEl.hidden = cards;
    if (!cards) this.renderHead(cols);
    this.renderRows();
  }

  renderBar() {
    const bar = this.querySelector('[data-bar]');
    if (!bar) return;
    // A grid that only picks a row (Insert fixtures' lists) has no selection bar.
    bar.hidden = this.spec.bar === false || this.spec.bar === 'head';
    if (this.spec.bar === 'head') { this.renderHeadSel(); return; }
    if (bar.hidden) return;
    const s = this.selection();
    const drag = dragSetting();
    const count = s.count ? `${s.count} selected${s.cells ? ` · ${s.cells} cell${s.cells === 1 ? '' : 's'}` : ''}${s.hidden ? ` (${s.hidden} hidden)` : ''}` : 'nothing selected';
    bar.innerHTML = `${this.spec.caption ? `<span class="lc-grid-cap">${esc(this.spec.caption)}</span>` : ''}
      <span class="lc-grid-count" data-count="${s.count}" data-cells="${s.cells}" data-hidden="${s.hidden}">${esc(count)}</span>
      <button type="button" class="lc-grid-mode${this.mode === '+' ? ' on' : ''}" data-mode="+" aria-pressed="${this.mode === '+'}" title="+ adds (stays on until pressed again)"><luma-icon name="plus"></luma-icon></button>
      <button type="button" class="lc-grid-mode${this.mode === '-' ? ' on' : ''}" data-mode="-" aria-pressed="${this.mode === '-'}" title="− removes (stays on until pressed again)"><luma-icon name="minus"></luma-icon></button>
      <button type="button" class="lc-grid-mode thru${this.thruNext ? ' on' : ''}" data-thru aria-pressed="${this.thruNext}" title="Thru: tap A, Thru, tap B">Thru</button>
      <button type="button" class="lc-grid-sel" data-all>All</button>
      <button type="button" class="lc-grid-sel" data-none>None</button>
      <button type="button" class="lc-grid-sel" data-invert>Invert</button>
      <button type="button" class="lc-grid-drag" data-drag title="One-finger drag on this device: Auto · Lasso · Scroll (hold-then-drag always lassos)"><luma-icon name="square-dashed"></luma-icon>Lasso: ${esc(drag)}</button>`;
    bar.querySelector('[data-mode="+"]').onclick = () => this.setMode('+');
    bar.querySelector('[data-mode="-"]').onclick = () => this.setMode('-');
    bar.querySelector('[data-thru]').onclick = () => this.thru();
    bar.querySelector('[data-all]').onclick = () => this.all();
    bar.querySelector('[data-none]').onclick = () => this.none();
    bar.querySelector('[data-invert]').onclick = () => this.invert();
    bar.querySelector('[data-drag]').onclick = () => setDragSetting(DRAG_MODES[(DRAG_MODES.indexOf(drag) + 1) % DRAG_MODES.length]);
  }

  /** A phone's selection bar in the header's No cell: the count; a tap opens the bar's tools. */
  renderHeadSel() {
    const cell = this.headEl?.querySelector('.lc-gh.gutter');
    if (!cell) return;
    const s = this.selection();
    const mark = this.mode || (this.thruNext ? 'Thru' : '');
    cell.innerHTML = `<button type="button" class="lc-gh-sel${s.count ? ' on' : ''}" data-selmenu data-count="${s.count}" data-cells="${s.cells}" aria-label="Selection: ${s.count} selected"
      title="${s.count} selected">${s.count ? esc(s.count) : 'No'}${mark ? `<small>${esc(mark)}</small>` : ''}</button>`;
    cell.querySelector('[data-selmenu]').onclick = e => {
      e.stopPropagation(); // the page closes a menu on a click outside it: not the one that opens it
      const r = e.currentTarget.getBoundingClientRect();
      const drag = dragSetting();
      openMenu({ x: r.left, y: r.bottom }, [
        { header: s.count ? `${s.count} selected${s.hidden ? ` (${s.hidden} hidden)` : ''}` : 'Nothing selected' },
        { icon: 'plus', label: '+ adds', check: this.mode === '+', fn: () => this.setMode('+') },
        { icon: 'minus', label: '− removes', check: this.mode === '-', fn: () => this.setMode('-') },
        { label: 'Thru: tap A, Thru, tap B', check: !!this.thruNext, fn: () => this.thru() },
        null,
        { label: 'All', fn: () => this.all() },
        { label: 'None', fn: () => this.none() },
        { label: 'Invert', fn: () => this.invert() },
        null,
        { icon: 'square-dashed', label: 'One-finger drag', value: drag, fn: () => setDragSetting(DRAG_MODES[(DRAG_MODES.indexOf(drag) + 1) % DRAG_MODES.length]) },
      ]);
    };
  }

  stick(c) { return this.sticky?.has(c.id) ? ` style="left:${this.sticky.get(c.id)}px"` : ''; }

  // ── Hooks for a grid built on this one (kit/devicegrid.js) ──
  /** A cell drawn by the subclass (HTML with `data-k` / `data-col`), or null for the grid's own. */
  cellHtml(_c, _r, _k, _j) { return null; }
  /** Rows above the header row (HTML on the same column tracks), '' for none. */
  headTop(_cols) { return ''; }

  renderHead(cols) {
    const sel = this.selection();
    const sort = this.spec.state.sort;
    const fullCol = id => sel.count > 0 && sel.rows.every(k => this.cellOn(k, id));
    const lvl = this.level();
    this.headEl.innerHTML = `${this.headTop(cols)}<div class="lc-grid-hrow">
        ${this.spec.reorder ? '<div class="lc-gh grip"></div>' : ''}<div class="lc-gh gutter">No</div>
        ${cols.map(c => `<div class="lc-gh${c.frozen ? ' frozen' : ''}${fullCol(c.id) ? ' on' : ''}${this.filters[c.id] ? ' filtered' : ''}${this.width(c) < 88 ? ' narrow' : ''}" data-hcol="${esc(c.id)}"${this.stick(c)}>
          <button type="button" class="lc-gh-tap" data-htap="${esc(c.id)}">
            <span class="lc-gh-label">${esc(c.label)}</span>
            ${this.filters[c.id] ? '<luma-icon name="funnel" class="lc-gh-funnel"></luma-icon>' : ''}
            ${sort?.col === c.id ? `<luma-icon name="${sort.dir === 'desc' ? 'arrow-down' : 'arrow-up'}" class="lc-gh-sort"></luma-icon>` : ''}
            ${c.kind === 'list' && c.prop ? '<luma-icon name="chevron-down" class="lc-gh-kind"></luma-icon>' : ''}
            ${(c.prop && needs(c.rights, lvl)) || c.lock ? `<luma-icon name="lock" class="lc-gh-kind"${c.lock ? ` title="${esc(c.lock)}"` : ''}></luma-icon>` : ''}
          </button>
          <button type="button" class="lc-gh-more" data-hmore="${esc(c.id)}" aria-label="${esc(c.label)} column menu"><luma-icon name="ellipsis-vertical"></luma-icon></button>
        </div>`).join('')}
        ${this.spec.reorder ? '<div class="lc-gh order"></div>' : ''}${this.spec.open ? '<div class="lc-gh open"></div>' : ''}<div class="lc-gh fill"></div>
      </div>
      ${this.showFilter ? `<div class="lc-grid-frow" data-kit="grid-filter">
        <div class="lc-gf gutter"><luma-icon name="funnel"></luma-icon></div>
        ${cols.map(c => `<div class="lc-gf${c.frozen ? ' frozen' : ''}"${this.stick(c)}><input type="text" data-filter="${esc(c.id)}" value="${esc(this.filters[c.id] || '')}" placeholder="${c.kind === 'bool' || c.kind === 'list' ? esc((c.options || []).join(' / ')) : ''}" aria-label="Filter ${esc(c.label)}" autocomplete="off" spellcheck="false"></div>`).join('')}
        <div class="lc-gf fill"></div>
      </div>` : ''}`;
    for (const b of this.headEl.querySelectorAll('[data-htap]')) {
      const col = this.colById(b.dataset.htap);
      b.addEventListener('click', e => this.tapHeader(col.id, e));
      hold(b, { onHold: ({ x, y }) => this.headerMenu(col, { x, y }) });
    }
    for (const b of this.headEl.querySelectorAll('[data-hmore]')) {
      b.addEventListener('click', e => { e.stopPropagation(); const r = b.getBoundingClientRect(); this.headerMenu(this.colById(b.dataset.hmore), { x: r.left, y: r.bottom + 2 }); });
    }
    for (const inp of this.headEl.querySelectorAll('[data-filter]')) {
      inp.addEventListener('input', () => {
        this.filters[inp.dataset.filter] = inp.value;
        this.order = null;
        this.renderRows();
        this.renderBar();
        const h = this.headEl.querySelector(`[data-hcol="${CSS.escape(inp.dataset.filter)}"]`);
        h?.classList.toggle('filtered', !!inp.value);
      });
    }
    if (this.spec.bar === 'head') this.renderHeadSel();
  }

  /** Heights: a row, plus its folded fields when expanded (phone). */
  rowHeight(r, folded) {
    if (!folded.length || !this.expanded.has(this.keyOf(r))) return this.rowH;
    return this.rowH + Math.ceil(folded.length / 2) * (this.rowH + 6) + 10;
  }

  renderRows() {
    const cards = this.spec.view === 'cards';
    const rows = this.shownRows();
    const cols = this.columns();
    const folded = this.folded();
    if (cards) return this.renderCards(rows);
    this.virtual = rows.length > VIRTUAL;
    let from = 0, to = rows.length, top = 0, bottom = 0;
    if (this.virtual) {
      const view = this.scroller.clientHeight || 600;
      const st = Math.max(0, this.scroller.scrollTop - (this.headEl.offsetHeight || 0));
      let y = 0, i = 0;
      while (i < rows.length && y + this.rowHeight(rows[i], folded) < st - view) { y += this.rowHeight(rows[i], folded); i++; }
      from = i; top = y;
      while (i < rows.length && y < st + 2 * view) { y += this.rowHeight(rows[i], folded); i++; }
      to = i;
      for (let j = to; j < rows.length; j++) bottom += this.rowHeight(rows[j], folded);
    }
    const marks = this.spec.marks || {};
    const lvl = this.level();
    const html = [];
    if (top) html.push(`<div style="height:${top}px"></div>`);
    for (let i = from; i < to; i++) html.push(this.rowHtml(rows[i], i, cols, folded, marks, lvl));
    if (bottom) html.push(`<div style="height:${bottom}px"></div>`);
    if (this.spec.new_line && this.spec.state.new_line !== false) {
      html.push(`<div class="lc-grid-row new" data-new><div class="lc-gc gutter"><luma-icon name="plus"></luma-icon></div><div class="lc-gc newline">${esc(this.spec.new_line)}</div></div>`);
    } else if (this.spec.hardware_note) {
      html.push(`<div class="lc-grid-row new note"><div class="lc-gc gutter"></div><div class="lc-gc newline">${esc(this.spec.hardware_note)}</div></div>`);
    }
    if (!rows.length && !this.spec.rows.length) html.push(`<div class="lc-grid-empty">${esc(this.spec.empty || 'Nothing here yet')}</div>`);
    else if (!rows.length) html.push(`<div class="lc-grid-empty">No rows match the filter</div>`);
    // The rows swapped in one step (a template): emptying the body first lets WebKit clamp the
    // scroller to 0 and every paint on a big patch landed back at the top. Should it still move,
    // it is put back — once, without a paint of its own.
    const kept = this.scroller.scrollTop;
    const tpl = document.createElement('template');
    tpl.innerHTML = html.join('');
    this.bodyEl.replaceChildren(tpl.content);
    if (kept && this.scroller.scrollTop !== kept) {
      this.restoring = true;
      this.scroller.scrollTop = kept;
    }
    if (this.spec.reorder) this.wireReorder();
    // open (§30 "Users & Profiles follows storage"): › opens the row's detail.
    for (const b of this.bodyEl.querySelectorAll('[data-open]')) {
      b.addEventListener('click', e => { e.stopPropagation(); this.dispatchEvent(new CustomEvent('open', { detail: { key: b.dataset.open }, bubbles: true })); });
    }
    this.renderBar();
    if (!this.spec.view || this.spec.view === 'grid') this.paintHead();
  }

  /**
   * The miniature (`view: 'mini'`, decisions §30 "12e round 5"): the whole
   * list drawn small to fill its pane — never a small table floating in it.
   * The window type declares the columns (`mini_columns`, else the shown
   * ones) and each column its renderer (`c.mini`): text (default), num (right,
   * monospace, a zero faint), level (a bar with the value on top; `c.tone` = a
   * row field that paints it in the programmer colour), swatch (the value is
   * a colour), bar (`c.max`), icon (`c.icons`: value → icon name). A row's
   * `mini` state paints it: run (gold, `progress` 0–1 along the bottom), next
   * (an edge mark), done (dimmed), sel (blue edge). Columns keep `c.mini_min`
   * (px at full size); past the pane's edge they fade. The live row (run or
   * sel) stays in view. Drawn at the kit's small size and scaled to
   * `spec.mini_row` px rows, like every miniature.
   */
  renderMini() {
    const s = this.spec;
    this.shell.hidden = true;
    if (!this.mini) {
      this.mini = document.createElement('div');
      this.mini.className = 'lc-grid-mini';
      this.appendChild(this.mini);
    }
    const W = this.clientWidth, H = this.clientHeight;
    if (!W || !H) return;
    const ROW = 16; // the drawn row at the kit's small size
    const scale = (s.mini_row || 14) / ROW;
    const w = W / scale, h = H / scale;
    const all = (s.columns || []).filter(c => c.id !== 'no');
    const cols = s.mini_columns ? s.mini_columns.map(id => all.find(c => c.id === id)).filter(Boolean) : this.columns();
    const weight = c => c.mini_w ?? (c.width || 120) / 60;
    const tw = cols.reduce((a, c) => a + weight(c), 0) || 1;
    const widths = cols.map(c => Math.max(c.mini_min ?? 30, (w * weight(c)) / tw));
    const total = widths.reduce((a, b) => a + b, 0);
    const fit = Math.max(1, Math.floor((h - ROW) / ROW));
    const rows = s.rows;
    const live = Math.max(0, rows.findIndex(r => r.mini === 'run' || r.mini === 'sel'));
    let from = Math.max(0, live - Math.floor(fit / 3));
    from = Math.min(from, Math.max(0, rows.length - fit));
    const cell = (c, r) => {
      const v = r[c.field ?? c.id];
      switch (c.mini) {
        case 'num': return `<td class="n">${v === 0 || v === '0' ? '<i class="z">0</i>' : esc(cellText(c, v))}</td>`;
        case 'level': return `<td class="lvl${c.tone && r[c.tone] ? ' tone' : ''}">${v ? `<i style="width:${Math.max(0, Math.min(100, Number(v)))}%"></i>` : ''}<b>${v ? esc(v) : '<i class="z">0</i>'}</b></td>`;
        case 'swatch': return `<td>${v ? `<span class="sw" style="background:${esc(v)}"></span>` : ''}</td>`;
        case 'bar': return `<td>${v ? `<span class="bar" style="width:${Math.min(100, (Number(v) / (c.max || 1)) * 100)}%"></span>` : ''}</td>`;
        case 'icon': return `<td>${c.icons?.[v] ? `<luma-icon name="${esc(c.icons[v])}"></luma-icon>` : esc(v ?? '')}</td>`;
        default: return `<td class="t${c.frozen ? ' key' : ''}">${esc(cellText(c, v))}</td>`;
      }
    };
    const body = rows.slice(from, from + fit).map(r => {
      const prog = r.mini === 'run' && r.progress != null ? `<span class="prog" style="width:${Math.round(r.progress * 100)}%"></span>` : '';
      return `<tr class="${esc(r.mini || '')}">${cols.map((c, j) => (j === 0 ? cell(c, r).replace('</td>', `${prog}</td>`) : cell(c, r))).join('')}</tr>`;
    }).join('');
    this.mini.innerHTML = `<div class="lc-grid-mini-in" style="width:${w}px;transform:scale(${scale})">`
      + `<table style="width:${total}px"><colgroup>${widths.map(x => `<col style="width:${x}px">`).join('')}</colgroup>`
      + `<tr>${cols.map(c => `<th>${esc(c.mini_label || c.label)}</th>`).join('')}</tr>${body}</table></div>`
      + (total > w + 1 ? '<span class="lc-grid-mini-more"></span>' : '');
  }

  /** Reorder (spec.reorder): grips drag a row, ↑ ↓ step it; one event each. */
  wireReorder() {
    const at = k => this.spec.rows.findIndex(r => this.keyOf(r) === k);
    const emit = (key, to) => {
      const from = at(key);
      if (from < 0 || to === from || to < 0 || to >= this.spec.rows.length) return;
      this.dispatchEvent(new CustomEvent('reorder', { detail: { key, from, to }, bubbles: true }));
    };
    for (const b of this.bodyEl.querySelectorAll('[data-up]')) b.addEventListener('click', () => emit(b.dataset.up, at(b.dataset.up) - 1));
    for (const b of this.bodyEl.querySelectorAll('[data-down]')) b.addEventListener('click', () => emit(b.dataset.down, at(b.dataset.down) + 1));
    for (const g of this.bodyEl.querySelectorAll('[data-grip]')) {
      const row = g.closest('.lc-grid-row');
      const key = g.dataset.grip;
      let to = 0;
      drag(g, {
        onStart: () => { to = at(key); row.classList.add('lc-lifted'); this.busy++; },
        onMove: ({ dy }) => {
          row.style.transform = `translateY(${dy}px)`;
          to = Math.max(0, Math.min(this.spec.rows.length - 1, at(key) + Math.round(dy / (row.offsetHeight || this.rowH))));
        },
        onEnd: (_, cancelled) => {
          this.busy = Math.max(0, this.busy - 1);
          row.classList.remove('lc-lifted');
          row.style.transform = '';
          if (!cancelled) emit(key, to);
        },
      });
    }
  }

  /** The row-number gutter: the object's number, or the position when the
   *  rows are of several kinds (Output 1 … Input 1) or named (en0). */
  gutterNo(r, i) {
    if (r.gutter != null) return r.gutter;
    const mixed = this.spec.rows.some(x => (x.object || '') !== (this.spec.rows[0]?.object || ''));
    return (this.spec.id || 'no') === 'no' && !mixed && r.no != null ? r.no : i + 1;
  }

  rowHtml(r, i, cols, folded, marks, lvl) {
    const k = this.keyOf(r);
    const m = marks[k] || {};
    const whole = this.selRows.has(k);
    const touched = whole || this.selCells.has(k);
    const exp = folded.length && this.expanded.has(k);
    const cls = ['lc-grid-row', touched ? 'touched' : '', whole ? 'whole' : '', m.invalid ? 'invalid' : '', m.warn ? 'warn' : '', this.cut.has(k) ? 'cut' : '', exp ? 'expanded' : '', this.previewWay?.has(k) ? 'lc-way' : ''].filter(Boolean).join(' ');
    const focus = this.focus?.key === k ? this.focus.col : null;
    const cell = (c, j) => {
      const own = this.cellHtml(c, r, k, j);
      if (own != null) return own;
      const f = c.field ?? c.id;
      const v = r[f];
      const ro = !c.prop || c.readonly || v === undefined;
      const locked = !ro && needs(c.rights, lvl);
      const on = this.cellOn(k, c.id);
      // A value sent and not yet reported back (row.pending), or refused
      // (row.refused): the station says, the cell shows it meanwhile.
      const pend = r.pending?.[f];
      const refused = pend == null ? r.refused?.[f] : null;
      const state = pend != null ? ' pending' : refused ? ' refused' : '';
      const tip = pend != null ? `Sent ${cellText(c, pend)} — waiting for the station to report it (now ${cellText(c, v)})` : refused || '';
      const tone = c.tone ? r[c.tone] : (c.kind === 'bool' ? (isOn(v) ? 'ok' : 'off') : null);
      const flash = m.flash && j === 0 ? ' flash' : '';
      // A column locked by its host (the Patch dialog's Live columns): read only, saying why.
      const lockWhy = c.lock || '';
      // A value dialog's preview (§34 round 3, 18): old → new, red where it collides.
      const pv = this.previewMap?.get(k)?.[c.id];
      // A data cell is narrow: the new value, old → new on hover (the gutter shows both).
      const text = pv ? `<span class="lc-gc-new${pv.bad ? ' bad' : ''}" title="${esc(`${pv.from} → ${pv.to}`)}">${esc(pv.to)}</span>`
        : esc(cellText(c, pend ?? v));
      // A claim area's draft: a gold dot on the cells that differ from the live rig.
      // Violet: a cell someone else's live edit brought into the draft (yours wins).
      const mine = r.changed?.includes(f) ? '<i class="lc-dot accent lc-gc-mine" title="Your draft edit"></i>'
        : r.live_changed?.[f] ? `<i class="lc-dot lc-gc-mine lc-gc-theirs" title="${esc(`${r.live_changed[f]} changed it live`)}"></i>` : '';
      return `<div class="lc-gc${ro ? ' ro' : ' ed'}${locked || lockWhy ? ' locked' : ''}${on ? ' on' : ''}${focus === c.id ? ' focus' : ''}${c.frozen ? ' frozen' : ''}${c.kind === 'int' ? ' num' : ''}${tone ? ` t-${esc(tone)}` : ''}${flash}${state}"
          data-k="${esc(k)}" data-col="${esc(c.id)}"${this.stick(c)} title="${esc(tip || locked || lockWhy || (j === 0 && (m.invalid || m.warn)) || '')}">
          ${pend != null ? '<i class="lc-dot lc-gc-wait"></i>' : tone ? `<i class="lc-dot ${esc(tone)}"></i>` : ''}${r.color && j === 0 ? `<i class="lc-dot" style="background:${esc(r.color)}"></i>` : ''}
          <span class="lc-gc-text">${text}</span>${mine}${c.kind === 'list' && !ro ? '<luma-icon name="chevron-down" class="lc-gc-kind"></luma-icon>' : ''}
        </div>`;
    };
    const ri = this.spec.reorder ? this.spec.rows.findIndex(x => this.keyOf(x) === k) : -1;
    const grip = this.spec.reorder ? `<div class="lc-gc lc-gc-grip" data-grip="${esc(k)}" aria-label="Drag to reorder"><luma-icon name="grip-vertical"></luma-icon></div>` : '';
    const order = this.spec.reorder ? `<div class="lc-gc lc-gc-order">`
      + `<button type="button" data-up="${esc(k)}" aria-label="Move up"${ri <= 0 ? ' disabled' : ''}><luma-icon name="arrow-up"></luma-icon></button>`
      + `<button type="button" data-down="${esc(k)}" aria-label="Move down"${ri >= this.spec.rows.length - 1 ? ' disabled' : ''}><luma-icon name="arrow-down"></luma-icon></button></div>` : '';
    const foldBtn = folded.length ? `<button type="button" class="lc-gc-fold" data-fold="${esc(k)}" aria-label="${exp ? 'Fold' : 'More columns'}" aria-expanded="${!!exp}"><luma-icon name="${exp ? 'chevron-down' : 'chevron-right'}"></luma-icon></button>` : '';
    return `<div class="${cls}" data-row="${esc(k)}" style="height:${this.rowHeight(r, folded)}px">
        <div class="lc-grid-line">
          ${grip}<div class="lc-gc gutter${whole ? ' on' : ''}" data-gutter="${esc(k)}">${this.previewMap?.get(k)?.['#gutter'] ? `<span class="lc-gc-old">${esc(this.gutterNo(r, i))}</span><span class="lc-gc-new">${esc(this.previewMap.get(k)['#gutter'].to)}</span>` : esc(this.gutterNo(r, i))}</div>
          ${cols.map(cell).join('')}
          ${order}${this.spec.open ? `<div class="lc-gc lc-gc-open"><button type="button" data-open="${esc(k)}" aria-label="Open"><luma-icon name="chevron-right"></luma-icon></button></div>` : ''}<div class="lc-gc fill">${foldBtn}</div>
        </div>
        ${exp ? `<div class="lc-grid-fold">${folded.map(c => {
          const v = r[c.field ?? c.id];
          const ro = !c.prop;
          return `<div class="lc-gc${ro ? ' ro' : ' ed'}${this.cellOn(k, c.id) ? ' on' : ''}" data-k="${esc(k)}" data-col="${esc(c.id)}"><small>${esc(c.label)}</small><span class="lc-gc-text">${esc(cellText(c, v))}</span></div>`;
        }).join('')}</div>` : ''}
      </div>`;
  }

  renderCards(rows) {
    const cols = this.spec.columns.filter(c => (c.priority ?? 1) <= 2);
    const [title, ...rest] = cols;
    this.bodyEl.innerHTML = `<div class="lc-grid-cards">${rows.map(r => {
      const k = this.keyOf(r);
      const on = this.touched().has(k);
      const tone = title?.tone ? r[title.tone] : null;
      return `<div class="lc-grid-card${on ? ' on' : ''}" data-k="${esc(k)}" data-card="${esc(k)}">
        <div class="lc-grid-card-title">${tone ? `<i class="lc-dot ${esc(tone)}"></i>` : ''}<span>${esc(title ? cellText(title, r[title.field ?? title.id]) : this.idOf(r))}</span></div>
        ${rest.map(c => `<div class="lc-grid-card-row"><small>${esc(c.label)}</small><span${c.tone ? ` class="t-${esc(r[c.tone] || '')}"` : ''}>${esc(cellText(c, r[c.field ?? c.id]))}</span></div>`).join('')}
      </div>`;
    }).join('') || `<div class="lc-grid-empty">${esc(this.spec.empty || 'Nothing here yet')}</div>`}</div>`;
    this.renderBar();
  }

  /** Repaint the selection only (no rebuild under a finger). */
  paint() {
    if (this.spec.view === 'cards') {
      const t = this.touched();
      for (const el of this.bodyEl.querySelectorAll('[data-card]')) el.classList.toggle('on', t.has(el.dataset.card));
      this.renderBar();
      return;
    }
    for (const row of this.bodyEl.querySelectorAll('[data-row]')) {
      const k = row.dataset.row;
      const whole = this.selRows.has(k);
      row.classList.toggle('touched', whole || this.selCells.has(k));
      row.classList.toggle('whole', whole);
      row.classList.toggle('cut', this.cut.has(k));
      row.querySelector('[data-gutter]')?.classList.toggle('on', whole);
      for (const c of row.querySelectorAll('[data-col]')) {
        c.classList.toggle('on', this.cellOn(k, c.dataset.col));
        c.classList.toggle('focus', this.focus?.key === k && this.focus.col === c.dataset.col);
      }
    }
    this.paintHead();
    this.renderBar();
  }
  paintHead() {
    const sel = this.selection();
    for (const h of this.headEl.querySelectorAll('[data-hcol]')) {
      h.classList.toggle('on', sel.count > 0 && sel.rows.every(k => this.cellOn(k, h.dataset.hcol)));
    }
  }

  // ── The Edit chip: beside the selection after a pause ──
  armChip() {
    clearTimeout(this.chipTimer);
    this.hideChip();
    if (!this.selection().count || this.busy) return;
    this.chipTimer = setTimeout(() => this.showChip(), CHIP_MS);
  }
  showChip() {
    if (this.g || this.busy || !this.selection().count || !this.isConnected) return;
    const a = this.anchor;
    const el = a && (a.col != null
      ? this.bodyEl.querySelector(`[data-k="${CSS.escape(a.key)}"][data-col="${CSS.escape(a.col)}"]`)
      : this.bodyEl.querySelector(`[data-row="${CSS.escape(a.key)}"] .lc-grid-line > .lc-gc:not(.gutter):not(.fill)`))
      || this.bodyEl.querySelector('.lc-gc.on, [data-card].on');
    if (!el) return;
    const r = el.getBoundingClientRect(), box = this.shell.getBoundingClientRect();
    const s = this.scroller.getBoundingClientRect();
    if (r.bottom < s.top || r.top > s.bottom) return;
    this.chipEl.hidden = false;
    this.moreEl.hidden = false;
    // Beside the anchor cell, never over it (the cell stays tappable): to its
    // right when there is room, else to its left, else under it.
    const more = this.moreEl.offsetWidth + 6;
    const w = this.chipEl.offsetWidth + more;
    let top = r.top - box.top + (r.height - this.chipEl.offsetHeight) / 2;
    let left = r.right - box.left + 4;
    if (left + w > box.width - 4) left = r.left - box.left - w - 4;
    if (left < 4) { left = Math.max(4, Math.min(r.left - box.left, box.width - w - 8)); top = r.bottom - box.top + 4; }
    Object.assign(this.chipEl.style, { top: `${top}px`, left: `${left}px` });
    Object.assign(this.moreEl.style, { top: `${top}px`, left: `${left + this.chipEl.offsetWidth + 6}px` });
  }
  hideChip() { if (this.chipEl) { this.chipEl.hidden = true; this.moreEl.hidden = true; } }

  /** A value dialog's preview: Map(row key → { [column id | '#gutter']: { from, to, bad } }),
   *  `way`: rows in the way (red); null clears it. Nothing is sent until the dialog applies. */
  setPreview(map, way) {
    this.previewMap = map || null;
    this.previewWay = way || null;
    this.renderRows();
  }
  /** Is column `id` drawn (else its preview goes to the gutter)? */
  shows(id) { return this.columns().some(c => c.id === id); }

  /** The one context menu on the selection at (x, y) client px (§34 round 3, 18): the host draws
   *  it (`context` { x, y }, cancelled when it did); a grid nobody hosts edits instead. */
  context(x, y) {
    this.hideChip();
    if (!this.selection().count) return;
    const asked = new CustomEvent('context', { detail: { x, y }, bubbles: true, cancelable: true });
    if (this.dispatchEvent(asked)) this.edit();
  }

  // ── Gestures ──
  /** One-finger drag lassos (mouse / pen always; touch per the setting). */
  lassoOne(pointerType) {
    if (pointerType !== 'touch' || this.lassoLatch) return true;
    const d = dragSetting();
    if (d === 'Lasso') return true;
    if (d === 'Scroll') return false;
    return !this.narrow() && window.innerWidth >= 600;
  }
  hit(target) {
    const t = target?.closest?.('[data-new], [data-fold], [data-gutter], [data-col], [data-card]');
    if (!t || !this.bodyEl.contains(t)) return null;
    if (t.dataset.new != null) return { new: true };
    if (t.dataset.fold != null) return { fold: t.dataset.fold };
    if (t.dataset.gutter != null) return { key: t.dataset.gutter, col: null };
    if (t.dataset.card != null) return { key: t.dataset.card, col: null };
    return { key: t.dataset.k, col: t.dataset.col };
  }
  /** Content coordinates of a client point. */
  local(x, y) {
    const r = this.scroller.getBoundingClientRect();
    return { x: x - r.left + this.scroller.scrollLeft, y: y - r.top + this.scroller.scrollTop };
  }

  wire() {
    const sc = this.scroller;
    sc.addEventListener('pointerdown', e => this.onDown(e));
    sc.addEventListener('pointermove', e => this.onMove(e));
    sc.addEventListener('pointerup', e => this.onUp(e, false));
    sc.addEventListener('pointercancel', e => this.onUp(e, true));
    // A lasso in progress keeps the page from scrolling under the finger
    // (hold-then-drag in Scroll mode); two fingers in lasso mode scroll here.
    sc.addEventListener('touchmove', e => { if (this.g?.lasso || this.g?.two) e.preventDefault(); }, { passive: false });
    // A captured pointer makes the scroller the target: hit-test the point.
    const at = e => this.hit(document.elementFromPoint(e.clientX, e.clientY) || e.target);
    sc.addEventListener('dblclick', e => {
      const h = at(e);
      if (!h || h.new || h.fold) return;
      if (!this.cellOn(h.key, h.col) && !(h.col == null && this.selRows.has(h.key))) this.tapAt(h.key, h.col, null);
      this.edit(h.col ?? undefined);
    });
    sc.addEventListener('contextmenu', e => {
      // Only a real right-click edits: a Mac Ctrl+click is `+`, a long press
      // is a hold (then drag = lasso) — neither opens the browser's menu.
      if (e.button !== 2 || e.ctrlKey) { e.preventDefault(); return; }
      const h = at(e);
      if (!h || h.new || h.fold) return;
      e.preventDefault();
      if (!(h.col == null ? this.selRows.has(h.key) : this.cellOn(h.key, h.col))) this.tapAt(h.key, h.col, null);
      this.context(e.clientX, e.clientY);
    });
    sc.addEventListener('keydown', e => this.onKey(e));
  }

  onDown(e) {
    if (e.target.closest('.lc-grid-head, .lc-gc-grip, .lc-gc-order, .lc-gc-open')) return;
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    this.hideChip();
    this.touches.set(e.pointerId, { x: e.clientX, y: e.clientY, x0: e.clientX, y0: e.clientY, t: performance.now() });
    if (this.touches.size === 2 && e.pointerType === 'touch') {
      // A second finger: before the lasso started it scrolls (or, still, a
      // two-finger tap edits); after, it is ignored.
      if (this.g && !this.g.lasso) {
        clearTimeout(this.g.timer);
        this.g = { two: true, t: performance.now(), moved: false, sx: this.scroller.scrollLeft, sy: this.scroller.scrollTop, first: this.g.first };
      }
      return;
    }
    if (this.touches.size > 1) return;
    const h = this.hit(e.target);
    const p = this.local(e.clientX, e.clientY);
    this.g = { id: e.pointerId, type: e.pointerType, x: e.clientX, y: e.clientY, p, h, first: h, lasso: false, e, t0: performance.now(),
      one: this.lassoOne(e.pointerType), gutter: h?.col === null && !h?.fold };
    if (this.spec.view === 'cards') this.g.one = false;
    if (this.g.one) {
      this.g.timer = setTimeout(() => { if (this.g && !this.g.two && this.g.one) this.startLasso(); }, START_MS);
      if (e.pointerType !== 'touch') { try { this.scroller.setPointerCapture(e.pointerId); } catch {} }
    } else {
      // Scroll mode: hold still to lasso — or, on a selected row, to lift it.
      this.g.timer = setTimeout(() => { if (this.g && !this.g.two && this.spec.view !== 'cards') { if (this.liftable(this.g)) this.startLift(this.g); else this.startLasso(); navigator.vibrate?.(10); } }, holdTime());
    }
  }
  startLasso() {
    if (!this.g || this.g.lasso) return;
    this.g.lasso = true;
    this.g.base = { rows: new Set(this.selRows), cells: new Map([...this.selCells].map(([k, v]) => [k, new Set(v)])) };
    this.drawLasso(this.g.p, this.g.p);
  }
  onMove(e) {
    const t = this.touches.get(e.pointerId);
    if (t) { t.x = e.clientX; t.y = e.clientY; }
    const g = this.g;
    if (!g) return;
    if (g.two) {
      const pts = [...this.touches.values()];
      if (pts.length < 2) return;
      const dx = pts.reduce((s, q) => s + q.x - q.x0, 0) / pts.length;
      const dy = pts.reduce((s, q) => s + q.y - q.y0, 0) / pts.length;
      if (Math.hypot(dx, dy) > SLOP) g.moved = true;
      if (g.moved && this.lassoOne('touch')) { this.scroller.scrollLeft = g.sx - dx; this.scroller.scrollTop = g.sy - dy; }
      return;
    }
    if (e.pointerId !== g.id) return;
    const dist = Math.hypot(e.clientX - g.x, e.clientY - g.y);
    if (g.lift) { this.dispatchEvent(new CustomEvent('liftmove', { detail: { x: e.clientX, y: e.clientY }, bubbles: true })); return; }
    if (!g.lasso) {
      if (dist <= START_PX) return;
      // A drag on a selected row lifts the selection (the host said so: a drop on a node of the
      // tree, decisions §40 "40d hardened" 3); on another row, the lasso.
      if (g.one && this.liftable(g)) { this.startLift(g); return; }
      if (g.one) this.startLasso();
      else { clearTimeout(g.timer); this.g = null; return; } // the page scrolls
    }
    g.last = { x: e.clientX, y: e.clientY };
    this.drawLasso(g.p, this.local(e.clientX, e.clientY));
    this.autoScroll();
  }
  /** Whether a press lifts: the host wants lifts and the row is selected. */
  liftable(g) { return !!this.lift && g.h?.key != null && !g.h.new && !g.h.fold && this.selRows.has(g.h.key); }
  startLift(g) {
    clearTimeout(g.timer);
    g.lift = true;
    this.dispatchEvent(new CustomEvent('lift', { detail: { keys: [...this.selRows], x: g.x, y: g.y }, bubbles: true }));
  }
  onUp(e, cancelled) {
    const t = this.touches.get(e.pointerId);
    this.touches.delete(e.pointerId);
    const g = this.g;
    if (!g) return;
    if (g.two) {
      if (this.touches.size) return;
      this.g = null;
      const quick = performance.now() - g.t < TWO_TAP_MS * 2;
      if (!cancelled && !g.moved && quick) {
        // Two-finger tap: edit the selection (or what is under the first finger).
        if (!this.selection().count && g.first?.key) this.tapAt(g.first.key, g.first.col, null);
        this.edit();
      }
      return;
    }
    if (e.pointerId !== g.id) return;
    clearTimeout(g.timer);
    cancelAnimationFrame(this.scrollRaf);
    this.scrollRaf = 0;
    this.g = null;
    try { this.scroller.releasePointerCapture(e.pointerId); } catch {}
    if (g.lift) { this.dispatchEvent(new CustomEvent('drop', { detail: { keys: [...this.selRows], x: e.clientX, y: e.clientY, cancelled: !!cancelled }, bubbles: true })); return; }
    if (cancelled) { this.lassoEl.hidden = true; return; }
    if (g.lasso) {
      const moved = t && Math.hypot(t.x - g.x, t.y - g.y) > START_PX;
      this.lassoEl.hidden = true;
      if (moved) return this.endLasso(g, e);
    }
    // A finger held still past the hold time and let go: the context menu on that row (a mouse
    // has its right-click). Hold-then-drag still lassos.
    if (g.type !== 'mouse' && performance.now() - g.t0 >= holdTime() && g.h?.key != null && !g.h.new && !g.h.fold) {
      if (!(g.h.col == null ? this.selRows.has(g.h.key) : this.cellOn(g.h.key, g.h.col))) this.tapAt(g.h.key, g.h.col, null);
      return this.context(g.x, g.y);
    }
    // A tap.
    const h = g.h;
    if (!h) return;
    if (h.new) { this.dispatchEvent(new CustomEvent('new', { bubbles: true })); return; }
    if (h.fold) { this.expanded.has(h.fold) ? this.expanded.delete(h.fold) : this.expanded.add(h.fold); this.renderRows(); return; }
    this.tapAt(h.key, h.col, g.type === 'mouse' ? e : null);
  }
  drawLasso(a, b) {
    const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y);
    Object.assign(this.lassoEl.style, { left: `${x}px`, top: `${y}px`, width: `${Math.abs(a.x - b.x)}px`, height: `${Math.abs(a.y - b.y)}px` });
    this.lassoEl.hidden = false;
    this.g.rect = { x, y, w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) };
  }
  /** Edges scroll while a lasso is out. */
  autoScroll() {
    if (this.scrollRaf) return;
    const step = () => {
      this.scrollRaf = 0;
      const g = this.g;
      if (!g?.lasso || !g.last) return;
      const r = this.scroller.getBoundingClientRect();
      const edge = 32;
      const vy = g.last.y < r.top + edge ? -12 : g.last.y > r.bottom - edge ? 12 : 0;
      const vx = g.last.x < r.left + edge ? -12 : g.last.x > r.right - edge ? 12 : 0;
      if (!vx && !vy) return;
      this.scroller.scrollTop += vy;
      this.scroller.scrollLeft += vx;
      this.drawLasso(g.p, this.local(g.last.x, g.last.y));
      this.scrollRaf = requestAnimationFrame(step);
    };
    this.scrollRaf = requestAnimationFrame(step);
  }
  /** The lasso landed: rows (started in the gutter) or cells under it. */
  endLasso(g, e) {
    const rect = g.rect;
    if (!rect) return;
    const rows = [], cells = [];
    const sc = this.scroller.getBoundingClientRect();
    const toLocal = r => ({ x: r.left - sc.left + this.scroller.scrollLeft, y: r.top - sc.top + this.scroller.scrollTop, w: r.width, h: r.height });
    const hits = q => q.x < rect.x + rect.w && q.x + q.w > rect.x && q.y < rect.y + rect.h && q.y + q.h > rect.y;
    if (this.spec.view === 'cards') {
      for (const el of this.bodyEl.querySelectorAll('[data-card]')) if (hits(toLocal(el.getBoundingClientRect()))) rows.push(el.dataset.card);
    } else if (g.gutter) {
      for (const el of this.bodyEl.querySelectorAll('[data-gutter]')) {
        const q = toLocal(el.getBoundingClientRect());
        if (q.y < rect.y + rect.h && q.y + q.h > rect.y) rows.push(el.dataset.gutter);
      }
    } else {
      for (const el of this.bodyEl.querySelectorAll('.lc-grid-line > [data-col]')) {
        if (hits(toLocal(el.getBoundingClientRect()))) cells.push([el.dataset.k, el.dataset.col]);
      }
    }
    // A lasso that starts on a selected item replaces too (it never drags).
    const op = this.opFor(g.type === 'mouse' ? e : null);
    if (rows.length) this.applyRows(rows, op);
    else if (cells.length) this.applyCells(cells, op);
    else if (op === 'replace') this.clearSel();
    const last = rows.length ? { key: rows[rows.length - 1], col: null } : cells.length ? { key: cells[cells.length - 1][0], col: cells[cells.length - 1][1] } : null;
    if (last) { this.anchor = last; this.focus = { key: last.key, col: last.col ?? this.columns()[0]?.id }; }
    this.paint();
    this.emitSel();
  }

  onKey(e) {
    if (e.target.closest('input')) return;
    // An editor, the card or a dialog over the grid has the keys.
    if (this.busy || document.querySelector('.lc-kp-scrim, .lc-pick-scrim, .lc-card-scrim, [data-kit="dialog"]:not([hidden])')) return;
    const cols = this.columns();
    const rows = this.shownRows();
    if ((e.key === 'Enter' || e.key === 'F2') && !e.altKey) { e.preventDefault(); this.edit(); return; }
    // The Menu key: the context menu beside the selection (Shift+F10 is Meow+F10, a core key).
    if (e.key === 'ContextMenu') {
      e.preventDefault();
      const on = this.bodyEl.querySelector('.lc-gc.on, [data-card].on')?.getBoundingClientRect();
      this.context(on ? on.left + 12 : 0, on ? on.bottom : 0);
      return;
    }
    if (e.key === 'Escape') { if (this.selection().count) { e.preventDefault(); e.stopPropagation(); this.none(); } return; }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') { e.preventDefault(); this.all(); return; }
    const arrows = { ArrowUp: [0, -1], ArrowDown: [0, 1], ArrowLeft: [-1, 0], ArrowRight: [1, 0] };
    if (arrows[e.key] && rows.length) {
      e.preventDefault();
      const [dx, dy] = arrows[e.key];
      const ri = Math.max(0, rows.findIndex(r => this.keyOf(r) === this.focus?.key));
      const ci = Math.max(0, cols.findIndex(c => c.id === this.focus?.col));
      const r = rows[Math.max(0, Math.min(rows.length - 1, ri + dy))];
      const c = cols[Math.max(0, Math.min(cols.length - 1, ci + dx))];
      this.focus = { key: this.keyOf(r), col: c.id };
      if (e.shiftKey && this.anchor) { this.range(this.anchor, this.focus, 'add'); } else if (!e.ctrlKey) { this.applyCells([[this.focus.key, c.id]], 'replace'); this.anchor = { ...this.focus }; }
      this.paint();
      this.emitSel();
      this.bodyEl.querySelector(`[data-k="${CSS.escape(this.focus.key)}"][data-col="${CSS.escape(c.id)}"]`)?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
      return;
    }
    // Typing starts editing the focused cell with what was typed.
    if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey && this.focus) {
      const col = this.colById(this.focus.col);
      if (!col?.prop) return;
      e.preventDefault();
      if (!this.cellOn(this.focus.key, col.id)) { this.applyCells([[this.focus.key, col.id]], 'replace'); this.paint(); }
      const card = this.edit(col.id);
      card?.typed?.(e.key);
    }
  }
}
customElements.define('luma-grid', LumaGrid);

