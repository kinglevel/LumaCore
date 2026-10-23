// <luma-maxbar> — the header of a window held full screen (decisions §30
// "12c2", the HoldFullscreen board): ‹ the neighbour pane before, the
// window's title, › the one after, × back to the whole screen. Holding the
// title goes back too. The page decides what a step means (its own device,
// or `Set Self Maximized On` for every client of the live screen).
//
//   el.update({ title, sub, count, index }) — count: panes on the screen,
//   index: this one's place — ‹ is off on the first, › on the last (they
//   stop at the ends, §30 "12e round 5 questions")
//   event `maxbar` → { go: 'prev' | 'next' | 'close' }

import { hold } from './pointer.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaMaxbar extends HTMLElement {
  constructor() { super(); this.spec = { title: '', sub: '', count: 1 }; }
  connectedCallback() { this.dataset.kit = 'maxbar'; this.render(); }
  update(spec) { this.spec = { ...this.spec, ...spec }; this.render(); }
  go(go) { this.dispatchEvent(new CustomEvent('maxbar', { detail: { go }, bubbles: true })); }
  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    const n = s.count ?? 1, i = s.index ?? 0;
    const prev = n > 1 && i > 0, next = n > 1 && i < n - 1;
    this.innerHTML = `
      <button type="button" class="lc-mx-step" data-go="prev" aria-label="Previous pane" ${prev ? '' : 'disabled'}><luma-icon name="chevron-left"></luma-icon></button>
      <span class="lc-mx-title" title="Hold: back to the whole screen">${esc(s.title)}${s.sub ? ` <small>${esc(s.sub)}</small>` : ''}</span>
      <button type="button" class="lc-mx-step" data-go="next" aria-label="Next pane" ${next ? '' : 'disabled'}><luma-icon name="chevron-right"></luma-icon></button>
      <button type="button" class="lc-mx-close" data-go="close" aria-label="Back to the whole screen"><luma-icon name="x"></luma-icon></button>`;
    for (const b of this.querySelectorAll('[data-go]')) b.addEventListener('click', () => this.go(b.dataset.go));
    hold(this.querySelector('.lc-mx-title'), { onHold: () => this.go('close') });
  }
}
customElements.define('luma-maxbar', LumaMaxbar);
