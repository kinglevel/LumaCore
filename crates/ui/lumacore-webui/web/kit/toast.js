// The kit's toast (decisions §30 "12c2", the HoldFullscreen board): one short
// line at the bottom of the screen, above the command line bar, that goes by
// itself — "Too small — hold to open Groups". Nothing to tap (a pop-up or a
// card is for anything that matters: the feedback system).
//
//   toast(text, { ms = 1600 })
//
// One variant holds a text for the user to copy (the NAV inspector's copy when
// the browser's clipboard can't be reached, docs/dev/ui-kit/mockups/2026-10-23-nav-mode):
// the text stands selected in a read-only field that has the focus, ready for
// Ctrl+C; a copy (or Esc, or the time) ends it and the focus goes back.
//
//   toast(text, { hold: 'the text', ms = 10000, onCopy })

let el = null;
let timer = 0;
let back = null; // the element that had the focus before a hold

function end() {
  if (!el) return;
  el.classList.remove('on', 'lc-toast-hold');
  const field = el.querySelector('input');
  if (field) {
    field.remove();
    if (back?.isConnected) back.focus({ preventScroll: true });
  }
  back = null;
}

export function toast(text, { ms, hold = null, onCopy = null } = {}) {
  if (!el) {
    el = document.createElement('div');
    el.className = 'lc-toast';
    el.dataset.kit = 'toast';
    el.setAttribute('role', 'status');
    el.setAttribute('aria-live', 'polite');
  }
  if (!el.isConnected) document.body.append(el);
  end();
  el.textContent = text;
  if (hold != null) {
    back = document.activeElement;
    el.classList.add('lc-toast-hold');
    const field = document.createElement('input');
    field.readOnly = true;
    field.value = hold;
    field.spellcheck = false;
    field.setAttribute('aria-label', text);
    // The page's key handlers leave an input alone; Ctrl+C copies its selection.
    field.addEventListener('copy', () => { setTimeout(() => { end(); onCopy?.(); }, 0); });
    field.addEventListener('keydown', e => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); end(); } });
    el.append(field);
  }
  el.classList.remove('on');
  void el.offsetWidth; // restart the fade
  el.classList.add('on');
  const field = el.querySelector('input');
  if (field) { field.focus({ preventScroll: true }); field.select(); }
  clearTimeout(timer);
  timer = setTimeout(end, ms ?? (hold != null ? 10000 : 1600));
}

/** The toast's text while it shows (tests), else ''. */
export function toastText() { return el?.classList.contains('on') ? el.textContent : ''; }
