// <luma-tiles> — the kit's tile grid (decisions §30 "12c2", the MenuTiles
// board): Menu, Shutdown and Backup. Compact tiles — a 24 px icon over a
// one-line label, ~88 px high, never under the 44 px floor; the columns
// auto-fill from the box's width (--lc-tile-min, so 4–5 on a desktop, 2 on a
// phone), and a narrow box (a container query) lays each tile out as icon +
// label in a row. The element scrolls inside; its box is never larger than
// the screen (the overlay contract). Every tile is one gesture: the page maps
// its id to a command line (or a menu of its own).
//
//   el.update([{ id, icon, label, value, title, disabled, tone: 'please' | 'clear' }])
//   event `tile` → { id }

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaTiles extends HTMLElement {
  constructor() { super(); this.tiles = []; }
  connectedCallback() { this.dataset.kit = 'tiles'; this.render(); }
  update(tiles) {
    // The same tiles again draw nothing (a press in flight keeps its button).
    const key = JSON.stringify(tiles || []);
    if (key === this.key && this.firstChild) return;
    this.key = key;
    this.tiles = tiles || [];
    this.render();
  }
  render() {
    if (!this.isConnected) return;
    this.innerHTML = `<div class="lc-tiles">${this.tiles.map((t, i) => `
      <button type="button" class="lc-tile${t.tone ? ` ${esc(t.tone)}` : ''}" data-tile="${esc(t.id)}" data-i="${i}"
        ${t.disabled ? 'disabled' : ''} title="${esc(t.title || t.label)}">
        <luma-icon name="${esc(t.icon || 'box')}"></luma-icon>
        <span class="lc-tile-text"><span class="lc-tile-label">${esc(t.label)}</span>${t.value != null ? `<small class="lc-tile-value">${esc(t.value)}</small>` : ''}</span>
      </button>`).join('')}</div>`;
    for (const b of this.querySelectorAll('[data-tile]')) {
      b.addEventListener('click', () => this.dispatchEvent(new CustomEvent('tile', { detail: { id: b.dataset.tile }, bubbles: true })));
    }
  }
}
customElements.define('luma-tiles', LumaTiles);
