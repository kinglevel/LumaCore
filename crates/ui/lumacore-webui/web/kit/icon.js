// <luma-icon name="settings"> — one icon from the vendored sprite (Lucide +
// LumaCore glyphs, vendor/lucide/sprite.svg). The sprite is inlined into the
// page once, so `<use href="#i-…">` resolves in every webview and in
// screenshots.

let sprite = null;
/** Inline the sprite (once). Resolves when the icons can draw. */
export function loadSprite() {
  if (!sprite) sprite = fetch('/vendor/lucide/sprite.svg')
    .then(r => r.ok ? r.text() : '')
    .then(svg => {
      const host = document.createElement('div');
      host.id = 'lc-sprite';
      host.hidden = true;
      host.innerHTML = svg;
      document.body.prepend(host);
      return [...host.querySelectorAll('symbol')].map(s => s.id.replace(/^i-/, ''));
    })
    .catch(() => []);
  return sprite;
}

/** Every icon name in the sprite (for the Showroom). */
export async function iconNames() { return loadSprite(); }

class LumaIcon extends HTMLElement {
  static get observedAttributes() { return ['name']; }
  connectedCallback() { this.render(); }
  attributeChangedCallback() { this.render(); }
  render() {
    const name = this.getAttribute('name') || '';
    this.dataset.kit = 'icon';
    // Stroke and fill as attributes, not only CSS: the screenshot capture
    // clones the page and keeps attributes (the <use> content inherits them).
    this.innerHTML = `<svg class="lc-ico" aria-hidden="true" focusable="false" fill="none" stroke="currentColor"`
      + ` stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><use href="#i-${name}"></use></svg>`;
  }
}
customElements.define('luma-icon', LumaIcon);

/** Markup for an icon, for templates. */
export function icon(name) { return `<luma-icon name="${String(name).replace(/[^a-z0-9-]/gi, '')}"></luma-icon>`; }
