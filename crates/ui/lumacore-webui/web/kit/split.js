// The split handle (decisions §30 "12e hardening", the Split board of
// mockups/2026-10-13-12e): a 4 px line between two panes. With a mouse the
// whole line drags (a thin band round it). On a touch screen only the grip
// pill in its middle is a target (44 × 64 px round a small pill) — the line
// itself never steals a touch meant for a pane. Double-tap resets to 50 / 50;
// with a keyboard the handle is a `role="separator"`: arrows move it 5 %,
// Home / End to the minimum / maximum, Enter back to 50 / 50. A drag, a reset
// or a key is one layout step, filed when it ends (onCommit).
//
//   const el = handle('h' | 'v', {
//     size: () => px,               // the split's length along its direction
//     ratio: () => r,               // the first pane's share, 0…1
//     set: r => …,                  // show the new share (live, while dragging)
//     commit: () => …,              // save it (one step)
//     minPx: 120,                   // neither pane gets smaller
//   })

import { drag, doubleTime } from './pointer.js';

const STEP = 0.05;

export function handle(dir, { size, ratio, set, commit, minPx = 120 } = {}) {
  const el = document.createElement('div');
  el.className = `lc-split lc-split-${dir}`;
  el.dataset.kit = 'split';
  el.tabIndex = 0;
  el.setAttribute('role', 'separator');
  el.setAttribute('aria-orientation', dir === 'h' ? 'vertical' : 'horizontal');
  el.innerHTML = '<span class="lc-split-pill" data-kit="split-pill"></span>';
  const isH = dir === 'h';
  const clamp = r => {
    const lo = Math.min(0.5, Math.max(0.1, minPx / Math.max(1, size())));
    return Math.max(lo, Math.min(1 - lo, r));
  };
  const show = r => { set(r); el.setAttribute('aria-valuenow', String(Math.round(r * 100))); };
  show(ratio());

  let r0 = 0, moved = false, lastTap = 0;
  drag(el, {
    // Touch: only the pill. Mouse and pen: the line too.
    filter: e => e.pointerType === 'mouse' || e.pointerType === 'pen' || !!e.target.closest('.lc-split-pill'),
    onStart: () => {
      r0 = ratio(); moved = false;
      el.classList.add('active');
      document.body.classList.add(isH ? 'lc-dragging-h' : 'lc-dragging-v');
    },
    onMove: ({ dx, dy }) => {
      const d = isH ? dx : dy;
      if (Math.abs(d) > 2) moved = true;
      show(clamp(r0 + d / Math.max(1, size())));
    },
    onEnd: () => {
      el.classList.remove('active');
      document.body.classList.remove('lc-dragging-h', 'lc-dragging-v');
      if (moved) { lastTap = 0; if (ratio() !== r0) commit(); return; }
      const now = performance.now();
      if (now - lastTap < doubleTime()) { lastTap = 0; show(clamp(0.5)); commit(); }
      else lastTap = now;
    },
  });
  el.addEventListener('keydown', e => {
    const fwd = isH ? 'ArrowRight' : 'ArrowDown', back = isH ? 'ArrowLeft' : 'ArrowUp';
    let r = ratio();
    if (e.key === fwd) r += STEP;
    else if (e.key === back) r -= STEP;
    else if (e.key === 'Home') r = 0;
    else if (e.key === 'End') r = 1;
    else if (e.key === 'Enter') r = 0.5;
    else return;
    e.preventDefault();
    show(clamp(r));
    commit();
  });
  return el;
}
