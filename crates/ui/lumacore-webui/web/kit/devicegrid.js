// <luma-device-grid> — the device grid (decisions device-grid-and-value-marks, device-sheet-window;
// the approved mockup 2026-10-22-device-sheet). A <luma-grid> whose rows are devices with the
// patch tree's folders as tree rows and whose columns are the attributes of the types in scope
// under their feature groups; every value cell is painted by kit/marks.js. It keeps the grid's
// keys, virtual scroll, column sets, header menu and miniature; the selection is the station's
// (the feed's `sel`), so every gesture is a line:
//
//   tap a name            `Fixture 785` · again `- Fixture 785` · a folder `Fixture Media 7003`
//   + / − latched, Ctrl / Alt   `+ Fixture 785` · `- Fixture 785`;  Shift  a range (`Fixture 1 Thru 9`)
//   lasso                 `Fixture 1 Thru 4 + 9` (with + / −: `+ …` / `- …`)
//   tap a value           selects its device; a second tap on the same cell opens the keypad and
//                         Enter runs `Fixture 785 Attribute "Pan" At -45` (Fade / Delay layer:
//                         `Attribute "Pan" Fade 2` on the selection; Output / DMX are read only)
//   hold / right-click    the one context menu on the selection (kit/context.js)
//
//   el.show(data, { mode: 'Fixture' | 'Channel', merge, level })  — data = the `devices` feed's
//   events: line { line } · range { from, count } (the rows the window shows: the host asks the
//     feed for their cells)
//   A small pane draws the hybrid miniature (decisions §30 "12e round 5"); a phone draws Channel.

import { LumaGrid, rowsWord, paintOf } from './grid.js';
import { valueParts, nameBar } from './marks.js';
import { openKeypad } from './keypad.js';
import { objectMenus, openContext } from './context.js';
import { phoneMode } from './phone.js';
import { toast } from './toast.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const FROZEN = [
  { id: 'name', label: 'Name', frozen: true, width: 220, mini_min: 50, mini_w: 2 },
  { id: 'fid', label: 'FID', frozen: true, width: 64, kind: 'int', mini: 'num', mini_min: 28 },
  { id: 'id_text', label: 'ID', frozen: true, width: 96 },
];
const VALUE_W = 66;
const MIX = '#mix';
// The miniature's columns (the hybrid): Fix, Name, Dim as a level, the colour, Pan.
const MINI = ['fid', 'name', 'Dimmer', MIX, 'Pan'];
const TILE_W = 96, TILE_H = 60;
const PAD_ROWS = 12;

export class LumaDeviceGrid extends LumaGrid {
  constructor() {
    super();
    this.data = null;
    this.cfg = { mode: 'Fixture', merge: false };
    this.lastCell = null;
    this.anchorKey = null;
    this.asked = { from: -1, count: -1 };
  }

  connectedCallback() {
    super.connectedCallback();
    this.classList.add('lc-dg');
  }

  // ── Data ──
  /** The feed's data and the window's settings. */
  show(data, cfg = {}) {
    this.data = data || { rows: [], columns: [], cells: {} };
    this.cfg = { ...this.cfg, ...cfg };
    const cells = this.data.cells || {};
    const cols = FROZEN.map(c => ({ ...c }));
    for (const c of this.data.columns || []) {
      if (c.group === 'Color' && !cols.some(x => x.id === MIX)) cols.push({ id: MIX, label: '', kind: 'value', group: 'Color', priority: 1, width: 40, mini: 'swatch', mini_min: 26, mini_w: 0.8 });
      const mini = c.id === 'Dimmer' ? { mini: 'level', tone: 'prg', mini_min: 46, mini_w: 1.8 } : c.id === 'Pan' ? { mini: 'num', mini_min: 34 } : {};
      cols.push({ id: c.id, label: c.label, kind: 'value', group: c.group, priority: c.priority, width: VALUE_W, ...mini });
    }
    const rows = (this.data.rows || []).map(r => {
      const own = cells[r.key];
      const dim = own?.Dimmer;
      return {
        ...r, fid: r.fid ?? '', no: r.fid ?? '',
        id_text: r.cid != null ? `${r.id_type} ${r.cid}` : r.id_type,
        // The miniature reads these.
        Dimmer: dim ? Math.round((dim.n ?? 0) * 100) : '', [MIX]: own?.[MIX] || '', Pan: own?.Pan?.t ?? '',
        prg: dim?.s === 'prgA' || dim?.s === 'prgI', mini: r.sel != null ? 'sel' : '',
      };
    });
    // The station's selection is the grid's.
    this.selRows = new Set(rows.filter(r => r.sel != null).map(r => r.key));
    this.selCells.clear();
    this.update({
      key: 'devices', object: 'Fixture', row_key: 'key', id: 'fid', columns: cols, rows, bar: false,
      empty: 'No devices in this scope', state: this.spec?.state, view: this.small() ? 'mini' : 'grid',
      mini_row: 14, mini_columns: MINI, level: this.cfg.level,
    });
  }

  small() { return !!this.clientWidth && (this.clientWidth < 300 || this.clientHeight < 180); }
  viewMode() { return phoneMode() ? 'Channel' : this.cfg.mode; }
  cellOf(k, col) {
    const own = this.data?.cells?.[k];
    if (!own) return undefined;
    return col in own ? own[col] : null;
  }

  // ── Drawing ──
  renderAll() {
    if (this.viewMode() === 'Channel' && this.shell && this.spec.view !== 'mini') return this.renderTiles();
    this.tiles?.remove();
    this.tiles = null;
    super.renderAll();
  }

  renderRows() {
    super.renderRows();
    this.askRange();
  }

  headTop(cols) {
    const out = ['<div class="lc-gh gutter"></div>'];
    for (let i = 0; i < cols.length;) {
      const c = cols[i];
      if (c.frozen) { out.push(`<div class="lc-gh frozen"${this.stick(c)}></div>`); i++; continue; }
      let n = 1;
      while (i + n < cols.length && !cols[i + n].frozen && cols[i + n].group === c.group) n++;
      out.push(`<div class="lc-gh lc-dg-group" style="grid-column: span ${n}">${esc(c.group || '')}</div>`);
      i += n;
    }
    out.push('<div class="lc-gh fill"></div>');
    return `<div class="lc-grid-hrow lc-dg-groups">${out.join('')}</div>`;
  }

  /** A device's paint is its name cell's bar, never a tint over its values (the marks own those). */
  rowPaint() { return ''; }

  cellHtml(c, r, k, j) {
    const focus = this.focus?.key === k && this.focus.col === c.id ? ' focus' : '';
    const on = this.cellOn(k, c.id) ? ' on' : '';
    if (c.id === 'name') {
      const folder = r.kind === 'folder';
      // The fixture's paint as a bar along the name's left edge (decisions 0294); the row stays untinted.
      const paint = paintOf(r.paint);
      const st = this.stick(c).replace(/^ style="/, '').replace(/"$/, '');
      return `<div class="lc-gc ro frozen lc-dg-name${folder ? ' folder' : ''}${paint ? ' lc-painted' : ''}${on}${focus}" data-k="${esc(k)}" data-col="name"${st || paint ? ` style="${st}${paint ? `${st ? ';' : ''}--lc-name-paint:${paint}` : ''}"` : ''} title="${esc(r.id)}">`
        + `${nameBar(r.bar)}<span class="lc-dg-indent" style="width:${(r.depth || 0) * 14}px"></span>`
        + `${folder ? '<luma-icon name="folder-open" class="lc-dg-fold"></luma-icon>' : ''}`
        + `<span class="lc-gc-text">${esc(r.name || r.id)}</span>${folder ? `<small>${esc(r.count)}</small>` : ''}</div>`;
    }
    if (c.kind !== 'value') return null;
    if (c.id === MIX) {
      const mix = this.data?.cells?.[k]?.[MIX];
      return `<div class="lc-gc lc-v swatch${on}${focus}" data-k="${esc(k)}" data-col="${MIX}" data-s="def">${mix ? `<span class="lc-v-sym" style="background:${esc(mix)}"></span>` : ''}</div>`;
    }
    const cell = this.cellOf(k, c.id);
    const p = valueParts(cell, { layer: this.data?.layer });
    // Merge cells: a neighbour in the same group with the same value draws once.
    const prev = this.cfg.merge && j > 0 ? this.columns()[j - 1] : null;
    const was = prev && prev.kind === 'value' && prev.group === c.group ? this.cellOf(k, prev.id) : null;
    const cont = was && cell && was.t === cell.t && was.s === cell.s ? ' cont' : '';
    return `<div class="lc-gc ${p.cls}${cont}${on}${focus}" data-k="${esc(k)}" data-col="${esc(c.id)}" ${p.attrs} title="${esc(p.title)}">${p.html}</div>`;
  }

  /** Channel mode: a tile per device (the dimmer as a level, the colour, its bar); folders part them. */
  renderTiles() {
    this.shell.hidden = true;
    this.mini?.remove();
    this.mini = null;
    if (!this.tiles) {
      this.tiles = document.createElement('div');
      this.tiles.className = 'lc-dg-tiles';
      this.tiles.addEventListener('click', e => {
        const t = e.target.closest('[data-tile]');
        if (t) this.tapAt(t.dataset.tile, null, e);
      });
      this.tiles.addEventListener('scroll', () => this.askRange(), { passive: true });
      this.tiles.addEventListener('contextmenu', e => { e.preventDefault(); this.context(e.clientX, e.clientY); });
      this.appendChild(this.tiles);
    }
    const html = [];
    for (const r of this.spec.rows) {
      if (r.kind === 'folder') { html.push(`<div class="lc-dg-tsep">${esc(r.name || r.id)}</div>`); continue; }
      const dim = this.cellOf(r.key, 'Dimmer');
      const p = valueParts(dim, { layer: this.data?.layer });
      const mix = this.data?.cells?.[r.key]?.[MIX];
      html.push(`<button type="button" class="lc-dg-tile${r.sel != null ? ' on' : ''}" data-tile="${esc(r.key)}" title="${esc(r.id)}">`
        + `${nameBar(r.bar)}<span class="lc-dg-tid"><span>${esc(r.fid !== '' ? r.fid : r.id_text)}</span>${mix ? `<span class="lc-v-sym" style="background:${esc(mix)}"></span>` : ''}</span>`
        + `<span class="lc-dg-lv ${p.cls}" ${p.attrs}><i style="width:${Math.round((dim?.n ?? 0) * 100)}%"></i>${p.html}</span></button>`);
    }
    this.tiles.innerHTML = html.join('') || `<div class="lc-grid-empty">${esc(this.spec.empty)}</div>`;
    this.askRange();
  }

  /** Tell the host which rows show (with a margin), once they change: it asks the feed for them. */
  askRange() {
    clearTimeout(this.rangeTimer);
    this.rangeTimer = setTimeout(() => {
      let from, count;
      if (this.tiles) {
        const per = Math.max(1, Math.floor(this.tiles.clientWidth / TILE_W));
        from = Math.floor(this.tiles.scrollTop / TILE_H) * per;
        count = (Math.ceil(this.tiles.clientHeight / TILE_H) + 2) * per;
      } else {
        const top = Math.max(0, this.scroller.scrollTop - (this.headEl.offsetHeight || 0));
        from = Math.floor(top / (this.rowH || 40));
        count = Math.ceil((this.scroller.clientHeight || 600) / (this.rowH || 40));
      }
      from = Math.max(0, from - PAD_ROWS);
      count += 2 * PAD_ROWS;
      if (from === this.asked.from && count === this.asked.count) return;
      this.asked = { from, count };
      this.dispatchEvent(new CustomEvent('range', { detail: { from, count }, bubbles: true }));
    }, 90);
  }

  // ── Gestures: every one a line ──
  tapAt(k, col, e) {
    const r = this.rowByKey(k);
    if (!r) return;
    this.focus = { key: k, col: col ?? 'name' };
    const op = this.mode || (e?.ctrlKey || e?.metaKey ? '+' : e?.altKey ? '-' : null);
    const sign = op === '+' ? '+ ' : op === '-' ? '- ' : '';
    const value = col && this.colById(col)?.kind === 'value' && col !== MIX;
    const sel = r.sel != null;
    const at = `${k}\u0001${col}`;
    if (value && sel && this.lastCell === at && !op) { this.lastCell = null; this.paint(); this.editCell(r, col); return; }
    this.lastCell = value ? at : null;
    if ((e?.shiftKey || this.thruNext) && this.anchorKey && this.anchorKey !== k) {
      this.thruNext = false;
      const keys = this.shownRows().map(x => this.keyOf(x));
      const [i, j] = [keys.indexOf(this.anchorKey), keys.indexOf(k)].sort((a, b) => a - b);
      const fids = keys.slice(i, j + 1).map(x => this.rowByKey(x)).filter(x => x?.kind === 'fixture' && x.fid !== '').map(x => x.fid);
      if (fids.length) this.line(`${sign}Fixture ${rowsWord(fids)}`);
    } else if (value && sel && !op) {
      // A value of a selected device: the first tap only focuses it (the next one edits).
    } else {
      this.line(`${sign || (sel && !value ? '- ' : '')}${r.id}`);
    }
    this.anchorKey = k;
    this.paint();
  }

  /** A header tap only focuses (the device grid has no column selection). */
  tapHeader() {}

  endLasso(g, e) {
    // Collect only what the lasso covers; the line says add, take out or replace.
    const op = this.mode || (g.type === 'mouse' && (e?.ctrlKey || e?.metaKey) ? '+' : g.type === 'mouse' && e?.altKey ? '-' : null);
    const kept = this.mode;
    this.mode = null;
    this.clearSel();
    this.lassoing = op || 'replace';
    super.endLasso(g, e && { ...e, ctrlKey: false, metaKey: false, altKey: false, shiftKey: false });
    this.lassoing = null;
    this.mode = kept;
  }

  emitSel() {
    if (!this.lassoing) return;
    const rows = this.pickOrder().map(k => this.rowByKey(k)).filter(Boolean);
    const fids = rows.filter(r => r.kind === 'fixture' && r.fid !== '').map(r => r.fid);
    const sign = this.lassoing === '+' ? '+ ' : this.lassoing === '-' ? '- ' : '';
    const lines = [];
    if (fids.length) lines.push(`${sign}Fixture ${rowsWord(fids)}`);
    for (const r of rows.filter(x => x.kind === 'folder' || x.fid === '')) lines.push(`${sign || (lines.length ? '+ ' : '')}${r.id}`);
    if (lines.length) this.line(lines.join(' ; '));
    // The station answers on the feed; meanwhile the grid shows what it had.
    this.selRows = new Set(this.spec.rows.filter(r => r.sel != null).map(r => r.key));
    this.selCells.clear();
    this.paint();
  }

  /** Enter / F2 / the second tap: the keypad on the focused value. */
  edit(column) {
    const f = this.focus;
    const r = f && this.rowByKey(f.key);
    const col = column ?? f?.col;
    if (r && this.colById(col)?.kind === 'value' && col !== MIX) this.editCell(r, col);
    return null;
  }

  async editCell(r, col) {
    const layer = this.data?.layer || 'Absolute';
    if (layer === 'Output' || layer === 'DMX') { toast(`${layer} shows what leaves the station: switch to Absolute to set a value`); return; }
    const c = this.colById(col);
    const cell = this.cellOf(r.key, col);
    this.busy++;
    const got = await openKeypad({ title: `${r.id} · ${c.label}${layer === 'Absolute' ? '' : ` · ${layer}`}`, value: String(cell?.t ?? '').replace(/[^0-9.\-]/g, '') });
    this.busy = Math.max(0, this.busy - 1);
    if (!got?.text) return;
    const quoted = `"${col.replace(/"/g, '')}"`;
    this.line(layer === 'Absolute' ? `${r.id} Attribute ${quoted} At ${got.text}` : `Attribute ${quoted} ${layer} ${got.text}`);
  }

  /** The one context menu on the station's selection (or the held device). */
  async context(x, y) {
    const menus = await objectMenus();
    const menu = menus.Fixture;
    let rows = this.spec.rows.filter(r => r.sel != null && r.fid !== '');
    if (!rows.length && this.focus) rows = [this.rowByKey(this.focus.key)].filter(r => r && r.fid !== '');
    if (!menu || !rows.length) return;
    openContext({
      at: { x, y }, menu, object: 'Fixture', rows: rowsWord(rows.map(r => r.fid)), count: rows.length,
      value: () => null, props: [], level: this.level(), run: l => this.line(l), edit: () => this.edit(),
    });
  }
}
customElements.define('luma-device-grid', LumaDeviceGrid);
