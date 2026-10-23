// <luma-tree> — a tree of named nodes numbered by their path (decisions §12 "Tags hardened",
// the locked mockups docs/dev/ui-kit/mockups/2026-10-17-tags/round2.html): the tag tree in the
// Tags pool window (browse: tap selects, the selected row has its ≡ menu) and in EditSetting's
// Tags tab (pick: a three-way box per row — on, part of several objects, off). Each row: − / +
// to fold (a node with children), its number, colour swatch, name, count and action badge. A
// search box narrows the rows (a match keeps its parents). Every change is the host's command
// line; the tree itself only keeps what is folded and what the search says.
//
//   el.update({ nodes: [{ path, name, colour, state, count, badge }], mode: 'browse' | 'pick',
//     selected, search: true, newHere: true, empty, numbers: false (no path numbers: the Patch
//     dialog's MVR tree), menu: false (no ≡ on the selected row), unit: 'fixtures' (after each
//     count), guides: true (a guide line for every level) })
//   events: select { path } · toggle { path, on } · menu { path, x, y } · new { parent }

import { lift } from './pointer.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const depthOf = path => String(path).split('.').length - 1;
const parentOf = path => { const i = String(path).lastIndexOf('.'); return i < 0 ? null : path.slice(0, i); };

class LumaTree extends HTMLElement {
  constructor() { super(); this.spec = { nodes: [] }; this.folded = new Set(); this.query = ''; }
  connectedCallback() { this.dataset.kit = 'tree'; this.render(); }
  update(spec) { this.spec = spec; this.render(); }

  /** Paths that have children. */
  parents() {
    const out = new Set();
    for (const n of this.spec.nodes || []) { const p = parentOf(n.path); if (p) out.add(p); }
    return out;
  }

  /** The rows shown: not under a folded node; with a search, the matches and their parents. */
  shown() {
    const nodes = this.spec.nodes || [];
    const q = this.query.trim().toLowerCase();
    if (q) {
      const keep = new Set();
      for (const n of nodes) {
        if (!String(n.name).toLowerCase().includes(q) && !String(n.path).startsWith(q)) continue;
        for (let p = n.path; p; p = parentOf(p)) keep.add(p);
      }
      return nodes.filter(n => keep.has(n.path));
    }
    return nodes.filter(n => { for (let p = parentOf(n.path); p; p = parentOf(p)) if (this.folded.has(p)) return false; return true; });
  }

  row(n, hasKids) {
    const s = this.spec;
    const pick = s.mode === 'pick';
    const sel = s.selected === n.path;
    const open = !this.folded.has(n.path) || this.query;
    const fold = hasKids ? `<button type="button" class="lc-tree-fold" data-fold="${esc(n.path)}" aria-label="${open ? 'Fold' : 'Unfold'} ${esc(n.name)}"><luma-icon name="${open ? 'minus' : 'plus'}"></luma-icon></button>` : '<span class="lc-tree-fold leaf"></span>';
    const box = pick ? `<button type="button" class="lc-tree-box" data-box="${esc(n.path)}" data-state="${esc(n.state || 'off')}" role="checkbox" aria-checked="${n.state === 'on' ? 'true' : n.state === 'part' ? 'mixed' : 'false'}" aria-label="${esc(n.name)}"></button>` : '';
    return `<div class="lc-tree-row${sel ? ' sel' : ''}" role="treeitem" aria-level="${depthOf(n.path) + 1}" ${hasKids ? `aria-expanded="${!!open}"` : ''} data-path="${esc(n.path)}" style="--lc-tree-depth:${depthOf(n.path)}">
      ${fold}${box}
      ${s.numbers === false ? '' : `<span class="lc-tree-no">${esc(n.path)}</span>`}
      ${n.colour ? `<i class="lc-tree-sw" style="background:${esc(n.colour)}"></i>` : ''}
      <span class="lc-tree-name">${esc(n.name) || '<i>no name</i>'}</span>
      ${n.id ? `<span class="lc-tree-id">${esc(n.id)}</span>` : ''}
      ${n.badge ? `<span class="lc-tree-badge">${esc(n.badge)}</span>` : ''}
      ${n.count != null ? `<small class="lc-tree-count">${esc(n.count)}${s.unit ? ` ${esc(s.unit)}` : ''}</small>` : ''}
      ${!pick && sel && s.menu !== false ? `<button type="button" class="lc-tree-menu" data-menu="${esc(n.path)}" aria-label="Menu of ${esc(n.name)}"><luma-icon name="menu"></luma-icon></button>` : ''}
    </div>`;
  }

  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    const kids = this.parents();
    const rows = this.shown();
    const had = this.querySelector('.lc-tree-search input') === document.activeElement;
    this.innerHTML = `<div class="lc-tree${s.mode === 'pick' ? ' pick' : ''}${s.guides ? ' guides' : ''}">
      ${s.search ? `<div class="lc-tree-head"><label class="lc-tree-search"><luma-icon name="search"></luma-icon><input type="search" placeholder="Search tags…" value="${esc(this.query)}" aria-label="Search"></label>
        ${/* tidy: slab-ok (the picker's own New tag here, in its head as the locked tags mockup draws it; not a list of objects) */ ''}
        ${s.newHere ? `<button type="button" class="lc-tree-new" data-new="1"><luma-icon name="plus"></luma-icon>New tag here</button>` : ''}</div>` : ''}
      <div class="lc-tree-rows" role="tree">${rows.map(n => this.row(n, kids.has(n.path))).join('') || `<div class="lc-tree-empty">${esc(s.empty || 'No tags yet.')}</div>`}</div>
    </div>`;
    const input = this.querySelector('.lc-tree-search input');
    if (input) {
      input.addEventListener('input', () => { this.query = input.value; const at = input.selectionStart; this.render(); const again = this.querySelector('.lc-tree-search input'); again.focus(); again.setSelectionRange(at, at); });
      if (had) input.focus();
    }
    const emit = (name, detail) => this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
    for (const b of this.querySelectorAll('[data-fold]')) b.addEventListener('click', ev => {
      ev.stopPropagation();
      const p = b.dataset.fold;
      this.folded.has(p) ? this.folded.delete(p) : this.folded.add(p);
      this.render();
    });
    for (const b of this.querySelectorAll('[data-box]')) b.addEventListener('click', ev => {
      ev.stopPropagation();
      emit('toggle', { path: b.dataset.box, on: b.dataset.state !== 'on' });
    });
    for (const b of this.querySelectorAll('[data-menu]')) b.addEventListener('click', ev => {
      ev.stopPropagation();
      const r = b.getBoundingClientRect();
      emit('menu', { path: b.dataset.menu, x: r.right - 200, y: r.bottom + 2 });
    });
    for (const r of this.querySelectorAll('.lc-tree-row')) r.addEventListener('click', () => emit('select', { path: r.dataset.path }));
    this.querySelector('[data-new]')?.addEventListener('click', () => emit('new', { parent: s.selected || null }));
    if (s.lift) this.wireLift();
  }
  /** A node dragged onto another node (decisions §40 "40d hardened" 3): the pointer core's one
   *  lift; the host runs the line on `drop` (it has the rows' lines). */
  wireLift() {
    this.unlift?.();
    const item = e => { const r = e.target.closest?.('.lc-tree-row'); if (!r) return null; return e.target.closest('button') ? undefined : { path: r.dataset.path, el: r }; };
    const emit = (name, detail) => this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
    this.unlift = lift(this, {
      item, scroller: () => this.querySelector('.lc-tree-rows') || this,
      mode: () => 'lasso', phone: () => false,
      onTap: (it) => { if (it) emit('select', { path: it.path }); },
      onLift: (it, info) => emit('lift', { path: it.path, x: info.x, y: info.y }),
      onMove: info => emit('liftmove', { x: info.x, y: info.y }),
      onDrop: (info, cancelled) => emit('drop', { x: info.x, y: info.y, cancelled: !!cancelled }),
      onContext: (it, at) => { if (it) emit('menu', { path: it.path, x: at.x, y: at.y }); },
    });
  }
}
customElements.define('luma-tree', LumaTree);

export { LumaTree };
