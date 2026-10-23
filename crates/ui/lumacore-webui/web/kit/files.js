// <luma-files> — the one way to load and save files (decisions §38 5): the drives rail (the
// station's drives from the `station` feed, then **This device** — the browser's upload — last),
// the crumbs (drive › LumaCore › library › folder; ★ Library jumps back), a <luma-grid> of the
// folder's files with the kind's columns from the `files` feed, a drop zone, and a name field
// for a save. Every file dialog is this element with a kind: Backup › Library (shows), Import
// MVR (mvr), Insert fixtures' library (gdtf), Import / Export UserProfile (userprofiles), Import
// Image (media). The element owns its two subscriptions on the client it is given.
//
//   el.update({ feeds: client, kind: 'shows', drive: 'local-library', folder: 'shows', save: false, name: '', level })
//   el.update({ host: true, feeds, kind, drive })   the host fills el.grid() itself (Backup › Library on the
//                                                  shows feed): the rail and the crumbs only, `where` says what was picked
//   el.selection() → { drive, folder, names: [file names], rows }   el.where() → { drive, folder }
//   event `pick`   → { drive, folder, name, row }   a file double-tapped (or Enter on the name field)
//   event `save`   → { drive, folder, name }        the name field's Enter / the Save button
//   event `select` → { drive, folder, names }       the grid's selection changed
//   event `where`  → { drive, folder }              the drive or folder changed
//   event `upload` → { drive, folder, name }        a file from this device landed

import './grid.js';
import { phoneMode, onPhoneMode } from './phone.js';
import { toast } from './toast.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** The library folder a kind opens on. */
const FOLDER = { shows: 'shows', gdtf: 'gdtf', mvr: 'mvr', media: 'media', userprofiles: 'userprofiles' };
const LIBRARY = ['LumaCore', 'library'];

const col = (id, label, field, width, priority = 1) => ({ id, label, field, width, priority, readonly: true });
const COLUMNS = {
  shows: [col('name', 'Show', 'name', 180), col('saved', 'Last saved', 'saved_text', 130), col('size', 'Size', 'size_text', 80), col('format', 'Format', 'format_text', 90, 2), col('saved_by', 'Saved by', 'saved_by', 100, 2), col('note', 'Note', 'note', 160, 3)],
  any: [col('name', 'Name', 'name', 220), col('size', 'Size', 'size_text', 90), col('modified', 'Modified', 'modified_text', 140, 2)],
  // A GDTF's own facts (decisions §38 5): what Insert fixtures' Library list showed.
  gdtf: [col('name', 'File', 'name', 200), col('manufacturer', 'Manufacturer', 'manufacturer', 140), col('type', 'Type', 'type', 160), col('modes', 'Modes', 'modes_text', 220, 2), col('size', 'Size', 'size_text', 80, 2)],
  // An MVR's head (decisions §38 5, "Import MVR hardened" 1): the scene, its layers and fixtures, the GDTFs it holds.
  mvr: [col('name', 'File', 'name', 200), col('scene', 'Scene', 'scene', 150, 2), col('layers', 'Layers', 'layers', 70, 2), col('fixtures', 'Fixtures', 'fixtures', 80), col('gdtf', 'GDTF', 'gdtf_text', 150), col('size', 'Size', 'size_text', 80, 2), col('modified', 'Modified', 'modified_text', 120, 3)],
};

const size = n => (n >= 1e9 ? `${(n / 1e9).toFixed(1)} GB` : n >= 1e6 ? `${(n / 1e6).toFixed(1)} MB` : n >= 1e3 ? `${Math.round(n / 1e3)} KB` : `${n} B`);
function when(ms) {
  if (!ms) return '—';
  const d = new Date(ms), now = new Date();
  const hm = d.toTimeString().slice(0, 5);
  if (d.toDateString() === now.toDateString()) return `today ${hm}`;
  const y = new Date(now); y.setDate(now.getDate() - 1);
  if (d.toDateString() === y.toDateString()) return `yesterday ${hm}`;
  return d.toISOString().slice(0, 10);
}

class LumaFiles extends HTMLElement {
  constructor() {
    super();
    this.spec = { feeds: null, kind: 'shows', drive: 'local-library', folder: null, save: false, name: '', level: '', host: false };
    this.drives = [];
    this.rows = [];
    this.error = '';
  }
  connectedCallback() {
    this.dataset.kit = 'files';
    // The phone rule (decisions §34 "Import MVR hardened" 1b): in portrait the rail folds into a
    // chip strip over a full-width grid; landscape keeps the desk layout.
    this.offPhone = onPhoneMode(() => { this.phoneUp(); this.renderRail(); });
    this.render();
  }
  disconnectedCallback() { this.stationSub?.close(); this.filesSub?.close(); this.stationSub = this.filesSub = null; this.offPhone?.(); this.offPhone = null; }
  /** Sets the element's phone attribute; true in a phone's portrait. */
  phoneUp() {
    const up = phoneMode() === 'up';
    if (up) this.dataset.phone = 'up'; else delete this.dataset.phone;
    return up;
  }

  update(spec) {
    const was = { ...this.spec };
    this.spec = { ...this.spec, ...(spec || {}) };
    if (this.spec.folder == null) this.spec.folder = FOLDER[this.spec.kind] || '';
    if (this.spec.feeds && (!this.stationSub || was.feeds !== this.spec.feeds)) this.subscribe();
    else if (this.filesSub && (was.drive !== this.spec.drive || was.folder !== this.spec.folder || was.kind !== this.spec.kind)) this.refile();
    this.render();
  }
  subscribe() {
    this.stationSub?.close(); this.filesSub?.close();
    const f = this.spec.feeds;
    this.stationSub = f.sub('station', {}, { batch: m => { this.drives = m.data?.drives || []; this.renderRail(); } });
    if (this.spec.host) return;
    this.filesSub = f.sub('files', this.params(), { batch: m => {
      // The same rows again (the feed is rebuilt when the station or the library moves): the
      // grid keeps its rows, so a tap on one lands.
      const key = JSON.stringify([m.data?.files || [], m.data?.error || '']);
      if (key === this.lastRows) return;
      this.lastRows = key;
      this.rows = m.data?.files || []; this.error = m.data?.error || ''; this.renderGrid();
    } });
  }
  params() { return { drive: this.spec.drive, folder: this.spec.folder, kind: this.spec.kind }; }
  refile() {
    this.lastRows = null; this.rows = []; this.error = ''; this.filesSub?.set(this.params()); }

  /** Go to a drive (its library folder of the kind) or a folder (relative to the library). */
  go(drive, folder) {
    const same = drive === this.spec.drive && folder === this.spec.folder;
    this.spec.drive = drive; this.spec.folder = folder;
    if (!same) { this.refile(); this.dispatchEvent(new CustomEvent('where', { detail: this.where(), bubbles: true })); }
    this.render();
  }
  where() { return { drive: this.spec.drive, folder: this.spec.folder }; }
  selection() {
    const g = this.grid();
    const keys = g?.selection().rows || [];
    const rows = keys.map(k => this.rows.find(r => r.file === k)).filter(Boolean);
    return { ...this.where(), names: rows.map(r => r.name), rows };
  }
  grid() { return this.querySelector('luma-grid'); }
  /** The kind's folder in a library (`gdtf`, `shows`). */
  folderOf(kind) { return FOLDER[kind] || ''; }

  // ── Drawing ──
  render() {
    if (!this.isConnected) return;
    if (!this.querySelector('.lc-files')) {
      this.innerHTML = `<div class="lc-files">
        <div class="lc-files-rail" data-rail></div>
        <div class="lc-files-main">
          <div class="lc-files-crumbs" data-crumbs></div>
          <luma-grid class="lc-files-grid"></luma-grid>
          <div class="lc-files-note" data-note hidden></div>
          <label class="lc-files-drop" data-drop><span>or <b>drop a file</b> here · it uploads to this folder</span></label>
          <div class="lc-files-name" data-namerow hidden><label for="lc-files-name">Name</label><input id="lc-files-name" class="lc-input" data-name type="text" autocomplete="off"></div>
        </div>
      </div>`;
      this.wire();
    }
    this.renderRail();
    this.renderCrumbs();
    this.renderGrid();
    const nr = this.querySelector('[data-namerow]');
    nr.hidden = !this.spec.save;
    this.querySelector('[data-drop]').hidden = !!this.spec.host; // a host's grid takes no drop
    const input = this.querySelector('[data-name]');
    if (this.spec.save && input.value !== (this.spec.name || '') && document.activeElement !== input) input.value = this.spec.name || '';
  }
  renderRail() {
    const rail = this.querySelector('[data-rail]');
    if (!rail) return;
    if (this.phoneUp()) {
      // The chip strip: id and label, the picked one on, Out ones dimmed; This device at the end.
      const chips = this.drives.map(d => `<button type="button" class="lc-files-drive chip${d.id === this.spec.drive ? ' on' : ''}${d.state === 'In' ? '' : ' out'}" data-drive="${esc(d.id)}"${d.state === 'In' ? '' : ' aria-disabled="true"'} title="${esc(d.label)}${d.state === 'In' ? '' : ' · out'}"><b>${esc(d.id)}</b><small>${esc(d.label)}</small></button>`).join('');
      rail.innerHTML = `${chips}<label class="lc-files-drive chip dev" title="Upload a file from this device into this folder"><b><luma-icon name="upload"></luma-icon>This device</b><input type="file" data-upload hidden></label>`;
      return;
    }
    const rows = this.drives.map(d => `<button type="button" class="lc-files-drive${d.id === this.spec.drive ? ' on' : ''}${d.state === 'In' ? '' : ' out'}" data-drive="${esc(d.id)}"${d.state === 'In' ? '' : ' aria-disabled="true"'} title="${esc(d.path || '')}">
        <b>${esc(d.id)}<i>${esc(d.kind === 'Internal' ? '' : d.kind)}</i></b><small>${esc(d.label)}${d.state === 'In' ? (d.free ? ` · ${esc(d.free)} free` : '') : ' · out'}</small></button>`).join('');
    rail.innerHTML = `<h4>Drives</h4>${rows}<label class="lc-files-drive dev" title="Upload a file from this device into this folder"><b><luma-icon name="upload"></luma-icon>This device</b><small>upload from the browser</small><input type="file" data-upload hidden></label>`;
  }
  renderCrumbs() {
    const c = this.querySelector('[data-crumbs]');
    if (!c) return;
    const { drive, folder } = this.spec;
    const parts = (folder || '').split('/').filter(Boolean);
    const local = drive === 'local-library';
    let html = `<button type="button" class="lc-files-crumb" data-go="">${esc(drive)}</button>`;
    if (!local) html += `<i>›</i><span class="lc-files-crumb dim">LumaCore</span><i>›</i><span class="lc-files-crumb dim">library</span>`;
    let acc = '';
    for (const p of parts) { acc = acc ? `${acc}/${p}` : p; html += `<i>›</i><button type="button" class="lc-files-crumb${acc === folder ? ' on' : ''}" data-go="${esc(acc)}">${esc(p)}</button>`; }
    const home = FOLDER[this.spec.kind] || '';
    html += `<button type="button" class="lc-files-lib" data-go="${esc(home)}" title="The library folder of this kind"><luma-icon name="star"></luma-icon>Library</button>`;
    c.innerHTML = html;
  }
  renderGrid() {
    const g = this.grid();
    if (!g || this.spec.host) return;
    const columns = COLUMNS[this.spec.kind] || COLUMNS.any;
    const rows = this.rows.map(r => ({
      ...r,
      key: r.file,
      name: r.dir ? `${r.name}/` : r.name,
      size_text: r.dir ? '' : size(r.size),
      modified_text: when(r.modified),
      saved_text: r.dir ? '' : when(r.saved_at || r.modified),
      format_text: r.dir ? '' : r.format ? `${r.format}` : '—',
    }));
    g.update({ key: `files.${this.spec.kind}`, id: 'name', row_key: 'key', columns, rows, state: {}, level: this.spec.level, caption: '', bar: false });
    const note = this.querySelector('[data-note]');
    note.hidden = !(this.error || !this.rows.length);
    note.textContent = this.error || (this.filesSub ? 'Nothing here yet' : '');
  }

  // ── Gestures ──
  wire() {
    this.addEventListener('click', e => {
      const d = e.target.closest('[data-drive]');
      if (d) { if (d.getAttribute('aria-disabled')) return toast(`${d.dataset.drive} is out`); return this.go(d.dataset.drive, FOLDER[this.spec.kind] || ''); }
      const go = e.target.closest('[data-go]');
      if (go) return this.go(this.spec.drive, go.dataset.go);
    });
    const g = this.grid();
    g.addEventListener('select', e => {
      e.stopPropagation();
      const keys = g.selection().rows || [];
      const row = keys.length === 1 ? this.rows.find(r => r.file === keys[0]) : null;
      if (row?.dir) { g.none?.(); return this.go(this.spec.drive, this.spec.folder ? `${this.spec.folder}/${row.file}` : row.file); }
      if (row && this.spec.save) { const input = this.querySelector('[data-name]'); input.value = row.name; }
      this.dispatchEvent(new CustomEvent('select', { detail: this.selection(), bubbles: true }));
    });
    // A double-tap on a file picks it (the grid's own double-tap would edit).
    g.addEventListener('dblclick', e => {
      e.stopPropagation(); e.preventDefault();
      // The grid captures the pointer, so the event's target is its scroller: hit-test the point.
      const k = (document.elementFromPoint(e.clientX, e.clientY) || e.target).closest('[data-k]')?.dataset.k;
      const row = this.rows.find(r => r.file === k);
      if (this.spec.host) { if (k) this.dispatchEvent(new CustomEvent('pick', { detail: { ...this.where(), name: k, row: null }, bubbles: true })); return; }
      if (!row) return;
      if (row.dir) return this.go(this.spec.drive, this.spec.folder ? `${this.spec.folder}/${row.file}` : row.file);
      this.dispatchEvent(new CustomEvent('pick', { detail: { ...this.where(), name: row.name, row }, bubbles: true }));
    }, true);
    this.addEventListener('keydown', e => {
      if (e.key !== 'Enter' || !e.target.matches('[data-name]')) return;
      const name = e.target.value.trim();
      if (name) this.dispatchEvent(new CustomEvent('save', { detail: { ...this.where(), name }, bubbles: true }));
    });
    this.addEventListener('change', e => {
      const input = e.target.closest('[data-upload]');
      if (input?.files?.[0]) this.upload(input.files[0]);
    });
    const drop = this.querySelector('[data-drop]');
    for (const ev of ['dragenter', 'dragover']) this.addEventListener(ev, e => { e.preventDefault(); drop.classList.add('hot'); });
    this.addEventListener('dragleave', e => { if (!this.contains(e.relatedTarget)) drop.classList.remove('hot'); });
    this.addEventListener('drop', e => { e.preventDefault(); drop.classList.remove('hot'); const f = e.dataTransfer?.files?.[0]; if (f) this.upload(f); });
  }
  /** This device: the file into the current drive's folder through the one route (§38 5). */
  async upload(file) {
    const { drive, folder } = this.spec;
    toast(`Uploading ${file.name} to ${drive} · ${folder || 'library'}…`);
    const q = `drive=${encodeURIComponent(drive)}&folder=${encodeURIComponent(folder || '')}&name=${encodeURIComponent(file.name)}`;
    const r = await fetch(`/api/upload?${q}`, { method: 'POST', body: file }).then(x => x.json()).catch(() => ({ ok: false, message: 'The upload failed' }));
    if (!r.ok) return toast(r.message || 'The upload failed');
    toast(`${file.name} is on ${drive}`);
    this.dispatchEvent(new CustomEvent('upload', { detail: { drive, folder, name: file.name }, bubbles: true }));
  }
}
customElements.define('luma-files', LumaFiles);
