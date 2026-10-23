// <luma-slab> — the kit's one action block (decisions §30 "12d hardening":
// "The slab is the kit's one action block", a function block). Every
// window or menu that lists objects puts its add / delete / copy / … actions
// here, on the selection of its <luma-grid>.
//
//   Left half, fixed order, on the selection: Select · New · Copy · Cut ·
//   Paste · Delete · Oops · Import · Export · Edit — each with its one
//   minimal icon in its standard colour, the same in every menu (Select /
//   Edit accent, New ok, Copy / Paste info, Cut warn, Delete alert — its label
//   alert only when it applies —, Oops amber, Import / Export teal). Labels
//   carry the count ("Delete 3"). An action that doesn't apply is greyed (a
//   grey icon) and says why on a tap.
//   Right half: the menu tab's own actions (optional icon + tone: the show
//   actions Load (gold primary), Save / Save as (ok), New show (neutral),
//   Info (blue), History (violet) carry a faint tint), up to 4 bound setting
//   cells (one Set line each) and the power toggle (<luma-power>, look A).
//   On a phone (a container query) it is a 2-column block: the power toggle
//   first, the tab's `phone` actions, then New · Delete · Edit · Copy · Oops ·
//   Import.
//   `bar` (the setup frame on a phone, decisions §37 1–2): one bar instead —
//   the actions that work now (the tab's `phone` actions, then New · Delete ·
//   Edit · Copy · Import, those that apply: 2 upright, 4 on its side), Oops,
//   ⋯ More (every cell in a sheet, the tab's own and the rows' — greyed ones
//   say why on a tap) and the power as a small cell; a claim button is a lock
//   cell (a tap: what the state means in a sheet, with the full button; a hold
//   claims). 'up' is a row, 'side' a column on the right.
//   A narrow slab (under 1000 px) shows an action's `short` label; one without
//   wraps to two lines (§37 3).
//
//   el.update({ edit: { select|new|copy|cut|paste|delete|oops|import|export|
//     edit: { line, off, rights } }, actions: [{ id, label, icon, tone, line,
//     rights, needs: 'selection' | 'one' | null, list (opens a list: the
//     yellow `===` mark), short (its label in a narrow cell) }], phone_edit: [ids] (the standard actions a
//     narrow slab shows; default New · Delete · Edit · Copy · Oops · Import), bound: [{ label, value,
//     line, input, rights, off }], power: { state: 'on' | 'off' | 'failing', word,
//     press, reason, rights, claim (a claim button: last on a phone too), explain: { title, text }
//     (what the state means: the lock's sheet) } | null, phone: [ids], level, count, clip: {
//     op, count } | null, lasso: bool, bar: 'up' | 'side' | '', label (the More sheet's
//     first group: the tab's name), title (the More sheet's title: what the lines name) })
//   el.openLock() — the lock's sheet (the setup frame's state chip opens it too)
//   events: act { id, action } (the host runs the line, opens the edit card,
//     pastes) · power {} · bound { index }
//
// <luma-power> — the power toggle, look A (the PowerToggle board): the usual
// tile, a big power icon, the state word (ONLINE / OFFLINE / SENDING /
// OUTPUT HELD), what a press does under it and a 5 px state bar along the
// bottom; "on but failing" fills the tile dark alert with the reason as the
// line. `compact` lays it out in a row (the phone's bar).
//   el.update({ state, word, press, note, reason, label, off, icon, hold, hold_ms, tap })   events:
//   press, hold, tap (`tap`: a press fires `tap` instead of the hold hint — the phone's lock cell)
//   `mini`: the phone bar's small cell (icon over the word)
//   (`icon` replaces the power icon; `note` is the small line as given, without "Press:";
//   `hold`: a hold of `hold_ms` (600) fires `hold` and a press only shows this hint — the Patch
//   dialog's claim button, decisions §34 round 3, 9: gold fills left → right over the hold and
//   drains back on an early release; Live · "Hold to claim and edit", state `draft` = Claimed
//   (gold), state `theirs` = someone else's draft, greyed by `off`)

import { toast } from './toast.js';
import { hold } from './pointer.js';
import { listMark } from './button.js';
import { openSheet } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];
const below = (need, have) => (need && have && LEVELS.indexOf(have) < LEVELS.indexOf(need) ? `Needs ${need} — you are ${have}` : '');

/** The standard edit actions: order, label, icon, colour role. */
export const EDIT_ACTIONS = [
  { id: 'select', label: 'Select', icon: 'mouse-pointer-2', tone: 'select' },
  { id: 'new', label: 'New', icon: 'plus', tone: 'new' },
  { id: 'copy', label: 'Copy', icon: 'copy', tone: 'copy', count: true, sel: true },
  { id: 'cut', label: 'Cut', icon: 'scissors', tone: 'cut', count: true, sel: true },
  { id: 'paste', label: 'Paste', icon: 'clipboard-paste', tone: 'copy' },
  { id: 'delete', label: 'Delete', icon: 'trash-2', tone: 'delete', count: true, sel: true },
  { id: 'oops', label: 'Oops', icon: 'undo-2', tone: 'oops' },
  { id: 'import', label: 'Import', icon: 'arrow-down-to-line', tone: 'io' },
  { id: 'export', label: 'Export', icon: 'arrow-up-from-line', tone: 'io' },
  { id: 'edit', label: 'Edit', icon: 'pencil', tone: 'select', count: true, sel: true },
];
/** On a phone: these, after the tab's own main actions. */
const PHONE_EDIT = ['new', 'delete', 'edit', 'copy', 'oops', 'import'];
/** The phone bar's candidates after the tab's `phone` actions (Oops has its own cell). */
const BAR_EDIT = ['new', 'delete', 'edit', 'copy', 'import'];
/** The show actions' icons (decisions §30 "History matrix + coloured show actions"). */
export const SHOW_ICONS = { load: 'folder-open', save: 'save', saveas: 'save-plus', newshow: 'file-plus-2', info: 'info', history: 'history' };

class LumaSlab extends HTMLElement {
  constructor() { super(); this.spec = { edit: {}, actions: [], bound: [] }; }
  connectedCallback() { this.dataset.kit = 'slab'; this.render(); }
  disconnectedCallback() { this.sheet?.close(); }
  update(spec) { this.spec = { edit: {}, actions: [], bound: [], ...spec }; this.render(); }

  /** Why the standard action `a` can't run now ('' = it can). */
  whyEdit(a) {
    const s = this.spec;
    const d = s.edit?.[a.id];
    const count = s.count || 0;
    if (!d) return `${a.label} isn't available here`;
    if (d.off) return d.off;
    const r = below(d.rights, s.level);
    if (r) return r;
    if (a.sel && !count) return 'Select rows first';
    // The station's clipboard (a Paste line) is the station's to judge (decisions §35 9).
    if (a.id === 'paste' && !s.clip && !d.line?.startsWith('Paste')) return 'Copy or Cut first';
    if (!d.line && !['select', 'edit', 'paste', 'copy', 'cut'].includes(a.id)) return `${a.label} isn't available here`;
    return '';
  }
  whyAction(a) {
    const r = below(a.rights, this.spec.level);
    if (r) return r;
    if (a.off) return a.off;
    if (a.needs === 'selection' && !(this.spec.count > 0)) return 'Select rows first';
    if (a.needs === 'one' && this.spec.count !== 1) return 'Select one row';
    return '';
  }

  editCell(a) {
    const why = this.whyEdit(a);
    const s = this.spec;
    const n = a.id === 'paste' ? s.clip?.count : a.count ? s.count : 0;
    const on = a.id === 'select' && s.lasso;
    return `<button type="button" class="lc-slab-cell edit t-${a.tone}${why ? ' off' : ''}${on ? ' latched' : ''}${a.id === 'delete' && !why ? ' danger' : ''}"
        data-act="${a.id}" ${why ? `aria-disabled="true" data-why="${esc(why)}"` : ''} title="${esc(why || a.label)}" aria-pressed="${on ? 'true' : 'false'}">
        <luma-icon name="${a.icon}"></luma-icon><span>${a.label}${n ? ` ${n}` : ''}</span></button>`;
  }
  actionCell(a, i) {
    const why = this.whyAction(a);
    const icon = a.icon || SHOW_ICONS[a.tone] || '';
    return `<button type="button" class="lc-slab-cell own${a.tone ? ` t-${esc(a.tone)} tinted` : ''}${why ? ' off' : ''}" data-own="${i}" data-id="${esc(a.id)}"
        ${why ? `aria-disabled="true" data-why="${esc(why)}"` : ''} title="${esc(why || a.line || a.label)}">
        ${icon ? `<luma-icon name="${esc(icon)}"></luma-icon>` : ''}${a.short ? `<span class="l-full">${esc(a.label)}</span><span class="l-short">${esc(a.short)}</span>` : `<span class="l-wrap">${esc(a.label)}</span>`}${a.list ? listMark() : ''}</button>`;
  }
  boundCell(b, i) {
    const why = below(b.rights, this.spec.level) || b.off || '';
    return `<button type="button" class="lc-slab-cell bound${why ? ' off' : ''}" data-bound="${i}" ${why ? `aria-disabled="true" data-why="${esc(why)}"` : ''} title="${esc(why || b.line || '')}">
        <small>${esc(b.label)}</small><span>${esc(b.value || '—')}</span></button>`;
  }

  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    if (s.bar) return this.renderBar();
    this.sheet?.close();
    const own = (s.actions || []).map((a, i) => this.actionCell(a, i));
    const bound = (s.bound || []).slice(0, 4).map((b, i) => this.boundCell(b, i));
    const right = [...own, ...bound];
    const cols = Math.max(1, Math.ceil(right.length / 2));
    // Phone order: the tab's main actions first, then the standard six.
    const phoneOwn = (s.phone || []).map(id => (s.actions || []).findIndex(a => a.id === id)).filter(i => i >= 0);
    // A host may name fewer (`phone_edit`: an editor's slab in a narrow column).
    const phoneEdit = (s.phone_edit || PHONE_EDIT).map(id => EDIT_ACTIONS.find(a => a.id === id)).filter(Boolean);
    const html = `<div class="lc-slab">
      <div class="lc-slab-desk">
        <div class="lc-slab-left">${EDIT_ACTIONS.map(a => this.editCell(a)).join('')}</div>
        ${right.length ? `<div class="lc-slab-right" style="--lc-slab-cols:${cols}">${right.join('')}</div>` : ''}
        ${s.power ? `<luma-power class="lc-slab-power${s.power.claim ? ' claim' : ''}"></luma-power>` : ''}
      </div>
      <div class="lc-slab-phone${s.power?.claim ? ' claim-last' : ''}">
        ${s.power ? '<luma-power compact class="lc-slab-power"></luma-power>' : ''}
        <div class="lc-slab-pgrid">${phoneOwn.map(i => this.actionCell(s.actions[i], i)).join('')}${phoneEdit.map(a => this.editCell(a)).join('')}</div>
      </div>
    </div>`;
    if (this.same(html)) return;
    this.innerHTML = html;
    this.wire(this);
  }

  /** The cells, the power and the bound cells in `root` (the slab, or a sheet of it) do their
   *  thing; `done` runs first (a sheet closes). */
  wire(root, done) {
    const s = this.spec;
    for (const p of root.querySelectorAll('luma-power')) {
      p.update(p.hasAttribute('mini') && s.power?.claim ? { ...s.power, tap: true } : s.power);
      p.addEventListener('press', () => { done?.(); this.dispatchEvent(new CustomEvent('power', { bubbles: true })); });
      p.addEventListener('hold', () => { done?.(); this.dispatchEvent(new CustomEvent('power', { bubbles: true, detail: { hold: true } })); });
      p.addEventListener('tap', () => this.openLock());
    }
    root.querySelector('[data-more]')?.addEventListener('click', () => this.openMore());
    for (const b of root.querySelectorAll('[data-act]')) b.addEventListener('click', () => {
      if (b.dataset.why) { toast(b.dataset.why); return; }
      const a = b.dataset.act;
      done?.();
      this.dispatchEvent(new CustomEvent('act', { detail: { id: a, action: s.edit?.[a] }, bubbles: true }));
    });
    for (const b of root.querySelectorAll('[data-own]')) b.addEventListener('click', () => {
      if (b.dataset.why) { toast(b.dataset.why); return; }
      const a = s.actions[+b.dataset.own];
      done?.();
      this.dispatchEvent(new CustomEvent('act', { detail: { id: a.id, action: a, own: true }, bubbles: true }));
    });
    for (const b of root.querySelectorAll('[data-bound]')) b.addEventListener('click', () => {
      if (b.dataset.why) { toast(b.dataset.why); return; }
      done?.();
      this.dispatchEvent(new CustomEvent('bound', { detail: { index: +b.dataset.bound, bound: s.bound[+b.dataset.bound] }, bubbles: true }));
    });
  }

  /** Nothing changed since the last draw (a feed update with the same cells): keep the elements, so
   *  a press that started on one isn't released on its replacement and lost. */
  same(html) {
    const key = html + JSON.stringify(this.spec.power || null);
    if (key === this.drawn && this.firstElementChild) return true;
    this.drawn = key;
    return false;
  }

  // ── The phone's one bar (decisions §37 1–2) ──
  /** The cells that work now: the tab's `phone` actions, then the standard ones that apply. */
  nowCells(n) {
    const s = this.spec;
    const out = [];
    for (const id of s.phone || []) {
      const i = (s.actions || []).findIndex(a => a.id === id);
      if (i >= 0 && !this.whyAction(s.actions[i])) out.push(this.actionCell(s.actions[i], i));
    }
    for (const id of BAR_EDIT) {
      const a = EDIT_ACTIONS.find(x => x.id === id);
      if (!this.whyEdit(a)) out.push(this.editCell(a));
    }
    return out.slice(0, n);
  }
  renderBar() {
    const s = this.spec;
    const oops = EDIT_ACTIONS.find(a => a.id === 'oops');
    const p = s.power;
    const html = `<div class="lc-slab lc-slab-bar1 ${esc(s.bar)}">
      ${this.nowCells(s.bar === 'side' ? 4 : 2).join('')}
      ${s.edit?.oops ? this.editCell(oops) : ''}
      <button type="button" class="lc-slab-cell more" data-more title="Every action"><luma-icon name="ellipsis"></luma-icon><span>More</span></button>
      ${p ? `<luma-power mini class="${p.claim ? 'lc-slab-lock' : 'lc-slab-power'}"></luma-power>` : ''}
    </div>`;
    if (!this.same(html)) {
      this.innerHTML = html;
      this.wire(this);
    }
    // An open sheet follows the selection and the state.
    if (this.sheet?.kind === 'more') this.fillMore();
    else if (this.sheet?.kind === 'lock') this.fillLock();
  }
  sheetOf(kind, title, sub) {
    if (this.sheet?.kind !== kind) {
      this.sheet?.close();
      const sh = openSheet({ title, sub, side: this.spec.bar === 'side', host: this.closest('.lc-setup') || undefined, cls: `lc-slab-sheet ${kind}`,
        onClose: () => { if (this.sheet === sh) this.sheet = null; } });
      sh.kind = kind;
      this.sheet = sh;
    } else this.sheet.setTitle(title, sub);
    return this.sheet;
  }
  /** ⋯ More: every cell — the tab's own (and its bound cells), then the rows'. */
  openMore() { this.sheetOf('more', 'More'); this.fillMore(); }
  fillMore() {
    const s = this.spec;
    const sh = this.sheetOf('more', s.title || 'More', s.title ? '' : '');
    const own = (s.actions || []).map((a, i) => this.actionCell(a, i));
    const bound = (s.bound || []).slice(0, 4).map((b, i) => this.boundCell(b, i));
    sh.body.innerHTML = `${own.length || bound.length ? `<div class="lc-psheet-sec">${esc(s.label || 'Actions')}</div><div class="lc-slab-cells">${[...own, ...bound].join('')}</div>` : ''}
      <div class="lc-psheet-sec">Rows</div><div class="lc-slab-cells">${EDIT_ACTIONS.map(a => this.editCell(a)).join('')}</div>`;
    this.wire(sh.body, () => sh.close());
  }
  /** The lock's sheet: what the state means, and the full button (a hold claims there too). */
  openLock() { if (this.spec.power) { this.sheetOf('lock', ''); this.fillLock(); } }
  fillLock() {
    const p = this.spec.power;
    if (!p) { this.sheet?.close(); return; }
    const sh = this.sheetOf('lock', p.explain?.title || p.word || '');
    sh.body.innerHTML = `<luma-power class="lc-slab-power"></luma-power>${p.explain?.text ? `<p class="lc-psheet-note">${esc(p.explain.text)}</p>` : ''}`;
    this.wire(sh.body, () => sh.close());
  }
}
customElements.define('luma-slab', LumaSlab);

class LumaPower extends HTMLElement {
  constructor() { super(); this.spec = { state: 'off' }; }
  connectedCallback() { this.dataset.kit = 'power'; this.render(); }
  update(spec) { this.spec = { state: 'off', ...(spec || {}) }; this.render(); }
  render() {
    if (!this.isConnected) return;
    const s = this.spec;
    const why = below(s.rights, s.level) || s.off || '';
    const failing = s.state === 'failing';
    this.innerHTML = `<button type="button" class="lc-power ${esc(s.state)}${why ? ' off-limits' : ''}" data-state="${esc(s.state)}" aria-pressed="${s.state !== 'off'}"
        title="${esc(why || s.label || '')}" ${why ? `aria-disabled="true" data-why="${esc(why)}"` : ''}>
        <luma-icon name="${esc(s.icon || 'power')}"></luma-icon>
        <span class="lc-power-text"><b class="lc-power-word">${esc(s.word || (s.state === 'off' ? 'OFF' : 'ON'))}</b>
        <small class="lc-power-press">${esc(failing && s.reason ? s.reason : s.note ?? `Press: ${s.press || ''}`)}</small></span>
        ${s.hold && !why ? '<i class="lc-power-fill"></i>' : ''}<i class="lc-power-bar"></i></button>`;
    const b = this.querySelector('button');
    b.addEventListener('click', () => {
      if (b.dataset.why) { toast(b.dataset.why); return; }
      // `tap`: the phone's lock cell — a press opens what the state means.
      if (s.tap) { this.dispatchEvent(new CustomEvent('tap', { bubbles: false })); return; }
      // `hold`: a press only says what a hold does (the patch's claim button, 34g4 entry).
      if (s.hold) { toast(s.hold); return; }
      this.dispatchEvent(new CustomEvent('press', { bubbles: false }));
    });
    if (s.hold && !b.dataset.why) {
      const ms = s.hold_ms || 600;
      b.style.setProperty('--lc-hold-ms', `${ms}ms`);
      hold(b, {
        ms, contextmenu: false, ringOn: false,
        onArm: () => b.classList.add('filling'),
        onDisarm: () => b.classList.remove('filling'),
        onHold: () => this.dispatchEvent(new CustomEvent('hold', { bubbles: false })),
      });
    }
  }
}
customElements.define('luma-power', LumaPower);
