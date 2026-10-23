// The kit's toast (decisions §30 "12c2", the HoldFullscreen board): one short
// line at the bottom of the screen, above the command line bar, that goes by
// itself — "Too small — hold to open Groups". Nothing to tap (a pop-up or a
// card is for anything that matters: the feedback system).
//
//   toast(text, { ms = 1600 })

let el = null;
let timer = 0;

export function toast(text, { ms = 1600 } = {}) {
  if (!el) {
    el = document.createElement('div');
    el.className = 'lc-toast';
    el.dataset.kit = 'toast';
    el.setAttribute('role', 'status');
    el.setAttribute('aria-live', 'polite');
  }
  if (!el.isConnected) document.body.append(el);
  el.textContent = text;
  el.classList.remove('on');
  void el.offsetWidth; // restart the fade
  el.classList.add('on');
  clearTimeout(timer);
  timer = setTimeout(() => { el.classList.remove('on'); }, ms);
}

/** The toast's text while it shows (tests), else ''. */
export function toastText() { return el?.classList.contains('on') ? el.textContent : ''; }
