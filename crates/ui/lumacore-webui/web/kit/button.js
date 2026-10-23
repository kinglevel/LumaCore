// <luma-button icon="save" variant="primary|danger|quiet" disabled list>Save</luma-button>
// A real <button> inside (keyboard, focus, form semantics), drawn by the kit.
// Disabled stays visible (a Guest sees actions disabled, not hidden).
// Every action is still a command line: the page wires `click`.
// `list`: the button opens a list or a menu first — the yellow `===` mark
// (decisions §30 "12e design locked": the kit's one way to say so; tools that
// ask, picks, any "choose from" button). `listMark()` is the same mark for
// the kit's other buttons (slab actions, tool buttons).

/** The yellow `===` mark: this button opens a list or a menu. */
export function listMark() {
  return '<span class="lc-list-mark" aria-label="opens a list"><i></i><i></i><i></i></span>';
}

class LumaButton extends HTMLElement {
  static get observedAttributes() { return ['icon', 'variant', 'disabled', 'label', 'list']; }
  connectedCallback() {
    if (!this.btn) {
      this.text = this.getAttribute('label') ?? this.textContent.trim();
      this.textContent = '';
      this.btn = document.createElement('button');
      this.btn.type = 'button';
      this.appendChild(this.btn);
    }
    this.render();
  }
  attributeChangedCallback() { if (this.btn) this.render(); }
  render() {
    this.dataset.kit = 'button';
    const icon = this.getAttribute('icon');
    const label = this.getAttribute('label') ?? this.text ?? '';
    this.btn.className = `lc-btn ${this.getAttribute('variant') || ''}`.trim();
    this.btn.disabled = this.hasAttribute('disabled');
    this.btn.innerHTML = (icon ? `<luma-icon name="${icon}"></luma-icon>` : '') + (label ? '<span></span>' : '');
    if (label) this.btn.querySelector('span').textContent = label;
    if (this.hasAttribute('list')) this.btn.insertAdjacentHTML('beforeend', listMark());
    if (!label && this.getAttribute('title')) this.btn.setAttribute('aria-label', this.getAttribute('title'));
  }
  set label(v) { this.setAttribute('label', v); }
}
customElements.define('luma-button', LumaButton);
