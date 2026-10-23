// <luma-fader> — a fader (decisions §30 "12f", mockups 2026-10-14-12f round 7: look B, the filled
// bar with the cap line). A fader only drags:
//   · drag moves it relative to where it is — a tap never jumps, there is no hold and no
//     double-tap (its settings come from the window);
//   · each finger owns one fader, so several move at once; a second finger on the same fader
//     gives fine control (×0.1);
//   · near the ends it snaps to 0 / 100.
// Every move is a real command line on the gesture channel (quiet on the station, one entry at
// the end): the element emits `gesture` { id, line, phase: 'move' | 'end', label, from, value },
// which the page sends with `wsGesture`.
//
//   el.update({ value, label, sub, line: 'Master 2.1 At {value}', min: 0, max: 100 })
//   A value the station sends while a finger holds the fader waits until it lets go.

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const SLOP = 4;       // px a finger moves before it drags
const SNAP = 2;       // within this of an end, the fader sits on it
const EVERY_MS = 33;  // at most one move line a frame (30 a second)
let gestures = 0;

const fmt = v => (Math.round(v * 10) / 10).toFixed(Math.round(v * 10) % 10 ? 1 : 0);

class LumaFader extends HTMLElement {
  constructor() {
    super();
    this.spec = { value: 0, min: 0, max: 100, label: '', sub: '', line: '' };
    this.v = 0;
    this.g = null;            // the gesture under way
    this.fingers = new Map(); // pointer id → { y }
  }

  connectedCallback() {
    this.dataset.kit = 'fader';
    if (!this.firstChild) this.build();
  }

  update(spec) {
    this.spec = { ...this.spec, ...spec };
    if (!this.track) this.build();
    if (!this.g && spec.value != null) this.v = Number(spec.value);
    this.querySelector('.lc-fd-lab').innerHTML = `${esc(this.spec.label)}<small>${esc(this.spec.sub)}</small>`;
    this.paint();
  }

  build() {
    this.innerHTML = `<div class="lc-fd-val"></div><div class="lc-fd-track"><div class="lc-fd-groove"><div class="lc-fd-fill"></div></div><div class="lc-fd-cap"></div></div><div class="lc-fd-lab"></div>`;
    this.track = this.querySelector('.lc-fd-track');
    this.track.addEventListener('pointerdown', e => this.down(e));
    this.track.addEventListener('pointermove', e => this.move(e));
    for (const t of ['pointerup', 'pointercancel', 'lostpointercapture']) this.track.addEventListener(t, e => this.up(e));
  }

  /** 0–1 of the travel. */
  frac() {
    const { min, max } = this.spec;
    return max > min ? Math.min(1, Math.max(0, (this.v - min) / (max - min))) : 0;
  }

  paint() {
    if (!this.track) return;
    const f = this.frac();
    this.querySelector('.lc-fd-fill').style.height = `${f * 100}%`;
    this.querySelector('.lc-fd-cap').style.bottom = `calc(${f} * (100% - 20px) + 10px)`;
    this.querySelector('.lc-fd-val').textContent = fmt(this.v);
    const bubble = this.querySelector('.lc-fd-bubble');
    if (bubble) {
      bubble.style.bottom = `calc(${f} * (100% - 20px) + 10px)`;
      bubble.textContent = this.line();
    }
  }

  line() {
    return String(this.spec.line || '').replace('{value}', fmt(this.v));
  }

  travel() {
    return Math.max(40, this.track.clientHeight - 20);
  }

  down(e) {
    e.preventDefault();
    try { this.track.setPointerCapture(e.pointerId); } catch { /* a synthetic pointer */ }
    this.fingers.set(e.pointerId, { y: e.clientY });
    // A second finger on the fader: fine control from where it is.
    if (this.fingers.size === 2 && this.g) {
      Object.assign(this.g, { fine: true, y0: this.fingers.get(this.g.owner).y, v0: this.v });
      this.classList.add('fine');
      return;
    }
    if (this.fingers.size > 2) return;
    this.g = { id: `f${Date.now().toString(36)}.${++gestures}`, owner: e.pointerId, y0: e.clientY, v0: this.v, start: this.v, moved: false, fine: false, last: 0, sent: false };
  }

  move(e) {
    const g = this.g;
    const f = this.fingers.get(e.pointerId);
    if (!g || !f) return;
    f.y = e.clientY;
    // The owner drives; while fine, its moves count ×0.1.
    if (e.pointerId !== g.owner) return;
    const dy = e.clientY - g.y0;
    if (!g.moved && Math.abs(dy) <= SLOP) return;
    if (!g.moved) {
      g.moved = true;
      g.y0 = e.clientY;
      this.classList.add('active');
      const b = document.createElement('div');
      b.className = 'lc-fd-bubble';
      this.track.append(b);
      return;
    }
    const { min, max } = this.spec;
    let v = g.v0 - (dy / this.travel()) * (max - min) * (g.fine ? 0.1 : 1);
    v = Math.min(max, Math.max(min, v));
    if (!g.fine) {
      if (v > max - SNAP) v = max;
      if (v < min + SNAP) v = min;
    }
    v = Math.round(v * 10) / 10;
    if (v === this.v) return;
    this.v = v;
    this.paint();
    const now = performance.now();
    if (now - g.last >= EVERY_MS) {
      g.last = now;
      this.send('move');
    }
  }

  up(e) {
    if (!this.fingers.delete(e.pointerId)) return;
    const g = this.g;
    if (!g) return;
    // The second finger lifts: back to coarse from where it is.
    if (e.pointerId !== g.owner) {
      if (g.fine) {
        Object.assign(g, { fine: false, y0: this.fingers.get(g.owner)?.y ?? g.y0, v0: this.v });
        this.classList.remove('fine');
      }
      return;
    }
    this.classList.remove('active', 'fine');
    this.querySelector('.lc-fd-bubble')?.remove();
    // A tap never jumps; a drag that came back where it started sends its end only if it moved.
    if (g.moved && (g.sent || this.v !== g.start)) this.send('end');
    this.g = null;
    this.fingers.clear();
  }

  send(phase) {
    const g = this.g;
    const detail = { id: g.id, line: this.line(), phase, value: fmt(this.v) };
    if (!g.sent) Object.assign(detail, { label: this.spec.label_line || this.spec.sub || this.spec.label, from: fmt(g.start) });
    g.sent = true;
    this.dispatchEvent(new CustomEvent('gesture', { detail, bubbles: true }));
  }
}
customElements.define('luma-fader', LumaFader);
