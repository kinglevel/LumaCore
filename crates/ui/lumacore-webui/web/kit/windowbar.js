// <luma-window-bar> — a window's bar (decisions §30 "12c", the WindowBar
// board): the row under the tabs with the window's pinned settings and its
// type's actions, drawn from the window type (`/api/window-types`: actions,
// settings, `bar_min`) and the window's live state (the `windows` feed:
// `items`, `states`, `values`, `chips`, `picks`).
//
//   bar.update({ type, view })   // type = a window type, view = its window on the feed
//   bar.addEventListener('run', e => send(e.detail.line))   // always with the window as owner
//   bar.hiddenBySize             // the window is below the type's bar_min
//   bar.menuItems()              // the items as kit menu rows (window menu)
//
// Items, left to right as `WindowBarItems` lists them:
//   an on/off setting  → a small square box + its label; tap = `Set Self "<Key>" Toggle`
//   an action          → a small button (`Clear`), a box when it has a state
//                        (`Pause` lit = !AutoScroll); tap = the action's line
//   any other setting  → `Label: value ▾`; tap opens the kit's pick list /
//                        keypad / dialog; the answer = `Set Self "<Key>" <value>`
//                        (numbers may be relative: `+ 1`)
//   a setting followed by actions that step it (`Set Self "Universe" - 1`,
//   `+ 1`) is drawn as one stepper: `Universe: ‹ 1 ›`
//   chips              → `× Search “act 2”` only while the station shows them
// What doesn't fit goes into a `…` menu at the right; below the type's
// minimum size the bar hides and the window menu (tab dot) lists the items
// instead. Every gesture is a command line (`run` event) — never a layout
// write. Every drawn item has `data-id="<window type>/<setting or action id>"`.

import { askField, valueWord, keyWord } from './settings.js';
import { openMenu } from './menu.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const norm = k => String(k ?? '').replace(/_/g, '').toLowerCase();
const STEP = /^Set Self "?(\w+)"? ([-+]) /i;

/** The settings-window field for a setting def (the kit's askField takes it). */
function fieldOf(def, view) {
  const value = view.values?.[def.key] ?? '';
  const base = { name: def.label, key: def.key, hint: def.hint, value };
  switch (def.type) {
    case 'enum': return { ...base, input: { type: 'choice', options: (def.options || []).map(o => o[1]) } };
    case 'int': return { ...base, input: { type: 'number', min: def.min, max: def.max, named: (def.named || []).map(n => n[1]) } };
    case 'link': {
      const pick = view.picks?.[def.key];
      return pick ? { ...base, input: { type: 'link' }, pick } : { ...base, input: { type: 'number', min: 1 } };
    }
    case 'colour': return { ...base, input: { type: 'colour' } };
    default: return { ...base, input: { type: 'text', max_len: def.max_len } };
  }
}

/** The bar's entries from the type and the window's state. */
export function barEntries(type, view) {
  if (!type || !view) return [];
  const setting = id => (type.settings || []).find(s => norm(s.key) === norm(id));
  const action = id => (type.actions || []).find(a => norm(a.id) === norm(id));
  const raw = [];
  for (const id of view.items || []) {
    const a = action(id);
    if (a) { raw.push({ id: a.id, kind: 'action', label: a.label, icon: a.icon, line: a.line, stateful: !!a.state, on: !!view.states?.[a.id] }); continue; }
    const d = setting(id);
    if (!d || d.type === 'list') continue;
    if (d.type === 'bool') raw.push({ id: d.key, kind: 'bool', label: d.label, on: !!view.states?.[d.key], line: `Set Self "${keyWord(d.key)}" Toggle` });
    else raw.push({ id: d.key, kind: 'value', label: d.label, value: view.values?.[d.key] ?? '', def: d });
  }
  // A setting with the actions that step it right after: one stepper.
  const out = [];
  for (let i = 0; i < raw.length; i++) {
    const e = raw[i];
    if (e.kind === 'value') {
      const steps = [];
      while (raw[i + 1]?.kind === 'action' && !raw[i + 1].stateful) {
        const m = STEP.exec(raw[i + 1].line || '');
        if (!m || norm(m[1]) !== norm(e.id)) break;
        steps.push({ ...raw[i + 1], dir: m[2] });
        i++;
      }
      if (steps.length) { out.push({ ...e, kind: 'stepper', minus: steps.filter(s => s.dir === '-'), plus: steps.filter(s => s.dir === '+') }); continue; }
    }
    out.push(e);
  }
  for (const c of view.chips || []) out.push({ id: c.id, kind: 'chip', label: c.label, line: c.line });
  return out;
}

class LumaWindowBar extends HTMLElement {
  constructor() {
    super();
    this.type = null;
    this.view = null;
    this.entries = [];
    this.hiddenBySize = false;
    this.key = '';
  }

  connectedCallback() {
    this.dataset.kit = 'window-bar';
    this.setAttribute('role', 'toolbar');
    if (!this.ro) {
      this.ro = new ResizeObserver(() => this.layout());
      this.ro.observe(this);
    }
    if (this.parentElement && this.parentElement !== this.watched) {
      this.parentRo?.disconnect();
      this.watched = this.parentElement;
      this.parentRo = new ResizeObserver(() => this.sizeCheck());
      this.parentRo.observe(this.watched);
    }
    this.render();
  }

  /** New type / window state: redraw (only when something changed). */
  update({ type, view }) {
    this.type = type;
    this.view = view;
    const key = JSON.stringify([type?.name, type?.actions, view?.bar, view?.items, view?.states, view?.values, view?.chips, view?.picks]);
    if (key === this.key) return;
    this.key = key;
    this.render();
  }

  emit(line) { if (line) this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }
  dataId(id) { return `${this.type?.name || 'window'}/${id}`; }

  /** The window (this bar's parent) is below the type's minimum size. */
  sizeCheck() {
    const min = this.type?.bar_min;
    const host = this.watched;
    if (!min || !host || !host.isConnected) return;
    const small = host.clientWidth > 0 && (host.clientWidth < min.width_px || host.clientHeight < min.height_px);
    if (small === this.hiddenBySize) return;
    this.hiddenBySize = small;
    this.render();
    this.dispatchEvent(new CustomEvent('barhidden', { detail: { hidden: small }, bubbles: true }));
  }

  /** What a tap on an entry does. */
  async activate(e) {
    if (e.kind === 'value' || e.kind === 'stepper') {
      const f = fieldOf(e.def, this.view);
      const got = await askField(f);
      if (got == null || typeof got === 'object') return;
      return this.emit(`Set Self "${keyWord(e.def.key)}" ${valueWord(f, got)}`);
    }
    this.emit(e.line);
  }

  itemHtml(e) {
    const id = esc(this.dataId(e.id));
    const box = on => `<i class="lc-wb-box${on ? ' on' : ''}">${on ? '<luma-icon name="check"></luma-icon>' : ''}</i>`;
    const value = `<span class="lc-wb-val">${esc(e.value) || '—'}<luma-icon name="chevron-down"></luma-icon></span>`;
    const step = s => `<button type="button" class="lc-wb-step" data-id="${esc(this.dataId(s.id))}" data-line="${esc(s.line)}" aria-label="${esc(s.label)}" title="${esc(s.label)}"><luma-icon name="${esc(s.icon || (s.dir === '-' ? 'chevron-left' : 'chevron-right'))}"></luma-icon></button>`;
    switch (e.kind) {
      case 'bool':
        return `<button type="button" class="lc-wb-item lc-wb-check${e.on ? ' on' : ''}" data-id="${id}" aria-pressed="${e.on}">${box(e.on)}<span>${esc(e.label)}</span></button>`;
      case 'action':
        return e.stateful
          ? `<button type="button" class="lc-wb-item lc-wb-check${e.on ? ' on' : ''}" data-id="${id}" aria-pressed="${e.on}" title="${esc(e.line)}">${box(e.on)}<span>${esc(e.label)}</span></button>`
          : `<button type="button" class="lc-wb-item lc-wb-act" data-id="${id}" title="${esc(e.line)}">${esc(e.label)}</button>`;
      case 'chip':
        return `<button type="button" class="lc-wb-item lc-wb-chip" data-id="${id}" title="${esc(e.line)}">${esc(e.label)}</button>`;
      case 'stepper':
        return `<span class="lc-wb-item lc-wb-stepper" data-group="${id}"><span class="lc-wb-lbl">${esc(e.label)}:</span>${e.minus.map(step).join('')}<button type="button" class="lc-wb-pick" data-id="${id}">${esc(e.value) || '—'}</button>${e.plus.map(step).join('')}</span>`;
      default:
        return `<button type="button" class="lc-wb-item lc-wb-pick" data-id="${id}"><span class="lc-wb-lbl">${esc(e.label)}:</span>${value}</button>`;
    }
  }

  render() {
    if (!this.isConnected) return;
    const view = this.view;
    this.entries = barEntries(this.type, view);
    const show = !!view?.bar && this.entries.length > 0 && !this.hiddenBySize;
    this.hidden = !show;
    if (!show) { this.innerHTML = ''; return; }
    this.innerHTML = `<div class="lc-wb-row">${this.entries.map((e, i) => this.itemHtml(e).replace('class="lc-wb-item', `data-n="${i}" class="lc-wb-item`)).join('')}</div>
      <button type="button" class="lc-wb-more" data-id="${esc(this.dataId('more'))}" aria-label="More bar items" title="Didn't fit in the bar" hidden><luma-icon name="ellipsis"></luma-icon></button>`;
    for (const el of this.querySelectorAll('.lc-wb-row > [data-n]')) {
      const e = this.entries[+el.dataset.n];
      if (e.kind === 'stepper') {
        el.querySelector('.lc-wb-pick').addEventListener('click', () => this.activate(e));
        for (const s of el.querySelectorAll('.lc-wb-step')) s.addEventListener('click', () => this.emit(s.dataset.line));
      } else {
        el.addEventListener('click', () => this.activate(e));
      }
    }
    const more = this.querySelector('.lc-wb-more');
    more.addEventListener('click', ev => {
      ev.stopPropagation();
      const r = more.getBoundingClientRect();
      more.classList.add('open');
      // Lit while its menu is open: the next press anywhere closes the menu.
      setTimeout(() => document.addEventListener('pointerdown', () => more.classList.remove('open'), { once: true, capture: true }));
      openMenu({ x: r.right - 250, y: r.bottom + 2 }, [{ header: 'Didn’t fit in the bar' }, ...this.menuItems(this.overflow)]);
    });
    this.layout();
  }

  /** Hide what doesn't fit (from the right); the `…` menu lists it. */
  layout() {
    const row = this.querySelector('.lc-wb-row');
    const more = this.querySelector('.lc-wb-more');
    if (!row || !more || this.hidden) return;
    const items = [...row.children];
    items.forEach(el => { el.hidden = false; });
    more.hidden = true;
    this.overflow = [];
    if (row.scrollWidth <= row.clientWidth + 1) return;
    more.hidden = false;
    for (let i = items.length - 1; i >= 0 && row.scrollWidth > row.clientWidth + 1; i--) {
      items[i].hidden = true;
      this.overflow.unshift(this.entries[+items[i].dataset.n]);
    }
  }

  /** Entries as kit menu rows: value on the right, a box for on/off. */
  menuItems(entries = this.entries) {
    const row = e => ({
      id: this.dataId(e.id),
      label: e.label,
      value: e.kind === 'value' || e.kind === 'stepper' ? (e.value || '—') : undefined,
      check: e.kind === 'bool' || (e.kind === 'action' && e.stateful) ? e.on : undefined,
      keep: false,
      fn: () => this.activate(e),
    });
    // A stepper is its value plus its step actions.
    return (entries || []).flatMap(e => (e.kind === 'stepper' ? [row(e), ...e.minus.map(row), ...e.plus.map(row)] : [row(e)]));
  }
}
customElements.define('luma-window-bar', LumaWindowBar);
