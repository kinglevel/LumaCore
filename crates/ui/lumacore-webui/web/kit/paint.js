// <luma-paint> — the paint picker (decisions 0294, the approved round 4 board
// docs/dev/ui-kit/mockups/2026-10-23-paint/04-paint.html). An object's Paint in three modes:
//
//   Palette  the station's 12 hues over its four tones (the named tone is the hue's word), the
//            greys, and `none · tags, then the pool`
//   Free     any colour: a saturation / value square and a hue strip
//   Theme    a link to one of the viewer's theme paint slots (`Theme "Position"`): it follows each
//            viewer's own theme
//
// then the Recent colours, an Examples switch (the colour on a pool cell, a list row, a tree row,
// a Device Sheet name, a view bar button and a cue row), the Hex field, a preview and the Line
// the answer runs. Please runs ONE line: the field's `line` with `{value}` = a palette word when a
// named swatch was picked, `"#rrggbb"`, `Theme "<slot>"` or `none`. Switching the mode runs the
// feed's `mode_line`, so the picker reopens on the mode used last.
//
// Everything it shows comes from the `paint` feed (the palette is the station's, record 0205;
// colours are screen sRGB, record 0144): `openPaint` subscribes while it is open, on the page's one
// FeedClient (`window.feeds()`, as luma-files does) unless the spec brings `feeds` or `data`.
// Colours from the station are data, set as inline custom properties (`--c`, `--obj`); the hue
// strip's spectrum and the square's hue are computed here the same way (no colour in the source).
//
// A drag on the square or the strip is the pointer core's captured `drag` (a fader's gesture: it
// sets a value, it lifts no object), never a hand-made pointer handler.
//
//   openPaint({ object, line, value, modes, slot, pool, name, no, profile, feeds, data, mode },
//     { run(line) }) → Promise<word | null>   (the line already ran when it resolves a word)
//   el.update(spec) · el.setData(feed data) · events: mode { line } · please { word, line } · cancel
//
// `slot: true` (a theme slot's own colour, the theme editor's Paint tab): Palette and Free only,
// the word is always `"#rrggbb"` (a slot takes a colour, not a word), no `none`, no examples.

import { drag } from './pointer.js';
import { phoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const HEX = /^#[0-9a-f]{6}$/i;
const MODES = ['Palette', 'Free', 'Theme'];
/** A pool type's own colour token, for `none` (decisions §30 "12e"). */
const POOL = { Group: 'groups', Sequence: 'sequences', Macro: 'macros', View: 'views' };

export const hsvHex = (h, s, v) => {
  const f = n => { const k = (n + h / 60) % 6; return v - v * s * Math.max(0, Math.min(k, 4 - k, 1)); };
  return `#${[f(5), f(3), f(1)].map(x => Math.round(x * 255).toString(16).padStart(2, '0')).join('')}`;
};
export const hexHsv = hex => {
  const [r, g, b] = [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255);
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
  let h = 0;
  if (d) h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return { h: (h * 60 + 360) % 360, s: mx ? d / mx : 0, v: mx };
};
/** A Paint value as the line writes it: `#rrggbb` → `Theme "<slot>"` / hex / ''. */
export function parsePaint(value) {
  const v = String(value ?? '').trim();
  const slot = /^Theme\s+"?([^"]+)"?$/i.exec(v);
  if (slot) return { slot: slot[1] };
  if (HEX.test(v)) return { hex: v.toLowerCase() };
  return { none: true, word: v };
}
/** The colour's palette word (the station's named colours), '' when it has none. */
export const namedOf = (palette, hex) => (palette?.named || []).find(n => n.hex.toLowerCase() === String(hex).toLowerCase())?.name || '';

class LumaPaint extends HTMLElement {
  constructor() {
    super();
    this.spec = { modes: MODES };
    this.data = null;
    this.st = { mode: null, hex: '', name: '', slot: '', h: 220, s: 0.6, v: 0.8, examples: false };
    this.built = '';
  }
  connectedCallback() { this.dataset.kit = 'paint'; this.render(); }

  update(spec) {
    this.spec = { modes: MODES, ...spec };
    if (this.spec.slot) this.spec.modes = this.spec.modes.filter(m => m !== 'Theme');
    const p = parsePaint(this.spec.value);
    Object.assign(this.st, { hex: p.hex || '', slot: p.slot || '', name: '' });
    if (p.hex) Object.assign(this.st, hexHsv(p.hex));
    if (this.spec.mode && this.spec.modes.includes(this.spec.mode)) this.st.mode = this.spec.mode;
    this.built = '';
    this.render();
  }

  /** The `paint` feed's data: the palette, the slots, recent, the mode it opens on. */
  setData(data) {
    const first = !this.data;
    this.data = data || null;
    const st = this.st;
    if (first && !st.mode) st.mode = this.spec.modes.includes(data?.mode) ? data.mode : null;
    if (st.hex && !st.slot) st.name = namedOf(data?.palette, st.hex);
    if (st.slot) {
      const s = this.slotOf(st.slot);
      if (s) { st.hex = s.colour; Object.assign(st, hexHsv(s.colour)); }
    }
    this.built = '';
    this.render();
  }

  slotOf(name) { return (this.data?.slots || []).find(s => s.name.toLowerCase() === String(name).toLowerCase()); }
  mode() { return this.st.mode || (this.st.slot && this.spec.modes.includes('Theme') ? 'Theme' : this.spec.modes[0]); }

  /** The `{value}` word of what is picked now. */
  word() {
    const st = this.st;
    if (st.slot) return `Theme "${st.slot.replace(/"/g, '')}"`;
    if (!st.hex) return 'none';
    if (!this.spec.slot && st.name) return st.name;
    return `"${st.hex}"`;
  }
  line() { return String(this.spec.line || '{value}').replace('{value}', this.word()); }

  // ── Drawing ──
  palette() {
    const pal = this.data?.palette;
    if (!pal) return '<p class="lc-pk-note">Waiting for the station\'s palette…</p>';
    const hues = pal.hues || [];
    const tones = hues[0]?.tones?.length || 0;
    const cells = hues.map(h => `<span class="lc-pk-nm">${esc(h.name)}</span>${h.tones.map((t, i) =>
      `<button type="button" style="--c:${esc(t)}" data-hex="${esc(t)}" data-name="${i === pal.named_tone ? esc(h.name) : ''}" title="${esc(i === pal.named_tone ? `${h.name} · ${t}` : t)}" aria-label="${esc(i === pal.named_tone ? h.name : t)}"></button>`).join('')}`).join('');
    const greys = (pal.greys || []).map(g => {
      const n = namedOf(pal, g);
      return `<button type="button" style="--c:${esc(g)}" data-hex="${esc(g)}" data-name="${esc(n)}" title="${esc(n ? `${n} · ${g}` : g)}" aria-label="${esc(n || g)}"></button>`;
    }).join('');
    const none = this.spec.slot ? '' : '<button type="button" class="none" data-hex="" data-name="none">none · tags, then the pool</button>';
    return `<div class="lc-pk-pal" style="--pk-hues:${hues.length};--pk-tones:${tones}">${cells}</div>
      <div class="lc-pk-greys${none ? '' : ' bare'}" style="--pk-greys:${(pal.greys || []).length}">${greys}${none}</div>`;
  }
  free() {
    // The strip's spectrum: computed hues, as data (no colour in the source).
    const spectrum = [0, 60, 120, 180, 240, 300, 360].map(h => hsvHex(h % 360, 1, 1)).join(', ');
    return `<div class="lc-pk-free"><div class="lc-pk-sv" data-sv><i class="lc-pk-knob"></i></div>
      <div class="lc-pk-hue" data-hue style="--pk-spectrum:${spectrum}"><i class="lc-pk-bar"></i></div></div>`;
  }
  theme() {
    const slots = this.data?.slots || [];
    return `<div class="lc-pk-theme">${slots.map(s => `<button type="button" style="--c:${esc(s.colour)}" data-slot="${esc(s.name)}" data-hex="${esc(s.colour)}"><span>${esc(s.name)}</span><i></i></button>`).join('')
      || '<p class="lc-pk-note">No paint slots in your theme.</p>'}</div>
      <p class="lc-pk-note">A theme colour is a link: it follows each viewer's own theme, and the theme editor's Paint tab changes it everywhere.</p>`;
  }
  examples() {
    const label = this.spec.name || '';
    const no = this.spec.no ?? '';
    const obj = this.spec.object || '';
    return `<div class="lc-pk-ex" data-examples ${this.st.examples ? '' : 'hidden'}>
      <div><small>Pool</small><div class="cell"><small>${esc(no)}</small>${esc(label || obj)}</div></div>
      <div><small>List row</small><div class="row"><span>${esc(no)}</span><span>${esc(label || obj)}</span></div></div>
      <div><small>Tree</small><div class="tree"><i></i><span>${esc([no, label].filter(Boolean).join(' · ') || obj)}</span></div></div>
      <div><small>Device Sheet</small><div class="name">${esc(label || obj)}</div></div>
      <div><small>View bar</small><div class="vb">${esc(obj)}</div></div>
      <div><small>Cue sheet</small><div class="row"><span>Cue ${esc(no || 1)}</span><span>${esc(label || obj)}</span></div></div>
    </div>`;
  }

  render() {
    if (!this.isConnected) return;
    const mode = this.mode();
    const key = `${mode}|${!!this.data}`;
    if (key !== this.built) {
      this.built = key;
      const s = this.spec;
      const modes = s.modes.length > 1 ? `<span class="lc-dlg-seg lc-pk-modes" role="group" aria-label="Mode">${s.modes.map(m =>
        `<button type="button" data-mode="${m}" class="${m === mode ? 'on' : ''}" aria-pressed="${m === mode}">${m}</button>`).join('')}</span>` : '';
      const body = mode === 'Free' ? this.free() : mode === 'Theme' ? this.theme() : this.palette();
      this.innerHTML = `<section class="lc-dlg lc-box lc-pk" role="dialog" aria-modal="true" aria-label="Paint · ${esc(s.object)}">
        <header class="lc-dlg-title"><span>Paint · ${esc(s.object)}${s.name ? ` "${esc(s.name)}"` : ''}</span><span class="lc-dlg-meta" data-meta></span>
          <button type="button" class="lc-close" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
        <div class="lc-dlg-body">
          ${modes}
          <div class="lc-pk-mode" data-body="${mode}">${body}</div>
          <div class="lc-pk-recent"><span>Recent</span><span data-recent></span></div>
          ${s.slot ? '' : `<div class="lc-dlg-toggle"><span>Examples</span><button type="button" role="switch" data-ex aria-label="Examples" aria-checked="${this.st.examples}"><i></i></button></div>${this.examples()}`}
          <div class="lc-pk-row">
            <label class="lc-dlg-field"><span data-hexlabel>Hex</span><span class="lc-dlg-input"><input data-hexin autocomplete="off" spellcheck="false" enterkeyhint="done" aria-label="Hex">
              <button type="button" class="lc-dlg-kb" data-kb aria-label="Keyboard"><luma-icon name="keyboard"></luma-icon></button></span></label>
            <div class="lc-pk-prev" data-prev><small>${esc(s.no ?? '')}</small><span>${esc(s.name || s.object || '')}</span></div>
          </div>
          <div class="lc-dlg-rows"><div><span>Line</span><b class="lc-selectable" data-line></b></div></div>
          <div class="lc-dlg-buttons" style="--lc-dlg-cols:2"><button type="button" class="lc-dlg-btn plain" data-cancel>Cancel</button><button type="button" class="lc-dlg-btn primary" data-please data-primary="1">Please</button></div>
        </div>
      </section>`;
      this.wire();
    }
    this.refresh();
  }

  /** What changes as you pick, without rebuilding (a drag keeps its capture). */
  refresh() {
    const st = this.st;
    const q = sel => this.querySelector(sel);
    if (!q('.lc-pk')) return;
    this.querySelector('.lc-pk').classList.toggle('phone', phoneMode() === 'up');
    for (const b of this.querySelectorAll('.lc-pk-pal button, .lc-pk-greys button')) {
      b.classList.toggle('on', !st.slot && b.dataset.hex === st.hex);
    }
    for (const b of this.querySelectorAll('.lc-pk-theme button')) b.classList.toggle('on', !!st.slot && b.dataset.slot.toLowerCase() === st.slot.toLowerCase());
    const sv = q('[data-sv]');
    if (sv) {
      sv.style.setProperty('--pk-hue', hsvHex(st.h, 1, 1));
      const kn = sv.querySelector('.lc-pk-knob');
      kn.style.left = `${st.s * 100}%`; kn.style.top = `${(1 - st.v) * 100}%`;
      kn.style.setProperty('--c', st.hex || 'transparent');
      q('[data-hue] .lc-pk-bar').style.top = `${(st.h / 360) * 100}%`;
    }
    const recent = q('[data-recent]');
    const rec = this.data?.recent || [];
    const recKey = rec.join(',');
    if (recent && recent.dataset.k !== recKey) {
      recent.dataset.k = recKey;
      recent.innerHTML = rec.map(c => `<button type="button" style="--c:${esc(c)}" data-hex="${esc(c)}" data-name="${esc(namedOf(this.data?.palette, c))}" title="${esc(c)}" aria-label="${esc(c)}"></button>`).join('')
        || '<span class="lc-pk-note">none yet</span>';
    }
    // `none`: the pool type's own colour stands in.
    const pool = POOL[this.spec.object?.split(' ')[0]] || this.spec.pool || 'default';
    const obj = st.hex || `var(--lc-pool-${pool})`;
    this.querySelector('.lc-pk').style.setProperty('--obj', obj);
    const hx = q('[data-hexin]');
    if (document.activeElement !== hx) hx.value = st.hex || 'none';
    hx.disabled = !!st.slot;
    q('[data-hexlabel]').textContent = st.slot ? 'Your theme shows it as' : 'Hex';
    q('[data-meta]').textContent = st.slot ? `Theme · ${st.slot}` : st.hex ? (st.name || st.hex) : (this.spec.slot ? '' : 'tags, then the pool');
    q('[data-line]').textContent = this.line();
  }

  pick(hex, name = '', slot = '') {
    Object.assign(this.st, { hex: hex || '', name: name === 'none' ? '' : (name || ''), slot: slot || '' });
    if (hex) Object.assign(this.st, hexHsv(hex));
    this.refresh();
  }

  wire() {
    const box = this.querySelector('.lc-pk');
    box.addEventListener('click', e => {
      const b = e.target.closest('button');
      if (!b) return;
      if (b.dataset.x != null || b.dataset.cancel != null) return this.dispatchEvent(new CustomEvent('cancel'));
      if (b.dataset.please != null) return this.please();
      if (b.dataset.mode) return this.setMode(b.dataset.mode);
      if (b.dataset.ex != null) {
        this.st.examples = !this.st.examples;
        b.setAttribute('aria-checked', String(this.st.examples));
        this.querySelector('[data-examples]').hidden = !this.st.examples;
        return;
      }
      if (b.dataset.kb != null) return this.dispatchEvent(new CustomEvent('keyboard', { detail: { field: this.querySelector('[data-hexin]') } }));
      if (b.dataset.slot) return this.pick(b.dataset.hex, '', b.dataset.slot);
      if (b.dataset.hex !== undefined) this.pick(b.dataset.hex, b.dataset.name || namedOf(this.data?.palette, b.dataset.hex));
    });
    const hx = this.querySelector('[data-hexin]');
    hx.addEventListener('input', () => {
      const v = hx.value.trim();
      if (/^#?[0-9a-f]{6}$/i.test(v)) {
        const hex = (v.startsWith('#') ? v : `#${v}`).toLowerCase();
        this.pick(hex, namedOf(this.data?.palette, hex));
      } else if (/^none$/i.test(v) && !this.spec.slot) this.pick('');
    });
    hx.addEventListener('blur', () => this.refresh());
    const sv = this.querySelector('[data-sv]');
    if (sv) {
      const at = ({ x, y }) => {
        const r = sv.getBoundingClientRect();
        this.st.s = Math.min(1, Math.max(0, (x - r.left) / r.width));
        this.st.v = Math.min(1, Math.max(0, 1 - (y - r.top) / r.height));
        this.fromHsv();
      };
      drag(sv, { onStart: at, onMove: at });
      const hue = this.querySelector('[data-hue]');
      const atHue = ({ y }) => {
        const r = hue.getBoundingClientRect();
        this.st.h = Math.min(359.9, Math.max(0, ((y - r.top) / r.height) * 360));
        this.fromHsv();
      };
      drag(hue, { onStart: atHue, onMove: atHue });
    }
  }
  fromHsv() {
    const hex = hsvHex(this.st.h, this.st.s, this.st.v);
    Object.assign(this.st, { hex, slot: '', name: namedOf(this.data?.palette, hex) });
    this.refresh();
  }

  setMode(m) {
    if (m === this.mode()) return;
    this.st.mode = m;
    // The mode is the viewer's (PaintMode): the next picker opens on it.
    const ml = this.data?.mode_line;
    if (ml) this.dispatchEvent(new CustomEvent('mode', { detail: { line: ml.replace('{value}', m) } }));
    this.render();
  }

  please() {
    if (this.spec.slot && !this.st.hex) return;
    this.dispatchEvent(new CustomEvent('please', { detail: { word: this.word(), line: this.line() } }));
  }
}
customElements.define('luma-paint', LumaPaint);

let current = null;

/** The picker over everything; resolves the word Please ran (null: cancelled). */
export function openPaint(spec, { run, closed } = {}) {
  current?.cancel();
  return new Promise(resolve => {
    const scrim = document.createElement('div');
    scrim.className = 'lc-dlg-scrim lc-pk-scrim';
    scrim.dataset.kit = 'paint-picker';
    if (phoneMode() === 'up') scrim.classList.add('sheet');
    const el = document.createElement('luma-paint');
    scrim.append(el);
    document.body.append(scrim);
    el.update(spec);
    let keyboard = null;
    let sub = null;
    if (spec.data) el.setData(spec.data);
    else {
      const client = spec.feeds || window.feeds?.();
      sub = client?.sub('paint', spec.profile ? { profile: spec.profile } : {}, { batch: m => { if (m.data) el.setData(m.data); } }) || null;
    }
    const done = word => {
      if (!scrim.isConnected) return;
      window.removeEventListener('keydown', onKey, true);
      sub?.close();
      keyboard?.remove();
      scrim.remove();
      if (current?.scrim === scrim) current = null;
      closed?.();
      resolve(word);
    };
    el.addEventListener('cancel', () => done(null));
    el.addEventListener('mode', e => run?.(e.detail.line));
    el.addEventListener('please', e => { run?.(e.detail.line); done(e.detail.word); });
    // The built-in keyboard under the card (the screen's Keyboard setting), as the dialog does.
    el.addEventListener('keyboard', e => {
      const field = e.detail.field;
      if (keyboard) { keyboard.remove(); keyboard = null; scrim.style.paddingBottom = ''; field.focus(); return; }
      keyboard = document.createElement('luma-keyboard');
      keyboard.setAttribute('mode', 'text');
      keyboard.className = 'lc-dlg-keyboard';
      scrim.append(keyboard);
      keyboard.target = field;
      const room = () => { scrim.style.paddingBottom = keyboard?.isConnected ? `${keyboard.offsetHeight + 8}px` : ''; };
      new ResizeObserver(room).observe(keyboard);
      keyboard.addEventListener('please', () => el.please());
      keyboard.addEventListener('escape', () => done(null));
      keyboard.addEventListener('hide', () => { keyboard?.remove(); keyboard = null; room(); });
    });
    scrim.addEventListener('pointerdown', e => { if (e.target === scrim) done(null); });
    // Before every other overlay's keys (window capture): Esc cancels, Enter is Please.
    const onKey = e => {
      if (e.target?.closest?.('luma-keyboard')) return;
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); done(null); }
      else if (e.key === 'Enter' && !e.isComposing && !e.target?.closest?.('.lc-dlg-btn, .lc-pk-mode button, .lc-pk-recent button')) { e.preventDefault(); e.stopPropagation(); el.please(); }
    };
    window.addEventListener('keydown', onKey, true);
    current = { scrim, el, cancel: () => done(null) };
  });
}

export const paintOpen = () => !!current;
