// The phone's one sheet (decisions §37 7: "one sheet look for every phone pop-up" — the value
// dialog, the context menu, the setup frame's tree / More / lock sheets). Upright it rises from
// the bottom: a grab handle, a title (and a small line under it) with ✕, then its body, which
// scrolls inside; at most about two-thirds of the screen, so the rows it acts on stay in sight.
// On its side (§37 2) it opens from the right. A swipe down (right, on its side) on the handle or
// the title, a tap outside, ✕ or Esc close it.
//
//   openSheet({ title, sub, body: Node | html, side, host, cls, onClose })
//     → { el, body, close(), setTitle(title, sub) }
//     host: the element it opens over (default the page); it covers the host's box
//   grabToClose(handle, el, close, { side }) — the swipe that closes, for a pop-up that is a sheet
//     only on a phone (the context menu, the value dialog)

import { drag } from './pointer.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const open = [];

const onKey = e => {
  if (e.key !== 'Escape' || !open.length) return;
  e.preventDefault();
  e.stopPropagation();
  open.at(-1)();
};

/** A drag on `handle` moves `el` down (or right): past a third of its size, or a quick flick past
 *  60 px, it closes; else it springs back. */
export function grabToClose(handle, el, close, { side = false } = {}) {
  let t0 = 0;
  drag(handle, {
    filter: e => !e.target.closest('button, input, select, a'),
    onStart: () => { t0 = performance.now(); el.style.transition = 'none'; },
    onMove: ({ dx, dy }) => { el.style.transform = side ? `translateX(${Math.max(0, dx)}px)` : `translateY(${Math.max(0, dy)}px)`; },
    onEnd: ({ dx, dy }) => {
      const d = side ? dx : dy;
      const size = side ? el.offsetWidth : el.offsetHeight;
      const quick = performance.now() - t0 < 300 && d > 60;
      el.style.transition = '';
      if (d > size / 3 || quick) close();
      else el.style.transform = '';
    },
  });
}

export function openSheet({ title = '', sub = '', body, side = false, host, cls = '', onClose } = {}) {
  const scrim = document.createElement('div');
  scrim.className = `lc-psheet-scrim${side ? ' side' : ''}${host ? ' in' : ''}`;
  scrim.dataset.kit = 'sheet';
  const el = document.createElement('div');
  el.className = `lc-psheet${cls ? ` ${cls}` : ''}`;
  el.setAttribute('role', 'dialog');
  el.innerHTML = `<div class="lc-psheet-head" data-head><i class="lc-grab" aria-hidden="true"></i>
      <div class="lc-psheet-title"><b data-title>${esc(title)}</b><small data-sub${sub ? '' : ' hidden'}>${esc(sub)}</small></div>
      <button type="button" class="lc-close" data-close aria-label="Close"><luma-icon name="x"></luma-icon></button></div>
    <div class="lc-psheet-body" data-body></div>`;
  const b = el.querySelector('[data-body]');
  if (typeof body === 'string') b.innerHTML = body;
  else if (body) b.append(body);
  scrim.append(el);
  (host || document.body).append(scrim);
  let shut = false;
  const close = () => {
    if (shut) return;
    shut = true;
    const i = open.indexOf(close);
    if (i >= 0) open.splice(i, 1);
    if (!open.length) document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    onClose?.();
  };
  if (!open.length) document.addEventListener('keydown', onKey, true);
  open.push(close);
  el.querySelector('[data-close]').onclick = close;
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  grabToClose(el.querySelector('[data-head]'), el, close, { side });
  return {
    el, body: b, close,
    setTitle(t, s = '') {
      el.querySelector('[data-title]').textContent = t;
      const small = el.querySelector('[data-sub]');
      small.textContent = s;
      small.hidden = !s;
    },
  };
}

/** Is a sheet open (a host's Esc waits for it)? */
export function sheetOpen() { return open.length > 0; }
