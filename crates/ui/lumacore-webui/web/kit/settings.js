// <luma-settings> — the one settings window (decisions §30 "12b2", the
// WindowSettings board; 12c: the BarSettings board): every window's settings
// and every `EditSetting` object. A title bar (icon, title, a subtitle such
// as `from preset 2 "Big DMX" · changed`, the header buttons, Close), tabs on
// the left from the fields' sections (none when there is one), a grid of
// cells — each cell takes its field's input: a toggle flips, a choice opens
// the pick list, a number the keypad (named numbers like Fit as choices), a
// link the pick list (with the follow row), a colour or a text the standard
// dialog, an object's Paint (input `paint`, decisions 0294) the paint picker
// (kit/paint.js: its swatch and word in the cell, empty = `none`). Every change runs the field's command line at once (no OK): the
// station answers with fresh fields and `update` redraws in place, keeping
// the tab.
//
// A window's settings (12c) also carry its bar: a field of input `list`
// (`window_bar_items`) turns on a pin per cell (pinned = in the window bar,
// accent) and draws the "Window bar" tab — on/off, the items in order (drag
// the handle or ↑ ↓, × removes), the type's actions to add, Load defaults.
// A changed cell shows ↺ (the field's `reset` line). Pins and the list run
// `Set Self "WindowBarItems" "a,b,c"`: the page sends them with the window as
// the owner (`Self`).
//
// A field of input `tags` (decisions §12 "Tags hardened": every show object's Tags tab) takes its
// whole tab: the tag tree as a `<luma-tree>` picker — a tick runs `assign`, an untick `unassign`,
// "New tag here" `store` / `store_top` with the name asked; the tree keeps what is folded across
// the station's fresh fields.
//
//   openSettings({ title, heading, subtitle, icon, fields: [{ name, key, section,
//     input: { type, min, max, named, options, max_len, items }, hint, value,
//     line, pick, changed, reset }], actions: [{ label, line, variant:
//     'warn'|'danger'|'' }], delete_line }, { run(line) }) → { update(spec),
//     close(), title }
// Every cell shows its kind (12d, the SetupSettings board) with one neutral
// outline badge: Aa text (an underlined field), 123 number (right-aligned
// value + unit, − / + steppers, the keypad on a tap), I/O switch, ≡ list (a
// box with a chevron, the pick list); a read-only cell greys the badge. A
// field's `home` marks where it lives with the kit's icon (decisions §39: show file `save`,
// user `user`, station `server`; grey, gold where it differs from `spec.home`, the tab's; the
// words in the tooltip); `rights` greys it below the viewer's
// level (`spec.level`) with the reason, `readonly` with its own.
// `<luma-settings embedded>` draws without the title bar, inside another
// window (Users & Profiles, a setup menu's settings tab); `spec.note` is a
// line above the cells. `readonly: true | "why"` draws the cells without
// changing anything (UserProfile 1 "Default", 12c2). Below ~600 px wide (a
// container query) the header buttons go into ⋯; title and × always show.

import { openKeypad } from './keypad.js';
import { openPick } from './popup.js';
import { openDialog } from './dialog.js';
import { drag } from './pointer.js';
import { openMenu } from './menu.js';
import { openPaint } from './paint.js';

import { toast } from './toast.js';
import './tree.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** The rights ladder, lowest first (lumacore_tree::UserRights). */
const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];
const below = (need, have) => (need && have && LEVELS.indexOf(have) < LEVELS.indexOf(need) ? `Needs ${need} — you are ${have}` : '');

/** A field's kind for its badge: text | number | toggle | list | colour. */
export function kindOf(input) {
  const t = input?.type ?? input;
  if (t === 'number' || t === 'int') return 'number';
  if (t === 'toggle' || t === 'bool') return 'toggle';
  if (t === 'choice' || t === 'link' || t === 'list') return 'list';
  if (t === 'colour') return 'colour';
  if (t === 'paint') return 'paint';
  return 'text';
}
const BADGE = { text: 'Aa', number: '123', toggle: 'I/O', list: '≡', colour: '#', paint: '#' };
const BADGE_TITLE = { text: 'Text', number: 'Number', toggle: 'Switch (on / off)', list: 'A list to pick from', colour: 'Colour', paint: 'Paint: a colour, a palette word or a theme link (the paint picker)' };
/** The kind badge (neutral outline; greyed when read-only). */
export function kindBadge(kind, ro = false) {
  return `<b class="lc-kind${ro ? ' ro' : ''}" data-kind="${kind}" title="${BADGE_TITLE[kind] || ''}">${esc(BADGE[kind] || 'Aa')}</b>`;
}
/** Where a setting lives (decisions §39): the kit's icon and the words of the tooltip. */
export const HOME = {
  show: ['save', 'Show file · every station'],
  user: ['user', 'Your profile · in the show'],
  station: ['server', 'This station · never in the show'],
};
/** The home icon of a cell: grey, gold where it differs from the tab's (`tabHome`). */
export function homeMark(home, tabHome = '') {
  const h = HOME[home];
  if (!h) return '';
  const odd = tabHome && HOME[tabHome] && tabHome !== home;
  return `<luma-icon name="${h[0]}" class="lc-home${odd ? ' odd' : ''}" data-home="${home}" title="${h[1]}${odd ? ` — this tab: ${HOME[tabHome][1]}` : ''}"></luma-icon>`;
}
/** The tab's or menu's home line: the icon and the words once. */
export function homeLine(home) {
  const h = HOME[home];
  return h ? `<luma-icon name="${h[0]}"></luma-icon>${h[1]}` : '';
}
/** A value as a command-line word: plain tokens as they are, the rest quoted. */
export const quote = v => (/^[A-Za-z0-9#.+\-*/]+$/.test(v) ? v : `"${String(v).replace(/[";]/g, '')}"`);
/** `window_bar_items` → `WindowBarItems` (the station matches keys either way). */
export const keyWord = k => String(k).split('_').map(p => p.charAt(0).toUpperCase() + p.slice(1)).join('');
const BAR_LIST = 'window_bar_items';
const BAR_ON = 'window_bar';

/** What a field's tap asks for (the kit's keypad, pick list or dialog);
 *  resolves the `{value}` text, `{ add, value }` for Add New, or null. */
export async function askField(f) {
  const t = f.input?.type;
  if (t === 'toggle') return /^(yes|on|true)$/i.test(f.value) ? 'Off' : 'On';
  if (t === 'choice') {
    const items = f.input.options.map((o, i) => ({ no: i + 1, name: o }));
    const got = await openPick({ title: f.name, value: f.value, pick: { kind: 'Choice', items, plain: true, note: f.hint || ' ' } });
    return got?.no ? f.input.options[got.no - 1] : null;
  }
  if (t === 'number') {
    const named = f.input.named || [];
    if (named.length) {
      const items = named.map((o, i) => ({ no: i + 1, name: o }));
      const got = await openPick({ title: f.name, value: f.value,
        pick: { kind: 'Number', items, plain: true, add: 'number', add_label: `Number ${f.input.min}–${f.input.max}…`, note: f.hint || ' ' } });
      if (!got) return null;
      if (got.no) return named[got.no - 1];
    }
    const typed = await openKeypad({ title: f.name, value: named.includes(f.value) ? '' : f.value, min: f.input.min, max: f.input.max });
    return typed?.text || null;
  }
  if (t === 'paint') return null; // the paint picker runs its own line (LumaSettings.paint)
  if (t === 'link' && f.pick) {
    const got = await openPick({ title: f.name, pick: f.pick, value: f.value });
    if (!got) return null;
    if (got.follow) return 'selected';
    if (got.none) return 'none';
    if (got.add) return { add: f.pick.add, value: String(f.pick.add_no) };
    return String(got.no);
  }
  const colour = t === 'colour';
  const got = await openDialog({
    title: f.name, text: f.hint || '',
    inputs: [{ name: 'v', label: colour ? 'Colour #rrggbb' : f.name, value: colour && f.value === 'Theme' ? '' : f.value, kind: 'text' }],
    buttons: [{ label: 'Please', value: 'set', kind: 'please' }, ...(colour ? [{ label: 'Theme', value: 'theme' }] : []), { label: 'Cancel', value: null }],
    primary: 0, cancel: colour ? 2 : 1,
  });
  if (!got || !got.value) return null;
  if (got.value === 'theme') return 'Theme';
  return got.inputs?.v ?? '';
}

/** The `{value}` text of an answer: a text is always quoted. */
export function valueWord(f, got) {
  return f.input?.type === 'text' ? `"${String(got).replace(/[";]/g, '')}"` : quote(got);
}

const ids = v => String(v ?? '').split(',').map(s => s.trim()).filter(Boolean);
const barLine = list => `Set Self "WindowBarItems" "${list.join(',')}"`;

class LumaSettings extends HTMLElement {
  constructor() { super(); this.spec = { fields: [] }; this.tab = 0; this.last = ''; }
  connectedCallback() { this.dataset.kit = 'settings'; this.render(); }
  update(spec) { this.spec = spec; this.render(); }

  sections() {
    const out = [];
    for (const f of this.spec.fields || []) if (!out.includes(f.section || '')) out.push(f.section || '');
    return out;
  }

  /** The tag tree picker of a Tags field (one element, kept across renders). */
  tagTree(f) {
    if (!this.tree) {
      this.tree = document.createElement('luma-tree');
      this.tree.addEventListener('toggle', ev => {
        const t = this.treeField;
        if (!t || this.why(t)) { toast(this.why(t) || 'Locked'); return; }
        const tpl = ev.detail.on ? t.input.assign : t.input.unassign;
        this.run(tpl.replace('{value}', ev.detail.path));
      });
      this.tree.addEventListener('select', ev => {
        this.treeSel = this.treeSel === ev.detail.path ? null : ev.detail.path;
        this.tree.update({ ...this.tree.spec, selected: this.treeSel });
      });
      this.tree.addEventListener('new', async ev => {
        const t = this.treeField;
        if (!t || this.why(t)) return;
        const got = await openDialog({
          title: ev.detail.parent ? `New tag under ${ev.detail.parent}` : 'New tag',
          inputs: [{ name: 'v', label: 'Name', value: '', kind: 'text' }],
          buttons: [{ label: 'Please', value: 'set', kind: 'please' }, { label: 'Cancel', value: null }],
          primary: 0, cancel: 1,
        });
        const name = got?.value && got.inputs?.v?.trim();
        if (!name) return;
        const quoted = `"${name.replace(/[";]/g, '')}"`;
        this.run(ev.detail.parent ? t.input.store.replace('{value}', ev.detail.parent).replace('{name}', quoted) : t.input.store_top.replace('{name}', quoted));
      });
    }
    this.treeField = f;
    this.tree.update({ nodes: f.input.nodes || [], mode: 'pick', selected: this.treeSel || null, search: true, newHere: !this.why(f),
      empty: 'No tags yet. New tag here makes the first.' });
    return this.tree;
  }

  /** The window bar's list field, when these are a window's settings. */
  barField() { return (this.spec.fields || []).find(f => f.input?.type === 'list'); }

  /** Why a field can't change here ('' = it can). */
  why(f) { return f.readonly || below(f.rights, this.spec.level) || (this.spec.readonly ? (this.spec.readonly === true ? 'Locked' : this.spec.readonly) : ''); }

  async tap(f) {
    const why = this.why(f);
    if (why) { toast(why); return; }
    if (f.input?.type === 'paint') { this.paint(f); return; }
    const got = await askField(f);
    if (got == null) return;
    if (typeof got === 'object') { this.run(got.add); return this.run(f.line.replace('{value}', got.value)); }
    this.run(f.line.replace('{value}', valueWord(f, got)));
  }

  /** An object's Paint (decisions 0294): the kit's picker; its Please runs the field's line once. */
  paint(f) {
    const object = /^Set (.+?) "Paint"/i.exec(f.line || '')?.[1] || this.spec.title || '';
    const own = (this.spec.fields || []).find(x => x.key === 'Name' || x.name === 'Name')?.value;
    const name = (own && own !== '—' ? own : '') || /"([^"]+)"/.exec(this.spec.heading || this.spec.title || '')?.[1] || '';
    openPaint({ object, name, no: /(\d+)$/.exec(object)?.[1] ?? '', line: f.line, value: f.value }, { run: l => this.run(l) });
  }

  run(line) {
    if (!line) return;
    this.last = line;
    const last = this.querySelector('.lc-set-last');
    if (last) last.textContent = `› ${line}`;
    this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true }));
  }

  /** One cell: kind badge + name + home mark, the value drawn by kind
   *  (number: − value unit +, switch, list box, underlined text), then
   *  (window settings) the pin and ↺. */
  cell(f, i, bar) {
    const why = this.why(f);
    const ro = !!why;
    const kind = kindOf(f.input);
    const pinnable = !this.spec.readonly && bar && f.key && f.key !== BAR_ON && f.input?.type !== 'list';
    const pinned = pinnable && ids(bar.value).includes(f.key);
    const side = pinnable || (!ro && f.changed);
    const on = /^(yes|on|true)$/i.test(f.value);
    // A paint (decisions 0294): its swatch (a colour as the station sends it) and its word; empty is
    // `none` — only a window's or the theme's colour reads "Theme" when empty.
    const paint = f.input?.type === 'paint';
    const sw = paint ? [f.colour, f.value].find(c => /^#[0-9a-f]{6}$/i.test(c || '')) || '' : '';
    const value = paint
      ? `<i class="lc-set-sw${sw ? '' : ' none'}"${sw ? ` style="--c:${esc(sw)}"` : ''}></i>${esc(f.value) || 'none'}`
      : `${f.input?.type === 'colour' && f.value !== 'Theme' ? `<i style="background:${esc(f.value)}"></i>` : ''}${esc(f.value) || (f.input?.type === 'colour' ? 'Theme' : '—')}`;
    const steps = kind === 'number' && !ro && !(f.input?.named || []).includes(f.value);
    return `<div class="lc-set-cell k-${kind}${pinned ? ' pinned' : ''}${side ? ' sided' : ''}${ro ? ' ro' : ''}${steps ? ' steps' : ''}" data-kind="${kind}" ${f.key ? `data-setting="${esc(f.key)}"` : ''} ${ro ? `data-why="${esc(why)}"` : ''}>
        <button type="button" class="lc-set-tap" data-i="${i}" title="${esc(why || f.hint)}">
          <span class="lc-set-label">${kindBadge(kind, ro)}<span class="lc-set-fname">${esc(f.name)}</span>${homeMark(f.home, this.spec.home)}${ro && !this.spec.readonly ? '<luma-icon name="lock" class="lc-set-lock"></luma-icon>' : ''}</span>
          <span class="lc-set-row">${kind === 'toggle' ? `<i class="lc-switch${on ? ' on' : ''}" aria-hidden="true"></i>` : ''}<span class="lc-set-value">${value}</span>${f.unit ? `<small class="lc-set-unit">${esc(f.unit)}</small>` : ''}${kind === 'list' ? '<luma-icon name="chevron-down" class="lc-set-drop"></luma-icon>' : ''}</span>
        </button>
        ${steps ? `<button type="button" class="lc-set-step minus" data-step="-" data-si="${i}" aria-label="${esc(f.name)} − 1"><luma-icon name="minus"></luma-icon></button>
          <button type="button" class="lc-set-step plus" data-step="+" data-si="${i}" aria-label="${esc(f.name)} + 1"><luma-icon name="plus"></luma-icon></button>` : ''}
        ${side ? `<div class="lc-set-side">
          ${pinnable ? `<button type="button" class="lc-set-pin${pinned ? ' on' : ''}" data-pin="${i}" aria-pressed="${pinned}"
            aria-label="${pinned ? 'Unpin from the window bar' : 'Pin to the window bar'}" title="${pinned ? 'In the window bar — tap to unpin' : 'Pin to the window bar'}"><luma-icon name="pin"></luma-icon></button>` : ''}
          ${!ro && f.changed && f.reset ? `<button type="button" class="lc-set-reset" data-reset="${i}" aria-label="Reset to default" title="${esc(f.reset)}"><luma-icon name="rotate-ccw"></luma-icon></button>` : ''}
        </div>` : ''}
      </div>`;
  }

  /** The Window bar tab: on/off, the items in order, the actions to add. */
  barTab(fields, bar) {
    const on = fields.find(f => f.key === BAR_ON);
    const items = bar.input.items || [];
    const byId = new Map(items.map(it => [it.id, it]));
    const list = ids(bar.value);
    const isOn = on && /^(yes|on|true)$/i.test(on.value);
    const actions = items.filter(it => it.kind === 'action');
    return `<div class="lc-set-bar" data-kit="settings-bar">
      <div class="lc-set-bar-list">
        <div class="lc-set-bar-head">
          ${on ? `<button type="button" class="lc-set-bar-on${isOn ? ' on' : ''}" data-setting="${BAR_ON}" aria-pressed="${!!isOn}"><i>${isOn ? '<luma-icon name="check"></luma-icon>' : ''}</i>${esc(on.name)}</button>` : ''}
          <span class="lc-set-bar-hint">In the bar, left → right · drag the handle or use ↑ ↓</span>
          <button type="button" class="lc-set-bar-def" data-defaults="1">Load defaults</button>
        </div>
        <div class="lc-set-bar-items">
          ${list.map((id, i) => {
            const it = byId.get(id) || { id, label: id, kind: 'setting' };
            return `<div class="lc-set-bar-item" data-item="${esc(id)}">
              <span class="lc-set-grip" data-grip="${i}" aria-label="Drag to move"><luma-icon name="grip-vertical"></luma-icon></span>
              <span class="lc-set-bar-name">${esc(it.label)}</span>
              <span class="lc-set-kind ${it.kind === 'action' ? 'action' : 'setting'}">${it.kind === 'action' ? 'action' : 'setting'}</span>
              <button type="button" data-up="${i}" aria-label="Move up" ${i === 0 ? 'disabled' : ''}><luma-icon name="arrow-up"></luma-icon></button>
              <button type="button" data-down="${i}" aria-label="Move down" ${i === list.length - 1 ? 'disabled' : ''}><luma-icon name="arrow-down"></luma-icon></button>
              <button type="button" class="drop" data-drop="${i}" aria-label="Remove from the bar"><luma-icon name="x"></luma-icon></button>
            </div>`;
          }).join('') || '<div class="lc-set-bar-empty">Nothing in the bar — pin settings on the other tabs or add an action.</div>'}
        </div>
      </div>
      <div class="lc-set-bar-acts">
        <span class="lc-set-bar-hint">Actions of the window (declared by its type)</span>
        ${actions.map(a => {
          const inBar = list.includes(a.id);
          return `<button type="button" class="lc-set-bar-act${inBar ? ' in' : ''}" data-add="${esc(a.id)}" ${inBar ? 'aria-disabled="true"' : ''}>
            <span><luma-icon name="${inBar ? 'check' : 'plus'}"></luma-icon>${esc(a.label)}</span><small>${esc(a.line || '')}</small></button>`;
        }).join('') || '<span class="lc-set-bar-hint">This window type has no actions.</span>'}
        <span class="lc-set-bar-hint">Settings are added with their pin on the other tabs.</span>
      </div>
    </div>`;
  }

  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    const secs = this.sections();
    this.tab = Math.max(0, Math.min(this.tab, secs.length - 1));
    const shown = (s.fields || []).filter(f => (f.section || '') === secs[this.tab]);
    const tabs = secs.length > 1;
    const embedded = this.hasAttribute('embedded');
    const bar = this.barField();
    const barHere = bar && shown.includes(bar);
    const treeHere = shown.find(f => f.input?.type === 'tags');
    // tidy: slab-ok (a window's own settings header, not a list of objects)
    const actions = [...(s.actions || []), ...(s.delete_line && !(s.actions || []).length ? [{ label: 'Delete window', line: s.delete_line, variant: 'danger' }] : [])];
    this.innerHTML = `
      <div class="lc-set${embedded ? ' embedded' : ''}${s.readonly ? ' readonly' : ''}">
        ${embedded ? '' : `<header class="lc-set-title"><luma-icon name="${esc(s.icon || 'settings')}"></luma-icon>
          <span class="lc-set-name">${esc(s.heading || s.title)}${s.subtitle ? ` <small class="lc-set-sub">${esc(s.subtitle)}</small>` : ''}</span>
          ${actions.map((a, i) => `<button type="button" class="lc-set-act ${esc(a.variant || '')}" data-act="${i}">${esc(a.label)}</button>`).join('')}
          ${actions.length ? '<button type="button" class="lc-set-more" data-kit="settings-more" aria-label="More" title="Load / Save preset, Reset, Delete"><luma-icon name="ellipsis"></luma-icon></button>' : ''}
          <button type="button" class="lc-set-x" aria-label="Close"><luma-icon name="x"></luma-icon></button></header>`}
        <div class="lc-set-body">
          ${tabs ? `<nav class="lc-set-tabs">${secs.map((n, i) => `<button type="button" data-tab="${i}" class="${i === this.tab ? 'on' : ''}">${esc(n || 'Settings')}</button>`).join('')}</nav>` : ''}
          <div class="lc-set-main">
            ${s.readonly ? `<div class="lc-set-locked"><luma-icon name="lock"></luma-icon>${esc(s.readonly === true ? 'Locked' : s.readonly)}</div>` : ''}
            ${s.note ? `<div class="lc-set-note">${esc(s.note)}</div>` : ''}
            ${barHere ? this.barTab(shown, bar) : treeHere ? `<div class="lc-set-note">${esc(treeHere.hint || '')}</div><div class="lc-set-tree" data-kit="settings-tags"></div>` : `<div class="lc-set-grid">${shown.map((f, i) => this.cell(f, i, bar)).join('')}</div>`}
            <div class="lc-set-last lc-selectable">${this.last ? `› ${esc(this.last)}` : ''}</div>
          </div>
        </div>
      </div>`;
    this.querySelector('.lc-set-x')?.addEventListener('click', () => this.dispatchEvent(new CustomEvent('close', { bubbles: true })));
    for (const b of this.querySelectorAll('[data-act]')) b.addEventListener('click', () => this.run(actions[+b.dataset.act].line));
    // Narrow: the header buttons in ⋯ (a container query shows it below ~600 px).
    const more = this.querySelector('.lc-set-more');
    more?.addEventListener('click', ev => {
      ev.stopPropagation();
      const r = more.getBoundingClientRect();
      more.classList.add('open');
      openMenu({ x: r.right - 220, y: r.bottom + 2 }, actions.map(a => ({
        label: `${a.label}…`, danger: a.variant === 'danger', warn: a.variant === 'warn', fn: () => this.run(a.line),
      })));
      setTimeout(() => document.addEventListener('pointerdown', () => more.classList.remove('open'), { once: true, capture: true }));
    });
    for (const b of this.querySelectorAll('[data-tab]')) b.addEventListener('click', () => { this.tab = +b.dataset.tab; this.render(); });
    for (const b of this.querySelectorAll('.lc-set-tap')) b.addEventListener('click', () => this.tap(shown[+b.dataset.i]));
    // − / + run the field's line with a relative step (the station works it out).
    for (const b of this.querySelectorAll('[data-step]')) b.addEventListener('click', () => {
      const f = shown[+b.dataset.si];
      if (!this.why(f)) this.run(f.line.replace('{value}', `${b.dataset.step} 1`));
    });
    for (const b of this.querySelectorAll('[data-reset]')) b.addEventListener('click', () => this.run(shown[+b.dataset.reset].reset));
    for (const b of this.querySelectorAll('[data-pin]')) b.addEventListener('click', () => {
      const key = shown[+b.dataset.pin].key;
      const list = ids(bar.value);
      this.setBar(bar, list.includes(key) ? list.filter(k => k !== key) : [...list, key]);
    });
    if (barHere) this.wireBar(bar);
    if (treeHere) this.querySelector('.lc-set-tree').append(this.tagTree(treeHere));
  }

  /** A new bar list: drawn at once (the next tap builds on it), then the
   *  line; the station's fresh fields follow (and win). */
  setBar(bar, next) {
    bar.value = next.join(',');
    const line = barLine(next);
    this.last = line;
    this.render();
    this.run(line);
  }

  wireBar(bar) {
    const list = ids(bar.value);
    const set = next => this.setBar(bar, next);
    const move = (i, j) => {
      if (j < 0 || j >= list.length || i === j) return;
      const next = list.slice();
      const [it] = next.splice(i, 1);
      next.splice(j, 0, it);
      set(next);
    };
    this.querySelector('[data-setting="window_bar"]')?.addEventListener('click', () => this.run('Set Self "WindowBar" Toggle'));
    this.querySelector('[data-defaults]')?.addEventListener('click', () => this.run('Reset Self "WindowBarItems"'));
    for (const b of this.querySelectorAll('[data-up]')) b.addEventListener('click', () => move(+b.dataset.up, +b.dataset.up - 1));
    for (const b of this.querySelectorAll('[data-down]')) b.addEventListener('click', () => move(+b.dataset.down, +b.dataset.down + 1));
    for (const b of this.querySelectorAll('[data-drop]')) b.addEventListener('click', () => set(list.filter((_, k) => k !== +b.dataset.drop)));
    for (const b of this.querySelectorAll('[data-add]')) b.addEventListener('click', () => {
      if (!list.includes(b.dataset.add)) set([...list, b.dataset.add]);
    });
    // Drag a row by its handle: it follows the pointer; dropped, the new order is one line.
    for (const g of this.querySelectorAll('[data-grip]')) {
      const row = g.closest('.lc-set-bar-item');
      const from = +g.dataset.grip;
      let h = 1;
      drag(g, {
        onStart: () => { h = row.offsetHeight + 6; row.classList.add('dragging'); },
        onMove: ({ dy }) => { row.style.transform = `translateY(${dy}px)`; row.dataset.to = String(Math.max(0, Math.min(list.length - 1, from + Math.round(dy / h)))); },
        onEnd: () => { row.classList.remove('dragging'); row.style.transform = ''; move(from, +(row.dataset.to ?? from)); },
      });
    }
  }
}
customElements.define('luma-settings', LumaSettings);

let current = null;

/** The settings window over everything; a second open with the same title
 *  updates the open one in place (the station's answer after a change). */
export function openSettings(spec, { run }) {
  if (current && current.title === spec.title) { current.el.update(spec); return current; }
  current?.close();
  const scrim = document.createElement('div');
  scrim.className = 'lc-set-scrim';
  const el = document.createElement('luma-settings');
  scrim.append(el);
  document.body.append(scrim);
  el.update(spec);
  const close = () => {
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    if (current?.el === el) current = null;
    el.dispatchEvent(new CustomEvent('closed', { bubbles: false }));
  };
  const onKey = e => {
    if (e.key === 'Escape' && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, [data-kit="dialog"], [data-kit="paint-picker"]')) { e.preventDefault(); e.stopPropagation(); close(); }
  };
  document.addEventListener('keydown', onKey, true);
  el.addEventListener('run', e => run(e.detail.line));
  el.addEventListener('close', close);
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  current = { el, close, title: spec.title, update: s => el.update(s) };
  return current;
}

export function settingsOpen() { return !!current; }
/** The open settings window's title (null: none open). */
export function settingsTitle() { return current?.title ?? null; }
