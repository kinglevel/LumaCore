// <luma-keymap> — the key editor (decisions 0295, the approved mockups
// docs/dev/ui-kit/mockups/2026-10-23-profile-editor/02-keys.html and 03-cycles.html): your
// profile's keyboard in each shortcut layer, drawn from the `keys` feed (lumacore_views::keys) —
// the station says what every key does and where it comes from; nothing is resolved here but which
// physical key a name is.
//
//   the bar        All · KBD · PRG · NAV (the layer drawn), Ctrl · Alt · Meow (the modifier layer;
//                  holding the real keys does the same), Find (a key or a line; Record = press a
//                  key), the keyboard's shape (ISO / ANSI)
//   the keyboard   every physical key where it sits: core keys striped with a lock, LumaCore's
//                  defaults raised, your own in the accent, the command line's and NAV's own keys
//                  tinted, a turned-off key crossed; a corner when this mode's binding overrides
//                  All, a badge with the count of a press cycle
//   the key panel  what the picked key does in every layer; its press cycle (each press a line that
//                  types or runs; ↑ ↓ ×, + a press), its Meow presses, the other gestures — each
//                  change saved as it is made (a row when it is left, Enter or Tab; a toggle, ↑ ↓ ×
//                  at once; the window's close saves the row being typed); Unbind, Turn off, Back
//                  on — every change one line (`run`): your own key's `Set UIKey`, else a `Bind`
//   the list       <luma-grid> of the layer's bindings; a row picks its key
//
//   el.show(data)  ·  events: run { line } (bubbles)
//   openKeyEditor({ line, close }) → { update(data), close() }: the pop-up the page opens on the
//   `key_editor` event.

import './grid.js';
import { icon } from './icon.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** A template from the feed with its `{name}`s filled. */
const fill = (tpl, words) => String(tpl || '').replace(/\{(\w+)\}/g, (m, k) => (k in words ? String(words[k]) : m));
const LAYERS = ['All', 'KBD', 'PRG', 'NAV'];
const SOURCE = { core: 'core · locked', lumacore: 'LumaCore', profile: 'yours', off: 'turned off', line: 'the command line', nav: 'NAV', types: 'types', free: 'free' };
const GESTURES = [['Release', 'Release'], ['Hold', 'Hold'], ['ReleaseAfterHold', 'Release after hold'], ['DoublePress', 'Double press']];

// ── The keyboards: [code, base, shifted, x, y, w, h] in key units (y: 0 = the F row) ──
function row(y, x, keys) {
  return keys.map(([code, base, sh, w = 1]) => { const k = [code, base, sh, x, y, w, 1]; x += w; return k; });
}
const letters = s => [...s].map(c => [`Key${c.toUpperCase()}`, c, c.toUpperCase()]);
const FN = [['Escape', 'Esc', '', 0, 0, 1, 1], ...[2, 3, 4, 5, 6.5, 7.5, 8.5, 9.5, 11, 12, 13, 14].map((x, i) => [`F${i + 1}`, `F${i + 1}`, '', x, 0, 1, 1])];
const CLUSTER = [['PrintScreen', 'PrtSc', '', 15.25, 0], ['ScrollLock', 'ScrLk', '', 16.25, 0], ['Pause', 'Pause', '', 17.25, 0],
  ['Insert', 'Ins', '', 15.25, 1.5], ['Home', 'Home', '', 16.25, 1.5], ['PageUp', 'PgUp', '', 17.25, 1.5],
  ['Delete', 'Del', '', 15.25, 2.5], ['End', 'End', '', 16.25, 2.5], ['PageDown', 'PgDn', '', 17.25, 2.5],
  ['ArrowUp', '↑', '', 16.25, 4.5], ['ArrowLeft', '←', '', 15.25, 5.5], ['ArrowDown', '↓', '', 16.25, 5.5], ['ArrowRight', '→', '', 17.25, 5.5]].map(k => [...k, 1, 1]);
const PAD = [['NumLock', 'Num', '', 18.5, 1.5, 1, 1], ['NumpadDivide', '/', '', 19.5, 1.5, 1, 1], ['NumpadMultiply', '*', '', 20.5, 1.5, 1, 1], ['NumpadSubtract', '-', '', 21.5, 1.5, 1, 1],
  ['Numpad7', '7', '', 18.5, 2.5, 1, 1], ['Numpad8', '8', '', 19.5, 2.5, 1, 1], ['Numpad9', '9', '', 20.5, 2.5, 1, 1], ['NumpadAdd', '+', '', 21.5, 2.5, 1, 2],
  ['Numpad4', '4', '', 18.5, 3.5, 1, 1], ['Numpad5', '5', '', 19.5, 3.5, 1, 1], ['Numpad6', '6', '', 20.5, 3.5, 1, 1],
  ['Numpad1', '1', '', 18.5, 4.5, 1, 1], ['Numpad2', '2', '', 19.5, 4.5, 1, 1], ['Numpad3', '3', '', 20.5, 4.5, 1, 1], ['NumpadEnter', 'Enter', '', 21.5, 4.5, 1, 2],
  ['Numpad0', '0', '', 18.5, 5.5, 2, 1], ['NumpadDecimal', ',', '', 20.5, 5.5, 1, 1]];
const bottom = altgr => row(5.5, 0, [['ControlLeft', 'Ctrl', '', 1.25], ['MetaLeft', 'Super', '', 1.25], ['AltLeft', 'Alt', '', 1.25], ['Space', 'Space', '', 6.25],
  ['AltRight', altgr, '', 1.25], ['MetaRight', 'Super', '', 1.25], ['ContextMenu', 'Menu', '', 1.25], ['ControlRight', 'Ctrl', '', 1.25]]);
const DIGITS_ISO = [['1', '!'], ['2', '"'], ['3', '#'], ['4', '¤'], ['5', '%'], ['6', '&'], ['7', '/'], ['8', '('], ['9', ')'], ['0', '=']].map(([d, s]) => [`Digit${d}`, d, s]);
const DIGITS_ANSI = [['1', '!'], ['2', '@'], ['3', '#'], ['4', '$'], ['5', '%'], ['6', '^'], ['7', '&'], ['8', '*'], ['9', '('], ['0', ')']].map(([d, s]) => [`Digit${d}`, d, s]);
export const KEYBOARDS = {
  ISO: [...FN, ...CLUSTER, ...PAD,
    ...row(1.5, 0, [['Backquote', '§', '½'], ...DIGITS_ISO, ['Minus', '+', '?'], ['Equal', '´', '`'], ['Backspace', 'Backspace', '', 2]]),
    ...row(2.5, 0, [['Tab', 'Tab', '', 1.5], ...letters('qwertyuiop'), ['BracketLeft', 'å', 'Å'], ['BracketRight', '¨', '^']]),
    ['Enter', 'Enter', '', 13.5, 2.5, 1.5, 2],
    ...row(3.5, 0, [['CapsLock', 'Caps', '', 1.75], ...letters('asdfghjkl'), ['Semicolon', 'ö', 'Ö'], ['Quote', 'ä', 'Ä'], ['Backslash', "'", '*']]),
    ...row(4.5, 0, [['ShiftLeft', 'Shift', '', 1.25], ['IntlBackslash', '<', '>'], ...letters('zxcvbnm'), ['Comma', ',', ';'], ['Period', '.', ':'], ['Slash', '-', '_'], ['ShiftRight', 'Shift', '', 2.75]]),
    ...bottom('AltGr')],
  ANSI: [...FN, ...CLUSTER, ...PAD,
    ...row(1.5, 0, [['Backquote', '`', '~'], ...DIGITS_ANSI, ['Minus', '-', '_'], ['Equal', '=', '+'], ['Backspace', 'Backspace', '', 2]]),
    ...row(2.5, 0, [['Tab', 'Tab', '', 1.5], ...letters('qwertyuiop'), ['BracketLeft', '[', '{'], ['BracketRight', ']', '}'], ['Backslash', '\\', '|', 1.5]]),
    ...row(3.5, 0, [['CapsLock', 'Caps', '', 1.75], ...letters('asdfghjkl'), ['Semicolon', ';', ':'], ['Quote', "'", '"'], ['Enter', 'Enter', '', 2.25]]),
    ...row(4.5, 0, [['ShiftLeft', 'Shift', '', 2.25], ...letters('zxcvbnm'), ['Comma', ',', '<'], ['Period', '.', '>'], ['Slash', '/', '?'], ['ShiftRight', 'Shift', '', 2.75]]),
    ...bottom('Alt')],
};
const MODS = { ControlLeft: 'Ctrl', ControlRight: 'Ctrl', AltLeft: 'Alt', AltRight: 'Alt', ShiftLeft: 'Shift', ShiftRight: 'Shift' };
const NAMED = { Escape: 'Esc', Space: 'Space', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right' };
const SPECIAL = /^(F\d+|Tab|Enter|Backspace|Delete|Home|End|PageUp|PageDown|Insert|Pause|PrintScreen)$/;
/** Nordic and other ISO keyboards by the browser's language; ANSI otherwise. */
const defaultShape = () => (/^(sv|nb|nn|no|da|fi|de|fr|it|es|pt|nl|is)\b/i.test(navigator.language || '') ? 'ISO' : 'ANSI');
const printable = k => k[1].length === 1 && !k[0].startsWith('Numpad');

class LumaKeymap extends HTMLElement {
  constructor() {
    super();
    this.data = null;
    this.layer = 'PRG';
    this.mods = { Ctrl: false, Alt: false, Shift: false };
    this.shape = defaultShape();
    this.sel = 'KeyE';
    this.find = '';
    this.recording = false;
    this.draft = null;
  }

  connectedCallback() {
    this.dataset.kit = 'keymap';
    this.tabIndex = 0;
    if (this.built) return;
    this.built = true;
    this.addEventListener('keydown', e => this.realKey(e, true));
    this.addEventListener('keyup', e => this.realKey(e, false));
    this.addEventListener('click', e => this.click(e));
    this.addEventListener('change', e => this.change(e));
  }

  /** The feed's data; `layer` = the screen's shortcut mode the first time. */
  show(data, { layer } = {}) {
    if (layer && !this.data) this.layer = layer === 'KBD' || layer === 'NAV' ? layer : layer === 'PRG' ? 'PRG' : this.layer;
    this.data = data;
    this.draft = null;
    this.render();
  }

  run(line) { this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }

  // ── Names ──
  keys() { return KEYBOARDS[this.shape]; }
  keyOf(code) { return this.keys().find(k => k[0] === code); }
  meowMod() { return this.data?.meow || 'Shift'; }
  /** The station's name for physical key `k` under modifiers `m` (`E`, `Ctrl-C`, `Shift-Tab`, `?`). */
  nameOf(k, m = this.mods) {
    const [code, base, shifted] = k;
    if (code.startsWith('Numpad') || code === 'NumLock' || MODS[code] || ['CapsLock', 'MetaLeft', 'MetaRight', 'ContextMenu', 'ScrollLock'].includes(code)) return null;
    let name = NAMED[code] || (SPECIAL.test(code) ? code : null);
    let shift = m.Shift;
    if (!name && printable(k)) {
      if (m.Shift && !/^[a-zå-öA-ZÅ-Ö]$/.test(base) && shifted) { name = shifted; shift = false; } else name = base.toUpperCase();
    }
    if (!name) return null;
    return `${m.Ctrl ? 'Ctrl-' : ''}${m.Alt ? 'Alt-' : ''}${shift ? 'Shift-' : ''}${name}`;
  }
  layerData(l = this.layer) { return this.data?.layers?.find(x => x.layer === l) || { keys: [], own: 'user.' }; }
  entry(name, l = this.layer) { return name ? this.layerData(l).keys.find(e => e.key === name) : null; }
  core(name) { const base = String(name || '').split('-').pop(); return (this.data?.core || []).some(c => c.toLowerCase() === base.toLowerCase()); }
  /** What key `k` does in layer `l` under the modifiers drawn: { name, e, meow, src, text }. */
  look(k, l = this.layer) {
    const name = this.nameOf(k);
    if (!name) return { name: null, src: 'none', text: '' };
    if (this.core(name)) return { name, e: this.entry(name, l) || this.entry(name.split('-').pop(), 'All'), src: 'core', text: this.entry(name, l)?.text || '' };
    let e = this.entry(name, l);
    let meow = false;
    // Meow held: the key's Meow presses (`Shift-…` entries of their own win: NAV's N).
    if (!e && this.mods[this.meowMod()]) {
      const plain = this.nameOf(k, { ...this.mods, [this.meowMod()]: false });
      const p = this.entry(plain, l);
      if (p?.lines?.meow?.length) { e = p; meow = true; }
    }
    if (e) {
      const rows = e.lines ? (meow ? e.lines.meow : e.lines.press) : [];
      // LumaCore's keys say their label (Highlight, Clear); your own their first line.
      const label = e.source === 'lumacore' && !meow ? e.text : '';
      return { name, e, meow, src: e.source, text: label || rows?.[0]?.line || e.text || '' };
    }
    const typing = printable(k) && !this.mods.Ctrl && !this.mods.Alt;
    return { name, src: typing ? 'types' : 'free', text: '' };
  }

  // ── Drawing ──
  render() {
    if (!this.data) { this.innerHTML = '<div class="lc-km-empty">No keys yet</div>'; return; }
    const meow = this.meowMod();
    const modeBtns = LAYERS.map(l => `<button type="button" class="lc-ins-chip" data-layer="${l}" aria-pressed="${l === this.layer}">${l === 'All' ? 'All modes' : `<b>${l}</b>`}</button>`).join('');
    const modBtns = ['Ctrl', 'Alt', 'Shift'].map(m => `<button type="button" class="lc-ins-chip" data-mod="${m}" aria-pressed="${this.mods[m]}">${m === meow ? `Meow (${m})` : m}</button>`).join('');
    this.innerHTML = `<div class="lc-km">
      <div class="lc-km-bar">
        <span class="lc-ins-seg">${modeBtns}</span><span class="lc-ins-seg">${modBtns}</span><span class="lc-km-sp"></span>
        <label class="lc-ins-find">${icon('search')}<input data-find placeholder="Find a key or a line" value="${esc(this.find)}" aria-label="Find a key or a line"></label>
        <button type="button" class="lc-ins-chip" data-rec aria-pressed="${this.recording}">${this.recording ? 'Press a key…' : 'Record'}</button>
        <button type="button" class="lc-ins-chip" data-shape>${this.shape}</button>
      </div>
      <div class="lc-km-board">${this.keys().map(k => this.keyHtml(k)).join('')}</div>
      <div class="lc-km-legend"><span><i class="core"></i>core · locked</span><span><i class="lumacore"></i>LumaCore</span><span><i class="profile"></i>yours</span><span><i class="line"></i>the command line · NAV</span><span><i class="off"></i>turned off</span><span><i class="free"></i>free · types</span><span><i class="over"></i>this mode over All</span></div>
      <div class="lc-km-lower"><section class="lc-km-panel">${this.panel()}</section><section class="lc-km-list"><luma-grid></luma-grid></section></div>
    </div>`;
    this.fillList();
    const f = this.querySelector('[data-find]');
    f.addEventListener('input', () => { this.find = f.value; const p = f.selectionStart; this.render(); const g = this.querySelector('[data-find]'); g.focus(); g.setSelectionRange(p, p); });
  }

  keyHtml(k) {
    const [code, base, shifted, x, y, w, h] = k;
    const style = `grid-column:${Math.round(x * 4) + 1} / span ${Math.round(w * 4)};grid-row:${Math.round(y * 2) + 1} / span ${h * 2}`;
    const mod = MODS[code];
    if (mod) {
      const meow = mod === this.meowMod();
      return `<button type="button" class="lc-km-key mod${this.mods[mod] ? ' on' : ''}" data-mod="${mod}" style="${style}"><span class="cap">${esc(base)}</span><span class="act">${meow ? 'Meow' : ''}</span></button>`;
    }
    const r = this.look(k);
    const cls = ['lc-km-key', r.src];
    if (r.e && r.e.from === 'All' && this.layer !== 'All' && r.src !== 'core' && r.src !== 'line') cls.push('all');
    if (r.e && r.e.from !== 'All' && this.layer !== 'All' && this.entry(r.name, 'All') && r.src !== 'line' && r.src !== 'nav') cls.push('over');
    if (code === this.sel) cls.push('sel');
    if (this.hit(r)) cls.push('hit');
    if (code === 'Enter' && this.shape === 'ISO') cls.push('iso-enter');
    const cap = printable(k) && this.mods.Shift && shifted && !this.mods.Ctrl && !this.mods.Alt ? shifted : base;
    const rows = r.e?.lines ? (r.meow ? r.e.lines.meow : r.e.lines.press) : [];
    const badge = rows?.length > 1 ? `<b class="lc-km-cyc">${rows.length}</b>` : '';
    return `<button type="button" class="${cls.join(' ')}" data-code="${code}" style="${style}" title="${esc(r.name || base)}${r.text ? ` · ${esc(r.text)}` : ''}">`
      + `<span class="cap">${esc(cap)}</span><span class="act">${r.src === 'types' || r.src === 'free' || r.src === 'none' ? '' : esc(r.text)}</span>${badge}</button>`;
  }

  hit(r) {
    const q = this.find.trim().toLowerCase();
    if (!q || !r.name) return false;
    const lines = r.e?.lines ? [...(r.e.lines.press || []), ...(r.e.lines.meow || [])].map(x => x.line) : [];
    return r.name.toLowerCase() === q || [r.text, ...lines].some(t => String(t || '').toLowerCase().includes(q));
  }

  /** The picked key: every layer, its presses, its other gestures, the actions. */
  panel() {
    const k = this.keyOf(this.sel);
    if (!k) return '';
    const r = this.look(k);
    if (!r.name) return `<div class="lc-km-note">${esc(k[1])}: ${k[0].startsWith('Numpad') ? 'the numpad\'s own keys aren\'t names yet (planned)' : 'a modifier'}.</div>`;
    const layers = LAYERS.map(l => {
      const x = this.look(k, l);
      const what = x.e?.lines?.press?.length ? x.e.lines.press.map(p => p.line).join(' › ') : x.text || (x.src === 'types' ? 'types' : '—');
      return `<span class="m${l === this.layer ? ' cur' : ''}">${l}</span><span class="${l === this.layer ? 'cur' : ''}">${esc(what)}${x.e && x.e.from !== l && l !== 'All' ? ' <small>from All</small>' : ''}</span><span class="src${l === this.layer ? ' cur' : ''}">${SOURCE[x.src] || ''}</span>`;
    }).join('');
    const own = r.src === 'profile' && !r.e?.locked;
    const editable = !['core', 'line', 'nav', 'none'].includes(r.src);
    const d = this.draftFor(r);
    const rows = (list, which) => list.map((p, i) => `<div class="lc-km-row" data-row="${i}" data-which="${which}"><b>${i + 1}</b>`
      + `<input class="lc-km-line" data-line value="${esc(p.line)}" ${editable ? '' : 'disabled'} aria-label="Press ${i + 1}">`
      + `<span class="lc-ins-seg"><button type="button" class="lc-ins-chip" data-types="1" aria-pressed="${p.types}" ${editable ? '' : 'disabled'}>Types</button><button type="button" class="lc-ins-chip" data-types="0" aria-pressed="${!p.types}" ${editable ? '' : 'disabled'}>Runs</button></span>`
      + (editable ? `<span class="lc-km-tools"><button type="button" class="lc-ins-chip" data-up title="Earlier">↑</button><button type="button" class="lc-ins-chip" data-down title="Later">↓</button><button type="button" class="lc-ins-chip" data-del title="Remove">${icon('x')}</button></span>` : '')
      + '</div>').join('');
    const cycle = (label, which, list) => `<div class="lc-km-cycle"><div class="lc-km-cap">${label}<small>${list.length > 1 ? 'each press while its word is still last on the line goes one on; anything else starts over' : 'add a press to make a cycle'}</small></div>`
      + rows(list, which) + (editable ? `<button type="button" class="lc-ins-chip lc-km-add" data-add="${which}">+ Add a press</button>` : '') + '</div>';
    const gestures = r.e?.lines ? `<div class="lc-km-ge">${GESTURES.map(([g, label]) => `<span>${label}${g === 'DoublePress' ? '' : ' <em>app · web</em>'}</span><input class="lc-km-line" data-gesture="${g}" value="${esc(r.e.lines[g] || '')}" ${own ? '' : 'disabled'} aria-label="${label}">`).join('')}</div>` : '';
    const acts = [];
    if (own) acts.push(`<button type="button" class="lc-ins-chip" data-act="unbind">Unbind</button>`);
    if (r.src === 'lumacore') acts.push(`<button type="button" class="lc-ins-chip" data-act="unbind">Turn off</button>`);
    if (r.src === 'off') acts.push(`<button type="button" class="lc-ins-chip" data-act="unbind">Back on</button>`);
    const note = r.src === 'core' ? 'A core key: LumaCore\'s in every mode, with any modifier (E-KEY-005).'
      : r.src === 'line' || r.src === 'nav' ? 'Built into the command line and NAV: shown so every key says what it does. Bind it in this mode to take it.'
      : r.src === 'types' && (this.layer === 'KBD' || this.layer === 'All') ? 'Letters, digits and signs type in KBD (E-KEY-006): bind them in PRG or NAV.' : '';
    const editNow = editable && !(r.src === 'types' && (this.layer === 'KBD' || this.layer === 'All'));
    return `<header class="lc-km-head"><b>${esc(r.name)}</b><span>${r.e?.ui_key ? `UiKey ${esc(r.e.ui_key)} · ` : ''}${esc(SOURCE[r.src] || '')}</span></header>
      <div class="lc-km-layers">${layers}</div>
      ${note ? `<div class="lc-km-note">${note}</div>` : ''}
      ${editNow ? cycle('Press', 'press', d.press) + cycle(`Meow (${esc(this.meowMod())}) + press`, 'meow', d.meow) : ''}
      ${editNow ? gestures : ''}
      ${editNow ? `<div class="lc-km-note">Changes save as you make them${own ? '' : r.e ? ' — the first one makes this key yours on this profile' : ' — the first one binds this key'}.</div>` : ''}
      ${acts.length && editNow ? `<div class="lc-km-acts">${acts.join('')}</div>` : r.src === 'off' ? `<div class="lc-km-acts">${acts.join('')}</div>` : ''}`;
  }

  baseRows(r) {
    const l = r.e?.lines;
    const copy = list => (list || []).map(p => ({ line: p.line, types: !!p.types }));
    return { press: l ? copy(l.press) : [{ line: '', types: this.layer !== 'All' }], meow: l ? copy(l.meow) : [] };
  }
  draftFor(r) {
    const key = `${this.layer}|${r.name}`;
    if (!this.draft || this.draft.key !== key) this.draft = { key, ...this.baseRows(r) };
    return this.draft;
  }

  // ── The lines ──
  /** `` `a` `b` `` and the presses that type (`"1 2"` or None). */
  rowsValue(list) {
    const rows = list.filter(p => p.line.trim());
    const types = rows.map((p, i) => (p.types ? i + 1 : 0)).filter(Boolean);
    return { lines: rows.map(p => `\`${p.line.trim().replace(/`/g, '')}\``).join(' '), types: types.length ? `"${types.join(' ')}"` : 'None', count: rows.length };
  }
  /** Save the draft when it differs from what the station holds (empty rows don't count). */
  commit(r) {
    if (!r || !this.draft) return;
    const filled = list => list.filter(p => p.line.trim()).map(p => ({ line: p.line.trim(), types: !!p.types }));
    const base = this.baseRows(r);
    const same = JSON.stringify([filled(this.draft.press), filled(this.draft.meow)]) === JSON.stringify([filled(base.press), filled(base.meow)]);
    if (!same) this.save(r);
  }
  save(r) {
    const L = this.data.lines;
    const d = this.draft;
    const press = this.rowsValue(d.press);
    const meow = this.rowsValue(d.meow);
    if (!press.count) return;
    const mode = this.layer === 'All' ? '' : fill(L.mode, { layer: this.layer });
    const own = r.src === 'profile' && !r.e?.locked;
    const id = own ? r.e.ui_key : `${this.layerData().own}${r.name.toLowerCase()}`;
    const set = (prop, value) => fill(L.set, { id, prop, value });
    const lines = [own ? set('Press', press.lines) : fill(L.bind, { key: r.name, lines: press.lines, mode }), set('Types', press.types)];
    if (meow.count || r.e?.lines?.meow?.length) lines.push(set('MeowPress', meow.count ? meow.lines : '""'), set('MeowTypes', meow.types));
    this.run(lines.join('; '));
  }

  click(e) {
    const b = e.target.closest('button, [data-code]');
    if (!b || !this.contains(b)) return;
    const k = this.keyOf(this.sel);
    const r = k ? this.look(k) : null;
    const d = this.draft;
    const at = b.closest('[data-row]');
    const list = at && d ? d[at.dataset.which] : null;
    const i = at ? +at.dataset.row : -1;
    if (b.dataset.layer) { this.layer = b.dataset.layer; this.draft = null; }
    else if (b.dataset.mod) { this.mods[b.dataset.mod] = !this.mods[b.dataset.mod]; this.draft = null; }
    else if (b.dataset.code) { this.sel = b.dataset.code; this.draft = null; }
    else if ('rec' in b.dataset) { this.recording = !this.recording; this.focus(); }
    else if ('shape' in b.dataset) this.shape = this.shape === 'ISO' ? 'ANSI' : 'ISO';
    else if (b.dataset.types && list) { list[i].types = b.dataset.types === '1'; this.commit(r); }
    else if ('up' in b.dataset && list && i > 0) { [list[i - 1], list[i]] = [list[i], list[i - 1]]; this.commit(r); }
    else if ('down' in b.dataset && list && i < list.length - 1) { [list[i + 1], list[i]] = [list[i], list[i + 1]]; this.commit(r); }
    else if ('del' in b.dataset && list) { list.splice(i, 1); this.commit(r); }
    else if (b.dataset.add && d) d[b.dataset.add].push({ line: '', types: this.layer !== 'All' });
    else if (b.dataset.act === 'unbind' && r) {
      this.run(fill(this.data.lines.unbind, { key: r.name, mode: this.layer === 'All' ? '' : fill(this.data.lines.mode, { layer: this.layer }) }));
      return;
    } else return;
    this.render();
  }

  change(e) {
    const t = e.target;
    const d = this.draft;
    if ('line' in t.dataset && d) {
      // Saved at once; no redraw here (a click that blurred the field still lands) — the
      // station's answer redraws it.
      const at = t.closest('[data-row]');
      d[at.dataset.which][+at.dataset.row].line = t.value;
      this.commit(this.look(this.keyOf(this.sel)));
    } else if (t.dataset.gesture) {
      const r = this.look(this.keyOf(this.sel));
      if (!r.e?.ui_key) return;
      const value = t.value.trim() ? `\`${t.value.trim().replace(/`/g, '')}\`` : '""';
      this.run(fill(this.data.lines.set, { id: r.e.ui_key, prop: t.dataset.gesture, value }));
    }
  }

  /** The real keyboard: held modifiers relabel the board; Record picks the key pressed. */
  realKey(e, down) {
    if (e.target.matches?.('input') && !this.recording) return;
    const mod = MODS[e.code];
    if (mod) { if (this.mods[mod] !== down) { this.mods[mod] = down; this.draft = null; this.render(); } return; }
    if (!down || !this.recording || !this.keyOf(e.code)) return;
    e.preventDefault();
    this.recording = false;
    this.sel = e.code;
    this.find = this.nameOf(this.keyOf(e.code), { Ctrl: e.ctrlKey, Alt: e.altKey, Shift: e.shiftKey }) || '';
    this.draft = null;
    this.render();
  }

  fillList() {
    const grid = this.querySelector('luma-grid');
    if (!grid) return;
    const q = this.find.trim().toLowerCase();
    const rows = this.layerData().keys
      .filter(e => e.source !== 'line' && e.source !== 'nav')
      .map(e => ({
        key: `${e.key}`,
        name: e.key,
        from: e.from,
        ui_key: e.ui_key || '',
        press: (e.lines?.press || []).map(p => p.line).join(' › ') || (e.source === 'off' ? 'off' : ''),
        source: SOURCE[e.source] || e.source,
      }))
      .filter(r => !q || r.name.toLowerCase() === q || r.press.toLowerCase().includes(q) || r.ui_key.toLowerCase().includes(q));
    grid.update({
      key: `keys-${this.layer}`, object: 'KeyBindings', id: 'key', row_key: 'key', bar: false, rows,
      columns: [
        { id: 'name', label: 'Key', width: 110, frozen: true },
        { id: 'from', label: 'Mode', width: 64, priority: 2 },
        { id: 'press', label: 'Press · cycle', width: 220, priority: 1 },
        { id: 'ui_key', label: 'UiKey', width: 130, priority: 2 },
        { id: 'source', label: 'Source', width: 110, priority: 1 },
      ],
      state: { set: 'Full' },
      empty: 'No bindings in this mode',
    });
    grid.addEventListener('select', e => {
      e.stopPropagation();
      const name = e.detail.rows?.[0];
      if (!name) return;
      const base = name.split('-').pop();
      const mods = { Ctrl: name.includes('Ctrl-'), Alt: name.includes('Alt-'), Shift: name.includes('Shift-') };
      const k = this.keys().find(k => this.nameOf(k, mods) === name) || this.keys().find(k => this.nameOf(k, { Ctrl: false, Alt: false, Shift: false }) === base);
      if (k) { this.sel = k[0]; this.mods = mods; this.draft = null; this.render(); }
    }, { once: true });
  }
}

customElements.define('luma-keymap', LumaKeymap);

/** The key editor as a pop-up (the `key_editor` event): fed with `update(data)`. */
export function openKeyEditor({ line, close, layer } = {}) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-km-scrim';
  scrim.dataset.kit = 'key-editor';
  const card = document.createElement('div');
  card.className = 'lc-card lc-km-card';
  card.innerHTML = `<div class="lc-box-title"><span class="lc-km-title">Keys</span><button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div><luma-keymap></luma-keymap>`;
  scrim.appendChild(card);
  document.body.appendChild(scrim);
  const map = card.querySelector('luma-keymap');
  map.addEventListener('run', e => line?.(e.detail.line));
  // Closing saves the row being typed: leaving the field is its change.
  const done = () => {
    if (card.contains(document.activeElement)) document.activeElement.blur();
    scrim.remove();
    close?.();
  };
  card.querySelector('.lc-close').onclick = done;
  scrim.addEventListener('keydown', e => { if (e.key === 'Escape' && !e.target.matches('input')) done(); });
  return {
    el: map,
    update(data) {
      card.querySelector('.lc-km-title').textContent = `Keys · UserProfile ${data.profile ?? ''}${data.name ? ` "${data.name}"` : ''}`;
      map.show(data, { layer });
    },
    close: () => scrim.remove(),
  };
}
