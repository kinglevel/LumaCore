// <luma-graph> — a canvas graph of the last N samples (decisions §30 "12b2
// hardening": Realtime window, the Realtime board): one point per sample,
// newest at the right; a dashed budget line; late samples marked with a red
// dot; a gap (NaN rows in the SampleRing) breaks the line and is shaded.
// Colours are theme roles (`tone` = a --lc-* token name), read at draw time.
//
//   <luma-graph tone="accent" points="600"></luma-graph>
//   graph.budget = 16667;            // dashed line (same unit as the values)
//   graph.draw(ring, 'frame_us', 'late');   // a SampleRing (feeds.js) + columns
//
// Draw on the window's requestAnimationFrame (feeds.drawLoop), never per
// message.

class LumaGraph extends HTMLElement {
  constructor() {
    super();
    this.budget = 0;
    this.scale = 0;
  }

  connectedCallback() {
    if (this.canvas) return;
    this.dataset.kit = 'graph';
    this.canvas = document.createElement('canvas');
    this.append(this.canvas);
    this.setAttribute('role', 'img');
    new ResizeObserver(() => { this.sized = false; if (this.lastArgs) this.draw(...this.lastArgs); }).observe(this);
  }

  get points() { return parseInt(this.getAttribute('points') || '600', 10); }

  color(token) { return getComputedStyle(this).getPropertyValue(`--lc-${token}`).trim(); }

  draw(ring, col, lateCol) {
    this.lastArgs = [ring, col, lateCol];
    const c = this.canvas;
    if (!c) return;
    const dpr = window.devicePixelRatio || 1;
    const w = this.clientWidth, h = this.clientHeight;
    if (!w || !h) return;
    if (!this.sized || c.width !== Math.round(w * dpr) || c.height !== Math.round(h * dpr)) {
      c.width = Math.round(w * dpr);
      c.height = Math.round(h * dpr);
      this.sized = true;
    }
    const g = c.getContext('2d');
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, w, h);
    const n = this.points;
    const len = Math.min(ring.length, n);
    const from = ring.length - len;
    // Scale: the budget (with room) or what the window holds; it eases down.
    let peak = 0;
    for (let i = from; i < ring.length; i++) { const v = ring.at(col, i); if (v > peak) peak = v; }
    const want = Math.max(this.budget * 1.1, peak * 1.25, 1);
    this.scale = want > this.scale ? want : this.scale * 0.98 + want * 0.02;
    const pad = 4;
    const x = i => (i - from + (n - len)) / (n - 1) * w;
    const y = v => h - pad - Math.min(1, v / this.scale) * (h - 2 * pad);

    if (this.budget) {
      g.strokeStyle = this.color('line');
      g.setLineDash([4, 4]);
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(0, Math.round(y(this.budget)) + 0.5);
      g.lineTo(w, Math.round(y(this.budget)) + 0.5);
      g.stroke();
      g.setLineDash([]);
    }

    // Gaps: shaded, the line breaks.
    g.fillStyle = this.color('warn');
    g.globalAlpha = 0.18;
    let gapStart = -1;
    for (let i = from; i <= ring.length; i++) {
      const missing = i < ring.length && Number.isNaN(ring.at(col, i));
      if (missing && gapStart < 0) gapStart = i;
      if (!missing && gapStart >= 0) { g.fillRect(x(gapStart), 0, Math.max(1, x(i) - x(gapStart)), h); gapStart = -1; }
    }
    g.globalAlpha = 1;

    g.strokeStyle = this.color(this.getAttribute('tone') || 'info');
    g.lineWidth = 1.4;
    g.lineJoin = 'round';
    g.beginPath();
    let pen = false;
    for (let i = from; i < ring.length; i++) {
      const v = ring.at(col, i);
      if (Number.isNaN(v)) { pen = false; continue; }
      if (pen) g.lineTo(x(i), y(v)); else { g.moveTo(x(i), y(v)); pen = true; }
    }
    g.stroke();

    if (lateCol) {
      g.fillStyle = this.color('alert');
      for (let i = from; i < ring.length; i++) {
        if (ring.at(lateCol, i) > 0) {
          g.beginPath();
          g.arc(x(i), y(ring.at(col, i)), 3.5, 0, Math.PI * 2);
          g.fill();
        }
      }
    }
  }
}
customElements.define('luma-graph', LumaGraph);