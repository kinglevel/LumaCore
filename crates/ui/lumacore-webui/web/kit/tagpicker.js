// <luma-tag-picker> — the tag picker (decisions 0297, the approved mockup
// docs/dev/ui-kit/mockups/2026-10-24-tag-assign/01-tag-assign.html): `Tag Group 3` opens it on
// those objects. The tag tree in pick mode with each tag's square, a box per row (on: every one
// carries it, part: some, off: none) from the `tags` feed's `on`; a tap on a row or its box is
// one `Assign` / `Unassign Tag <path> At <objects>` line, so each tap applies at once (one Oops
// step). A search with no tag of that name offers New tag: `Store Tag "name"; Assign Tag "name"
// At <objects>`. Done (or Esc) closes; on a phone it is a sheet.
//
//   openTagPicker({ object, name, guids, data? }, { run, closed })   run(line) sends a line
//   el.update(spec) · el.setData(feedData)
//   events: run { line } · cancel

import './tree.js';
import { phoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** A tag name as a quoted Lum value (no quote or `;` inside). */
const quoted = s => `"${String(s).replace(/[";]/g, '').trim()}"`;

class LumaTagPicker extends HTMLElement {
  constructor() { super(); this.spec = {}; this.data = { nodes: [] }; this.query = ''; this.ran = ''; }

  connectedCallback() {
    this.dataset.kit = 'tag-picker';
    if (this.tree) return;
    this.innerHTML = `<section class="lc-dlg lc-box lc-tagpk" role="dialog" aria-modal="true">
      <header class="lc-dlg-title"><span data-title></span><span class="lc-dlg-meta" data-meta></span>
        <button type="button" class="lc-close" data-cancel aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
      <div class="lc-dlg-body">
        <luma-tree></luma-tree>
        <button type="button" class="lc-tagpk-new" data-new hidden><luma-icon name="plus"></luma-icon><span></span></button>
        <div class="lc-dlg-rows"><div><span>Ran</span><b class="lc-selectable" data-ran>—</b></div></div>
        <div class="lc-dlg-buttons" style="--lc-dlg-cols:1"><button type="button" class="lc-dlg-btn primary" data-done data-primary="1">Done</button></div>
      </div>
    </section>`;
    this.tree = this.querySelector('luma-tree');
    const cancel = () => this.dispatchEvent(new CustomEvent('cancel'));
    this.querySelector('[data-cancel]').addEventListener('click', cancel);
    this.querySelector('[data-done]').addEventListener('click', cancel);
    this.tree.addEventListener('toggle', e => { e.stopPropagation(); this.tick(e.detail.path, e.detail.on); });
    this.tree.addEventListener('select', e => { e.stopPropagation(); this.tick(e.detail.path, this.stateOf(e.detail.path) !== 'on'); });
    this.tree.addEventListener('query', e => { e.stopPropagation(); this.query = e.detail.query.trim(); this.renderNew(); });
    this.querySelector('[data-new]').addEventListener('click', () => this.newTag());
    this.render();
  }

  update(spec) { this.spec = spec || {}; this.render(); }
  setData(data) { this.data = data || { nodes: [] }; this.render(); }

  stateOf(path) { return (this.data.nodes || []).find(n => n.path === path)?.state || 'off'; }

  /** One tap: tag them with it, or take it off them. */
  tick(path, on) { this.send(`${on ? 'Assign' : 'Unassign'} Tag ${path} At ${this.spec.object}`); }

  /** New tag: store it at the top (the next free number) and tag them with it, in one line. */
  newTag() {
    if (!this.query) return;
    const name = quoted(this.query);
    this.send(`Store Tag ${name}; Assign Tag ${name} At ${this.spec.object}`);
    this.query = '';
    this.tree.query = '';
    this.render();
  }

  send(line) {
    this.ran = line;
    this.querySelector('[data-ran]').textContent = line;
    this.dispatchEvent(new CustomEvent('run', { detail: { line } }));
  }

  /** The New tag button: a search that names no tag. */
  renderNew() {
    const b = this.querySelector('[data-new]');
    if (!b) return;
    const q = this.query.toLowerCase();
    const named = (this.data.nodes || []).some(n => (n.name || '').toLowerCase() === q);
    b.hidden = !q || named;
    b.querySelector('span').innerHTML = `New tag <b>${esc(quoted(this.query))}</b> on ${esc(this.spec.object)}`;
  }

  render() {
    if (!this.tree) return;
    const s = this.spec, nodes = this.data.nodes || [];
    this.querySelector('[data-title]').textContent = `Tags · ${s.object || ''}${s.name ? ` "${s.name}"` : ''}`;
    this.querySelector('section').setAttribute('aria-label', `Tags · ${s.object || ''}`);
    const on = nodes.filter(n => n.state && n.state !== 'off').length;
    this.querySelector('[data-meta]').textContent = on ? `${on} tag${on === 1 ? '' : 's'}` : 'no tags';
    const many = (s.guids || []).length > 1;
    this.tree.update({
      mode: 'pick', search: true, squares: true, placeholder: 'Search or name a new tag…',
      empty: 'No tags yet. Type a name to make one.',
      nodes: nodes.map(n => ({ path: n.path, name: n.name, colour: n.colour, state: n.state || 'off',
        count: many && n.state === 'part' ? `${n.carried} of ${s.guids.length}` : null })),
    });
    this.renderNew();
  }
}
customElements.define('luma-tag-picker', LumaTagPicker);

let current = null;

/** Open the tag picker on `spec.object` (the line's words) — `spec.guids` the objects for the
 *  `tags` feed, or `spec.data` given (the Showroom). Resolves when it closes. */
export function openTagPicker(spec, { run, closed } = {}) {
  current?.cancel();
  return new Promise(resolve => {
    const scrim = document.createElement('div');
    scrim.className = 'lc-dlg-scrim lc-tagpk-scrim';
    scrim.dataset.kit = 'tag-picker-scrim';
    if (phoneMode() === 'up') scrim.classList.add('sheet');
    const el = document.createElement('luma-tag-picker');
    scrim.append(el);
    document.body.append(scrim);
    el.update(spec);
    let sub = null;
    if (spec.data) el.setData(spec.data);
    else {
      const client = spec.feeds || window.feeds?.();
      sub = client?.sub('tags', { tag: '', deep: false, on: spec.guids || [] }, { batch: m => { if (m.data) el.setData(m.data); } }) || null;
    }
    const done = () => {
      if (!scrim.isConnected) return;
      window.removeEventListener('keydown', onKey, true);
      sub?.close();
      scrim.remove();
      if (current?.scrim === scrim) current = null;
      closed?.();
      resolve();
    };
    el.addEventListener('cancel', done);
    el.addEventListener('run', e => run?.(e.detail.line));
    scrim.addEventListener('pointerdown', e => { if (e.target === scrim) done(); });
    // Before every other overlay's keys: Esc closes; Enter in the search makes the new tag.
    const onKey = e => {
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); done(); }
      else if (e.key === 'Enter' && e.target?.closest?.('.lc-tree-search')) { e.preventDefault(); e.stopPropagation(); el.newTag(); }
    };
    window.addEventListener('keydown', onKey, true);
    current = { scrim, el, cancel: done };
  });
}

export const tagPickerOpen = () => !!current;
