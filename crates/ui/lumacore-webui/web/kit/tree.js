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
//   A node may carry its own shown number (`no`), `tickable: false` (pick mode: no box — a
//   heading, a circular reference) and a `tone` for its name (`alert`): the station's Update ask
//   (decisions 0293) draws its tree of presets and cues this way.
//
// Host mode (the Tree window, decisions 0292: NAV's rows from the station): the host sends the
// rows it shows, already opened and filtered — each { path, name, depth, no, count, badge,
// branch, open, hit, dir } — and keeps what is open and the search itself: a fold is the event
// `fold { path, open }`, the search box `search { query }` (typing settles first) and
// `searchenter { query }` (Enter). Many rows draw only the part in sight (virtual); the scroll
// stays across updates; `reveal` scrolls a path into sight once each time it changes; a hold or
// a right-click on a row is `menu`.
//
//   el.update({ host: true, nodes, selected, search: true, query, placeholder, reveal, empty })

import { lift } from './pointer.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const depthOf = path => String(path).split('.').length - 1;
/** Host mode: past this many rows only those in sight are drawn. */
const VIRTUAL = 200;
/** Rows drawn above and below the ones in sight. */
const OVERSCAN = 12;
/** Typing settles this long before the host's search runs. */
const SEARCH_SETTLE = 300;
const parentOf = path => { const i = String(path).lastIndexOf('.'); return i < 0 ? null : path.slice(0, i); };

class LumaTree extends HTMLElement {
  constructor() { super(); this.spec = { nodes: [] }; this.folded = new Set(); this.query = ''; }
  connectedCallback() {
    this.dataset.kit = 'tree';
    if (!this.spec.host) { this.render(); return; }
    // Moved back into the page (a layout redraw): where it was scrolled to.
    const rows = this.querySelector('.lc-tree-rows');
    if (rows && this.scrolled) rows.scrollTop = this.scrolled;
    else this.renderHost();
  }
  update(spec) { this.spec = spec; if (spec.host) this.renderHost(); else this.render(); }

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
    const open = s.host ? !!n.open : !this.folded.has(n.path) || this.query;
    if (s.host) hasKids = !!n.branch;
    const depth = n.depth ?? depthOf(n.path);
    const fold = hasKids ? `<button type="button" class="lc-tree-fold" data-fold="${esc(n.path)}" aria-label="${open ? 'Fold' : 'Unfold'} ${esc(n.name)}"><luma-icon name="${open ? 'minus' : 'plus'}"></luma-icon></button>` : '<span class="lc-tree-fold leaf"></span>';
    const box = pick && n.tickable === false ? '<span class="lc-tree-box gap" aria-hidden="true"></span>' : pick ? `<button type="button" class="lc-tree-box" data-box="${esc(n.path)}" data-state="${esc(n.state || 'off')}" role="checkbox" aria-checked="${n.state === 'on' ? 'true' : n.state === 'part' ? 'mixed' : 'false'}" aria-label="${esc(n.name)}"></button>` : '';
    const no = s.host ? (n.no ?? '') : (n.no ?? n.path);
    return `<div class="lc-tree-row${sel ? ' sel' : ''}${n.hit ? ' hit' : ''}${n.dir ? ' dir' : ''}" role="treeitem" aria-level="${depth + 1}" ${hasKids ? `aria-expanded="${!!open}"` : ''} ${sel ? 'aria-selected="true"' : ''} data-path="${esc(n.path)}" style="--lc-tree-depth:${depth}">
      ${fold}${box}
      ${s.numbers === false ? '' : `<span class="lc-tree-no">${esc(no)}</span>`}
      ${n.colour ? `<i class="lc-tree-sw" style="background:${esc(n.colour)}"></i>` : ''}
      <span class="lc-tree-name${n.tone ? ` tone-${esc(n.tone)}` : ''}">${esc(n.name) || '<i>no name</i>'}</span>
      ${n.id ? `<span class="lc-tree-id">${esc(n.id)}</span>` : ''}
      ${n.badge ? `<span class="lc-tree-badge"${s.host ? ` title="Bookmark ${esc(n.badge)}"` : ''}>${esc(n.badge)}</span>` : ''}
      ${n.count != null ? `<small class="lc-tree-count${n.tone ? ` tone-${esc(n.tone)}` : ''}">${esc(n.count)}${s.unit ? ` ${esc(s.unit)}` : ''}</small>` : ''}
      ${!pick && sel && s.menu !== false && !s.host ? `<button type="button" class="lc-tree-menu" data-menu="${esc(n.path)}" aria-label="Menu of ${esc(n.name)}"><luma-icon name="menu"></luma-icon></button>` : ''}
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

  // ── Host mode (the Tree window) ──
  /** Draw the head once and the rows in sight; the search box keeps its focus and text. */
  renderHost() {
    if (!this.isConnected) return;
    const s = this.spec;
    if (!this.hostBuilt) {
      this.hostBuilt = true;
      this.innerHTML = `<div class="lc-tree host">
        ${s.search ? `<div class="lc-tree-head"><label class="lc-tree-search"><luma-icon name="search"></luma-icon><input type="search" aria-label="Search"></label></div>` : ''}
        <div class="lc-tree-rows" role="tree"></div>
      </div>`;
      const input = this.querySelector('.lc-tree-search input');
      const emit = (name, detail) => this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
      if (input) {
        input.addEventListener('input', () => {
          clearTimeout(this.settle);
          this.settle = setTimeout(() => emit('search', { query: input.value }), SEARCH_SETTLE);
        });
        input.addEventListener('keydown', e => {
          if (e.key !== 'Enter') return;
          e.preventDefault();
          clearTimeout(this.settle);
          emit('searchenter', { query: input.value });
        });
      }
      const rows = this.querySelector('.lc-tree-rows');
      rows.addEventListener('scroll', () => {
        this.scrolled = rows.scrollTop;
        if ((this.spec.nodes || []).length > VIRTUAL && !this.raf) this.raf = requestAnimationFrame(() => { this.raf = 0; this.paintRows(); });
      });
      rows.addEventListener('click', e => {
        const b = e.target.closest('[data-fold]');
        if (!b) return;
        e.stopPropagation();
        const row = b.closest('.lc-tree-row');
        emit('fold', { path: b.dataset.fold, open: row?.getAttribute('aria-expanded') !== 'true' });
      });
      this.unlift?.();
      this.unlift = lift(rows, {
        item: e => { const r = e.target.closest?.('.lc-tree-row'); if (!r) return null; return e.target.closest('button') ? undefined : { path: r.dataset.path }; },
        scroller: rows, mode: () => 'scroll', phone: () => true,
        onTap: it => { if (it) emit('select', { path: it.path }); },
        onContext: (it, at) => { if (it) emit('menu', { path: it.path, x: at.x, y: at.y }); },
      });
    }
    const input = this.querySelector('.lc-tree-search input');
    if (input) {
      input.placeholder = s.placeholder || 'Search…';
      if (document.activeElement !== input && s.query != null && input.value !== s.query) input.value = s.query;
    }
    this.paintRows();
    if (s.reveal && s.reveal !== this.revealed) {
      this.revealed = s.reveal;
      this.revealPath(s.reveal);
    }
  }

  /** The rows in sight (all of them when there are few). */
  paintRows() {
    const s = this.spec;
    const rows = this.querySelector('.lc-tree-rows');
    if (!rows) return;
    const nodes = s.nodes || [];
    if (!nodes.length) { rows.innerHTML = `<div class="lc-tree-empty">${esc(s.empty || 'Nothing here.')}</div>`; return; }
    const top = this.scrolled ?? rows.scrollTop;
    if (nodes.length <= VIRTUAL) {
      rows.innerHTML = nodes.map(n => this.row(n)).join('');
      rows.scrollTop = top;
      return;
    }
    const h = this.rowH || 40;
    const from = Math.max(0, Math.floor(top / h) - OVERSCAN);
    const to = Math.min(nodes.length, Math.ceil((top + (rows.clientHeight || 600)) / h) + OVERSCAN);
    rows.innerHTML = `<div class="lc-tree-gap" style="height:${from * h}px"></div>${nodes.slice(from, to).map(n => this.row(n)).join('')}<div class="lc-tree-gap" style="height:${(nodes.length - to) * h}px"></div>`;
    const first = rows.querySelector('.lc-tree-row');
    if (first && !this.rowH && first.offsetHeight) { this.rowH = first.offsetHeight; this.paintRows(); return; }
    rows.scrollTop = top;
  }

  /** Scroll `path`'s row into sight (the middle), when it is out of it. */
  revealPath(path) {
    const rows = this.querySelector('.lc-tree-rows');
    const i = (this.spec.nodes || []).findIndex(n => n.path === path);
    if (!rows || i < 0) return;
    const h = this.rowH || rows.querySelector('.lc-tree-row')?.offsetHeight || 40;
    const y = i * h;
    if (y < rows.scrollTop || y + h > rows.scrollTop + rows.clientHeight) {
      rows.scrollTop = Math.max(0, y - rows.clientHeight / 2);
      this.scrolled = rows.scrollTop;
      this.paintRows();
    }
  }
}
customElements.define('luma-tree', LumaTree);

export { LumaTree };
