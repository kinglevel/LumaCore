// <luma-keypad> — the keypad / calculator for every number (decisions §30
// "12b", the Keyboards board and the 12b Inputs board). The locked 4×5 grid:
//   7 8 9 ⌫ / 4 5 6 Clear / 1 2 3 + / 0 . − * / Min Max / Please
// The old value is shown ("was 15") and preselected: the first digit replaces
// it, an operator works on it (`15 + 3`). What was typed goes to the station
// as it is — the station works relative input out (never 0 for nonsense).
// A unit swipe shows when the number has units (seconds ↔ fps, % ↔ DMX).
// A hardware keyboard types into it too (digits, + − * / . Backspace, Enter,
// Esc).
//
//   openKeypad({ title, value, min, max, unit, units }) → Promise<{ text,
//     unit } | null>

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const LAYOUT = ['7', '8', '9', 'back', '4', '5', '6', 'Clear', '1', '2', '3', '+', '0', '.', '-', '*', 'Min', 'Max', '/', 'Please'];

class LumaKeypad extends HTMLElement {
  constructor() {
    super();
    this.spec = {};
    this.entry = '';
    this.fresh = true;
    this.unit = 0;
  }
  connectedCallback() { this.dataset.kit = 'keypad'; this.render(); }

  /** Start on `spec` ({ title, value, min, max, unit, units }). */
  update(spec) {
    this.spec = { ...spec };
    this.entry = String(spec.value ?? '');
    this.fresh = true;
    this.unit = 0;
    this.render();
  }

  get text() { return this.fresh ? String(this.spec.value ?? '') : this.entry; }

  press(k) {
    if (k === 'Please') return this.emit('please', { text: this.text.trim(), unit: this.spec.units?.[this.unit] ?? this.spec.unit ?? null });
    if (k === 'Esc') return this.emit('escape');
    if (k === 'Clear') { this.entry = ''; this.fresh = false; }
    else if (k === 'back') { this.entry = (this.fresh ? '' : this.entry).replace(/\s*\S\s*$/, ''); this.fresh = false; }
    else if (k === 'Min' || k === 'Max') {
      const v = k === 'Min' ? this.spec.min : this.spec.max;
      if (v == null) return;
      this.entry = String(v); this.fresh = false;
    } else {
      const op = '+-*/'.includes(k);
      const base = this.fresh ? (op ? String(this.spec.value ?? '') : '') : this.entry;
      this.entry = base + (op ? ` ${k} ` : k);
      this.fresh = false;
    }
    this.render();
  }

  emit(name, detail = {}) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    const bounds = s.min != null && s.max != null ? `${s.min}–${s.max}` : s.min != null ? `≥ ${s.min}` : s.max != null ? `≤ ${s.max}` : '';
    const range = [bounds, s.unit].filter(Boolean).join(' ');
    const label = k => (k === 'back' ? '<luma-icon name="delete"></luma-icon>' : k === '-' ? '−' : esc(k));
    const cls = k => (k === 'Please' ? 'please' : k === 'Clear' ? 'clear' : /^[0-9.]$/.test(k) ? 'digit' : 'op');
    const disabled = k => (k === 'Min' && s.min == null) || (k === 'Max' && s.max == null);
    this.innerHTML = `
      <div class="lc-kp">
        <header class="lc-kp-title"><span>${esc(s.title || 'Value')}</span><span class="lc-kp-range">${esc(range)}</span>
          <button type="button" class="lc-kp-x" data-k="Esc" aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
        <div class="lc-kp-display"><span>${s.value !== undefined && s.value !== '' ? `was ${esc(s.value)}` : ''}</span><b class="${this.fresh ? 'fresh' : ''}">${esc(this.text) || '&nbsp;'}</b></div>
        ${s.units?.length > 1 ? `<div class="lc-kp-units">${s.units.map((u, i) => `<button type="button" data-unit="${i}" class="${i === this.unit ? 'on' : ''}">${esc(u)}</button>`).join('')}</div>` : ''}
        <div class="lc-kp-grid">${LAYOUT.map(k => `<button type="button" class="lc-kp-key ${cls(k)}" data-k="${esc(k)}" ${disabled(k) ? 'disabled' : ''}>${label(k)}</button>`).join('')}</div>
      </div>`;
    for (const b of this.querySelectorAll('[data-k]')) {
      b.addEventListener('pointerdown', e => e.preventDefault());
      b.addEventListener('click', () => this.press(b.dataset.k));
    }
    for (const b of this.querySelectorAll('[data-unit]')) b.addEventListener('click', () => { this.unit = +b.dataset.unit; this.render(); });
  }
}
customElements.define('luma-keypad', LumaKeypad);

/** The keypad as a pop-up over everything; resolves the typed text. */
export function openKeypad(spec) {
  return new Promise(resolve => {
    const scrim = document.createElement('div');
    scrim.className = 'lc-kp-scrim';
    const pad = document.createElement('luma-keypad');
    scrim.append(pad);
    document.body.append(scrim);
    pad.update(spec);
    const done = value => {
      document.removeEventListener('keydown', onKey, true);
      scrim.remove();
      resolve(value);
    };
    pad.addEventListener('please', e => done(e.detail));
    pad.addEventListener('escape', () => done(null));
    scrim.addEventListener('pointerdown', e => { if (e.target === scrim) done(null); });
    const onKey = e => {
      const map = { Backspace: 'back', Delete: 'Clear', Enter: 'Please', Escape: 'Esc' };
      const k = map[e.key] ?? (/^[0-9.+\-*/]$/.test(e.key) ? e.key : null);
      if (!k) return;
      e.preventDefault(); e.stopPropagation();
      pad.press(k);
    };
    document.addEventListener('keydown', onKey, true);
  });
}
