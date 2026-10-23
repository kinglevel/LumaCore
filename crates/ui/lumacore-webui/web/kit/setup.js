// <luma-setup> — the setup frame every menu is drawn in (decisions §30 "12d
// hardening": the usual setup layout, ours; the Setup* boards). It draws a
// MenuDef from the station's registry (GET /api/menu-types):
//
//   title row   icon, title, status chips (from state), the grid's Filter row
//               / Columns / Grid | Cards chips, the home mark, ×
//   left tabs   label + a small line under it (a top strip on a phone); a
//               menu with `left` takes them from a feed's rows (DMX
//               Protocols: the show's networks, a stripe for what is really
//               sent) and shows its own tabs as a strip on top of the content
//               (Lines · Routing · Discovery); grids filter their rows by it
//   content     by the tab's kind: a <luma-grid> (rows from the host),
//               cards (the same grid, the same selection), a grid's second
//               view `matrix` (History: List | Matrix, a column per object
//               kind; a tapped cell's `detail` field goes to the last line),
//               a grid's `dmx_map` view (Universes: List | Map, <luma-dmxmap>
//               on the area feed's runs and conflicts), settings cells
//               (<luma-settings embedded>, fields from `EditSetting`), or a
//               custom / empty note
//   last line   the last line run, or what the lines name ("lines name their
//               rows: Output 3")
//   slab        <luma-slab>: the standard edit actions on the grid's
//               selection, the tab's own actions, bound cells, the power
//               toggle
//
// An overlay never larger than the screen − 32 px; full screen on a phone.
// A phone (decisions §37 1–2: its short side under 500 px, kit/phone.js) folds the chrome and the
// slab: upright two rows on top — the title with the area's state as a chip (a tap: what Live /
// your draft / theirs means, the slab's lock sheet) and one band with the tree as a chip
// ("▾ Patch · all · 12", the tree in a sheet) and the tabs —, the grid's selection bar in its
// header, the slab one bar (<luma-slab bar>); on its side one row of chrome and the bar a column
// on the right. A desktop and a tablet keep this layout.
// Every gesture is a command line (`run`); a tab switch runs `Menu "<name>"
// /Tab <tab>` (the profile keeps the last tab, never an Oops step).
//
//   openSetup(def, { tab, level, viewer, grids, run(line), chips, power }) → { el,
//     def, close(), setTab(name), tab(), setRows(tab, rows, extra),
//     setFields(target, fields), setChips(chips), setPower(state),
//     setBound(values), setCustom(tab, node), setLeft(rows), grid(),
//     openRow(tab, word, group, part), setDetail({ fields, data, custom }),
//     detailInfo() }
//   detail (§30 "Users & Profiles follows storage"): a grid declaring
//     `detail` ends its rows with ›; a row opens <luma-detail> in place of the
//     grid (‹ back to the grid); `onDetail(info | null)` tells the host which
//     object (`{ kind, no, owner, edit, path, row }`) so it answers with
//     setDetail (the `edit` fields, the `detail` feed, the page's elements).
//   extra (setRows): { caption, marks, empty, networks: [{ guid, name }]
//     (columns with `expand: 'networks'` become one per network), options:
//     { [column id]: [values] } (a list column's choices from the station),
//     owner (the grid's `owner`: lines name it after the object word —
//     `Delete Version "demo" …`; `{owner}` in a template), select: [keys]
//     (a selection the host makes, once) }
//   events: line { line } · close · tab { tab } · action { id, action } (a
//     tab action without a line: the host does it) · slab-act { id } before
//     a standard edit action runs (cancelable: the host takes it)
//   setPower({ state, reason, data }): `{a.b}` in the power's lines is a field
//     of `data` (`Join LumaSession "{station.lumasession}"`)
//   setupOpen() → the open controller or null
//   area (a menu over a claim area with a draft, the Patch dialog — decisions §34 "34g4 patch
//     hardened" 9, the locked mockup 2026-10-18-34g4): the title is a crumb (Devices › DMX ›
//     Patch), a rail of the area's kinds sits under it, the left side is a <luma-tree> sidebar
//     grouped by the area's `tree` modes (MVR tree / Class / Tag; Direct | Deep for a mode with
//     `deep`) that filters the grids whose rows carry the mode's field, the tabs are a strip on
//     top, the chip shows Live or whose draft, in Live the columns whose property isn't live
//     are locked, and the power slot is the claim button (§34 "34g4 patch entry hardened",
//     patch-lock.html): Live · a hold runs the area's `claim`; Claimed · Draft (gold) · a press
//     runs `exit` (Save / Discard asked); another's claim greyed. The ✕ while claimed runs
//     `close` (asked, then closed). setArea(data) gives it the feed's `mode`, `holder` and the
//     tree's rows.

import { idWord, rowsWord, cellText } from './grid.js';
import './slab.js';
import { askField, valueWord, homeLine } from './settings.js';
import { toast } from './toast.js';
import { openDialog } from './dialog.js';
import { openMenu } from './menu.js';
import { openPick } from './popup.js';
import './detail.js';
import { objectMenus, openContext } from './context.js';
import { openValue, renumberKind, addressKind } from './value.js';
import { openInsert } from './insert.js';
import { openMvrImport } from './mvrimport.js';
import { openEditPatch } from './editpatch.js';
import { phoneMode, onPhoneMode } from './phone.js';
import { openSheet, sheetOpen } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const SETS = ['Condensed', 'Full', 'Extended'];
/** A tab's home as the header shows it (decisions §39): the icon and the words once. */
const homeText = home => homeLine(home) || esc(home || '');

class LumaSetup extends HTMLElement {
  constructor() {
    super();
    this.def = { tabs: [] };
    this.at = 0;
    this.rows = {};     // tab name → rows
    this.extra = {};    // tab name → { caption, marks, empty }
    this.fields = {};   // settings target → fields
    this.custom = {};   // tab name → node
    this.grids = {};    // grid key → profile state
    this.kept = {};     // grid key → the rows selected when its tab was left (4d4 2: the map's cards)
    this.chips = [];
    this.power = null;  // { state, reason }
    this.boundValues = {};
    this.level = '';
    this.last = '';
    this.clip = null;
    this.views = {};    // grid key → 'grid' | 'cards'
    this.drive = 'local-library'; // a drives content's picked drive (§38 5): `{drive}` in its lines
    this.leftRows = null; // a `left` menu's tabs (feed rows)
    this.leftKey = null;  // the picked one (client only, like a row highlight)
    this.areaData = null; // an area menu's feed data: mode, holder, the tree's rows
    this.treeMode = null; // the sidebar's grouping (an area `tree` mode id)
    this.treeNode = 'all'; // the picked sidebar node (a path of the drawn tree)
    this.treePaths = {};  // drawn path → node GUID
    this.deep = false;    // Direct | Deep for a `deep` mode
  }
  connectedCallback() {
    this.dataset.kit = 'setup';
    // A turn or a resize across the phone rule draws the frame again.
    this.offPhone ||= onPhoneMode(() => this.render());
    if (!this.wiredPhone) {
      this.wiredPhone = true;
      this.addEventListener('click', e => {
        if (e.target.closest('[data-treechip]')) this.openTreeSheet();
        else if (e.target.closest('[data-lockinfo]')) this.querySelector('luma-slab')?.openLock();
      });
    }
    if (!this.built) this.render();
  }
  // The frame going closes its Edit Patch (which ends what it made flash) and its sheets.
  disconnectedCallback() { this.ep?.close(); this.offPhone?.(); this.offPhone = null; this.treeSheet?.close(); }

  configure({ def, tab, level, viewer, grids, chips, power }) {
    this.def = def;
    this.level = level || '';
    this.viewer = viewer || '';
    this.grids = grids || {};
    this.chips = chips || [];
    this.power = power || null;
    const i = (def.tabs || []).findIndex(t => t.name.toLowerCase() === String(tab || '').toLowerCase());
    this.at = i >= 0 ? i : 0;
    this.render();
  }

  tab() { return this.def.tabs?.[this.at]; }
  setTab(name) {
    const i = (this.def.tabs || []).findIndex(t => t.name.toLowerCase() === String(name || '').toLowerCase());
    if (i < 0 || i === this.at) return;
    this.at = i;
    this.backOnTab = true;
    this.clip = null;
    this.drill = null;
    this.render();
  }

  // ── A row's detail (§30 "Users & Profiles follows storage") ──
  /** Drill into row `key` of tab `t` (its grid declares `detail`). */
  openDetail(t, gd, key, at) {
    const row = (this.rows[t.name] || []).find(r => String(r[gd.row_key || 'guid']) === String(key));
    if (!row || !gd.detail) return;
    const d = gd.detail;
    const fill = tpl => String(tpl || '').replace(/\{(\w+)\}/g, (_, k) => String(row[k] ?? ''));
    const el = document.createElement('luma-detail');
    el.addEventListener('back', () => this.closeDetail());
    el.addEventListener('run', e => this.run(e.detail.line));
    const info = { kind: d.feed, no: row.no, owner: fill(d.owner), edit: fill(d.edit), path: fill(d.path), row };
    // A part's buttons name the row's fields (`/Open "{profile}"`).
    const groups = d.groups.map(g => ({ ...g, parts: g.parts.map(p => ({ ...p, actions: (p.actions || []).map(a => ({ ...a, line: fill(a.line.replace(/\{owner\}/g, info.owner)) })) })) }));
    this.drill = { tab: t.name, key: String(key), el, info, spec: {
      back: t.label || t.name, title: row.name || String(row.no), path: info.path, owner: info.owner, groups,
      level: this.level, at, custom: {},
    } };
    this.render();
    el.update(this.drill.spec);
    this.dispatchEvent(new CustomEvent('detail', { detail: info, bubbles: true }));
  }
  closeDetail() {
    if (!this.drill) return;
    this.drill = null;
    this.render();
    this.dispatchEvent(new CustomEvent('detail', { detail: null, bubbles: true }));
  }
  /** `/Open "<row> [<group> [<part>]]"`: drill in once the rows are here. */
  openRow(tab, word, group, part) {
    this.setTab(tab);
    this.pendingOpen = { tab: this.tab()?.name, word: String(word), group, part };
    this.tryOpen();
  }
  tryOpen() {
    const o = this.pendingOpen, t = this.tab(), gd = this.gridDef();
    if (!o || t?.name !== o.tab || !gd?.detail) return;
    const row = (this.rows[t.name] || []).find(r => [r.no, r.name, r[gd.id], r[gd.row_key || 'guid']].some(v => String(v ?? '').toLowerCase() === o.word.toLowerCase()));
    if (!row) return;
    this.pendingOpen = null;
    this.openDetail(t, gd, row[gd.row_key || 'guid'], { group: o.group, part: o.part });
  }
  /** The host's answers for the open detail: fields, the feed's data, elements. */
  setDetail(patch) {
    if (!this.drill) return;
    const spec = this.drill.spec;
    if (patch.custom) spec.custom = { ...spec.custom, ...patch.custom };
    if (patch.fields) spec.fields = patch.fields;
    if (patch.data) spec.data = patch.data;
    this.drill.el.update(spec);
  }
  detailInfo() { return this.drill?.info || null; }
  run(line) {
    if (!line) return;
    this.last = line;
    const el = this.querySelector('[data-last]');
    if (el) el.textContent = `› ${line}`;
    this.dispatchEvent(new CustomEvent('line', { detail: { line }, bubbles: true }));
  }
  grid() { return this.querySelector('luma-grid'); }
  gridDef() { const c = this.tab()?.content; return c?.grid || null; }

  /** A template with the selection: {object}, {rows}, {at}, {left}. */
  fill(t, extra = {}) {
    const g = this.grid();
    const gd = this.gridDef();
    const sel = g?.selection();
    // Rows of mixed objects (DMX lines: Output and Input): one statement per
    // object, never an Input's number under the Output word.
    if (gd?.row_object && extra.rows == null && sel?.count && String(t).includes('{rows}')) {
      const groups = new Map();
      for (const k of sel.rows) {
        const r = g.rowByKey(k);
        if (!r) continue;
        const o = r.object || gd.object;
        if (!groups.has(o)) groups.set(o, []);
        groups.get(o).push(r[gd.id || 'no']);
      }
      if (groups.size > 1 || (groups.size === 1 && !groups.has(gd.object))) {
        return [...groups].map(([o, ids]) => this.fillOne(t, o, rowsWord(ids), extra.at ?? ids[0])).join('; ');
      }
    }
    const rows = extra.rows ?? (sel?.count ? g.rowsWordFor(sel.rows) : '');
    const owner = this.ownerWord();
    return this.fillOne(t, owner && gd?.object ? `${gd.object} ${owner}` : gd?.object || '', rows, extra.at ?? (sel?.ids?.[0] ?? ''));
  }
  /** The grid's owner as a line word (`"demo"`), or ''. */
  ownerWord() {
    const t = this.tab();
    const o = t ? this.extra[t.name]?.owner : null;
    return o ? `"${String(o).replace(/[";]/g, '')}"` : '';
  }
  fillOne(t, object, rows, at) {
    return String(t)
      .replaceAll('{object}', object)
      .replaceAll('{owner}', this.ownerWord())
      .replaceAll('{rows}', rows)
      .replaceAll('{at}', at)
      .replaceAll('{left}', this.leftKey != null ? idWord(this.leftKey) : '"Default"')
      .replaceAll('{drive}', this.drive && this.drive !== 'local-library' ? ` /Drive "${this.drive}"` : '')
      .replace(/\s+/g, ' ').trim();
  }

  /** A `left` menu's tabs from its feed (DMX Protocols: the networks). */
  setLeft(rows) {
    const L = this.def.left;
    if (!L) return;
    this.leftRows = rows || [];
    if (!this.leftRows.some(r => String(r[L.key]) === String(this.leftKey))) this.leftKey = this.leftRows[0]?.[L.key] ?? null;
    const nav = this.querySelector('.lc-setup-tabs');
    if (nav) nav.innerHTML = this.leftHtml();
    this.wireLeft();
    const t = this.tab();
    if (t?.content?.grid) this.setRows(t.name, this.rows[t.name] || []);
  }
  leftHtml() {
    const L = this.def.left;
    if (!this.leftRows?.length) return `<div class="lc-setup-left-empty">${esc(this.leftRows ? L.empty : '…')}</div>`;
    return this.leftRows.map(r => {
      const k = r[L.key];
      const on = String(k) === String(this.leftKey);
      return `<button type="button" data-left="${esc(k)}" class="${on ? 'on' : ''}" aria-pressed="${on}">
        <i class="lc-setup-stripe ${esc(r[L.stripe] || 'off')}"></i><span>${esc(k)}</span>${r[L.sub] ? `<small>${esc(r[L.sub])}</small>` : ''}</button>`;
    }).join('');
  }
  wireLeft() {
    for (const b of this.querySelectorAll('[data-left]')) b.onclick = () => {
      if (String(this.leftKey) === b.dataset.left) return;
      this.leftKey = b.dataset.left;
      for (const x of this.querySelectorAll('[data-left]')) { x.classList.toggle('on', x === b); x.setAttribute('aria-pressed', x === b); }
      const t = this.tab();
      if (t?.content?.grid) { this.grid()?.none?.(); this.setRows(t.name, this.rows[t.name] || []); }
    };
  }
  /** The rows a grid shows: a `left` menu's grid only those of the picked tab. */
  shownRows(t, gd) {
    const rows = this.def.area ? this.areaRows(this.rows[t.name] || []) : this.rows[t.name] || [];
    const f = gd.left_filter;
    if (!this.def.left || !f) return rows;
    return rows.filter(r => r[f] === '*' || String(r[f]) === String(this.leftKey));
  }
  /** Columns with `expand: 'networks'` → one per network; list choices from the station. */
  columnsOf(t, gd) {
    const ex = this.extra[t.name] || {};
    const out = [];
    for (const c of gd.columns || []) {
      if (c.expand === 'networks') {
        for (const n of ex.networks || []) out.push({ ...c, expand: null, id: `nic:${n.guid}`, field: `nic:${n.guid}`, label: `${n.name} → NIC`, prop: `Network ${idWord(n.name)} "Interface"` });
        continue;
      }
      const col = ex.options?.[c.id] ? { ...c, options: ex.options[c.id] } : c;
      // Live: a column whose property isn't live is locked until the draft is yours.
      // So is a column that changes only in the draft (Fixture type, Mode, MVR group, the FID).
      const draftOnly = (col.prop && col.live === false) || col.draft;
      out.push(this.isLive() && draftOnly ? { ...col, readonly: true, lock: this.areaWhy() } : col);
    }
    return out;
  }

  // ── A claim area (the Patch dialog) ──
  /** The feed's data for an area menu: mode, holder, the sidebar's rows. */
  setArea(data) {
    if (!this.def.area) return;
    const was = this.areaData?.mode;
    const wasReview = this.areaData?.review?.file;
    this.areaData = data || {};
    // The grid changes shape when the review comes or goes, or another file's review replaces it:
    // redraw it (live ↔ draft keeps the grid, its selection with it).
    const mode = this.areaData.mode;
    if (was !== undefined && ((mode === 'review') !== (was === 'review') || (mode === 'review' && this.areaData.review?.file !== wasReview))) {
      this.drawSide();
      this.drawChips();
      this.render();
      return;
    }
    // An open Edit Patch follows the feed (its fixtures' addresses, the runs, the universes).
    if (this.ep) {
      const nos = new Set(this.epNos || []);
      this.ep.update({ fixtures: (this.areaData.fixtures || []).filter(r => nos.has(Number(r.no))).map(r => ({ no: Number(r.no), name: r.name, address: r.address, footprint: r.footprint })),
        universes: this.areaData.universes || this.rowsOf('universes'), runs: this.areaData.runs || [] });
    }
    this.drawSide();
    this.drawChips();
    const t = this.tab();
    if (this.isReview() && t?.content?.grid?.rows === 'fixtures') {
      // The review's rows follow the feed (a cell taken, a row's tick).
      const g = this.grid();
      if (g) this.drawReview(g, t, t.content.grid);
      this.drawSlab();
    } else if (t?.content?.grid) this.setRows(t.name, this.rows[t.name] || []);
    else this.drawSlab();
  }
  areaMode() { return this.def.area?.tree?.find(m => m.id === this.treeMode) || this.def.area?.tree?.[0] || null; }
  isLive() { return !!this.def.area && !['draft', 'review'].includes(this.areaData?.mode); }
  /** The claim area's grid is the MVR import review (decisions §34 "Import MVR hardened" 3). */
  isReview() { return !!this.def.area && this.areaData?.mode === 'review' && !!this.areaData?.review; }
  /** Why a draft-only change can't run now ('' in your own draft): the one reason every locked
   *  cell, header and slab cell gives (§34 round 3, 10). */
  areaWhy() {
    if (!this.isLive()) return '';
    const holder = this.areaData?.holder;
    return holder ? `${holder} holds the draft` : 'Claim the draft to change it';
  }
  /** The sidebar's nodes for <luma-tree>: All, the mode's rows nested by `parent`, None. */
  sideNodes(rows) {
    const m = this.areaMode();
    const nodes = (this.areaData?.[m.rows]) || [];
    const kids = new Map();
    for (const n of nodes) {
      const p = n.parent && nodes.some(x => x.guid === n.parent) ? n.parent : null;
      if (!kids.has(p)) kids.set(p, []);
      kids.get(p).push(n);
    }
    this.treePaths = {};
    this.treeRows = {};
    // The whole area's count only where the grid's rows are the ones the tree sorts.
    const sorts = rows.length && m.filter in rows[0];
    const out = [{ path: 'all', name: `${this.def.label} · all`, count: sorts ? rows.length : null }];
    const walk = (parent, prefix) => (kids.get(parent) || []).forEach((n, i) => {
      const path = prefix ? `${prefix}.${i + 1}` : String(i + 1);
      this.treePaths[path] = n.guid;
      this.treeRows[path] = n;
      // A folder fixture shows its second id (Media 1); a layer or a group its kind (§40 40d).
      out.push({ path, name: n.name || '', count: n.count, badge: n.kind === 'layer' ? 'layer' : n.kind === 'group' ? 'group' : '', id: n.kind === 'folder' ? n.id : '' });
      walk(n.guid, path);
    });
    walk(null, '');
    const none = rows.filter(r => { const v = r[m.filter]; return m.list ? !(v || []).length : !v; }).length;
    out.push({ path: 'none', name: m.none, count: sorts ? none : null });
    return out;
  }
  /** The GUIDs a picked node stands for: it and, nested (or Deep), the nodes under it. */
  nodeGuids() {
    const m = this.areaMode();
    const g = this.treePaths[this.treeNode];
    if (!g) return null;
    const out = new Set([g]);
    if (m.nested && (!m.deep || this.deep)) {
      const prefix = `${this.treeNode}.`;
      for (const [p, guid] of Object.entries(this.treePaths)) if (p.startsWith(prefix)) out.add(guid);
    }
    return out;
  }
  /** An area grid's rows under the picked node (rows without the mode's field aren't filtered). */
  areaRows(rows) {
    const m = this.areaMode();
    if (!m || this.treeNode === 'all' || !rows.length || !(m.filter in rows[0])) return rows;
    const has = r => { const v = r[m.filter]; return m.list ? (v || []) : v ? [v] : []; };
    if (this.treeNode === 'none') return rows.filter(r => !has(r).length);
    const want = this.nodeGuids();
    return want ? rows.filter(r => has(r).some(v => want.has(v))) : rows;
  }
  sideHtml() {
    const a = this.def.area;
    const m = this.areaMode();
    return `<div class="lc-setup-side">
      <div class="lc-setup-gby" role="group" aria-label="Group by">${a.tree.map(t => `<button type="button" data-gby="${esc(t.id)}" class="${t.id === m.id ? 'on' : ''}" aria-pressed="${t.id === m.id}">${esc(t.label)}</button>`).join('')}</div>
      ${m.deep ? `<div class="lc-setup-gby sub" role="group" aria-label="Members"><button type="button" data-deep="0" class="${this.deep ? '' : 'on'}" aria-pressed="${!this.deep}">Direct</button><button type="button" data-deep="1" class="${this.deep ? 'on' : ''}" aria-pressed="${this.deep}">Deep</button></div>` : ''}
      <luma-tree class="lc-setup-tree"></luma-tree>
      ${(a.adds || []).length ? `<div class="lc-setup-treebar">${a.adds.map((x, i) => `<button type="button" data-add="${i}"${this.isLive() ? ` class="off" aria-disabled="true" data-why="${esc(this.areaWhy())}"` : ''}>${esc(x.label)}</button>`).join('')}</div>` : ''}
    </div>`;
  }
  /** The coloured bar under the strip (§34 round 3, 10): Live, your draft, or someone else's. */
  areaBar() {
    const a = this.def.area;
    if (!a?.bars?.length) return '';
    const holder = this.areaData?.holder || '';
    const [kind, bar] = !this.isLive() ? ['draft', a.bars[1]] : holder ? ['theirs', a.bars[2]] : ['live', a.bars[0]];
    const fill = s => String(s || '').replaceAll('{holder}', holder);
    return `<div class="lc-setup-bar ${kind}" data-areabar><b>${esc(fill(bar.title))}</b><span>${esc(fill(bar.text))}</span></div>`;
  }
  /** Where an area dialog is (§34 round 3): its path and the picked tree node, in the title. */
  areaHome() {
    const node = this.treeNode && this.treeNode !== 'all' ? this.sideNodes([]).find(n => n.path === this.treeNode)?.name : '';
    return [this.areaData?.path, node].filter(Boolean).join(' · ');
  }
  drawSide() {
    if (!this.def.area) return;
    const tree = this.querySelector('.lc-setup-tree');
    const gd = this.gridDef();
    const rows = gd ? (this.rows[this.tab().name] || []) : [];
    const nodes = this.sideNodes(rows);
    // A phone's tree chip: the picked node and its count.
    const chip = this.querySelector('[data-treechip] span');
    if (chip) chip.textContent = this.treeChipText(nodes);
    if (!tree) return;
    const menu = !this.areaMode().read_only && !!this.treeRows[this.treeNode]?.lines;
    tree.update({ nodes, mode: 'browse', selected: this.treeNode, numbers: false, menu, unit: this.def.area.unit || '', guides: true, empty: 'Nothing here yet.', lift: !this.areaMode().read_only });
    const home = this.querySelector('.lc-setup-home');
    if (home) home.textContent = this.areaHome();
  }
  treeChipText(nodes) {
    const n = nodes.find(x => x.path === this.treeNode) || nodes[0];
    return [n?.path === 'all' ? 'All' : n?.name, n?.count].filter(x => x != null && x !== '').join(' · ');
  }
  /** A phone's tree (§37 1): the sidebar — its grouping, the tree, + Layer / + Group — in a sheet
   *  (from the right on its side); a picked node closes it. */
  openTreeSheet() {
    const a = this.def.area;
    if (!a || !this.phone) return;
    this.treeSheet?.close();
    const rail = a.kinds.length > 1 ? `<div class="lc-setup-rail">${a.kinds.map(k => `<button type="button" data-kind="${esc(k.menu)}" class="${String(k.menu).toLowerCase() === String(this.def.name).toLowerCase() ? 'on' : ''}">${esc(k.label)}</button>`).join('')}</div>` : '';
    const sh = openSheet({
      title: `Show ${a.unit || 'rows'} from`, side: this.phone === 'side', host: this.querySelector('.lc-setup'), cls: 'lc-setup-treesheet',
      body: rail + this.sideHtml(), onClose: () => { if (this.treeSheet === sh) this.treeSheet = null; },
    });
    this.treeSheet = sh;
    for (const b of sh.body.querySelectorAll('[data-kind]')) b.onclick = () => { if (!b.classList.contains('on')) { sh.close(); this.run(`Menu "${b.dataset.kind}"`); } };
    this.wireSide();
    this.drawSide();
  }
  /** The chip that follows a lift ("6 fixtures → Smoke") and the node under the pointer. */
  dragChip(x, y) {
    if (!this.lifting) return;
    if (!this.chipEl) { this.chipEl = document.createElement('div'); this.chipEl.className = 'lc-dmap-chip'; this.chipEl.dataset.kit = 'chip'; document.body.append(this.chipEl); }
    this.chipEl.textContent = `${this.lifting.text} → ${this.lifting.over?.name || '…'}`;
    this.chipEl.style.left = `${x + 14}px`;
    this.chipEl.style.top = `${y + 14}px`;
  }
  dragOver(x, y) {
    if (!this.lifting) return;
    const row = document.elementFromPoint(x, y)?.closest?.('.lc-setup-tree .lc-tree-row');
    const n = row ? this.treeRows[row.dataset.path] : null;
    for (const r of this.querySelectorAll('.lc-tree-row.drop')) if (r !== row) r.classList.remove('drop');
    if (row && n?.lines?.at) row.classList.add('drop');
    this.lifting.over = n?.lines?.at ? { path: row.dataset.path, name: n.name || n.lines.at } : null;
    this.dragChip(x, y);
  }
  /** The node under the drop point, with its path; none when cancelled or off a node. */
  dropTarget(x, y, cancelled) {
    if (cancelled || !this.lifting) return null;
    const row = document.elementFromPoint(x, y)?.closest?.('.lc-setup-tree .lc-tree-row');
    const n = row ? this.treeRows[row.dataset.path] : null;
    return n?.lines?.at ? { ...n, path: row.dataset.path } : null;
  }
  dragEnd() {
    this.lifting = null;
    this.chipEl?.remove();
    this.chipEl = null;
    for (const r of this.querySelectorAll('.lc-tree-row.drop')) r.classList.remove('drop');
  }
  /** The picked node's ≡ menu (decisions §34 "34g4 patch hardened" 12): its own lines from the
   *  feed; edits only in the draft (live, a tap on Claim first). Move asks on the station. */
  async sideMenu(path, at) {
    const n = this.treeRows[path], L = n?.lines;
    if (!L) return;
    const edit = !this.isLive();
    const named = async (title, template, value = '') => {
      const got = await openDialog({
        title, inputs: [{ name: 'v', label: 'Name', value, kind: 'text' }],
        buttons: [{ label: 'Please', value: 'set', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
      });
      const name = got?.value && got.inputs?.v?.trim();
      if (name) this.run(template.replaceAll('{name}', `"${name.replace(/[";]/g, '')}"`));
    };
    const moveTo = async () => {
      // Every layer and group but this one and those under it.
      const under = new Set(Object.keys(this.treeRows).filter(p => p === path || p.startsWith(`${path}.`)));
      const dests = Object.entries(this.treeRows).filter(([p, r]) => !under.has(p) && r.lines?.at).map(([, r]) => r);
      const got = await openPick({ title: `Move ${L.select} into`, pick: { kind: 'MvrGroup', items: dests.map((r, i) => ({ no: i + 1, name: `${r.lines.at} · ${r.name || 'no name'}` })) } });
      const r = got?.no != null ? dests[got.no - 1] : null;
      if (r) this.run(L.move.replace('{to}', r.lines.at));
    };
    openMenu(at, [
      { header: `${L.select} · ${n.name || 'no name'}` },
      { icon: 'mouse-pointer-2', label: 'Select fixtures', fn: () => this.run(L.select) },
      { icon: 'pencil', label: 'Rename', off: !edit, fn: () => named(`Rename ${L.select}`, L.label, n.name || '') },
      // tidy: slab-ok (a tree node's own ≡ menu, as the locked Patch dialog mockup draws it; the tree is not a grid)
      { icon: 'plus', label: 'New group here', off: !edit, fn: () => named(`New group in ${L.select}`, L.child) },
      // tidy: slab-ok (the same ≡ menu: a copy of the node beside it)
      { icon: 'copy', label: 'Copy', off: !edit, fn: () => this.run(L.copy) },
      { icon: 'copy', label: 'Duplicate…', off: true },
      { icon: 'move', label: 'Move…', off: !edit || !L.move, fn: moveTo },
      null,
      // tidy: slab-ok (the same ≡ menu: Delete asks on the station, naming its groups and fixtures)
      { icon: 'trash-2', label: 'Delete…', danger: true, off: !edit, fn: () => this.run(L.delete) },
    ]);
  }
  wireSide() {
    if (!this.def.area) return;
    for (const b of this.querySelectorAll('[data-gby]')) b.onclick = () => {
      if (b.dataset.gby === this.treeMode) return;
      this.treeMode = b.dataset.gby;
      this.treeNode = 'all';
      this.render();
    };
    for (const b of this.querySelectorAll('[data-deep]')) b.onclick = () => { this.deep = b.dataset.deep === '1'; this.render(); };
    // The tree's add bar: the name is asked, then its line runs (greyed in Live, saying why).
    for (const b of this.querySelectorAll('[data-add]')) b.onclick = () => {
      if (b.dataset.why) { toast(b.dataset.why); return; }
      const x = this.def.area.adds[+b.dataset.add];
      this.askValue({ label: x.label.replace(/^\+\s*/, 'New '), line: x.line });
    };
    this.querySelector('.lc-setup-tree')?.addEventListener('menu', e => this.sideMenu(e.detail.path, { x: e.detail.x, y: e.detail.y }));
    // A node lifted onto another node: Move … At … (the station asks), §40 40d 3.
    const tree = this.querySelector('.lc-setup-tree');
    tree?.addEventListener('lift', e => { const n = this.treeRows[e.detail.path]; if (!n?.lines?.move) { this.lifting = null; return; } this.lifting = { node: e.detail.path, text: `${n.lines.select} "${n.name || ''}"` }; this.dragChip(e.detail.x, e.detail.y); });
    tree?.addEventListener('liftmove', e => this.dragOver(e.detail.x, e.detail.y));
    tree?.addEventListener('drop', e => {
      const dest = this.dropTarget(e.detail.x, e.detail.y, e.detail.cancelled);
      const from = this.lifting?.node && this.treeRows[this.lifting.node];
      this.dragEnd();
      if (!dest || !from?.lines?.move || dest.path === from.path || dest.path.startsWith(`${from.path}.`)) return;
      this.run(from.lines.move.replace('{to}', dest.lines.at));
    });
    this.querySelector('.lc-setup-tree')?.addEventListener('select', e => {
      this.treeNode = e.detail.path;
      this.drawSide();
      this.treeSheet?.close();
      const t = this.tab();
      if (t?.content?.grid) { this.grid()?.none?.(); this.setRows(t.name, this.rows[t.name] || []); }
    });
  }
  /** The state chip (decisions §34 "34g4 patch entry hardened"): Live, or whose draft. Claiming
   *  is the claim button's (bottom right). */
  areaChip() {
    const d = this.areaData || {};
    // A phone: the state's name, a tap says what it means (the slab's lock sheet).
    if (this.phone) {
      const [dot, text] = !this.isLive() ? ['accent', 'Draft'] : d.holder ? ['off', `Claimed · ${d.holder}`] : ['ok', 'Live'];
      return `<button type="button" class="lc-setup-chip btn state" data-chip="area" data-lockinfo><i class="lc-dot ${dot}"></i>${esc(text)}</button>`;
    }
    if (!this.isLive()) return `<span class="lc-setup-chip" data-chip="area"><i class="lc-dot accent"></i>${esc(`Devices draft · ${d.holder || ''}`)}</span>`;
    const who = d.holder ? `${d.holder} has claimed it` : 'nobody has claimed it';
    return `<span class="lc-setup-chip" data-chip="area"><i class="lc-dot ${d.holder ? 'off' : 'ok'}"></i>${esc(`Live patch · ${who}`)}</span>`;
  }
  drawChips() {
    const bar = this.querySelector('[data-areabar]');
    if (bar) { bar.insertAdjacentHTML('afterend', this.areaBar()); bar.remove(); }
    for (const b of this.querySelectorAll('[data-add]')) {
      b.classList.toggle('off', this.isLive());
      if (this.isLive()) { b.setAttribute('aria-disabled', 'true'); b.dataset.why = this.areaWhy(); } else { b.removeAttribute('aria-disabled'); delete b.dataset.why; }
    }
    const old = this.querySelector('[data-chip="area"]');
    if (!old) return;
    old.insertAdjacentHTML('afterend', this.areaChip());
    old.remove();
  }

  tabsHtml(kind) {
    return (this.def.tabs || []).map((tb, i) => `
          <button type="button" data-tab="${esc(tb.name)}" class="${i === this.at ? 'on' : ''}" aria-pressed="${i === this.at}">
            <span>${esc(tb.label || tb.name)}</span>${tb.sub && kind === 'tab' ? `<small>${esc(tb.sub)}</small>` : ''}</button>`).join('');
  }

  render() {
    this.built = true;
    const reopenTree = !!this.treeSheet;
    this.treeSheet?.close();
    this.phone = phoneMode();
    if (this.phone) this.dataset.phone = this.phone; else delete this.dataset.phone;
    this.parentElement?.classList.toggle('phone', !!this.phone);
    const side = this.phone === 'side';
    const d = this.def;
    const t = this.tab() || { content: { kind: 'empty' } };
    const c = t.content || { kind: 'empty' };
    const gd = c.grid;
    const view = gd ? (this.views[gd.key] || (c.kind === 'cards' ? 'cards' : 'grid')) : null;
    const state = gd ? (this.grids[gd.key] || {}) : null;
    const home = t.home ? homeText(t.home) : esc(d.home || '');
    // A phone: the tree is a chip, the tabs a strip — on its side both in the title row.
    const treeChip = d.area && this.phone ? `<button type="button" class="lc-setup-chip btn tree" data-treechip><luma-icon name="chevron-down"></luma-icon><span>${esc(this.treeChipText(this.sideNodes(gd ? this.rows[t.name] || [] : [])))}</span></button>` : '';
    const strip = d.left || d.area || side ? `<div class="lc-setup-strip" role="group" aria-label="${esc(d.label || d.name)} view">${treeChip}${this.tabsHtml('strip')}</div>` : '';
    const slab = '<luma-slab></luma-slab>';
    this.innerHTML = `<div class="lc-setup${this.phone ? ` phone ${this.phone}` : ''}">
      <header class="lc-setup-title">
        <luma-icon name="${esc(d.icon || 'settings')}"></luma-icon>
        ${d.area && this.phone ? `<span class="lc-setup-name">${esc(d.label || d.name)}</span>${this.areaChip()}` : d.area ? `<span class="lc-setup-crumb">${d.area.trail.map(x => `<span>${esc(x)}</span><i>›</i>`).join('')}<span class="lc-setup-name">${esc(d.label || d.name)}</span></span>${this.areaChip()}` : `<span class="lc-setup-name">${esc(d.label || d.name)}</span>`}
        ${(this.chips || []).map(ch => `<span class="lc-setup-chip"${ch.id ? ` data-chip="${esc(ch.id)}"` : ''}>${ch.tone ? `<i class="lc-dot ${esc(ch.tone)}"></i>` : ''}${esc(ch.text)}</span>`).join('')}
        ${gd && view === 'grid' ? `<button type="button" class="lc-setup-chip btn" data-filter><luma-icon name="funnel"></luma-icon>Filter row</button>
          <button type="button" class="lc-setup-chip btn" data-colset>Columns: ${esc(state?.columns?.length ? 'Custom' : state?.set || 'Full')}</button>` : ''}
        ${gd?.cards ? `<span class="lc-setup-switch" role="group" aria-label="View"><button type="button" data-view="grid" class="${view === 'grid' ? 'on' : ''}">Grid</button><button type="button" data-view="cards" class="${view === 'cards' ? 'on' : ''}">Cards</button></span>` : ''}
        ${gd?.matrix ? `<span class="lc-setup-switch" role="group" aria-label="View"><button type="button" data-view="grid" class="${view === 'grid' ? 'on' : ''}">List</button><button type="button" data-view="matrix" class="${view === 'matrix' ? 'on' : ''}">Matrix</button></span>` : ''}
        ${gd?.dmx_map ? `<span class="lc-setup-switch" role="group" aria-label="View"><button type="button" data-view="grid" class="${view === 'grid' ? 'on' : ''}">List</button><button type="button" data-view="map" class="${view === 'map' ? 'on' : ''}">Map</button></span>` : ''}
        ${side ? strip : ''}
        <span class="lc-setup-home">${d.area ? esc(this.areaHome()) : home}</span>
        <button type="button" class="lc-setup-x" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button>
      </header>
      ${d.area && !this.phone ? `<div class="lc-setup-rail"><span class="lc-setup-rail-label">${esc(d.area.kinds_label)}</span>${d.area.kinds.map(k => `<button type="button" data-kind="${esc(k.menu)}" class="${String(k.menu).toLowerCase() === String(d.name).toLowerCase() ? 'on' : ''}">${esc(k.label)}</button>`).join('')}<span class="lc-setup-rail-later">${esc(d.area.later)}</span></div>` : ''}
      <div class="lc-setup-body">
        ${d.area ? (this.phone ? '' : this.sideHtml()) : side && !d.left ? '' : `<nav class="lc-setup-tabs${d.left ? ' left' : ''}" aria-label="${esc(d.label || d.name)} tabs">${d.left ? this.leftHtml() : this.tabsHtml('tab')}</nav>`}
        <div class="lc-setup-main">
          ${side ? '' : strip}
          ${d.area && !this.phone ? this.areaBar() : ''}
          <div class="lc-setup-content" data-content></div>
          <div class="lc-setup-last lc-selectable" data-last>${esc(this.last ? `› ${this.last}` : '')}</div>
        </div>
        ${side ? slab : ''}
      </div>
      ${side ? '' : slab}
    </div>`;
    // The ✕ while your draft is open asks Save / Discard / Cancel on the station; the answer
    // closes the dialog (the area's `close` line). Otherwise it closes.
    this.querySelector('[data-x]').onclick = () => {
      if (d.area && !this.isLive()) return this.run(d.area.close);
      this.dispatchEvent(new CustomEvent('close', { bubbles: true }));
    };
    this.wireLeft();
    if (d.area) {
      this.treeMode ||= d.area.tree[0]?.id;
      this.wireSide();
      this.drawSide();
      for (const b of this.querySelectorAll('[data-kind]')) b.onclick = () => { if (!b.classList.contains('on')) this.run(`Menu "${b.dataset.kind}"`); };
    }
    for (const b of this.querySelectorAll('[data-tab]')) b.onclick = () => {
      if (b.classList.contains('on')) return;
      this.dispatchEvent(new CustomEvent('tab', { detail: { tab: b.dataset.tab }, bubbles: true }));
      this.setTab(b.dataset.tab);
    };
    const host = this.querySelector('[data-content]');
    if (this.drill && this.drill.tab === t.name) {
      host.append(this.drill.el);
    } else if (gd && view === 'map') {
      // The DMX map (§34 round 3, 12): the area feed's runs and conflicts; its gestures are lines.
      const m = document.createElement('luma-dmxmap');
      host.append(m);
      this.backOnTab = false;
      this.drawMap(m, t);
      m.addEventListener('line', e => { e.stopPropagation(); this.run(e.detail.line); });
      m.addEventListener('select', e => this.mapSelected(e.detail.fixtures));
      // Zoom in: Edit Patch on that universe with the map's selection.
      m.addEventListener('zoom', e => {
        const nos = new Set(m.selection());
        const rows = nos.size ? this.rowsOf('fixtures').filter(r => nos.has(Number(r.no))) : this.patchSelection();
        this.editPatch(rows, { universe: e.detail.universe });
      });
      m.addEventListener('context', e => { e.stopPropagation(); this.mapContext(e.detail.fixtures, { x: e.detail.x, y: e.detail.y }); });
      for (const b of this.querySelectorAll('[data-view]')) b.onclick = () => { this.views[gd.key] = b.dataset.view; this.render(); };
    } else if (gd && this.isReview() && gd.rows === 'fixtures') {
      // The review (decisions §34 "Import MVR hardened" 3): the file's rows with their status,
      // the taken cells ticked, the selection = the ticks (Take rows pre-selected).
      const g = document.createElement('luma-grid');
      host.append(g);
      this.drawReview(g, t, gd);
      g.addEventListener('line', e => e.stopPropagation());
      g.addEventListener('select', () => { this.kept[gd.key] = g.selection().rows || []; this.drawSlab(); });
      g.addEventListener('context', e => { e.preventDefault(); });
    } else if (gd) {
      let g;
      if (c.drives) {
        // On the drives (decisions §38 5): the kit's files element hosts the grid — its rail
        // picks the drive the tab's feed and lines work on, a double-tap is Load…
        const lf = document.createElement('luma-files');
        host.append(lf);
        lf.update({ host: true, feeds: window.feeds?.() || null, kind: 'shows', drive: this.drive });
        g = lf.grid();
        lf.addEventListener('where', e => { e.stopPropagation(); this.drive = e.detail.drive; this.dispatchEvent(new CustomEvent('where', { detail: { ...e.detail, tab: t.name }, bubbles: true })); });
        lf.addEventListener('pick', e => { e.stopPropagation(); this.dispatchEvent(new CustomEvent('action', { detail: { id: 'load', action: { id: 'load' } }, bubbles: true })); });
      } else {
        g = document.createElement('luma-grid');
        host.append(g);
      }
      this.drawGrid(g, t, gd, view, state);
      // The patch list's rows lift onto a sidebar node: one Assign line (§40 40d 3).
      if (this.def.area && gd.rows === 'fixtures' && !this.areaMode().read_only) {
        g.lift = true;
        g.addEventListener('lift', e => { this.lifting = { keys: e.detail.keys, text: `${e.detail.keys.length} fixture${e.detail.keys.length === 1 ? '' : 's'}` }; this.dragChip(e.detail.x, e.detail.y); });
        g.addEventListener('liftmove', e => this.dragOver(e.detail.x, e.detail.y));
        g.addEventListener('drop', e => {
          const dest = this.dropTarget(e.detail.x, e.detail.y, e.detail.cancelled);
          const keys = this.lifting?.keys || [];
          this.dragEnd();
          if (!dest?.lines?.at || !keys.length) return;
          this.run(`Assign ${gd.object} ${g.rowsWordFor(keys)} At ${dest.lines.at}`);
        });
      }
      g.addEventListener('line', e => { e.stopPropagation(); this.gridLine(gd, e.detail.line); });
      g.addEventListener('select', () => { this.kept[gd.key] = g.selection().rows || []; this.drawSlab(); if (view === 'matrix') this.matrixDetail(); });
      g.addEventListener('new', () => this.act('new'));
      g.addEventListener('context', e => { e.preventDefault(); this.gridContext(g, t, gd, e.detail); });
      g.addEventListener('open', e => this.openDetail(t, gd, e.detail.key));
      this.querySelector('[data-filter]')?.addEventListener('click', () => g.toggleFilter());
      this.querySelector('[data-colset]')?.addEventListener('click', e => {
        const now = g.columnSet();
        g.setColumnSet(SETS[(SETS.indexOf(now) + 1) % SETS.length]);
        e.currentTarget.textContent = `Columns: ${g.columnSet()}`;
      });
      for (const b of this.querySelectorAll('[data-view]')) b.onclick = () => { this.views[gd.key] = b.dataset.view; this.render(); };
    } else if (c.kind === 'settings') {
      const s = document.createElement('luma-settings');
      s.setAttribute('embedded', '');
      host.append(s);
      s.update({ title: c.settings, fields: this.fields[c.settings] || [], level: this.level, note: c.note || '', home: t.home || '' });
      s.addEventListener('run', e => { e.stopPropagation(); this.run(e.detail.line); });
    } else if (this.custom[t.name]) {
      host.append(this.custom[t.name]);
    } else {
      host.innerHTML = `<div class="lc-setup-note">${esc(c.note || 'Nothing here yet.')}</div>`;
    }
    this.drawSlab();
    if (gd && view === 'matrix') this.matrixDetail();
    if (reopenTree && this.phone) this.openTreeSheet();
  }

  drawGrid(g, t, gd, view, state) {
    const ex = this.extra[t.name] || {};
    const empty = this.def.left && !this.leftRows?.length ? this.def.left.empty : ex.empty || '';
    // The matrix is the same rows with the kind columns (read-only).
    const matrix = view === 'matrix' && gd.matrix;
    g.update({
      key: matrix ? `${gd.key}.matrix` : gd.key, object: ex.owner && gd.object ? `${gd.object} ${this.ownerWord()}` : gd.object, id: gd.id || 'no', row_key: gd.row_key || 'guid',
      columns: matrix ? gd.matrix : this.columnsOf(t, gd), rows: this.shownRows(t, gd), new_line: matrix ? null : gd.new_line || null,
      hardware_note: gd.hardware ? `No new-object line: ${gd.hardware.replace(/^./, ch => ch.toLowerCase())}` : '',
      state: matrix ? {} : state, level: this.level, view: matrix ? 'grid' : view, caption: ex.caption || '', marks: ex.marks || {}, empty, home: t.home || '',
      open: !matrix && !!gd.detail,
      // A phone: the selection bar folds into the grid's header (§37 1).
      bar: this.phone ? 'head' : true,
    });
    g.lassoLatch = !!this.lassoLatch;
    // A selection the host asks for (Library: the loaded show), once.
    if (ex.select?.length && !g.selection().count) { g.select(ex.select.map(String)); ex.select = null; }
    // Back on a tab: what was selected when it was left (a view switch within the tab clears, as before).
    else if (this.backOnTab && this.kept[gd.key]?.length && !g.selection().count) g.select(this.kept[gd.key]);
    this.backOnTab = false;
  }

  /** The review grid: one row per file fixture (and lost draft fixture), its cells as the file
   *  would set them — ✓ where taken, the live value in brackets where it differs. */
  drawReview(g, t, gd) {
    const r = this.areaData?.review || { rows: [] };
    // A cell's text: `take · <file value> (was <live>)`, `clash` where it can't be taken.
    const cellText = c => (c ? `${c.taken ? 'take · ' : ''}${c.file}${c.differs && c.live != null && c.live !== '' ? ` (was ${c.live})` : ''}${c.clash ? ' · clash' : ''}` : '');
    const tone = s => ({ Changed: 'warn', New: 'ok', Lost: 'alert', Clash: 'alert', Outside: 'warn' }[s] || 'off');
    const rows = (r.rows || []).map(row => {
      const cell = f => row.cells.find(c => c.field === f);
      return {
        key: row.no ? String(row.no) : `new:${row.uuid}`, no: row.no || row.id || '—',
        status: row.status + (row.match_key ? ` · ${row.match_key}` : ''), status_tone: tone(row.status),
        name: row.status === 'Lost' ? row.name : cellText(cell('name')),
        type: cellText(cell('type')), address: cellText(cell('address')), pos: cellText(cell('pos')), rot: cellText(cell('rot')),
        group: row.group_name + (row.group_missing ? ' (gone)' : ''), layer: row.layer_name,
        clash: row.clash || '', taken: row.taken,
      };
    });
    const col = (id, label, field, width, priority = 1, extra = {}) => ({ id, label, field, width, priority, readonly: true, ...extra });
    const columns = [
      col('status', 'Status', 'status', 130, 1, { tone: 'status_tone' }),
      col('name', 'Name', 'name', 180), col('type', 'Type · Mode', 'type', 200, 2), col('address', 'Address', 'address', 130),
      col('pos', 'Position (m)', 'pos', 170, 2), col('rot', 'Rotation (°)', 'rot', 150, 3),
      col('group', 'MVR group', 'group', 150, 2), col('layer', 'Layer', 'layer', 100, 3), col('clash', 'Clash', 'clash', 180, 3),
    ];
    const c = r.counts || {};
    const caption = `${r.file}${r.scope ? ` · ${r.scope}` : ''} · nothing written yet · ${(r.rows || []).filter(x => x.taken).length} ticked`;
    g.update({ key: `${gd.key}.review`, id: 'no', row_key: 'key', columns, rows, state: {}, level: this.level, caption, marks: {}, empty: 'The file has nothing in this scope.', bar: this.phone ? 'head' : true });
    // The ticks: Take rows pre-selected once per review.
    const stamp = `${r.file}:${(r.rows || []).length}`;
    if (this.reviewStamp !== stamp) {
      this.reviewStamp = stamp;
      g.select(rows.filter(x => x.taken).map(x => x.key));
    }
    void c;
  }

  /** Take cell / Offer cell: the field, picked from the review's cells. */
  async pickCell(action) {
    const fields = [['Name', 'name'], ['Address', 'address'], ['Position', 'pos'], ['Rotation', 'rot'], ['Type', 'type'], ['Unit', 'unit'], ['CustomId', 'custom_id']];
    const items = fields.map(([label], i) => ({ no: i + 1, name: label }));
    const got = await openPick({ title: action.label, pick: { kind: 'Cell', items, plain: true } });
    if (!got?.no) return;
    this.run(this.fill(action.line).replace('{value}', `"${fields[got.no - 1][0]}"`));
  }

  /** The Import MVR dialog (decisions §34 "Import MVR hardened" 1): File, then Options, then
   *  the review in this grid. */
  mvrImport() {
    const tree = (this.areaData?.tree || []).filter(n => n.lines?.at);
    openMvrImport({
      feeds: window.feeds?.() || null, level: this.level,
      groups: tree.map(n => ({ at: n.lines.at, name: n.name })),
      classes: (this.areaData?.classes || []).filter(c => c.at).map(c => ({ at: c.at, name: c.name })),
      run: line => this.run(line),
    });
  }

  /** The DMX map's data: the tab's universes and the feed's runs and conflicts. */
  drawMap(m, t) {
    const d = this.areaData || {};
    const sel = this.patchSelection();
    const offset = Number(this.patchState()?.offset) || 0;
    const every = offset && sel.length ? ` /Every ${Math.max(1, ...sel.map(r => Number(r.footprint) || 1)) + offset}` : '';
    m.update({ universes: this.rows[t.name] || [], runs: d.runs || [], conflicts: d.conflicts || [], level: this.level, zoomable: true, layout: 'spacing', selection: sel, every });
  }
  /** The Patch list's selected fixture rows (its grid, or what was selected when its tab was left). */
  patchSelection() {
    const t = (this.def.tabs || []).find(x => x.content?.grid?.rows === 'fixtures');
    if (!t) return [];
    const gd = t.content.grid;
    const g = this.tab() === t ? this.grid() : null;
    const keys = g ? g.selection().rows || [] : this.kept[gd.key] || [];
    const rows = this.rows[t.name] || [];
    const keyOf = r => String(r[gd.row_key || gd.id || 'no'] ?? r.no);
    const by = new Map(rows.map(r => [keyOf(r), r]));
    return keys.map(k => by.get(String(k))).filter(Boolean);
  }
  patchState() {
    const gd = (this.def.tabs || []).find(x => x.content?.grid?.rows === 'fixtures')?.content.grid;
    return gd ? this.grids[gd.key] : null;
  }
  /** The context menu on the map's fixtures (a hold on a block or run): the Patch list's menu. */
  async mapContext(nos, at) {
    const t = (this.def.tabs || []).find(x => x.content?.grid?.rows === 'fixtures');
    if (!t || !nos.length) return;
    const gd = t.content.grid;
    const menu = (await objectMenus())[gd.object];
    const rows = (this.rows[t.name] || []).filter(r => nos.includes(Number(r.no)));
    if (!menu || !rows.length) return;
    const cols = this.columnsOf(t, gd).filter(c => c.prop && !c.expand);
    const live = this.def.area ? (this.isLive() ? 'Live patch — live columns only' : 'Draft · claimed by you') : '';
    // Renumber… / Address… preview in the Patch list: it opens with these rows selected.
    const inList = kind => { this.kept[gd.key] = rows.map(r => String(r[gd.row_key || gd.id || 'no'] ?? r.no)); this.setTab(t.name); const g = this.grid(); return g ? this.valueDialog(g, gd, kind) : false; };
    openContext({
      at, menu, object: gd.object, rows: rowsWord(nos), count: rows.length,
      props: cols.map(c => c.prop), labels: Object.fromEntries(cols.map(c => [c.prop, c.label])),
      value: prop => { const c = cols.find(x => String(x.prop).toLowerCase() === String(prop).toLowerCase()); return c ? rows.map(r => cellText(c, r[c.field ?? c.id])) : null; },
      claim: this.areaWhy?.() || '', note: live, level: this.level,
      run: line => this.run(line), edit: () => inList(null) || this.grid()?.edit(),
      valueDialog: kind => (kind === 'editpatch' ? this.editPatch(rows) : inList(kind)),
    });
  }
  /** The rows a grid of this menu shows, by its feed rows' name (`fixtures`, `universes`). */
  rowsOf(name) {
    const t = (this.def.tabs || []).find(x => x.content?.grid?.rows === name);
    return (t && this.rows[t.name]) || [];
  }
  /** Edit Patch (kit/editpatch.js) on `rows` (fixture rows): the DMX map zoomed in. */
  editPatch(rows, opts = {}) {
    const fix = r => ({ no: Number(r.no), name: r.name, address: r.address, footprint: r.footprint });
    const patchGrid = (this.def.tabs || []).find(x => x.content?.grid?.rows === 'fixtures')?.content.grid;
    this.ep = openEditPatch({
      fixtures: rows.map(fix), universes: this.rowsOf('universes'), runs: this.areaData?.runs || [],
      universe: opts.universe, typed: opts.typed, level: this.level,
      offset: Number((patchGrid && this.grids[patchGrid.key]?.offset) ?? this.grid()?.spec.state?.offset) || 0,
      run: line => this.run(line), closed: () => { this.ep = null; },
      context: (nos, at) => this.mapContext(nos, at),
    });
    this.epNos = rows.map(r => Number(r.no));
    return true;
  }
  /** A block tapped in the map: the last line names the fixtures its lines would take. */
  mapSelected(nos) {
    const el = this.querySelector('[data-last]');
    if (el && !this.last) el.textContent = nos.length ? `lines name their fixtures: Fixture ${rowsWord(nos)}` : '';
  }

  /** The matrix: what the tapped cell holds goes to the last line. */
  matrixDetail() {
    const g = this.grid();
    const gd = this.gridDef();
    const sel = g?.selection();
    const el = this.querySelector('[data-last]');
    if (!g || !gd?.matrix || !el) return;
    const col = gd.matrix.find(c => c.id === sel?.columns?.[0] && c.detail);
    const r = sel?.rows?.length ? g.rowByKey(sel.rows[0]) : null;
    const when = r ? r[gd.matrix[0].field] : '';
    el.textContent = !r ? 'Tap a cell: which objects changed in that save (+ added · ~ changed · − removed), compared by GUID with the save before.'
      : col ? `${when} · ${col.label}: ${r[col.detail] || 'no changes'}` : `${when} · ${r.changes || ''}`;
  }

  /** A state line from the grid: kept here for the next tab switch, then run. */
  gridLine(gd, line) {
    const m = /^(?:Set GridColumns "[^"]*" "(\w+)"\s*(.*)|Reset GridColumns "[^"]*"(?: "\w+")?)$/.exec(line);
    if (m) {
      const st = { ...(this.grids[gd.key] || {}) };
      const g = this.grid();
      Object.assign(st, g?.spec?.state || {});
      this.grids[gd.key] = st;
    }
    this.run(line);
  }

  drawSlab() {
    const slab = this.querySelector('luma-slab');
    if (!slab) return;
    // A row's detail has its own buttons; the grid's slab waits.
    const drilled = !!(this.drill && this.drill.tab === this.tab()?.name);
    slab.style.display = drilled ? 'none' : '';
    if (drilled) return;
    const t = this.tab() || {};
    const g = this.grid();
    const sel = g?.selection();
    const power = this.def.power;
    let p = null;
    if (this.def.area) {
      // The claim button (decisions §34 "34g4 patch entry hardened", the locked patch-lock.html):
      // Live — a hold claims; Claimed (yours, gold) — a press leaves; another's — greyed.
      // Round 3, 9: bottom-right, a gold fill over the 0.6 s hold, the open lock in Live.
      const holder = this.areaData?.holder || '';
      if (!this.isLive()) p = { state: 'draft', word: 'Claimed', icon: 'lock', note: 'Draft · you', level: this.level, claim: true };
      else if (holder) p = { state: 'theirs', word: `Claimed · ${holder}`, icon: 'lock', note: `${holder}’s draft`, off: `${holder} has claimed it. Only ${holder} ends the draft (an Admin can use /Force on the command line).`, level: this.level, claim: true };
      else p = { state: 'on', word: 'Live', icon: 'lock-open', note: 'Hold to claim and edit', hold: 'Hold to claim and edit.', hold_ms: 600, level: this.level, claim: true };
    } else if (power) {
      const st = this.power?.state || 'off';
      const look = st === 'failing' ? { ...(power.on || {}), ...(power.failing || {}) } : (st === 'on' ? power.on : power.off) || {};
      p = { state: st, word: look.word, press: look.press, reason: this.power?.reason, rights: power.rights, level: this.level, label: power.label };
    }
    // A locked row selected (UserProfile Default): its row actions are off.
    const gdef = this.gridDef();
    const selRows = (sel?.rows || []).map(k => g?.rowByKey(k)).filter(Boolean);
    const locked = gdef?.locked && selRows.some(r => r[gdef.locked.field]) ? gdef.locked.why : '';
    const edit = { ...(t.slab || {}) };
    if (locked) for (const id of ['cut', 'delete', 'edit']) if (edit[id]) edit[id] = { ...edit[id], off: locked };
    // Slab states in a claim area (§34 round 3, 18): in Live (or another's draft) what makes,
    // copies or removes rows needs the claim — a copied fixture is a new fixture; Select, Oops,
    // Export and Edit (live fields) work. Paste counts the station's clipboard.
    let clip = this.clip;
    const review = this.isReview();
    if (this.def.area) {
      const why = this.areaWhy();
      if (why) for (const id of ['new', 'copy', 'cut', 'delete', 'import', 'paste']) if (edit[id]) edit[id] = { ...edit[id], off: edit[id].off || why };
      // The review: the standard cells wait for Apply or Cancel (Select and Oops work).
      if (review) for (const id of ['new', 'copy', 'cut', 'delete', 'import', 'paste', 'export', 'edit']) if (edit[id]) edit[id] = { ...edit[id], off: 'Apply or cancel the import first' };
      const c = this.areaData?.clipboard;
      const mineKind = c && String(c.object).toLowerCase() === String(gdef?.object || '').toLowerCase();
      clip = mineKind ? { op: c.cut ? 'cut' : 'copy', count: c.count, station: true } : null;
      if (!clip && edit.paste && !edit.paste.off) edit.paste = { ...edit.paste, off: 'The clipboard is empty' };
    }
    // Your own row (your password): no level needed.
    const me = String(this.viewer || '').toLowerCase();
    const draftWhy = this.areaWhy();
    // The review's own cells show in the review, the others hide; an action of value `import`
    // only marks the Import cell as the page's (never a cell of its own).
    const actions = (t.actions || []).filter(a => (a.when === 'review') === review && a.value !== 'import').map(a => {
      if (locked && a.needs) return { ...a, off: locked };
      // A function block action that needs the draft (Insert fixtures…): greyed in Live.
      if (a.claim && draftWhy) return { ...a, off: draftWhy };
      // The view cells show what they hold: the sort column, the spare slots.
      if (a.value === 'sort') {
        const col = (g?.columns?.() || []).find(c => c.id === g?.spec.state?.sort?.col);
        return { ...a, label: `Sort · ${col?.label || 'none'}` };
      }
      if (a.value === 'offset') return { ...a, label: `Offset · ${Number(g?.spec.state?.offset) || 0}` };
      if (a.mine && selRows.length === 1 && String(selRows[0][a.mine] ?? '').toLowerCase() === me) return { ...a, rights: null };
      return a;
    });
    // A phone's lock sheet says what the state means (the area's bar).
    if (p && this.def.area?.bars?.length) {
      const holder = this.areaData?.holder || '';
      const bar = this.def.area.bars[!this.isLive() ? 1 : holder ? 2 : 0];
      const fillH = x => String(x || '').replaceAll('{holder}', holder);
      p.explain = { title: fillH(bar.title), text: fillH(bar.text) };
    }
    const named = sel?.count && gdef ? `${gdef.object || ''} ${g.rowsWordFor(sel.rows)}`.trim() : '';
    slab.update({
      bar: this.phone, label: t.label || t.name,
      title: [named || 'Nothing selected', this.def.area ? (review ? 'Import review' : this.isLive() ? 'Live' : 'Draft') : ''].filter(Boolean).join(' · '),
      edit, actions, phone: t.phone || [], level: this.level,
      bound: (t.bound || []).map(b => this.boundOf(b, g, sel)),
      power: p, count: sel?.count || 0, clip, lasso: !!this.lassoLatch,
    });
    if (!slab.wired) {
      slab.wired = true;
      slab.addEventListener('act', e => this.act(e.detail.id, e.detail.action, e.detail.own));
      slab.addEventListener('power', e => this.pressPower(e.detail));
      slab.addEventListener('bound', e => this.tapBound(e.detail.bound));
    }
    const last = this.querySelector('[data-last]');
    if (last && !this.last && g && this.views[this.gridDef()?.key] !== 'matrix') last.textContent = sel?.count ? `lines name their rows: ${this.gridDef()?.object || ''} ${g.rowsWordFor(sel.rows)}` : '';
  }

  /** A bound cell: a station setting (from `EditSetting`), or — when it
   *  names the grid's object — the selected rows' value (one line for them;
   *  DMX Protocols' Rate / Send / Sync / Packet gap of the selected lines). */
  boundOf(b, g, sel) {
    const gd = this.gridDef();
    const same = gd?.object && String(b.object).toLowerCase() === String(gd.object).toLowerCase();
    if (!same) return { ...b, ...(this.boundValues[`${b.object}.${b.prop}`] || {}) };
    const col = this.columnsOf(this.tab(), gd).find(c => String(c.prop || '').toLowerCase() === String(b.prop).toLowerCase());
    const rows = (sel?.rows || []).map(k => g.rowByKey(k)).filter(r => r && (r.object || gd.object) === b.object);
    if (!col || !rows.length) return { ...b, value: '—', line: null, off: `Select ${String(b.object).toLowerCase()} rows first` };
    const vals = rows.map(r => r[col.field ?? col.id]).filter(v => v !== undefined);
    const shown = !vals.length ? '—' : vals.every(v => String(v) === String(vals[0])) ? cellText(col, vals[0]) : 'Mixed';
    const input = col.kind === 'int' ? { type: 'number', min: col.min, max: col.max, named: [] }
      : col.kind === 'bool' ? { type: 'toggle' } : col.kind === 'list' ? { type: 'choice', options: col.options || [] } : { type: 'text' };
    const rowsText = rowsWord(rows.map(r => r[gd.id || 'no']));
    return { ...b, value: shown, input, line: `Set ${b.object} ${rowsText} ${b.prop} {value}`, rights: col.rights || b.rights };
  }

  /** The one context menu on the grid's selection (§34 round 3, 18): the object's menu from the
   *  registry, the Set part limited to the grid's columns with the rows' values, the claim area's
   *  reason on what Live refuses. */
  async gridContext(g, t, gd, at) {
    const menus = await objectMenus();
    const sel = g.selection();
    const rows = sel.rows.map(k => g.rowByKey(k)).filter(Boolean);
    const objects = new Set(rows.map(r => r.object || gd.object));
    const menu = menus[gd.object];
    // Rows of several objects (DMX lines: Output and Input) have no one menu: the edit card.
    if (!menu || objects.size > 1 || !rows.length) { g.edit(); return; }
    const cols = this.columnsOf(t, gd).filter(c => c.prop && !c.expand);
    const live = this.def.area ? (this.isLive() ? 'Live patch — live columns only' : 'Draft · claimed by you') : '';
    openContext({
      at, menu, object: gd.object, rows: g.rowsWordFor(sel.rows), count: rows.length,
      props: cols.map(c => c.prop),
      labels: Object.fromEntries(cols.map(c => [c.prop, c.label])),
      value: prop => {
        const c = cols.find(x => String(x.prop).toLowerCase() === String(prop).toLowerCase());
        return c ? rows.map(r => cellText(c, r[c.field ?? c.id])) : null;
      },
      claim: this.areaWhy?.() || '', note: live, level: this.level,
      run: line => this.run(line), edit: () => g.edit(),
      valueDialog: kind => this.valueDialog(g, gd, kind),
      // A phone's sheet: the first selected row scrolls above it.
      // A short list gets room under it while the sheet is open.
      reveal: top => {
        const row = g.querySelector('.lc-grid-row.touched');
        const sc = g.querySelector('[data-scroll]');
        if (!row || !sc) return null;
        const over = row.getBoundingClientRect().bottom - (top - 8);
        if (over <= 0) return null;
        const pad = sc.style.paddingBottom;
        sc.style.paddingBottom = `${sc.getBoundingClientRect().bottom - top + 8}px`;
        sc.scrollTop += over;
        return () => { sc.style.paddingBottom = pad; };
      },
    });
  }

  /** The value dialog on the grid's selection (§34 round 3, 18): Renumber (`number`) or Address
   *  (`address`), previewing old → new in the grid; Apply runs the one line. */
  valueDialog(g, gd, kind, opts = {}) {
    if (kind === 'editpatch') return this.editPatch(g.pickOrder().map(k => g.rowByKey(k)).filter(Boolean)) || true;
    // Move into… (§40 40d 3): a pick of the tree's nodes, then the one Assign line.
    if (kind === 'node') {
      const dests = Object.values(this.treeRows).filter(r => r.lines?.at);
      const sel = g.selection();
      openPick({ title: `Move ${sel.rows.length} into`, pick: { kind: 'MvrGroup', items: dests.map((r, i) => ({ no: i + 1, name: `${r.lines.at} · ${r.name || 'no name'}` })) } })
        .then(got => { const r = got?.no != null ? dests[got.no - 1] : null; if (r) this.run(`Assign ${gd.object} ${g.rowsWordFor(sel.rows)} At ${r.lines.at}`); });
      return true;
    }
    const all = this.rows[this.tab()?.name] || [];
    const keyOf = r => g.keyOf(r);
    const order = g.pickOrder();
    const picked = order.map(k => g.rowByKey(k)).filter(Boolean);
    const set = new Set(order);
    const grid = g.shownRows().filter(r => set.has(keyOf(r)));
    const keyById = new Map(all.map(r => [r[gd.id || 'no'], keyOf(r)]));
    const sub = `${picked.length} ${picked.length === 1 ? gd.object : `${gd.object}s`} · ${g.rowsWordFor(order)}`;
    const avoid = () => {
      const on = [...g.querySelectorAll('.lc-grid-row.touched')].map(e => e.getBoundingClientRect());
      if (!on.length) return g.getBoundingClientRect();
      return { left: Math.min(...on.map(r => r.left)) + 40, right: Math.max(...on.map(r => r.right)), top: Math.min(...on.map(r => r.top)), bottom: Math.max(...on.map(r => r.bottom)) };
    };
    if (kind === 'number') {
      const id = r => Number(r[gd.id || 'no']);
      const col = g.shows(gd.id || 'no') ? (gd.id || 'no') : '#gutter';
      openValue({
        title: 'Renumber', sub, avoid: avoid(),
        kind: renumberKind({ object: gd.object, rows: picked.map(id), grid: grid.map(id), taken: all.filter(r => !set.has(keyOf(r))).map(id) }),
        preview: p => g.setPreview(p && new Map([...p.to].filter(([a, b]) => a !== b).map(([a, b]) => [keyById.get(a), { [col]: { from: a, to: b } }])),
          p && new Set([...p.way].map(n => keyById.get(n)).filter(Boolean))),
        apply: line => this.run(line),
      });
      return true;
    }
    // `copies` (Multipatch…): new copies at an address — the address kind, starting nowhere.
    if (kind === 'copies') opts = { ...opts, copies: true };
    if (kind === 'address' || kind === 'copies') {
      const run = r => {
        const m = String(r.address || '').match(/^(\d+)\.(\d+)$/);
        return { id: Number(r[gd.id || 'no']), u: m ? +m[1] : 0, s: m ? +m[2] : 0, len: Number(r.footprint) || 1 };
      };
      const a3 = n => String(n).padStart(3, '0');
      openValue({
        title: opts.copies ? 'Multipatch at' : 'Address', sub, avoid: avoid(),
        // New copies (Multipatch…) start nowhere, and their fixtures' own slots are taken.
        kind: addressKind({
          object: gd.object,
          rows: opts.copies ? picked.map(r => ({ ...run(r), u: 0, s: 0 })) : picked.map(run),
          others: (opts.copies ? all : all.filter(r => !set.has(keyOf(r)))).map(run).filter(o => o.u),
          template: opts.line, offset: Number(g.spec.state?.offset) || 0,
        }),
        // Slot grid…: the same fixtures in Edit Patch, what was typed kept (§34 round 3).
        more: opts.copies ? null : { label: 'Slot grid…', icon: 'grid-3x3', run: typed => this.editPatch(picked, { typed }) },
        // Copies (Multipatch…) land beside their fixtures: nothing in the grid moves.
        preview: opts.copies ? null : p => g.setPreview(p && new Map(picked.map(r => {
          const to = p.to.get(Number(r[gd.id || 'no']));
          return [keyOf(r), { address: { from: r.address || '—', to: `${to.u}.${a3(to.s)}`, bad: p.bad.has(Number(r[gd.id || 'no'])) } }];
        })), p && new Set([...p.way].map(n => keyById.get(n)).filter(Boolean))),
        apply: line => this.run(line),
      });
      return true;
    }
    return false;
  }

  act(id, action, own) {
    const t = this.tab() || {};
    const g = this.grid();
    if (own) {
      // Insert fixtures… (round 3 4): its own two-step dialog writes the line.
      if (action?.value === 'insert') return this.insertDialog();
      // Take / Offer cell (the review, decisions §34 "Import MVR hardened" 3): which cell, picked.
      if (action?.value === 'takecell' || action?.value === 'offercell') return this.pickCell(action);
      // Edit Patch… (§34 round 3, the ways into Edit Patch): the DMX map zoomed in on the selection.
      if (action?.value === 'editpatch') return this.editPatch(g ? g.pickOrder().map(k => g.rowByKey(k)).filter(Boolean) : []);
      // The view cells: the grid's order, the spare slots its DMX actions leave.
      if (action?.value === 'sort') return this.pickSort(g);
      if (action?.value === 'offset') return this.askOffset(g);
      if (!action?.line) { this.dispatchEvent(new CustomEvent('action', { detail: { id, action }, bubbles: true })); return; }
      // `{value}`: what the action needs is asked first — the value dialog for its kind
      // (Patch to… = an address), else the kit's dialog.
      if (action.line.includes('{value}')) {
        if (action.value && g && this.valueDialog(g, this.gridDef(), action.value, { line: action.line })) return;
        return this.askValue(action);
      }
      return this.run(this.fill(action.line) + (action.value === 'packed' ? this.everyWord(g) : ''));
    }
    const d = (t.slab || {})[id] || {};
    const sel = g?.selection();
    // The Import cell of a tab whose own action imports (Patch): the Import MVR dialog.
    if (id === 'import' && (t.actions || []).some(a => a.value === 'import')) return this.mvrImport();
    // The host may take a standard action (Backup › Library: Import = the
    // Load show dialog's parts).
    const taken = !this.dispatchEvent(new CustomEvent('slab-act', { detail: { id, tab: t.name }, bubbles: true, cancelable: true }));
    if (taken) return;
    switch (id) {
      case 'select':
        if (d.line) return this.run(this.fill(d.line));
        // No select line here: Select latches the lasso (one finger draws it).
        this.lassoLatch = !this.lassoLatch;
        if (g) g.lassoLatch = this.lassoLatch;
        toast(this.lassoLatch ? 'Select: one finger draws a lasso' : 'Select off');
        return this.drawSlab();
      case 'edit': return g?.edit();
      case 'copy': case 'cut':
        if (!sel?.count) return;
        // On the object door the station holds the clipboard (decisions §35 9): the line puts
        // the rows there; the page only remembers that Paste has something to place.
        if (d.line && !d.line.includes('{at}')) {
          this.run(this.fill(d.line));
          this.clip = { op: id, count: sel.count, station: true };
          g?.setCut([]);
          return this.drawSlab();
        }
        this.clip = { op: id, count: sel.count, rows: g.rowsWordFor(sel.rows), keys: sel.rows, line: d.line };
        g.setCut(id === 'cut' ? sel.rows : []);
        toast(`${sel.count} ${sel.count === 1 ? 'row' : 'rows'} ${id === 'cut' ? 'cut' : 'copied'} — select where it goes, then Paste`);
        return this.drawSlab();
      case 'paste': {
        if (d.line?.startsWith('Paste')) {
          const at = sel?.ids?.[0];
          this.run(at == null ? this.fill('Paste') : this.fill(d.line, { at }));
          if (this.clip?.op === 'cut') this.clip = null;
          return this.drawSlab();
        }
        if (!this.clip) return;
        const at = sel?.ids?.[0];
        if (at == null) { toast('Select where to paste first'); return; }
        const line = this.fill(this.clip.line || (this.clip.op === 'cut' ? 'Move {object} {rows} At {at}' : 'Copy {object} {rows} At {at}'), { rows: this.clip.rows, at });
        if (this.clip.op === 'cut') { this.clip = null; g?.setCut([]); }
        this.run(line);
        return this.drawSlab();
      }
      default: {
        // A tab whose function block inserts (Insert fixtures…): New is that dialog (round 3, 18).
        const ins = id === 'new' && (t.actions || []).find(a => a.value === 'insert');
        if (ins) return this.act(ins.id, ins, true);
      }
        // New with `{value}` (`Store Network {value}`): the name is asked first.
        if (d.line?.includes('{value}')) return this.askValue({ label: `New ${this.gridDef()?.object || ''}`.trim(), line: d.line });
        if (d.line) this.run(this.fill(d.line));
    }
  }

  /** The Offset cell's spare slots as `/Every`: after the selection's largest footprint ('' when 0). */
  everyWord(g) {
    const offset = Number(g?.spec.state?.offset) || 0;
    if (!offset) return '';
    const rows = (g.selection().rows || []).map(k => g.rowByKey(k)).filter(Boolean);
    return ` /Every ${Math.max(1, ...rows.map(r => Number(r.footprint) || 1)) + offset}`;
  }
  /** The Sort cell: the column the grid sorts by. */
  async pickSort(g) {
    if (!g) return;
    const cols = g.columns();
    const now = g.spec.state?.sort?.col;
    const got = await openPick({ title: 'Sort by', value: now, pick: { kind: 'Column', plain: true, items: cols.map((c, i) => ({ no: i + 1, name: c.label })) } });
    if (got?.no) { g.sortBy(cols[got.no - 1].id); this.drawSlab(); }
  }
  /** The Offset cell: spare slots after each fixture (0 = packed). */
  async askOffset(g) {
    if (!g) return;
    const got = await askField({ name: 'Offset', value: String(g.spec.state?.offset || 0), input: { type: 'number', min: 0, max: 512, named: [] }, hint: 'Spare DMX slots after each fixture when Patch to…, Next free address, Multipatch… and Insert pack a selection' });
    if (got == null || typeof got === 'object') return;
    g.setState('Offset', Math.max(0, Math.min(512, parseInt(got, 10) || 0)));
    this.drawSlab();
  }

  /** Insert fixtures… on the area's grid: its rows (taken numbers and addresses) and the tree's
   *  nodes (the MVR group a block goes into). */
  insertDialog() {
    const tree = (this.areaData?.tree || []).filter(n => n.lines?.at);
    // Under the picked node (§40 40d, suggestion 6): the pick is the Place step's default.
    const picked = this.treeRows[this.treeNode];
    openInsert({
      rows: this.rows[this.tab()?.name] || [], groups: tree.map(n => ({ at: n.lines.at, name: n.name })),
      group: picked?.lines?.at ? { at: picked.lines.at, name: picked.name } : null,
      classes: (this.areaData?.classes || []).filter(c => c.at).map(c => ({ at: c.at, name: c.name })),
      offset: Number(this.grid()?.spec.state?.offset) || 0,
      level: this.level, run: line => this.run(line),
    });
  }

  async askValue(action) {
    const label = String(action.label || '').replace(/…$/, '');
    const got = await openDialog({
      title: label, text: this.fill(action.line.replace('{value}', '…')),
      inputs: [{ name: 'v', label, value: '', kind: 'text' }],
      buttons: [{ label: 'Please', value: 'go', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
    });
    const v = String(got?.inputs?.v ?? '').trim();
    if (!got?.value || !v) return;
    // A name is quoted text (decisions §33: a bare word is a word); a number stays bare.
    this.run(this.fill(action.line).replace('{value}', /^[0-9.+\-*/]+$/.test(v) ? v : `"${v.replace(/[";]/g, '')}"`));
  }

  async tapBound(b) {
    if (!b?.line) return;
    const f = { name: b.label, value: b.value ?? '', input: b.input || { type: 'text' }, line: b.line, pick: b.pick };
    const got = await askField(f);
    if (got == null || typeof got === 'object') return;
    this.run(b.line.replace('{value}', valueWord(f, got)));
  }

  pressPower(how) {
    if (this.def.area) {
      if (how?.hold && this.isLive() && !this.areaData?.holder) this.run(this.def.area.claim);
      else if (!how?.hold && !this.isLive()) this.run(this.def.area.exit);
      return;
    }
    const p = this.def.power;
    if (!p) return;
    const st = this.power?.state || 'off';
    const look = st === 'off' ? p.off : p.on;
    if (!look?.line) return;
    // `{a.b}`: a field of the power's feed (the LumaSession to join).
    const data = this.power?.data || {};
    this.run(look.line.replace(/\{([a-z_.]+)\}/g, (_, path) => String(path.split('.').reduce((o, k) => o?.[k], data) ?? '').replace(/"/g, '')));
  }

  // ── Data from the host ──
  setRows(tab, rows, extra) {
    this.rows[tab] = rows || [];
    if (extra) this.extra[tab] = { ...(this.extra[tab] || {}), ...extra };
    const t = this.tab();
    if (t?.name !== tab) return;
    this.tryOpen();
    // The drilled row: a new name shows, a row that went closes the detail.
    if (this.drill?.tab === tab) {
      const gd = this.gridDef();
      const row = this.rows[tab].find(r => String(r[gd?.row_key || 'guid']) === this.drill.key);
      if (!row) return this.closeDetail();
      if ((row.name || String(row.no)) !== this.drill.spec.title) { this.drill.spec.title = row.name || String(row.no); this.drill.el.update(this.drill.spec); }
      return;
    }
    const g = this.grid();
    const gd = this.gridDef();
    if (this.def.area) this.drawSide();
    const map = this.querySelector('luma-dmxmap');
    if (map) this.drawMap(map, t);
    if (g && gd && this.isReview() && gd.rows === 'fixtures') {
      // The review owns the Patch grid while it stands (decisions §34 "Import MVR hardened" 3).
      this.drawReview(g, t, gd);
    } else if (g && gd) {
      this.drawGrid(g, t, gd, this.views[gd.key] || g.spec.view, g.spec.state);
      if (this.views[gd.key] === 'matrix') this.matrixDetail();
    }
    this.drawSlab();
  }
  setFields(target, fields) {
    this.fields[target] = fields || [];
    const c = this.tab()?.content;
    if (c?.kind === 'settings' && c.settings === target) {
      this.querySelector('luma-settings')?.update({ title: target, fields: this.fields[target], level: this.level, note: c.note || '' });
    }
  }
  setChips(chips) {
    this.chips = chips || [];
    const title = this.querySelector('.lc-setup-title');
    if (!title) return;
    for (const el of title.querySelectorAll('.lc-setup-chip:not(.btn):not([data-chip="area"])')) el.remove();
    const name = title.querySelector('.lc-setup-name');
    name.insertAdjacentHTML('afterend', this.chips.map(ch => `<span class="lc-setup-chip"${ch.id ? ` data-chip="${esc(ch.id)}"` : ''}>${ch.tone ? `<i class="lc-dot ${esc(ch.tone)}"></i>` : ''}${esc(ch.text)}</span>`).join(''));
  }
  setPower(state) { this.power = state; this.drawSlab(); }
  setBound(values) { this.boundValues = { ...this.boundValues, ...(values || {}) }; this.drawSlab(); }
  setCustom(tab, node) { this.custom[tab] = node; if (this.tab()?.name === tab) this.render(); }
}
customElements.define('luma-setup', LumaSetup);

let current = null;

/** The setup frame over everything (one at a time). */
export function openSetup(def, { tab, level, viewer, grids, run, chips, power, onTab, onClose, onAction, onSlab, onDetail, onWhere } = {}) {
  current?.close();
  const scrim = document.createElement('div');
  scrim.className = 'lc-setup-scrim';
  const el = document.createElement('luma-setup');
  scrim.append(el);
  document.body.append(scrim);
  el.configure({ def, tab, level, viewer, grids, chips, power });
  const close = () => {
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    if (current?.el === el) current = null;
    onClose?.();
  };
  const onKey = e => {
    if (e.key === 'Escape' && !sheetOpen() && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, .lc-card-scrim, [data-kit="dialog"]') && !e.target.closest?.('luma-grid')) {
      e.preventDefault(); e.stopPropagation(); close();
    }
  };
  document.addEventListener('keydown', onKey, true);
  el.addEventListener('line', e => run?.(e.detail.line));
  el.addEventListener('close', close);
  el.addEventListener('tab', e => onTab?.(e.detail.tab));
  el.addEventListener('action', e => onAction?.(e.detail));
  el.addEventListener('slab-act', e => { if (onSlab?.(e.detail)) e.preventDefault(); });
  el.addEventListener('detail', e => onDetail?.(e.detail));
  el.addEventListener('where', e => onWhere?.(e.detail));
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  current = {
    el, def, close,
    setTab: n => el.setTab(n), tab: () => el.tab()?.name,
    setRows: (t, r, x) => el.setRows(t, r, x), setFields: (t, f) => el.setFields(t, f),
    setChips: c => el.setChips(c), setPower: p => el.setPower(p), setBound: v => el.setBound(v),
    setCustom: (t, n) => el.setCustom(t, n), setLeft: r => el.setLeft(r), setArea: x => el.setArea(x), grid: () => el.grid(),
    openRow: (t, w, g, p) => el.openRow(t, w, g, p), setDetail: x => el.setDetail(x), detailInfo: () => el.detailInfo(),
    run: line => el.run(line), fill: t => el.fill(t),
  };
  return current;
}

export function setupOpen() { return current; }
