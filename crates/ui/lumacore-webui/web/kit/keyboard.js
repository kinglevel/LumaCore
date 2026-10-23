// <luma-keyboard mode="text|password"> — the built-in on-screen keyboard
// (decisions §30, the Keyboards board): for console and kiosk screens with no
// OS keyboard. A letters page and a 123 page from a layout data file
// (layouts/<id>.json: value / shifted / AltGr per key, a common keyboard-file
// shape; English (US) ships, more are files later); a language key cycles
// layouts when there are several. A command-word row (from the registry,
// `setWords`) in text mode; password mode hides the word row.
//
// It types into its `target`: an <input> (keeps its caret; the OS keyboard
// stays down with `inputmode="none"`), or a sink with `kbType(act)` for the
// command line (acts: `ch:x`, `back`, `left`, `right`, `please`, `esc`).
// Events: `please` (Enter), `escape`, `hide`.
//
// The screen's Keyboard setting (decisions §30 "12b": per device and screen,
// Auto | OS | Built-in | Off) is kept here: `mode()`, `setMode()`,
// `useBuiltIn()`; the page loads and stores it (app client.json / tab).

const LAYOUTS = new Map();
let layoutIds = ['en-US'];
let current = 'en-US';

/** Load the layout files (the kit's `ready` waits for it). */
export async function loadLayouts() {
  try {
    const base = new URL('./layouts/', import.meta.url);
    const ids = await (await fetch(new URL('index.json', base))).json();
    layoutIds = ids.length ? ids : layoutIds;
    for (const id of layoutIds) LAYOUTS.set(id, await (await fetch(new URL(`${id}.json`, base))).json());
    if (!LAYOUTS.has(current)) current = layoutIds[0];
  } catch {
    // No files (an old page cache): the keyboard still works in English.
  }
}

const FALLBACK = {
  id: 'en-US', short: 'EN',
  letters: ['qwertyuiop', 'asdfghjkl', 'zxcvbnm"'].map(r => [...r].map(c => [c, c.toUpperCase(), null])),
  symbols: ['1234567890', '-/:()@.,=', "*+#!?'\""].map(r => [...r].map(c => [c, c, null])),
};
const layout = () => LAYOUTS.get(current) || FALLBACK;

/** The layout in use (id) and the ones there are. */
export function layouts() { return { current, all: [...layoutIds] }; }
export function setLayout(id) { if (LAYOUTS.has(id)) current = id; }

// ── The screen's Keyboard setting ───────────────────────────────────────────
const MODES = ['Auto', 'OS', 'Built-in', 'Off'];
let keyboardMode = 'Auto';
let kiosk = false;
export const KEYBOARD_MODES = MODES;
export function mode() { return keyboardMode; }
export function setMode(m) {
  if (!MODES.includes(m)) return;
  keyboardMode = m;
  window.dispatchEvent(new CustomEvent('lc-keyboard-mode', { detail: { mode: m } }));
}
/** The app says this device is a console / kiosk (Auto = built-in). */
export function setKiosk(on) { kiosk = !!on; }
/** Phones and tablets bring their own keyboard. */
const mobileOs = () => /Android|iPhone|iPad|Mobile/i.test(navigator.userAgent) || (navigator.maxTouchPoints > 1 && /Macintosh/.test(navigator.userAgent));
/** Whether text fields get the built-in keyboard here. */
export function useBuiltIn() {
  if (keyboardMode === 'Built-in') return true;
  if (keyboardMode !== 'Auto') return false;
  return kiosk || (matchMedia('(any-pointer: coarse)').matches && !mobileOs());
}

class LumaKeyboard extends HTMLElement {
  static get observedAttributes() { return ['mode']; }
  constructor() {
    super();
    this.page = 'abc';
    this.shift = false;
    this.words = [];
    this._target = null;
  }
  connectedCallback() { this.render(); }
  attributeChangedCallback() { if (this.isConnected) this.render(); }

  /** The input (or sink) it types into. */
  set target(el) {
    if (this._target?.removeAttribute) this._target.removeAttribute('inputmode');
    this._target = el;
    if (el?.setAttribute) el.setAttribute('inputmode', 'none');
  }
  get target() { return this._target; }

  /** Command words for the top row (text mode). */
  setWords(words) { this.words = words || []; this.render(); }

  render() {
    this.dataset.kit = 'keyboard';
    const password = this.getAttribute('mode') === 'password';
    const l = layout();
    const rows = this.page === 'abc' ? l.letters : l.symbols;
    const key = (label, cls = '', act = label) =>
      `<button type="button" class="lc-kb-key ${cls}" data-act="${act.replace(/"/g, '&quot;')}">${label}</button>`;
    const esc = c => String(c).replace(/[&<>"]/g, x => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[x]));
    const ch = k => (this.shift ? k[1] ?? k[0] : k[0]);
    const keys = row => row.map(k => key(esc(ch(k)), '', `ch:${ch(k)}`)).join('');
    const lang = layoutIds.length > 1 ? key(esc(l.short || l.id), 'mod', 'lang') : '';
    let html = '';
    if (!password && this.words.length) {
      html += `<div class="lc-kb-row lc-kb-words">${this.words.map(w => key(esc(w), 'word', `word:${w}`)).join('')}</div>`;
    }
    html += `<div class="lc-kb-row">${keys(rows[0])}${key('<luma-icon name="delete"></luma-icon>', 'wide mod', 'back')}</div>`;
    html += `<div class="lc-kb-row lc-kb-indent">${keys(rows[1])}${key('Please', 'please', 'please')}</div>`;
    html += `<div class="lc-kb-row">${key('<luma-icon name="arrow-big-up"></luma-icon>', `wide mod${this.shift ? ' on' : ''}`, 'shift')}${keys(rows[2])}${key('<luma-icon name="arrow-big-up"></luma-icon>', `wide mod${this.shift ? ' on' : ''}`, 'shift')}</div>`;
    html += `<div class="lc-kb-row">${key(this.page === 'abc' ? '123' : 'ABC', 'wide mod', 'page')}${key('Esc', 'wide mod', 'esc')}${key('space', 'space', 'ch: ')}${key('<luma-icon name="chevron-left"></luma-icon>', 'mod', 'left')}${key('<luma-icon name="chevron-right"></luma-icon>', 'mod', 'right')}${lang}${key('Hide', 'wide mod', 'hide')}</div>`;
    this.innerHTML = `<div class="lc-kb${password ? ' password' : ''}">${html}</div>`;
    for (const b of this.querySelectorAll('.lc-kb-key')) {
      // Never take focus from the field: the caret stays where it is.
      b.addEventListener('pointerdown', e => e.preventDefault());
      b.addEventListener('click', () => this.press(b.dataset.act));
    }
  }

  press(act) {
    const t = this._target;
    if (act === 'shift') { this.shift = !this.shift; this.render(); return; }
    if (act === 'page') { this.page = this.page === 'abc' ? '123' : 'abc'; this.render(); return; }
    if (act === 'lang') {
      current = layoutIds[(layoutIds.indexOf(current) + 1) % layoutIds.length];
      this.render();
      return;
    }
    if (act === 'hide') { this.dispatchEvent(new CustomEvent('hide', { bubbles: true })); return; }
    if (act.startsWith('word:')) {
      const v = t?.value ?? '';
      act = `ch:${(t?.kbType || !v || v.endsWith(' ') ? '' : ' ') + act.slice(5)} `;
    }
    // A sink (the command line) takes the act itself.
    if (t?.kbType) {
      t.kbType(act);
    } else if (t && act.startsWith('ch:')) {
      const text = act.slice(3);
      const s = t.selectionStart ?? t.value.length, e = t.selectionEnd ?? t.value.length;
      t.value = t.value.slice(0, s) + text + t.value.slice(e);
      t.selectionStart = t.selectionEnd = s + text.length;
      t.dispatchEvent(new Event('input', { bubbles: true }));
    } else if (act === 'back' && t) {
      const s = t.selectionStart ?? t.value.length, e = t.selectionEnd ?? s;
      const from = s === e ? Math.max(0, s - 1) : s;
      t.value = t.value.slice(0, from) + t.value.slice(e);
      t.selectionStart = t.selectionEnd = from;
      t.dispatchEvent(new Event('input', { bubbles: true }));
    } else if (act === 'left' && t) {
      t.selectionStart = t.selectionEnd = Math.max(0, (t.selectionStart ?? 0) - 1);
    } else if (act === 'right' && t) {
      t.selectionStart = t.selectionEnd = Math.min(t.value.length, (t.selectionEnd ?? 0) + 1);
    }
    if (act.startsWith('ch:') && this.shift) { this.shift = false; this.render(); }
    if (act === 'please') this.dispatchEvent(new CustomEvent('please', { bubbles: true }));
    else if (act === 'esc') this.dispatchEvent(new CustomEvent('escape', { bubbles: true }));
  }
}
customElements.define('luma-keyboard', LumaKeyboard);
