// Miniature panes (decisions §30 "12c2", the HoldFullscreen board): a phone or
// tablet shows the desk's screen with every pane in its place. A pane smaller
// than its window's natural size draws its content scaled down — the desk,
// smaller — instead of squeezing it. In a miniature nothing under the 44 px
// floor is tappable (pointer-events off); a quick tap there says "Too small —
// hold to open <window>", and holding the pane (the profile's hold time, the
// kit's ring) opens it full screen. A cell big enough to tap keeps its own gestures
// (the pool's hold selector).
//
//   const m = miniature(pane, inner, {
//     natural: () => ({ w, h }),    // the content's natural (smallest real) size
//     name: () => 'Groups',          // for the toast
//     onHold: () => …,               // open it full screen
//     enabled: () => true,           // e.g. touch screens only
//   })
//   m.update()   m.destroy()   m.scale   (1 = real size)

import { hold } from './pointer.js';
import { toast } from './toast.js';

const FLOOR = 44;
const TAPPABLE = 'button, a[href], input, select, textarea, [role="button"], [data-key], .lc-pool-cell, .ptab, .tab-dot, .lc-wb-item, .dx-cell';

export function miniature(pane, inner, { natural, name, onHold, enabled = () => true } = {}) {
  const m = { scale: 1 };
  let markTimer = 0;

  // Mark what stays tappable at this scale (its drawn size ≥ the floor).
  const mark = () => {
    clearTimeout(markTimer);
    if (m.scale >= 1) return;
    for (const el of inner.querySelectorAll(TAPPABLE)) {
      const r = el.getBoundingClientRect();
      el.classList.toggle('lc-tap-ok', r.width >= FLOOR && r.height >= FLOOR);
    }
  };
  const markSoon = () => { clearTimeout(markTimer); markTimer = setTimeout(mark, 80); };

  m.update = () => {
    const w = pane.clientWidth, h = pane.clientHeight;
    const nat = natural?.() || { w, h };
    const s = enabled() && w > 0 && h > 0 ? Math.min(1, w / nat.w, h / nat.h) : 1;
    if (Math.abs(s - m.scale) < 0.005 && (s < 1) === pane.classList.contains('lc-mini')) {
      if (s < 1) { inner.style.width = `${w / s}px`; inner.style.height = `${h / s}px`; }
      return;
    }
    m.scale = s;
    if (s < 1) {
      pane.classList.add('lc-mini');
      pane.style.setProperty('--lc-mini', String(s));
      inner.style.width = `${w / s}px`;
      inner.style.height = `${h / s}px`;
      inner.style.transform = `scale(${s})`;
      markSoon();
    } else {
      pane.classList.remove('lc-mini');
      pane.style.removeProperty('--lc-mini');
      inner.style.width = inner.style.height = inner.style.transform = '';
      for (const el of inner.querySelectorAll('.lc-tap-ok')) el.classList.remove('lc-tap-ok');
    }
  };

  const ro = new ResizeObserver(() => m.update());
  ro.observe(pane);
  const mo = new MutationObserver(() => { if (m.scale < 1) markSoon(); });
  mo.observe(inner, { childList: true, subtree: true });
  const off = hold(pane, {
    // Only a miniature takes the hold, and never from a cell big enough to tap.
    filter: e => pane.classList.contains('lc-mini') && !e.target.closest?.('.lc-tap-ok'),
    onHold: () => onHold?.(),
    onTap: () => toast(`Too small — hold to open ${name?.() || 'the window'}`),
  });
  m.destroy = () => { ro.disconnect(); mo.disconnect(); off(); clearTimeout(markTimer); };
  requestAnimationFrame(() => m.update());
  return m;
}
