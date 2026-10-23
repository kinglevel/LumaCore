// <luma-avatar name="Bob" color="#758ba0" level="Program" badge="LD" state="on FOH" size="68">
// A user on the lock surface (decisions §30 "12a3"): first two letters in a
// ring of the user's own colour, their rights level (§30 "12d") and the
// profile's name as badges, where they
// are logged in. `dim` = can't log in from here (Remote Login off);
// `selected` = picked. A real <button> when `tap` is set.

class LumaAvatar extends HTMLElement {
  static get observedAttributes() { return ['name', 'color', 'level', 'badge', 'state', 'size', 'dim', 'selected', 'big']; }
  connectedCallback() { this.render(); }
  attributeChangedCallback() { if (this.isConnected) this.render(); }
  render() {
    this.dataset.kit = 'avatar';
    const name = this.getAttribute('name') || '';
    const size = Number(this.getAttribute('size')) || 68;
    const big = this.hasAttribute('big');
    this.classList.toggle('dim', this.hasAttribute('dim'));
    this.classList.toggle('big', big);
    this.style.setProperty('--lc-av-size', `${size}px`);
    this.style.setProperty('--lc-av-ring', this.hasAttribute('selected') || big ? (this.getAttribute('color') || 'var(--lc-line)') : 'var(--lc-av-idle)');
    this.innerHTML = '<button type="button" class="lc-av-btn"><span class="lc-av-circle"></span><span class="lc-av-name"></span><span class="lc-av-state"></span></button>';
    this.querySelector('.lc-av-circle').textContent = name.slice(0, 2).toUpperCase();
    const label = this.querySelector('.lc-av-name');
    label.textContent = name;
    for (const [attr, cls] of [['level', 'lc-av-badge lc-av-level'], ['badge', 'lc-av-badge']]) {
      const text = this.getAttribute(attr);
      if (!text) continue;
      const b = document.createElement('span');
      b.className = cls;
      b.textContent = text;
      label.append(' ', b);
    }
    this.querySelector('.lc-av-state').textContent = this.getAttribute('state') || '';
    const btn = this.querySelector('button');
    btn.setAttribute('aria-label', name);
    btn.disabled = this.hasAttribute('dim');
  }
}
customElements.define('luma-avatar', LumaAvatar);
