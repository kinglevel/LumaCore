// <luma-pathbar> — where a window stands in the show's tree (decisions 0292, the approved mockup
// docs/dev/ui-kit/mockups/2026-10-23-tree-window/01-tree-window.html): the path as crumbs, each
// one a step up (`Root › ShowData › DataPools › 1`); a tap on the bar's empty space (or its edit
// button) turns it into a field to type or paste a tree path or a GUID (whole or its first 8
// digits) — Enter goes, Esc or leaving it gives the crumbs back. The bar only says where to go;
// the host runs the line (`Set Self "Path" …`) and the station says whether it is there.
//
//   el.update({ path: 'Root.ShowData.DataPools.1', placeholder })
//   events: jump { path } (a crumb) · go { text } (typed or pasted, trimmed, not empty)

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaPathbar extends HTMLElement {
  constructor() { super(); this.spec = { path: '' }; this.editing = false; }
  connectedCallback() { this.dataset.kit = 'pathbar'; this.render(); }
  update(spec) { this.spec = spec; if (!this.editing) this.render(); }

  /** The crumbs: each step's own path. */
  crumbs() {
    const parts = String(this.spec.path || '').split('.').filter(Boolean);
    return parts.map((name, i) => ({ name, path: parts.slice(0, i + 1).join('.') }));
  }

  render() {
    if (!this.isConnected) return;
    const emit = (name, detail) => this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
    if (this.editing) {
      this.innerHTML = `<div class="lc-pathbar editing"><input type="text" class="lc-pathbar-in" spellcheck="false" autocomplete="off"
        aria-label="Go to a path or a GUID" placeholder="${esc(this.spec.placeholder || 'A tree path or a GUID — Enter goes, Esc leaves')}" value="${esc(this.spec.path)}"></div>`;
      const input = this.querySelector('input');
      const done = () => { if (!this.editing) return; this.editing = false; this.render(); };
      input.addEventListener('keydown', e => {
        if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); done(); }
        if (e.key === 'Enter') {
          e.preventDefault();
          const text = input.value.trim();
          done();
          if (text && text !== this.spec.path) emit('go', { text });
        }
      });
      input.addEventListener('blur', done);
      input.focus();
      input.select();
      return;
    }
    const crumbs = this.crumbs();
    this.innerHTML = `<div class="lc-pathbar" role="navigation" aria-label="Path">
      <span class="lc-pathbar-crumbs">${crumbs.map((c, i) => `${i ? '<span class="lc-pathbar-sep" aria-hidden="true">›</span>' : ''}<button type="button" class="lc-pathbar-crumb" data-crumb="${esc(c.path)}"${i === crumbs.length - 1 ? ' aria-current="location"' : ''}>${esc(c.name)}</button>`).join('')}</span>
      <button type="button" class="lc-pathbar-edit" data-edit aria-label="Type or paste a path or a GUID" title="Type or paste a path or a GUID"><luma-icon name="pencil"></luma-icon></button>
    </div>`;
    for (const b of this.querySelectorAll('[data-crumb]')) b.addEventListener('click', e => { e.stopPropagation(); emit('jump', { path: b.dataset.crumb }); });
    // The bar's empty space (or the pencil): the field.
    this.querySelector('.lc-pathbar').addEventListener('click', e => {
      if (e.target.closest('[data-crumb]')) return;
      this.editing = true;
      this.render();
    });
  }
}
customElements.define('luma-pathbar', LumaPathbar);

export { LumaPathbar };
