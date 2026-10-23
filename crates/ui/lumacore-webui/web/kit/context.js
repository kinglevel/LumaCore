// The one context menu (decisions §34 round 3, 18, the approved context-menu.html): every list,
// grid and pool opens the same menu on its selection — right-click, a hold without moving, the
// selection chip's ⋯, the Menu key (Shift+F10 is Meow+F10, a core key). It is drawn from the station's registry
// (GET /api/object-menus, `lumacore_views::context`), never a per-window list:
//
//   the common actions   Edit… (the edit card) · Renumber… · Label… · Copy · Move… — the
//                        object's verbs
//   Set                  every declared property: the rows' shared value or ‹different›, its
//                        editor by kind (pick list, keypad, text, colour)
//   <Object>             its own lines (Patch to…, Select, Info …), then Delete
//
// An item the rights or the claim refuse stays in place, greyed, with the reason under it (the
// menu never jumps). Every pick is one line naming the rows — one Oops step. A phone (decisions
// §37 7: its short side under 500 px) gets the same menu as the kit's bottom sheet, like the
// value dialog: a grab handle, the selection and ✕ on top, then the common actions, the object's
// own lines and Set folded to one line ("Set · 14 ▸", a tap opens it); at most about two-thirds
// of the screen, scrolling inside; a swipe down, a tap outside, ✕ or Esc close it. On its side it
// opens from the right.
//
//   openContext({
//     at: { x, y }, menu (one object's entry of /api/object-menus),
//     object: 'Fixture' (how lines name it, owner first: 'DataPool 1 Group'),
//     rows: '101 Thru 103', count: 3,
//     value(prop) → [values of the selected rows] | null (unknown: no value shown),
//     props: ['Name', 'Address', …] | null (a grid: only its columns; null = every property),
//     labels: { PosX: 'X', … } (a grid's column labels for its properties),
//     claim: '' | 'Claim the draft to change it' (a claim area's Live: what it greys),
//     note: 'Live patch — live columns only', level: 'Admin',
//     run(line), edit() (Edit…: the host's edit card), target(item) (optional: the host picks a
//     `{value}` itself — a pool's tap-the-target; return true when it took it),
//     reveal(top) (optional: a phone's sheet starts at `top` — the host scrolls the rows above it
//     and returns what puts the list back when the sheet closes),
//     valueDialog(kind) (optional: the host opens the value dialog for an item's `value` kind —
//     'number', 'address' — and returns true; else the plain ask),
//   })
//   closeContext() · contextOpen()

import { openDialog } from './dialog.js';
import { askField, valueWord, quote } from './settings.js';
import { phoneMode } from './phone.js';
import { grabToClose } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];
const below = (need, have) => (need && have && LEVELS.indexOf(have) < LEVELS.indexOf(need) ? `Needs ${need} — you are ${have}` : '');

let menus = null;
/** Every object's menu, once per page (the registry doesn't change at run time). */
export async function objectMenus() {
  if (!menus) menus = fetch('/api/object-menus').then(r => (r.ok ? r.json() : {})).catch(() => ({}));
  return menus;
}

let el = null;
let spec = null;
const onOutside = e => { if (el && !el.contains(e.target)) closeContext(); };
const onKey = e => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); closeContext(); } };

export function contextOpen() { return !!el; }
export function closeContext() {
  if (!el) return;
  el.scrim?.remove();
  el.unreveal?.();
  el.remove();
  el = null;
  spec = null;
  document.removeEventListener('pointerdown', onOutside, true);
  document.removeEventListener('keydown', onKey, true);
}

/** The shown value of a property over the selection: one shared value, or ‹different› n. */
function shared(s, prop) {
  const vals = s.value?.(prop);
  if (!vals) return { text: '' };
  const set = [...new Set(vals.map(v => String(v ?? '')))];
  if (set.length > 1) return { text: `‹different› ${set.length}`, diff: true };
  return { text: set[0] || '—', value: set[0] ?? '' };
}

export function openContext(s) {
  closeContext();
  spec = s;
  const m = s.menu || { common: [], set: [], own: [] };
  const fill = line => String(line).replaceAll('{object}', s.object).replaceAll('{rows}', s.rows).replace(/\s+/g, ' ').trim();
  const why = (rights, claim) => below(rights, s.level) || (claim && s.claim ? s.claim : '');
  const items = [];
  const row = (k, icon, label, off, extra = '', cls = '') => {
    items.push(`<button type="button" class="lc-cm-item${off ? ' off' : ''}${cls ? ` ${cls}` : ''}" role="menuitem" data-k="${esc(k)}"
      ${off ? `aria-disabled="true" data-why="${esc(off)}"` : ''}><span class="lc-cm-ic">${icon ? `<luma-icon name="${esc(icon)}"></luma-icon>` : ''}</span>
      <span class="lc-cm-lbl">${esc(label)}</span>${extra}${off ? `<span class="lc-cm-why">${esc(off)}</span>` : ''}</button>`);
  };
  const plural = s.count === 1 ? m.object : m.plural;
  const phone = phoneMode();
  items.push(`<div class="lc-cm-head"${phone ? ' data-head' : ''}>${phone ? '<i class="lc-grab" aria-hidden="true"></i>' : ''}<b>${esc(`${s.count} ${plural} · ${s.rows}`)}</b>${s.note ? `<small>${esc(s.note)}</small>` : ''}${phone ? '<button type="button" class="lc-close lc-cm-x" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button>' : ''}</div>`);
  const common = (m.common || []).filter(c => c.id !== 'delete');
  for (const c of common) row(`c:${c.id}`, c.icon, c.label, why(c.rights, c.claim));
  // A phone: the object's own lines come before Set, which folds to one line.
  const own = () => {
    const del = (m.common || []).find(c => c.id === 'delete');
    if (!(m.own || []).length && !del) return;
    items.push(`<div class="lc-cm-sep"></div><div class="lc-cm-sec">${esc(m.object)}</div>`);
    (m.own || []).forEach((o, i) => row(`o:${i}`, o.icon, o.label, why(o.rights, !o.live)));
    if (del) row('c:delete', del.icon, `Delete ${s.count > 1 ? `${s.count} ${m.plural}` : m.object}`, why(del.rights, del.claim), '', 'danger');
  };
  if (phone) own();
  // Set: a grid's columns (its declared properties), else every property — the object's own first.
  const wanted = s.props ? s.props.map(p => (m.set || []).find(x => x.name.toLowerCase() === String(p).toLowerCase())).filter(Boolean)
    : [...(m.set || []).filter(p => !p.meta), ...(m.set || []).filter(p => p.meta)];
  if (wanted.length) {
    items.push('<div class="lc-cm-sep"></div>');
    if (phone) items.push(`<button type="button" class="lc-cm-item lc-cm-fold" data-k="fold" aria-expanded="false"><span class="lc-cm-ic"><luma-icon name="chevron-right"></luma-icon></span><span class="lc-cm-lbl">Set · ${wanted.length}</span><span class="lc-cm-val">›</span></button><div class="lc-cm-set" hidden>`);
    else items.push('<div class="lc-cm-sec">Set</div>');
    for (const p of wanted) {
      const v = shared(s, p.name);
      const off = why(p.rights, !p.live);
      row(`s:${p.name}`, '', s.labels?.[p.name] || p.name, off, `<span class="lc-cm-val${v.diff ? ' diff' : ''}">${esc(v.text)}${off ? '' : ' ›'}</span>`);
    }
    if (phone) items.push('</div>');
  }
  if (!phone) own();
  el = document.createElement('div');
  el.className = 'lc-cm';
  el.dataset.kit = 'context';
  el.setAttribute('role', 'menu');
  el.innerHTML = items.join('');
  document.body.append(el);
  el.classList.toggle('sheet', !!phone);
  el.classList.toggle('side', phone === 'side');
  if (phone) {
    // The kit's sheet look: its scrim, the grab handle's swipe and ✕ close it.
    const scrim = document.createElement('div');
    scrim.className = 'lc-cm-scrim';
    el.before(scrim);
    el.scrim = scrim;
    grabToClose(el.querySelector('[data-head]'), el, closeContext, { side: phone === 'side' });
    // The rows it acts on stay in sight: the host scrolls them above the sheet.
    if (phone === 'up') el.unreveal = s.reveal?.(el.getBoundingClientRect().top);
    el.querySelector('[data-x]').onclick = closeContext;
  } else {
    const vw = window.innerWidth, vh = window.innerHeight;
    el.style.left = `${Math.max(4, Math.min(s.at?.x ?? 0, vw - el.offsetWidth - 4))}px`;
    el.style.top = `${Math.max(4, Math.min(s.at?.y ?? 0, vh - el.offsetHeight - 4))}px`;
  }
  el.querySelector('.lc-cm-item:not(.off)')?.focus();
  // The press that opened it must not close it: listen for presses from the
  // next turn. Esc is armed at once (no key opens a menu, and a key sent right
  // after the opening press must not slip in before the listener).
  document.addEventListener('keydown', onKey, true);
  setTimeout(() => document.addEventListener('pointerdown', onOutside, true));
  el.addEventListener('keydown', e => {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    e.preventDefault();
    const all = [...el.querySelectorAll('.lc-cm-item:not(.off)')];
    const i = all.indexOf(document.activeElement);
    all[(i + (e.key === 'ArrowDown' ? 1 : all.length - 1)) % all.length]?.focus();
  });
  el.addEventListener('click', e => {
    const b = e.target.closest('.lc-cm-item');
    if (!b) return;
    // A phone's folded Set opens in place.
    if (b.dataset.k === 'fold') {
      const set = el.querySelector('.lc-cm-set');
      set.hidden = !set.hidden;
      b.setAttribute('aria-expanded', String(!set.hidden));
      b.querySelector('luma-icon')?.setAttribute('name', set.hidden ? 'chevron-right' : 'chevron-down');
      return;
    }
    // A greyed item says why under itself; a tap does nothing more.
    if (b.dataset.why) return;
    const k = b.dataset.k;
    const [kind, id] = [k.slice(0, 1), k.slice(2)];
    const host = spec;
    closeContext();
    if (kind === 'c') pickCommon(host, m.common.find(c => c.id === id), fill);
    else if (kind === 'o') pickLine(host, m.own[+id], fill);
    else if (kind === 's') pickSet(host, m.set.find(p => p.name === id), fill);
  });
}

async function pickCommon(s, c, fill) {
  if (!c) return;
  if (c.id === 'edit') { s.edit?.(); return; }
  pickLine(s, c, fill);
}

/** A line with `{value}` asks it first (the host may pick it itself: a pool's tap-the-target). */
async function pickLine(s, item, fill) {
  // An item without a line opens what its value names (Edit Patch… — no line runs).
  if (item?.value && !item.line) { s.valueDialog?.(item.value); return; }
  if (!item?.line) return;
  if (!item.line.includes('{value}')) { s.run(fill(item.line)); return; }
  if (item.value && s.valueDialog?.(item.value)) return;
  if (s.target?.(item, v => s.run(fill(item.line).replace('{value}', v)))) return;
  const label = String(item.label).replace(/…$/, '');
  const got = await openDialog({
    title: `${label} · ${s.object} ${s.rows}`, text: fill(item.line.replace('{value}', '…')),
    inputs: [{ name: 'v', label, value: '', kind: 'text' }],
    buttons: [{ label: 'Please', value: 'go', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
  });
  const v = String(got?.inputs?.v ?? '').trim();
  if (!got?.value || !v) return;
  // A name is quoted text (decisions §33: a bare word is a word); a number or address stays bare.
  s.run(fill(item.line).replace('{value}', /^[0-9.]+( Thru [0-9.]+)?$/i.test(v) ? v : quote(v)));
}

async function pickSet(s, p, fill) {
  if (!p) return;
  const cur = shared(s, p.name);
  const f = { name: p.name, value: cur.diff ? '' : (cur.value ?? ''), input: p.input || { type: 'text' }, hint: `${s.object} ${s.rows}` };
  const got = await askField(f);
  if (got == null || typeof got === 'object') return;
  s.run(fill(`Set {object} {rows} "${p.name}" ${valueWord(f, got)}`));
}
