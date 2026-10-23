// The pointer core (decisions §30): every gesture in the UI goes through here.
// Pointer Events only, one entry per pointer (several fingers work several
// controls), every gesture ends on pointerup, pointercancel AND
// lostpointercapture. Two gestures for now:
//
//   hold(el, …)  "show me more": a tap stays a tap; holding still for the
//                profile's hold time (default 500 ms) fires onHold, with a ring
//                filling over the time. Right-click (mouse) and the browser's
//                own long-press `contextmenu` fire the same onHold, so one
//                handler serves mouse, pen and touch (iOS never sends
//                contextmenu on a long press).
//   drag(el, …)  a captured drag (split handles, faders later): the element
//                keeps the pointer until it is released, whatever it moves over.
//
//   lift(el, …)  the one drag engine for a surface of items (decisions §34 round 3, 18;
//                "4d4 hardened" 1, 3): tap selects, a drag on an item lifts and moves it, a
//                drag on empty ground lassos, hold-and-release is the context menu, pagers
//                spring under a lifted item. The DMX map's bars and slot grid; every pool,
//                tile set and tree later.
//
// One hold time (the profile's HoldTime) and one owner per press (decisions
// §30 "12e hardening"): the innermost hold that can be tapped takes the press
// and every other hold on the way up leaves it alone — a tab inside a pane
// never arms both. Inside a miniature, a target under the 44 px floor is not
// tappable, so the press goes on to the pane.

let holdMs = 500;
const SLOP = 8;       // px a finger may wander before a hold becomes a scroll/drag
const RING_DELAY = 140; // ms before the ring shows (a plain tap shows nothing)

/** The profile's hold time (GET /api/ui-keys `hold_ms`). */
export function setHoldTime(ms) { if (ms > 0) holdMs = ms; }
export function holdTime() { return holdMs; }
/** The profile's double-tap time (GET /api/ui-keys `double_click_ms`): one for every gesture. */
let doubleMs = 300;
export function setDoubleTime(ms) { if (ms > 0) doubleMs = ms; }
export function doubleTime() { return doubleMs; }

// ── Ring ──
let ringEl = null;
function ring(x, y, ms) {
  hideRing();
  const el = document.createElement('div');
  el.className = 'lc-ring';
  el.dataset.kit = 'ring';
  el.style.left = `${x}px`;
  el.style.top = `${y}px`;
  el.style.setProperty('--lc-ring-ms', `${Math.max(0, ms - RING_DELAY)}ms`);
  el.style.setProperty('--lc-ring-delay', `${RING_DELAY}ms`);
  el.innerHTML = '<svg viewBox="0 0 44 44" aria-hidden="true"><circle class="lc-ring-track" cx="22" cy="22" r="19"/><circle class="lc-ring-fill" cx="22" cy="22" r="19"/></svg>';
  document.body.appendChild(el);
  ringEl = el;
}
/**
 * Take this press (a pointerdown event) for one gesture. False when an inner
 * gesture already took it. Hand-made press handlers (UI keys) call this too.
 */
export function claim(e) {
  if (e.__lcOwned) return false;
  e.__lcOwned = true;
  return true;
}

/** The hold ring at (x, y) filling over `ms` — for hand-made holds (UI keys, pool buttons). */
export function showRing(x, y, ms = holdMs) { ring(x, y, ms); }
export function hideRing() { ringEl?.remove(); ringEl = null; }

// After a hold, the click that follows the release must not act as a tap, and
// the browser's own long-press menu (Android, WebView2) must not open as well.
let swallowClickUntil = 0;
document.addEventListener('click', e => {
  if (performance.now() < swallowClickUntil) { e.preventDefault(); e.stopPropagation(); swallowClickUntil = 0; }
}, true);
document.addEventListener('contextmenu', e => {
  if (performance.now() < swallowClickUntil && e.pointerType !== 'mouse') { e.preventDefault(); e.stopPropagation(); }
}, true);

/**
 * Hold = "show me more" on `el`.
 *   onHold(info)   info = { x, y, target, pointerType, event }
 *   ms             hold time (default: the profile's)
 *   filter(e)      return false to ignore a press (e.g. on an inner control)
 *   contextmenu    also fire on right-click / native long-press (default true)
 *   ringOn         show the ring (default true)
 *   onTap(info)    a press released before the hold time, without moving
 *                  (12c2: a miniature pane's "Too small — hold to open")
 *   onArm(ms)      a press that may become a hold started (a host's own
 *                  progress, the claim button's fill; ringOn false)
 *   onDisarm()     that press ended before the hold time
 * Returns a function that removes the listeners.
 */
export function hold(el, { onHold, onTap, onArm, onDisarm, ms, filter, contextmenu = true, ringOn = true } = {}) {
  let down = null; // { id, x, y, timer, fired }
  const cancel = () => {
    if (!down) return;
    clearTimeout(down.timer);
    if (!down.fired) { hideRing(); onDisarm?.(); }
    down = null;
  };
  // `held`: a press held still (its release would click) — swallow that click.
  // A right-click has no click to swallow.
  const fire = (e, x, y, held) => {
    if (held) {
      swallowClickUntil = performance.now() + 700;
      // Only the release's own click: once the finger is up (even when the
      // hold replaced the element under it), the next tap is a tap.
      document.addEventListener('pointerup', () => setTimeout(() => { swallowClickUntil = 0; }, 60), { once: true, capture: true });
    }
    onHold?.({ x, y, target: e.target, pointerType: e.pointerType || 'mouse', event: e });
  };
  const onDown = e => {
    if (down || (e.pointerType === 'mouse' && e.button !== 0) || !e.isPrimary) return;
    if (filter && !filter(e)) return;
    // In a miniature only what is big enough to tap keeps its own hold.
    const mini = e.target.closest?.('.lc-mini');
    if (mini && mini !== el && !e.target.closest('.lc-tap-ok')) return;
    if (!claim(e)) return;
    const t = ms ?? holdMs;
    down = { id: e.pointerId, x: e.clientX, y: e.clientY, fired: false };
    if (ringOn) ring(e.clientX, e.clientY, t);
    onArm?.(t);
    down.timer = setTimeout(() => {
      if (!down) return;
      down.fired = true;
      hideRing();
      fire(e, down.x, down.y, true);
    }, t);
  };
  const onMove = e => {
    if (!down || e.pointerId !== down.id) return;
    if (Math.hypot(e.clientX - down.x, e.clientY - down.y) > SLOP) cancel();
  };
  const onUp = e => {
    if (!down || e.pointerId !== down.id) return;
    const tap = e.type === 'pointerup' && !down.fired;
    cancel();
    if (tap) onTap?.({ x: e.clientX, y: e.clientY, target: e.target, pointerType: e.pointerType || 'mouse', event: e });
  };
  const onContext = e => {
    if (filter && !filter(e)) return;
    e.preventDefault();
    e.stopPropagation();
    const already = down?.fired;
    cancel();
    if (!already) fire(e, e.clientX, e.clientY, false);
  };
  el.addEventListener('pointerdown', onDown);
  el.addEventListener('pointermove', onMove);
  el.addEventListener('pointerup', onUp);
  el.addEventListener('pointercancel', cancel);
  el.addEventListener('lostpointercapture', cancel);
  el.addEventListener('pointerleave', onUp);
  if (contextmenu) el.addEventListener('contextmenu', onContext);
  return () => {
    cancel();
    el.removeEventListener('pointerdown', onDown);
    el.removeEventListener('pointermove', onMove);
    el.removeEventListener('pointerup', onUp);
    el.removeEventListener('pointercancel', cancel);
    el.removeEventListener('lostpointercapture', cancel);
    el.removeEventListener('pointerleave', onUp);
    if (contextmenu) el.removeEventListener('contextmenu', onContext);
  };
}

/**
 * A captured drag on `el`: onStart(info) → onMove(info) … → onEnd(info, cancelled).
 * info = { x, y, dx, dy, pointerType, event }. Each pointer is its own drag,
 * so two fingers can drag two elements at once.
 */
export function drag(el, { onStart, onMove, onEnd, filter } = {}) {
  const drags = new Map();
  const info = (d, e) => ({ x: e.clientX, y: e.clientY, dx: e.clientX - d.x, dy: e.clientY - d.y, pointerType: e.pointerType, event: e });
  const end = (e, cancelled) => {
    const d = drags.get(e.pointerId);
    if (!d) return;
    drags.delete(e.pointerId);
    try { el.releasePointerCapture(e.pointerId); } catch (_) {}
    onEnd?.(info(d, e), cancelled);
  };
  el.addEventListener('pointerdown', e => {
    if ((e.pointerType === 'mouse' && e.button !== 0) || (filter && !filter(e))) return;
    e.preventDefault();
    const d = { x: e.clientX, y: e.clientY };
    drags.set(e.pointerId, d);
    try { el.setPointerCapture(e.pointerId); } catch (_) {}
    onStart?.(info(d, e));
  });
  el.addEventListener('pointermove', e => {
    const d = drags.get(e.pointerId);
    if (d) onMove?.(info(d, e));
  });
  el.addEventListener('pointerup', e => end(e, false));
  el.addEventListener('pointercancel', e => end(e, true));
  el.addEventListener('lostpointercapture', e => end(e, false));
}

/**
 * A horizontal swipe on `el` (decisions §30 "12e": a full-screen pane swipes
 * to its neighbours). A finger that moves more sideways than up and down
 * past the slop follows (`onMove(dx)`, the host shifts the content); let go
 * past `share` of the width it is `onSwipe('next' | 'prev')`, else
 * `onCancel()` (spring back). Vertical moves stay the content's scroll
 * (`touch-action: pan-y` on `el`). Touch and pen only — a mouse uses ‹ ›.
 */
export function swipe(el, { onMove, onSwipe, onCancel, share = 0.2 } = {}) {
  let d = null;
  el.style.touchAction = 'pan-y';
  el.addEventListener('pointerdown', e => {
    if (e.pointerType === 'mouse' || !e.isPrimary) return;
    d = { id: e.pointerId, x: e.clientX, y: e.clientY, dx: 0, side: null };
  });
  el.addEventListener('pointermove', e => {
    if (!d || e.pointerId !== d.id) return;
    const dx = e.clientX - d.x, dy = e.clientY - d.y;
    if (d.side === null && Math.hypot(dx, dy) > SLOP) d.side = Math.abs(dx) > Math.abs(dy);
    if (!d.side) return;
    d.dx = dx;
    onMove?.(dx);
  });
  const end = (e, cancelled) => {
    if (!d || e.pointerId !== d.id) return;
    const { side, dx } = d;
    d = null;
    if (side && !cancelled && Math.abs(dx) > el.clientWidth * share) onSwipe?.(dx < 0 ? 'next' : 'prev');
    else if (side) onCancel?.();
  };
  el.addEventListener('pointerup', e => end(e, false));
  el.addEventListener('pointercancel', e => end(e, true));
}

/**
 * The one drag engine for a surface of items (`el` scrolls its content, or holds `scroller`: an
 * element, or a function returning it when the host redraws its content).
 *   item(e)                 the item under a press (any value), null on empty ground;
 *                           a press on a control the host keeps (a button) returns undefined
 *   mode()                  the device's one-finger rule for touch: 'auto' | 'lasso' | 'scroll'
 *                           (grid.js `dragSetting`; auto = scroll on a phone, lasso on a desk)
 *   onTap(item, info)       a press released without moving (item null: empty ground)
 *   onLift(item, info)      a move begins; then onMove(info) … onDrop(info, cancelled)
 *   onLasso(info)           a lasso begins; then onLassoMove(info) … onLassoEnd(info, cancelled)
 *   onContext(item, info)   hold and let go without moving (touch), right-click (mouse)
 *   springs()               [{ el, fire }]: a lifted item held still over `el` for the hold
 *                           time fires it (a pager, a tab; 4d4 3) — once per visit
 *   phone                   () → true on a phone (auto = scroll)
 * Rules — mouse / pen: a press on an item that moves lifts it at once, on empty ground it
 * lassos; right-click is the context. Touch: the hold is the press — hold still on an item and
 * it lifts (a short vibration), on empty ground the lasso starts; a plain finger scrolls in
 * Auto (phone) and Scroll, in Lasso it lifts / lassos at once and two fingers scroll; let go
 * after the hold without moving is the context. The ring fills while a touch press waits.
 * info = { x, y, dx, dy, add (Shift / Meow adds), pointerType, event }.
 * Returns a function that removes the listeners.
 */
export function lift(el, o = {}) {
  const sc = () => (typeof o.scroller === 'function' ? o.scroller() : o.scroller) || el;
  let g = null;           // the one gesture
  const fingers = new Map();
  const mk = (d, e, add) => ({ x: e.clientX, y: e.clientY, dx: e.clientX - d.x, dy: e.clientY - d.y, add: add ?? !!(e.shiftKey || e.metaKey), pointerType: e.pointerType || 'mouse', event: e });
  const info = (e, add) => mk(g, e, add);
  const oneFinger = () => { const m = o.mode?.() || 'auto'; return m === 'lasso' || (m === 'auto' && !o.phone?.()); };
  const disarmSpring = () => { if (!g?.spring) return; clearTimeout(g.spring.timer); g.spring.el.classList.remove('lc-sprung'); g.spring = null; };
  const end = (e, cancelled) => {
    if (!g || e.pointerId !== g.id) return;
    const d = g;
    g = null;
    clearTimeout(d.timer);
    hideRing();
    if (d.spring) { clearTimeout(d.spring.timer); d.spring.el.classList.remove('lc-sprung'); }
    cancelAnimationFrame(d.raf);
    try { el.releasePointerCapture(e.pointerId); } catch (_) {}
    if (d.lifted) {
      o.onDrop?.(mk(d, e, d.add), cancelled || !d.moved);
      if (!cancelled && !d.moved && d.held) { swallow(); o.onContext?.(d.item, { x: d.x, y: d.y, pointerType: e.pointerType, event: e }); }
      return;
    }
    if (d.lasso) { o.onLassoEnd?.(mk(d, e, d.add), cancelled || !d.moved); if (!d.moved && !cancelled && !d.held) o.onTap?.(d.item, mk(d, e, d.add)); return; }
    if (cancelled || d.moved) return;
    if (d.held && d.item == null) return;
    o.onTap?.(d.item, mk(d, e, d.add));
  };
  // The click after a hold must not act (the engine's release is its own gesture).
  const swallow = () => { swallowClickUntil = performance.now() + 700; document.addEventListener('pointerup', () => setTimeout(() => { swallowClickUntil = 0; }, 60), { once: true, capture: true }); };
  const begin = (e, kind) => {
    if (kind === 'lift') { g.lifted = true; hideRing(); if (e.pointerType === 'touch') { try { navigator.vibrate?.(12); } catch (_) {} } o.onLift?.(g.item, info(e, g.add)); }
    else { g.lasso = true; hideRing(); o.onLasso?.(info(e, g.add)); }
  };
  const edges = () => {
    if (g?.raf) return;
    const step = () => {
      if (!g) return;
      g.raf = 0;
      if (!(g.lifted || g.lasso) || !g.last) return;
      const r = sc().getBoundingClientRect();
      const vy = g.last.y < r.top + 28 ? -10 : g.last.y > r.bottom - 28 ? 10 : 0;
      const vx = g.last.x < r.left + 28 ? -10 : g.last.x > r.right - 28 ? 10 : 0;
      if (!vx && !vy) return;
      sc().scrollTop += vy; sc().scrollLeft += vx;
      (g.lifted ? o.onMove : o.onLassoMove)?.(info(g.last.e, g.add));
      g.raf = requestAnimationFrame(step);
    };
    g.raf = requestAnimationFrame(step);
  };
  const onDown = e => {
    if ((e.pointerType === 'mouse' && e.button !== 0)) return;
    const item = o.item?.(e);
    if (item === undefined) return;
    fingers.set(e.pointerId, { x: e.clientX, y: e.clientY, x0: e.clientX, y0: e.clientY });
    if (fingers.size === 2 && e.pointerType === 'touch' && g && !g.lifted && !g.lasso) {
      // A second finger before anything started: the two scroll (Lasso mode's scroll).
      clearTimeout(g.timer); hideRing();
      g = { two: true, sx: sc().scrollLeft, sy: sc().scrollTop };
      return;
    }
    if (g) return;
    if (!claim(e)) return;
    e.preventDefault();
    g = { id: e.pointerId, x: e.clientX, y: e.clientY, item, add: !!(e.shiftKey || e.metaKey), type: e.pointerType, moved: false, held: false, lifted: false, lasso: false, sx: sc().scrollLeft, sy: sc().scrollTop, t0: performance.now() };
    try { el.setPointerCapture(e.pointerId); } catch (_) {}
    if (e.pointerType !== 'mouse') {
      ring(e.clientX, e.clientY, holdMs);
      g.timer = setTimeout(() => {
        if (!g || g.id !== e.pointerId || g.moved) return;
        g.held = true;
        begin(e, item == null ? 'lasso' : 'lift');
      }, holdMs);
    }
  };
  const onMove = e => {
    const f = fingers.get(e.pointerId);
    if (f) { f.x = e.clientX; f.y = e.clientY; }
    if (!g) return;
    if (g.two) {
      const pts = [...fingers.values()];
      if (pts.length < 2) return;
      const dx = pts.reduce((s, q) => s + q.x - q.x0, 0) / pts.length, dy = pts.reduce((s, q) => s + q.y - q.y0, 0) / pts.length;
      sc().scrollLeft = g.sx - dx; sc().scrollTop = g.sy - dy;
      return;
    }
    if (e.pointerId !== g.id) return;
    const dist = Math.hypot(e.clientX - g.x, e.clientY - g.y);
    if (!g.lifted && !g.lasso) {
      if (dist <= SLOP) return;
      if (g.type === 'touch' && !oneFinger() && !g.held) {
        // A plain finger scrolls (the hold is gone).
        clearTimeout(g.timer); hideRing();
        if (g.scroll == null) g.scroll = true;
        sc().scrollLeft = g.sx - (e.clientX - g.x); sc().scrollTop = g.sy - (e.clientY - g.y);
        return;
      }
      clearTimeout(g.timer);
      begin(e, g.item == null ? 'lasso' : 'lift');
    }
    if (g.scroll) return;
    g.moved ||= dist > SLOP;
    g.last = { x: e.clientX, y: e.clientY, e };
    if (g.lifted) {
      // A pager under the lifted item springs after the hold time, once per visit.
      const over = (o.springs?.() || []).find(s => { const r = s.el.getBoundingClientRect(); return e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom; });
      if (over && g.spring?.el !== over.el) {
        disarmSpring();
        over.el.classList.add('lc-sprung');
        over.el.style.setProperty('--lc-dwell-ms', `${holdMs}ms`);
        g.spring = { el: over.el, timer: setTimeout(() => { if (!g || g.spring?.el !== over.el) return; over.el.classList.remove('lc-sprung'); g.spring = { el: over.el, fired: true }; over.fire(); o.onMove?.(info(e, g.add)); }, holdMs) };
      } else if (!over) disarmSpring();
      o.onMove?.(info(e, g.add));
    } else o.onLassoMove?.(info(e, g.add));
    edges();
  };
  const onUp = e => { fingers.delete(e.pointerId); if (g?.two) { if (!fingers.size) g = null; return; } end(e, false); };
  const onCancel = e => { fingers.delete(e.pointerId); if (g?.two) { if (!fingers.size) g = null; return; } end(e, true); };
  const onContext = e => {
    if (e.pointerType && e.pointerType !== 'mouse') { e.preventDefault(); return; }
    if (e.button !== 2 || e.ctrlKey) { e.preventDefault(); return; }
    const item = o.item?.(e);
    if (item === undefined) return;
    e.preventDefault();
    o.onContext?.(item, { x: e.clientX, y: e.clientY, pointerType: 'mouse', event: e });
  };
  el.addEventListener('pointerdown', onDown);
  el.addEventListener('pointermove', onMove);
  el.addEventListener('pointerup', onUp);
  el.addEventListener('pointercancel', onCancel);
  el.addEventListener('lostpointercapture', onCancel);
  el.addEventListener('contextmenu', onContext);
  return () => {
    el.removeEventListener('pointerdown', onDown);
    el.removeEventListener('pointermove', onMove);
    el.removeEventListener('pointerup', onUp);
    el.removeEventListener('pointercancel', onCancel);
    el.removeEventListener('lostpointercapture', onCancel);
    el.removeEventListener('contextmenu', onContext);
  };
}
