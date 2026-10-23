// <luma-overview> — screens overview and tiled mode (decisions §30 "12a3",
// the Overview board). Overview: this computer's displays with every screen
// window drawn on them as a live picture, and a list — bring forward, open /
// hide, send to a display (the app's side, through events). Tiled: several
// screens live and usable side by side in one window or browser tab; Pop out
// opens one as its own OS window (app only).
//
// update({ mode: 'overview'|'tiled', title, displays: [{name, x, y, w, h,
//   main}], screens: [{no, open, display, x, y, w, h, fullscreen, url}],
//   app: bool })
// Events: overview-mode {mode} · overview-bring {no} · overview-toggle {no}
//   · overview-send {no, display} · overview-popout {no}

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaOverview extends HTMLElement {
  constructor() { super(); this.data = { mode: 'overview', displays: [], screens: [] }; this.sel = null; }
  connectedCallback() { this.dataset.kit = 'overview'; this.render(); }
  update(data) { this.data = { ...this.data, ...data }; this.render(); }
  emit(name, detail = {}) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  render() {
    if (!this.isConnected) return;
    const d = this.data;
    const head = `<header class="lc-ov-head"><h1>${esc(d.title || 'Screens')}</h1>
      <button type="button" class="lc-ov-tab${d.mode === 'overview' ? ' on' : ''}" data-mode="overview">Overview</button>
      <button type="button" class="lc-ov-tab${d.mode === 'tiled' ? ' on' : ''}" data-mode="tiled">Tiled</button></header>`;
    this.innerHTML = head + (d.mode === 'tiled' ? this.tiled() : this.overview());
    for (const b of this.querySelectorAll('.lc-ov-tab')) b.addEventListener('click', () => this.emit('overview-mode', { mode: b.dataset.mode }));
    for (const b of this.querySelectorAll('[data-act]')) {
      b.addEventListener('click', () => {
        const no = Number(b.dataset.no);
        if (b.dataset.act === 'pick') { this.sel = no; this.render(); return; }
        this.sel = no;
        this.emit(`overview-${b.dataset.act}`, { no, display: b.dataset.display });
      });
    }
  }

  overview() {
    const d = this.data;
    const shown = d.screens.filter(s => s.open);
    // Fit every display into the map.
    const minX = Math.min(0, ...d.displays.map(m => m.x)), minY = Math.min(0, ...d.displays.map(m => m.y));
    const maxX = Math.max(1, ...d.displays.map(m => m.x + m.w)), maxY = Math.max(1, ...d.displays.map(m => m.y + m.h));
    const scale = Math.min(640 / (maxX - minX), 340 / (maxY - minY));
    const px = v => Math.round(v * scale);
    const maps = d.displays.map(m => {
      const wins = shown.filter(s => s.display === m.name).map(s => `
        <button type="button" class="lc-ov-win${this.sel === s.no ? ' on' : ''}" data-act="pick" data-no="${s.no}"
          style="left:${px(s.x)}px;top:${px(s.y)}px;width:${Math.max(40, px(s.w))}px;height:${Math.max(28, px(s.h))}px">
          <span class="lc-ov-win-title"><i></i>Screen ${s.no}${s.fullscreen ? ' · full screen' : ''}</span>
          ${s.url ? `<span class="lc-ov-live"><iframe src="${esc(s.url)}" tabindex="-1" loading="lazy" title="Screen ${s.no}"
             style="width:${s.w}px;height:${s.h}px;transform:scale(${Math.max(40, px(s.w)) / s.w})"></iframe></span>` : ''}
        </button>`).join('');
      return `<div class="lc-ov-display" style="left:${px(m.x - minX)}px;top:${px(m.y - minY)}px;width:${px(m.w)}px;height:${px(m.h)}px">
        ${wins}<span class="lc-ov-dname">${esc(m.name)}${m.main ? ' (main)' : ''}</span></div>`;
    }).join('');
    const others = name => d.displays.filter(m => m.name !== name);
    const list = d.screens.map(s => `
      <div class="lc-ov-row${this.sel === s.no ? ' on' : ''}">
        <span class="lc-ov-row-title">Screen ${s.no}<small>${s.open ? `${esc(s.display || '')}${s.fullscreen ? ' · native full screen' : ''}` : 'hidden'}</small></span>
        <span class="lc-ov-actions">
          ${s.open ? `<button type="button" class="lc-ov-btn" data-act="bring" data-no="${s.no}">Bring forward</button>` : ''}
          <button type="button" class="lc-ov-btn" data-act="toggle" data-no="${s.no}">${s.open ? 'Hide' : 'Open'}</button>
          ${d.app ? others(s.display).map(m => `<button type="button" class="lc-ov-btn" data-act="send" data-no="${s.no}" data-display="${esc(m.name)}">Send to ${esc(m.name)}</button>`).join('') : ''}
        </span>
      </div>`).join('');
    return `<div class="lc-ov-body">
      <section class="lc-ov-map"><span class="lc-ov-note">${d.app ? 'Displays of this computer' : 'Screens of this session'}</span>
        <div class="lc-ov-canvas" style="width:${px(maxX - minX)}px;height:${px(maxY - minY)}px">${maps}</div></section>
      <section class="lc-ov-list">${list}<span class="lc-ov-note">Each screen remembers its display and place (per server and user). A missing display → the main one, scaled. Meow+F12 in a screen = native full screen there.</span></section>
    </div>`;
  }

  tiled() {
    const d = this.data;
    const shown = d.screens.filter(s => s.open || d.screens.length <= 4);
    return `<div class="lc-ov-tiles" style="--lc-ov-cols:${Math.min(2, Math.max(1, shown.length))}">${shown.map(s => `
      <section class="lc-ov-tile">
        <header><i></i><span>Screen ${s.no}</span>${d.app ? `<button type="button" class="lc-ov-btn" data-act="popout" data-no="${s.no}">Pop out to its own window</button>` : ''}</header>
        <iframe src="${esc(s.url || '')}" title="Screen ${s.no}"></iframe>
      </section>`).join('')}</div>`;
  }
}
customElements.define('luma-overview', LumaOverview);
