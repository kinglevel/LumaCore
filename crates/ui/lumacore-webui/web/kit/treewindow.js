// <luma-tree-window> — the Tree window (decisions 0292, the approved mockup
// docs/dev/ui-kit/mockups/2026-10-23-tree-window/01-tree-window.html): the terminal's NAV as a
// window of the web and the app. A path bar on top (<luma-pathbar>: crumbs, type or paste a path
// or a GUID), the tree on the left (<luma-tree> in host mode, the `tree` feed: the same rows as
// `ls` and NAV) and the inspector on the right (<luma-inspector>, the `inspector` feed: identity
// with copy buttons, facts, every property as a <luma-settings> cell, Used by / Links to). Every
// gesture is a line: a value is its `Set` line, ↺ its `Reset`; where the window stands, what is
// open and the search are the window's own settings (`Set Self "Path" …`, `"Open"`, `"Search"`);
// Follow = Selected jumps when the station names a new pick. Copies go through the one
// clipboard door (kit/clipboard.js). The row menu is the registry's one context menu plus the
// copies and "Open in the terminal's NAV". A phone draws the tree only; a tap opens the
// inspector as the one phone sheet.
//
//   el.configure({ follow, show, split, path, open, search, level })   the window's settings
//   el.showTree(feedData) · el.showInspector(feedData)
//   events: run { line } (a line for the station, the window its owner)
//
// <luma-inspector> — one place's inspector (also the phone's sheet):
//   el.update(feedData, { show: 'All' | 'Changed', level })
//   events: run { line } · jump { path } · copy { label, text }

import './tree.js';
import './pathbar.js';
import { valueWord } from './settings.js';
import { openMenu } from './menu.js';
import { objectMenus, openContext } from './context.js';
import { copyText } from './clipboard.js';
import { toast } from './toast.js';
import { phoneMode, onPhoneMode } from './phone.js';
import { openSheet } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** A value as a quoted word of a line. */
const quoted = v => `"${String(v ?? '').replace(/[";]/g, '')}"`;
/** Where a window with no place stands (lumacore_views::tree_window::HOME). */
const HOME = 'Root.ShowData';
/** How long a value that changed under the inspector stays lit (NAV's FLASH). */
const FLASH_MS = 1500;
/** Branches kept in `Self.Open` (the newest; the setting's text has a limit). */
const OPEN_KEEP = 60;
const parentOf = p => { const i = String(p).lastIndexOf('.'); return i < 0 ? 'Root' : p.slice(0, i); };
const copy = (label, text) => copyText(label, text, { host: window.LumaHost || null });
/** The Set lines of a place's changed properties, one line (`; ` between). */
function setLines(insp) {
  return (insp?.rows || []).filter(r => r.field?.changed).map(r => r.field.line.replace('{value}', valueWord(r.field, r.field.value))).join('; ');
}

// ── The inspector ──
class LumaInspector extends HTMLElement {
  constructor() { super(); this.data = null; this.cfg = {}; this.flashes = new Map(); }
  connectedCallback() {
    this.dataset.kit = 'inspector';
    if (!this.built) {
      this.built = true;
      this.addEventListener('click', e => {
        const c = e.target.closest('[data-copy]');
        if (c) { e.stopPropagation(); this.emit('copy', { label: c.dataset.copy, text: c.dataset.text }); return; }
        const g = e.target.closest('[data-go]');
        if (g) { e.stopPropagation(); this.emit('jump', { path: g.dataset.go }); }
      });
    }
    this.render();
  }
  emit(name, detail) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  update(data, cfg = {}) {
    const before = this.data;
    if (data && before && !data.missing && data.path === before.path) {
      const was = new Map((before.rows || []).map(r => [`${r.section}|${r.name}|${r.link || ''}`, r.value]));
      const now = performance.now();
      for (const r of data.rows || []) {
        const k = `${r.section}|${r.name}|${r.link || ''}`;
        if (was.has(k) && was.get(k) !== r.value) this.flashes.set(r.name, now);
      }
    } else if (data?.path !== before?.path) this.flashes.clear();
    this.data = data;
    this.cfg = cfg;
    this.render();
    if (this.flashes.size) {
      clearTimeout(this.fade);
      this.fade = setTimeout(() => {
        const old = performance.now() - FLASH_MS + 50;
        for (const [k, t] of this.flashes) if (t < old) this.flashes.delete(k);
        this.paintFlashes();
      }, FLASH_MS);
    }
  }

  row(r, copyAs) {
    const lit = this.flashes.has(r.name) ? ' lc-flash' : '';
    const value = r.missing
      ? `<span class="lc-insp-link missing">${esc(r.value)}</span>`
      : r.link ? `<button type="button" class="lc-insp-link" data-go="${esc(r.link)}" title="Go to ${esc(r.link)}">${esc(r.value || r.link)}</button>`
        : `<span class="lc-insp-v${copyAs ? ' mono lc-selectable' : ''}">${esc(r.value) || '—'}</span>`;
    const btn = copyAs && r.value ? `<button type="button" class="lc-insp-copy" data-copy="${esc(copyAs)}" data-text="${esc(r.value)}" aria-label="Copy ${esc(copyAs)}" title="Copy ${esc(copyAs)}"><luma-icon name="copy"></luma-icon></button>` : '';
    return `<div class="lc-insp-row${lit}" data-row="${esc(r.name)}"><span class="lc-insp-k">${esc(r.name)}</span>${value}${btn}</div>`;
  }

  render() {
    if (!this.isConnected) return;
    const d = this.data;
    if (!d || d.missing || d.busy) {
      this.innerHTML = `<div class="lc-insp-empty">${esc(d?.missing || (d?.busy ? 'The station is busy — it comes when it can.' : 'Pick a row.'))}</div>`;
      return;
    }
    const rows = d.rows || [];
    const sec = name => `<div class="lc-insp-sec"><span>${esc(name)}</span><i></i></div>`;
    const kind = rows.find(r => r.section === 'identity' && r.name === 'Kind')?.value || '';
    let h = `<div class="lc-insp-title"><span class="lc-insp-name lc-selectable">${esc(d.title)}</span>${d.marks ? `<span class="lc-tree-badge" title="Bookmark ${esc(d.marks)}">${esc(d.marks)}</span>` : ''}${kind ? `<span class="lc-insp-kind">${esc(kind)}</span>` : ''}</div>`;
    h += sec('Identity');
    for (const r of rows.filter(x => x.section === 'identity')) {
      h += this.row(r, r.name === 'GUID' ? 'GUID' : r.name === 'Path' ? 'path' : '');
      if (r.name === 'Kind' && d.target) h += this.row({ name: 'Target', value: d.target }, 'target');
    }
    const info = rows.filter(x => x.section === 'info');
    if (info.length) { h += sec('Facts'); for (const r of info) h += this.row(r); }
    // A cell per value; a field that takes a whole tab (the tag tree) stays a row here (its
    // editor is EditSetting's Tags tab).
    const whole = r => ['tags', 'list'].includes(r.field?.input?.type);
    const props = rows.filter(x => x.section === 'property' && x.field && !whole(x));
    const tabs = rows.filter(x => x.section === 'property' && (!x.field || whole(x)));
    const changed = props.filter(r => r.field.changed).length;
    const shown = this.cfg.show === 'Changed' ? props.filter(r => r.field.changed) : props;
    if (props.length) h += `${sec(`Properties · ${props.length}${changed ? ` · ${changed} changed` : ''}`)}<div class="lc-insp-props"></div>`;
    if (this.cfg.show !== 'Changed') for (const r of tabs) h += this.row(r);
    const links = rows.filter(x => x.section === 'link');
    for (const name of [...new Set(links.map(r => r.name))]) {
      const of = links.filter(r => r.name === name);
      h += sec(`${name} · ${of.length}`);
      for (const r of of) h += this.row({ ...r, name: '' });
    }
    if (!links.length && d.guid) h += `${sec('Used by · 0')}<div class="lc-insp-note">Nothing uses it.</div>`;
    this.innerHTML = `<div class="lc-insp">${h}</div>`;
    if (props.length) {
      if (!this.cells) {
        this.cells = document.createElement('luma-settings');
        this.cells.setAttribute('embedded', '');
      }
      this.querySelector('.lc-insp-props').append(this.cells);
      this.cells.update({ fields: shown.map(r => r.field), level: this.cfg.level || '', note: shown.length ? '' : 'No property differs from its default.' });
    }
    this.paintFlashes();
  }

  /** Light the rows and cells whose value changed under the inspector; the rest go dark. */
  paintFlashes() {
    for (const el of this.querySelectorAll('.lc-flash')) el.classList.remove('lc-flash');
    for (const name of this.flashes.keys()) {
      for (const el of this.querySelectorAll(`[data-row="${CSS.escape(name)}"], .lc-set-cell[data-setting="${CSS.escape(name)}"]`)) el.classList.add('lc-flash');
    }
  }
}
customElements.define('luma-inspector', LumaInspector);

// ── The window ──
class LumaTreeWindow extends HTMLElement {
  constructor() {
    super();
    this.cfg = {};
    this.treeData = null;
    this.insp = null;
    this.pending = null;   // a place picked here, until the station's settings say it
    this.asked = null;     // what the path bar sent: { text, back }
    this.followKey = undefined;
  }

  connectedCallback() {
    this.dataset.kit = 'tree-window';
    if (this.built) return;
    this.built = true;
    this.innerHTML = `<div class="lc-tw">
      <luma-pathbar></luma-pathbar>
      <div class="lc-tw-body">
        <div class="lc-tw-tree"><luma-tree></luma-tree><div class="lc-tw-note" hidden></div></div>
        <div class="lc-tw-insp"><luma-inspector></luma-inspector></div>
      </div>
    </div>`;
    this.bar = this.querySelector('luma-pathbar');
    this.tree = this.querySelector('luma-tree');
    this.inspEl = this.querySelector('luma-inspector');
    this.tree.addEventListener('select', e => { e.stopPropagation(); this.select(e.detail.path, { sheet: true }); });
    this.tree.addEventListener('fold', e => { e.stopPropagation(); this.fold(e.detail.path, e.detail.open); });
    this.tree.addEventListener('search', e => { e.stopPropagation(); if ((e.detail.query || '') !== (this.cfg.search || '')) this.run(`Set Self "Search" ${quoted(e.detail.query)}`); });
    this.tree.addEventListener('searchenter', e => {
      e.stopPropagation();
      const q = e.detail.query || '';
      if (q !== (this.cfg.search || '')) { this.run(`Set Self "Search" ${quoted(q)}`); this.jumpFirst = true; return; }
      this.goFirstHit();
    });
    this.tree.addEventListener('menu', e => { e.stopPropagation(); this.rowMenu(e.detail.path, { x: e.detail.x, y: e.detail.y }); });
    this.bar.addEventListener('jump', e => { e.stopPropagation(); this.select(e.detail.path); });
    this.bar.addEventListener('go', e => {
      e.stopPropagation();
      this.asked = { text: e.detail.text, back: this.current() };
      this.pending = null;
      this.run(`Set Self "Path" ${quoted(e.detail.text)}`);
    });
    // The inspector's gestures (here and in the phone's sheet).
    this.addEventListener('jump', e => { if (e.target === this.bar) return; e.stopPropagation(); this.select(e.detail.path); });
    this.addEventListener('copy', e => { e.stopPropagation(); copy(e.detail.label, e.detail.text); });
    this.offPhone = onPhoneMode(() => this.paint());
  }

  run(line) { this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }

  /** Where the window stands: a pick on its way, else the station's place. */
  current() {
    if (this.pending) return this.pending;
    if (this.insp && !this.insp.missing && this.insp.path) return this.insp.path;
    return this.treeData?.path || HOME;
  }

  configure(cfg) {
    // Moved from elsewhere (another client, a line): the pick on its way is old.
    if (cfg.path !== this.cfg.path && cfg.path !== this.pending) this.pending = null;
    if (cfg.follow !== this.cfg.follow) this.followKey = undefined;
    this.cfg = cfg;
    this.paint();
  }

  showTree(d) {
    this.treeData = d;
    if (d.path && d.path === this.pending) this.pending = null;
    // Follow = Selected: a new pick the station names moves the window (the first one it sees
    // is where things stand, not a pick).
    if (d.follow) {
      const fresh = this.followKey !== undefined && d.follow.key !== this.followKey;
      this.followKey = d.follow.key;
      if (fresh && this.cfg.follow === 'Selected' && d.follow.path !== this.current()) this.select(d.follow.path);
    }
    if (this.jumpFirst) { this.jumpFirst = false; this.goFirstHit(); }
    this.paint();
  }

  showInspector(d) {
    // The path bar's place isn't one: say why and stay where it was.
    if (d.missing && this.asked && String(d.asked ?? '').trim() === this.asked.text.trim()) {
      const back = this.asked.back;
      this.asked = null;
      toast(d.missing);
      this.run(`Set Self "Path" ${quoted(back)}`);
      return;
    }
    if (!d.missing) this.asked = null;
    if (d.path && d.path === this.pending) this.pending = null;
    this.insp = d;
    this.paint();
  }

  /** Stand at `path` (one line); a phone opens the inspector's sheet. */
  select(path, { sheet = false } = {}) {
    if (sheet && phoneMode()) this.openSheet();
    if (!path || path === this.current()) return;
    this.pending = path;
    this.paint();
    this.run(`Set Self "Path" ${quoted(path)}`);
  }

  goFirstHit() {
    const hit = (this.treeData?.rows || []).find(r => r.hit);
    if (hit) this.select(hit.path);
  }

  /** A fold opens or closes one branch (`Self.Open`); closing one that holds the place moves the
   *  place up to it (its parents stay open by themselves). */
  fold(path, open) {
    let list = String(this.cfg.open || '').split(',').map(s => s.trim()).filter(Boolean);
    if (open) list = [...list.filter(p => p !== path), path].slice(-OPEN_KEEP);
    else {
      list = list.filter(p => p !== path && !p.startsWith(`${path}.`));
      if (this.current().startsWith(`${path}.`)) this.select(path);
    }
    this.run(`Set Self "Open" ${quoted(list.join(','))}`);
  }

  paint() {
    if (!this.built) return;
    const here = this.current();
    const phone = !!phoneMode();
    const root = this.querySelector('.lc-tw');
    root.classList.toggle('phone', phone);
    root.style.setProperty('--lc-tw-split', `${Number(this.cfg.split) || 45}%`);
    this.bar.update({ path: here });
    const d = this.treeData;
    const nodes = (d?.rows || []).map(r => ({
      path: r.path, depth: r.depth, no: r.no ?? '', name: r.quoted ? `"${r.name}"` : r.name,
      count: r.type || r.meta || null, badge: r.marks || '', branch: r.branch, open: r.open, hit: r.hit, dir: r.kind === 'node',
    }));
    this.tree.update({
      host: true, nodes, selected: here, search: true, query: this.cfg.search || '', reveal: here,
      placeholder: 'Search · a number finds its slot', empty: d ? 'Nothing here.' : 'Loading…',
    });
    const note = this.querySelector('.lc-tw-note');
    const said = [d?.hidden ? `${d.hidden} hidden by the search` : '', d?.cut ? `${d.cut} more — search to narrow` : ''].filter(Boolean).join(' · ');
    note.hidden = !said;
    note.textContent = said;
    const icfg = { show: this.cfg.show, level: this.cfg.level };
    // The inspector redraws only when what it shows changed (its scroll stays).
    if (this.inspEl.data !== this.insp || this.inspEl.cfg.show !== icfg.show || this.inspEl.cfg.level !== icfg.level) this.inspEl.update(this.insp, icfg);
    if (this.sheet) {
      this.sheetInsp.update(this.insp, icfg);
      this.sheet.setTitle(this.insp?.title || 'Inspector', this.insp?.path || '');
    }
  }

  /** The phone's inspector: the one sheet, kept live until it closes. */
  openSheet() {
    if (this.sheet) return;
    this.sheetInsp = document.createElement('luma-inspector');
    // Its gestures reach the window like the pane's.
    for (const ev of ['run', 'jump', 'copy']) this.sheetInsp.addEventListener(ev, e => { e.stopPropagation(); this.dispatchEvent(new CustomEvent(ev, { detail: e.detail, bubbles: true })); });
    this.sheet = openSheet({ title: this.insp?.title || 'Inspector', sub: this.insp?.path || '', body: this.sheetInsp, cls: 'lc-tw-sheet', onClose: () => { this.sheet = null; this.sheetInsp = null; } });
    this.sheetInsp.update(this.insp, { show: this.cfg.show, level: this.cfg.level });
  }

  /** A row's menu: the copies, the terminal's NAV, then the registry's one context menu. */
  async rowMenu(path, at) {
    const r = (this.treeData?.rows || []).find(x => x.path === path);
    if (!r) return;
    this.select(path);
    const inspected = () => (this.insp && this.insp.path === path ? this.insp : null);
    const extra = [];
    // tidy: slab-ok (a copy to the user's clipboard, not the station's object Copy)
    if (r.guid) extra.push({ icon: 'copy', label: 'Copy GUID', fn: () => copy('GUID', r.guid) });
    // tidy: slab-ok (a copy to the user's clipboard, not the station's object Copy)
    extra.push({ icon: 'copy', label: 'Copy path', fn: () => copy('path', r.path) });
    // tidy: slab-ok (a copy to the user's clipboard, not the station's object Copy)
    if (r.target) extra.push({ icon: 'copy', label: 'Copy as target', hint: r.target, fn: () => copy('target', r.target) });
    // tidy: slab-ok (a copy to the user's clipboard, not the station's object Copy)
    if (r.guid) extra.push({ icon: 'copy', label: 'Copy Set line', fn: () => { const line = setLines(inspected()); if (line) copy('Set line', line); else toast('Nothing of it differs from its default'); } });
    extra.push({ icon: 'terminal', label: 'Open in the terminal\'s NAV', hint: 'cd', fn: () => { this.run(`cd ${parentOf(path)}`); this.run('Shortcuts Navigator'); } });
    extra.push({ icon: 'pin', label: 'Bookmark…', off: 'In NAV: m and a letter (a word for it is open)' });
    const menu = r.type && r.object && r.no != null ? (await objectMenus())[r.type] : null;
    if (menu) {
      openContext({
        at, menu, object: r.object, rows: String(r.no), count: 1, level: this.cfg.level || '', extra,
        value: prop => { const f = inspected()?.rows?.find(x => x.field && x.name.toLowerCase() === String(prop).toLowerCase()); return f ? [f.value] : null; },
        run: line => this.run(line), edit: () => this.run(`EditSetting ${r.target}`),
      });
      return;
    }
    openMenu(at, [{ header: r.target || r.name || r.path }, ...extra.map(x => ({ icon: x.icon, label: x.label, hint: x.off || x.hint, off: !!x.off, fn: x.fn }))]);
  }
}
customElements.define('luma-tree-window', LumaTreeWindow);

export { LumaInspector, LumaTreeWindow, setLines };
