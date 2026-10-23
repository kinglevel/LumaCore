// <luma-tags-pool> — the Tags pool window (decisions §12 "Tags hardened", the locked mockup
// docs/dev/ui-kit/mockups/2026-10-17-tags/round2.html "Tags pool window"): an overview in two
// panes. Left the tag tree (<luma-tree> browse: − / + fold, colour, count, the Action badge; the
// picked tag has its ≡ menu). Right the picked tag's members (<luma-grid>, read only) with
// Direct | Deep and Run action. Drawn from the `tags` feed; every gesture is one station line
// (event `run { line }`): a tap picks the tag (`Set Self "Tag" 1.1`), Direct | Deep is
// `Set Self "Members" Deep`, Run action the tag's select line, the ≡ menu the tag's own lines
// (Label and New child tag ask for a name first). Folds and the search stay in the tree.
//
//   el.update(feedData, { action: 'Select' | 'None', counts: true })
//   events: run { line }

import './tree.js';
import './grid.js';
import './button.js';
import { openMenu } from './menu.js';
import { openDialog } from './dialog.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaTagsPool extends HTMLElement {
  constructor() { super(); this.data = { nodes: [], members: [] }; this.cfg = {}; }

  connectedCallback() {
    this.dataset.kit = 'tags-pool';
    if (this.tree) return;
    this.innerHTML = `<div class="lc-tp">
      <div class="lc-tp-tree"><luma-tree></luma-tree></div>
      <div class="lc-tp-pane">
        <div class="lc-tp-strip">
          <span class="lc-tp-title"></span>
          <span class="lc-tp-modes" role="group" aria-label="Members">
            <luma-button data-members="Direct">Direct</luma-button><luma-button data-members="Deep">Deep</luma-button>
          </span>
          <luma-button data-run variant="primary" icon="play">Run action</luma-button>
        </div>
        <div class="lc-tp-kinds"></div>
        <div class="lc-tp-grid"><luma-grid></luma-grid></div>
        <div class="lc-tp-empty"></div>
      </div>
    </div>`;
    this.tree = this.querySelector('luma-tree');
    this.grid = this.querySelector('luma-grid');
    const run = line => this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true }));
    this.run = run;
    this.tree.addEventListener('select', e => {
      e.stopPropagation();
      if (e.detail.path !== this.data.tag?.path) run(`Set Self "Tag" ${e.detail.path}`);
    });
    this.tree.addEventListener('menu', e => { e.stopPropagation(); this.menu(e.detail); });
    for (const b of this.querySelectorAll('[data-members]')) b.addEventListener('click', () => {
      if ((this.data.deep ? 'Deep' : 'Direct') !== b.dataset.members) run(`Set Self "Members" ${b.dataset.members}`);
    });
    this.querySelector('[data-run]').addEventListener('click', () => { const l = this.runLine(); if (l) run(l); });
    this.render();
  }

  update(data, cfg = {}) { this.data = data || { nodes: [], members: [] }; this.cfg = cfg; this.render(); }

  /** Run action: the tag's own Action, or the window's default (Select fixtures today). */
  runLine() {
    const t = this.data.tag;
    if (!t) return null;
    return t.own_action || this.cfg.action !== 'None' ? t.lines.select : null;
  }

  /** The picked tag's ≡ menu: its own lines. */
  menu(at) {
    const t = this.data.tag;
    if (!t) return;
    const named = async (title, template) => {
      const got = await openDialog({
        title, inputs: [{ name: 'v', label: 'Name', value: '', kind: 'text' }],
        buttons: [{ label: 'Please', value: 'set', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
      });
      const name = got?.value && got.inputs?.v?.trim();
      if (name) this.run(template.replace('{name}', `"${name.replace(/[";]/g, '')}"`));
    };
    openMenu(at, [
      { header: `Tag ${t.path} · ${t.name || 'no name'}` },
      { icon: 'settings', label: 'Settings', fn: () => this.run(t.lines.settings) },
      { icon: 'pencil', label: 'Label', fn: () => named(`Label tag ${t.path}`, t.lines.label) },
      // tidy: slab-ok (a tag's own ≡ menu, as the locked tags mockup draws it; the tree is not a grid)
      { icon: 'plus', label: 'New child tag', fn: () => named(`New tag under ${t.path}`, t.lines.child) },
      { icon: 'arrow-up', label: 'Move to the top', off: !t.path.includes('.'), fn: () => this.run(t.lines.top) },
      null,
      // tidy: slab-ok (the same ≡ menu: Delete asks with / keep children on the station)
      { icon: 'trash-2', label: 'Delete', danger: true, fn: () => this.run(t.lines.delete) },
    ]);
  }

  render() {
    if (!this.tree) return;
    const d = this.data, t = d.tag;
    const counts = this.cfg.counts !== false;
    this.tree.update({
      mode: 'browse', search: true, selected: t?.path || null, empty: 'No tags yet. Store Tag "name" makes one.',
      nodes: (d.nodes || []).map(n => ({ path: n.path, name: n.name, colour: n.colour, badge: n.badge, count: counts ? n.count : null })),
    });
    this.querySelector('.lc-tp-title').textContent = t ? `${t.name || 'no name'} · ${t.path}` : 'Pick a tag';
    for (const b of this.querySelectorAll('[data-members]')) {
      const on = (d.deep ? 'Deep' : 'Direct') === b.dataset.members;
      b.setAttribute('variant', on ? 'primary' : 'quiet');
      b.setAttribute('aria-pressed', String(on));
    }
    const runB = this.querySelector('[data-run]');
    if (this.runLine()) runB.removeAttribute('disabled'); else runB.setAttribute('disabled', '');
    this.querySelector('.lc-tp-kinds').innerHTML = (d.kinds || []).map(k => `<span class="lc-tp-kind">${esc(k.count)} ${esc(k.kind)}</span>`).join('');
    const empty = this.querySelector('.lc-tp-empty');
    const members = d.members || [];
    empty.hidden = !!(t && members.length);
    empty.textContent = d.missing ? 'The picked tag is gone. Pick another.' : !t ? 'Tap a tag to see what carries it.' : (d.deep ? 'Nothing carries this tag or the tags under it.' : 'Nothing carries this tag itself. Deep adds the tags under it.');
    this.querySelector('.lc-tp-grid').hidden = !t || !members.length;
    if (t) this.grid.update({ key: 'tags.members', object: null, id: 'target', row_key: 'guid', columns: d.columns || [], rows: members.map(m => ({ ...m, gutter: m.no })), state: { set: 'Full' }, level: 'Guest' });
  }
}
customElements.define('luma-tags-pool', LumaTagsPool);

export { LumaTagsPool };
