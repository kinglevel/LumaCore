// The slider panel (decisions §30 "12f", rounds 3–5: one vertical pop-up slider for the encoders
// and every timing field): − / + and a Keypad button; a drag on it moves the value relative to
// where it is; a tap anywhere outside closes it, keeping the value (rounds 10–11: the rule for
// every pop-up); the interface behind dims when the profile's PopupDim is on.
//
//   openSliderPanel({ at: {x, y}, title, value, min, max, step, unit, dim,
//                     onMove(value), onEnd(value), onKeypad(text) })
//   `onMove` is called while dragging (at most a frame apart), `onEnd` when a drag or a − / +
//   tap ends, `onKeypad` with what was typed (relative input like `+ 5` is the station's).

import { openKeypad } from './keypad.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const TRAVEL = 220;
const EVERY_MS = 33;

let open = null;

export function closeSliderPanel() {
  open?.close();
}

export function openSliderPanel(spec) {
  closeSliderPanel();
  const { min = 0, max = 100, step = 1, unit = '' } = spec;
  let v = Number(spec.value ?? min);
  const fmt = x => {
    const r = Math.round(x * 100) / 100;
    return String(r);
  };
  const scrim = document.createElement('div');
  scrim.className = 'lc-sp-scrim' + (spec.dim ? ' dim' : '');
  scrim.dataset.kit = 'slider-panel';
  const box = document.createElement('div');
  box.className = 'lc-sp';
  box.innerHTML = `<header class="lc-sp-title">${esc(spec.title)}</header>
    <div class="lc-sp-val"></div>
    <button type="button" class="lc-sp-step" data-d="1" aria-label="More">+</button>
    <div class="lc-sp-track"><div class="lc-sp-fill"></div><div class="lc-sp-cap"></div></div>
    <button type="button" class="lc-sp-step" data-d="-1" aria-label="Less">−</button>
    <button type="button" class="lc-sp-kp">Keypad</button>`;
  scrim.append(box);
  document.body.append(scrim);
  const track = box.querySelector('.lc-sp-track');
  const paint = () => {
    const f = max > min ? Math.min(1, Math.max(0, (v - min) / (max - min))) : 0;
    box.querySelector('.lc-sp-fill').style.height = `${f * 100}%`;
    box.querySelector('.lc-sp-cap').style.bottom = `calc(${f} * (100% - 4px))`;
    box.querySelector('.lc-sp-val').textContent = `${fmt(v)}${unit}`;
  };
  paint();
  // Beside the touch, inside the screen.
  const r = box.getBoundingClientRect();
  const x = Math.max(8, Math.min((spec.at?.x ?? innerWidth / 2) - r.width / 2, innerWidth - r.width - 8));
  const y = Math.max(8, Math.min((spec.at?.y ?? innerHeight / 2) - r.height / 2, innerHeight - r.height - 8));
  Object.assign(box.style, { left: `${x}px`, top: `${y}px` });

  const clamp = x => Math.min(max, Math.max(min, Math.round(x / step) * step));
  for (const b of box.querySelectorAll('.lc-sp-step')) {
    b.addEventListener('click', () => {
      v = clamp(v + Number(b.dataset.d) * step);
      paint();
      spec.onEnd?.(v);
    });
  }
  box.querySelector('.lc-sp-kp').addEventListener('click', async () => {
    const text = await openKeypad({ title: spec.title, value: fmt(v), min, max, unit: unit.trim() });
    if (text != null && text !== '') spec.onKeypad?.(text);
    close();
  });
  let drag = null;
  track.addEventListener('pointerdown', e => {
    e.preventDefault();
    try { track.setPointerCapture(e.pointerId); } catch { /* synthetic */ }
    drag = { y0: e.clientY, v0: v, last: 0, moved: false };
  });
  track.addEventListener('pointermove', e => {
    if (!drag) return;
    const dy = e.clientY - drag.y0;
    if (!drag.moved && Math.abs(dy) < 3) return;
    drag.moved = true;
    v = clamp(drag.v0 - (dy / TRAVEL) * (max - min));
    paint();
    const now = performance.now();
    if (now - drag.last >= EVERY_MS) {
      drag.last = now;
      spec.onMove?.(v);
    }
  });
  const end = () => {
    if (drag?.moved) spec.onEnd?.(v);
    drag = null;
  };
  track.addEventListener('pointerup', end);
  track.addEventListener('pointercancel', end);
  // A tap anywhere outside closes it (the value stays).
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  // Escape closes the panel only (the page behind doesn't see it).
  const onKey = e => { if (e.key === 'Escape') { e.stopPropagation(); close(); } };
  document.addEventListener('keydown', onKey, true);
  function close() {
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    if (open?.scrim === scrim) open = null;
  }
  open = { scrim, close };
  return open;
}
