// The value dialog (decisions §34 round 3, 18, the approved renumber.html v2): one small floating
// dialog for a value a selection takes — Renumber and Address first, any number a row has later.
// It opens beside the selection (never over it), drags by its header, and is a bottom sheet on a
// phone: a big field with − / +, one hint line, up to four free values that fit, the value's own
// toggles, Apply. Enter applies, Esc cancels, ↑ ↓ step. Every keystroke previews old → new in the
// list behind it (the host draws it); nothing is sent until Apply — one line, one Oops step.
//
//   openValue({
//     title: 'Renumber', sub: '3 Fixtures · 104 Thru 106', avoid: DOMRect (the selection),
//     kind: renumberKind(…) | addressKind(…),
//     preview(plan | null) (the host shows plan.preview / plan.way), apply(line),
//     more: { label, icon, run(typed) } (optional: the bigger tool for the same value — Address's
//       Slot grid… opens Edit Patch with what was typed),
//   })  → close()
//
// A kind: { start, placeholder, empty (the hint before typing), invalid (the hint for a bad
// value), parse(text) → value | null, step(text, ±1) → text, toggles: { key: [first = default,
// …] }, plan(value, toggles) → { hint: [tone ok|warn|dim, text], change, line, preview, way },
// free(toggles) → [[value, label]] }.

import { drag } from './pointer.js';
import { phoneMode } from './phone.js';
import { grabToClose } from './sheet.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const a3 = n => String(n).padStart(3, '0');

/** Numbers as a line names them: ascending runs as `a Thru b`, else in order with `+`. */
export function inOrder(list) {
  const ascending = list.every((v, i) => !i || v > list[i - 1]);
  if (!ascending) return list.join(' + ');
  const out = [];
  for (let i = 0; i < list.length;) {
    let j = i;
    while (j + 1 < list.length && list[j + 1] === list[j] + 1) j++;
    out.push(j > i ? `${list[i]} Thru ${list[j]}` : `${list[i]}`);
    i = j + 1;
  }
  return out.join(' + ');
}

let current = null;
export function closeValue() { current?.(); }

export function openValue({ title, sub = '', avoid, kind, preview, apply, more } = {}) {
  closeValue();
  const t = Object.fromEntries(Object.entries(kind.toggles || {}).map(([k, o]) => [k, o[0]]));
  const el = document.createElement('div');
  el.className = 'lc-vd';
  el.dataset.kit = 'value';
  el.setAttribute('role', 'dialog');
  el.innerHTML = `<div class="lc-vd-head" data-head><i class="lc-grab" aria-hidden="true"></i><b>${esc(title)}</b><small>${esc(sub)}</small>
      <button type="button" class="lc-vd-x" data-cancel aria-label="Cancel"><luma-icon name="x"></luma-icon></button></div>
    <div class="lc-vd-body">
      <div class="lc-vd-in"><button type="button" data-step="-1" aria-label="Step down"><luma-icon name="minus"></luma-icon></button>
        <input data-in inputmode="decimal" autocomplete="off" spellcheck="false" placeholder="${esc(kind.placeholder || '')}">
        <button type="button" data-step="1" aria-label="Step up"><luma-icon name="plus"></luma-icon></button></div>
      <div class="lc-vd-hint" data-hint></div>
      <div class="lc-vd-chips" data-chips></div>
      <div class="lc-vd-tg" data-tg></div>
      <div class="lc-vd-keys">Enter applies · Esc cancels · ↑ ↓ step</div>
    </div>
    <div class="lc-vd-foot">${more ? `<button type="button" class="lc-btn lc-vd-more" data-more><luma-icon name="${esc(more.icon || 'grid-3x3')}"></luma-icon>${esc(more.label)}</button>` : ''}<button type="button" class="lc-btn" data-cancel>Cancel</button><button type="button" class="lc-btn primary" data-apply disabled>Apply</button></div>`;
  document.body.append(el);
  const input = el.querySelector('[data-in]');
  input.value = kind.start?.() ?? '';
  let plan = null;
  const update = () => {
    const v = kind.parse(input.value);
    plan = v != null ? kind.plan(v, t) : null;
    const hint = el.querySelector('[data-hint]');
    const [tone, text] = plan ? plan.hint : ['dim', input.value ? kind.invalid : kind.empty];
    hint.className = `lc-vd-hint ${tone}`;
    hint.textContent = text || '';
    const free = kind.free?.(t) || [];
    el.querySelector('[data-chips]').innerHTML = free.length
      ? `<span>Free:</span>${free.slice(0, 4).map(([val, label]) => `<button type="button" class="lc-vd-chip${input.value === val ? ' on' : ''}" data-val="${esc(val)}">${esc(label)}</button>`).join('')}` : '';
    el.querySelector('[data-tg]').innerHTML = Object.entries(kind.toggles || {}).map(([k, opts]) =>
      `<button type="button" class="lc-vd-toggle" data-tg="${esc(k)}">${opts.map(o => (o === t[k] ? `<b>${esc(o)}</b>` : esc(o))).join(' · ')}</button>`).join('');
    el.querySelector('[data-apply]').disabled = !plan?.change;
    preview?.(plan?.change ? plan : null);
  };
  // Esc closes it wherever the focus is, before the page's own Esc (on the window: the setup
  // frame's Esc on the document would close the frame instead).
  const onEsc = e => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); } };
  const close = () => {
    if (!el.isConnected) return;
    el.remove();
    current = null;
    window.removeEventListener('keydown', onEsc, true);
    preview?.(null);
  };
  window.addEventListener('keydown', onEsc, true);
  const go = () => {
    if (!plan?.change) return;
    const line = plan.line;
    close();
    apply(line);
  };
  current = close;
  input.addEventListener('input', update);
  input.addEventListener('keydown', e => {
    if (e.key === 'Enter') { e.preventDefault(); go(); }
    else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') { e.preventDefault(); input.value = kind.step(input.value, e.key === 'ArrowUp' ? 1 : -1); update(); }
  });
  el.addEventListener('click', e => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.dataset.cancel != null) close();
    // The bigger tool for the same value (Address: Slot grid… = Edit Patch, the typed value kept).
    else if (b.dataset.more != null) { const v = input.value; close(); more.run(v); }
    else if (b.dataset.apply != null) go();
    else if (b.dataset.step) { input.value = kind.step(input.value, +b.dataset.step); update(); input.focus(); }
    else if (b.dataset.val != null) { input.value = b.dataset.val; update(); input.focus(); }
    else if (b.dataset.tg) {
      const opts = kind.toggles[b.dataset.tg];
      t[b.dataset.tg] = opts[(opts.indexOf(t[b.dataset.tg]) + 1) % opts.length];
      update();
      input.focus();
    }
  });
  // Beside the selection, never over it: below it when there is room, else above; a phone's
  // sheet (decisions §37 7: the kit's one sheet look, from the right on its side) stays put and a
  // swipe down on its head closes it.
  const phone = phoneMode() || (window.matchMedia('(max-width: 600px)').matches ? 'up' : '');
  el.classList.toggle('sheet', !!phone);
  el.classList.toggle('side', phone === 'side');
  if (phone) grabToClose(el.querySelector('[data-head]'), el, close, { side: phone === 'side' });
  else {
    const w = el.offsetWidth, h = el.offsetHeight, vw = window.innerWidth, vh = window.innerHeight;
    const r = avoid || { left: vw / 2 - w / 2, right: vw / 2 + w / 2, top: vh / 2, bottom: vh / 2 };
    const below = r.bottom + 8 + h <= vh;
    const top = below ? r.bottom + 8 : Math.max(8, r.top - 8 - h);
    el.style.left = `${Math.max(8, Math.min(r.left, vw - w - 8))}px`;
    el.style.top = `${Math.max(8, Math.min(top, vh - h - 8))}px`;
    // It grows as the hint and the free values come (a row near the bottom of a big patch):
    // kept inside the viewport whenever its size changes.
    new ResizeObserver(() => {
      const h = el.offsetHeight, w = el.offsetWidth, vw = window.innerWidth, vh = window.innerHeight;
      el.style.left = `${Math.max(8, Math.min(el.offsetLeft, vw - w - 8))}px`;
      el.style.top = `${Math.max(8, Math.min(el.offsetTop, vh - h - 8))}px`;
    }).observe(el);
    let start = null;
    drag(el.querySelector('[data-head]'), {
      filter: e => !e.target.closest('button'),
      onStart: () => { start = { x: el.offsetLeft, y: el.offsetTop }; },
      onMove: ({ dx, dy }) => { el.style.left = `${start.x + dx}px`; el.style.top = `${start.y + dy}px`; },
    });
  }
  update();
  input.focus();
  input.select();
  return close;
}

// ── Free values (the value dialog's chips, Insert fixtures' First FID and Address) ──

/** Up to four blocks of `n` free numbers, as [value, label]: from `from` on first, then empty
 *  hundreds. `taken` is a Set of the numbers in use. */
export function freeNumbers(taken, n, from = 1) {
  const out = [];
  const fits = s => { for (let i = 0; i < n; i++) if (taken.has(s + i)) return false; return true; };
  for (let s = Math.max(1, from); s < 100000 && out.length < 2; s++) if (fits(s)) { out.push(s); s += 99 - (s % 100); }
  for (let h = 100; h < 100000 && out.length < 4; h += 100) {
    if (![...taken].some(x => x > h && x <= h + 100) && fits(h + 1) && !out.includes(h + 1)) out.push(h + 1);
  }
  return out.map(s => [String(s), n > 1 ? `${s}–${s + n - 1}` : `${s}`]);
}

/** Up to four DMX starts that hold `need` slots, as [value, label]: the first free run in each
 *  universe in use (`others`: { u, s, len }; `also`: more universes to try), then the next one. */
export function freeAddresses(others, need, also = []) {
  const universes = [...new Set([...others.map(o => o.u), ...also].filter(Boolean))].sort((a, b) => a - b);
  const out = [];
  for (const u of [...universes, (universes.at(-1) || 0) + 1]) {
    const used = others.filter(o => o.u === u).sort((a, b) => a.s - b.s);
    let s = 1;
    for (const o of used) { if (s + need - 1 < o.s) break; s = Math.max(s, o.s + o.len); }
    if (s + need - 1 <= 512) out.push([`${u}.${a3(s)}`, `${u}.${a3(s)}`]);
    if (out.length >= 4) break;
  }
  return out;
}

// ── Value kinds ──

/** Renumber (`Move <object> … At n`): `rows` the selection in selection order, `grid` the same
 *  in the list's order, `taken` every other number of the pool. Skip taken (`/Skip`) by default;
 *  Swap trades with what is there (Move's own rule). */
export function renumberKind({ object, rows, grid, taken }) {
  const others = new Set(taken);
  const low = Math.min(...rows);
  return {
    placeholder: `first number (now ${low})`,
    empty: `${rows.length} number${rows.length === 1 ? '' : 's'} needed — type the first, or tap a free one`,
    invalid: 'Type a whole number',
    toggles: { order: ['Selection order', 'Grid order'], taken: ['Skip taken', 'Swap'] },
    parse: v => (/^\d+$/.test(String(v).trim()) && +v > 0 ? +v : null),
    step: (v, d) => String(Math.max(1, (parseInt(v, 10) || low) + d)),
    plan(v, t) {
      const order = t.order === 'Grid order' ? grid : rows;
      const to = new Map();
      const hit = [];
      let n = v;
      for (const id of order) {
        if (t.taken === 'Skip taken') while (others.has(n)) n++;
        else if (others.has(n)) hit.push(n);
        to.set(id, n++);
      }
      const vals = [...to.values()].sort((a, b) => a - b);
      const same = [...to].every(([a, b]) => a === b);
      let hint;
      if (same) hint = ['dim', 'Same numbers — nothing to change'];
      else if (hit.length) hint = ['warn', `→ ${inOrder(vals)} · ${hit.join(', ')} swap${hit.length === 1 ? 's' : ''} out of the way`];
      else if (vals[0] !== v) hint = ['warn', `${v}${vals[0] - 1 > v ? `–${vals[0] - 1}` : ''} taken → ${inOrder(vals)}`];
      else if (vals.at(-1) - vals[0] !== vals.length - 1) hint = ['warn', `→ ${inOrder(vals)} · split around taken numbers`];
      else hint = ['ok', `→ ${inOrder(vals)} · all free`];
      return {
        hint, change: !same, to, way: new Set(hit),
        line: `Move ${object} ${inOrder(order)} At ${v}${t.taken === 'Skip taken' ? ' /Skip' : ''}`,
      };
    },
    // Free blocks that fit: after the selection first, then an empty hundred.
    free: () => freeNumbers(others, rows.length, Math.max(...rows) + 1),
  };
}

/** A DMX address (`Assign <object> … At DMX u.aaa`): `rows` the selection in order ({ id, u, s,
 *  len }; u 0 = no address), `others` every other fixture's run. Bunch up (packed, the default for
 *  DMX) or Keep spacing (`/KeepSpacing`). An overlap stays red: both are parked until resolved.
 *  `template`: the line (`{object}`, `{rows}`, `{value}`; Multipatch… stores copies); `offset`:
 *  spare slots after each fixture (the patch's Offset cell) — `/Every <largest footprint + n>`. */
export function addressKind({ object, rows, others, template = 'Assign {object} {rows} At DMX {value}', offset = 0 }) {
  const parse = v => {
    const m = String(v).trim().match(/^(\d+)[.:](\d+)$/);
    return m && +m[1] > 0 && +m[2] >= 1 && +m[2] <= 512 ? { u: +m[1], s: +m[2] } : null;
  };
  const first = rows.find(r => r.u) || null;
  const every = offset > 0 ? Math.max(...rows.map(r => r.len || 1)) + offset : 0;
  const need = every ? every * (rows.length - 1) + (rows.at(-1)?.len || 1) : rows.reduce((n, r) => n + (r.len || 1), 0);
  const spanOf = r => (r.u ? r.s - (first?.s ?? r.s) : 0);
  return {
    start: () => (first ? `${first.u}.${a3(first.s)}` : ''),
    placeholder: 'u.aaa',
    empty: 'Type an address, or tap a free one',
    invalid: 'Universe.address, e.g. 1.150',
    toggles: { layout: ['Bunch up', 'Keep spacing'] },
    parse, step: (v, d) => { const p = parse(v) || { u: 1, s: 1 }; return `${p.u}.${a3(Math.max(1, Math.min(512, p.s + d)))}`; },
    plan(v, t) {
      const to = new Map();
      let at = v.s;
      for (const r of rows) {
        const s = t.layout === 'Keep spacing' && first ? v.s + spanOf(r) : at;
        to.set(r.id, { u: v.u, s });
        at = every ? s + every : s + (r.len || 1);
      }
      const bad = new Set(), way = new Set();
      let past = false;
      for (const r of rows) {
        const m = to.get(r.id);
        if (m.s + (r.len || 1) - 1 > 512) { bad.add(r.id); past = true; }
        for (const o of others) {
          if (o.u === m.u && m.s <= o.s + o.len - 1 && o.s <= m.s + (r.len || 1) - 1) { bad.add(r.id); way.add(o.id); }
        }
      }
      const same = rows.every(r => to.get(r.id).u === r.u && to.get(r.id).s === r.s);
      const last = rows.at(-1), end = to.get(last.id).s + (last.len || 1) - 1;
      const hint = same ? ['dim', 'Same addresses — nothing to change']
        : past ? ['warn', `past 512 — the selection fits in one universe or the line is refused`]
        : bad.size ? ['warn', `overlaps ${inOrder([...way].sort((a, b) => a - b))} — both stay red and parked until resolved`]
        : ['ok', `→ ${v.u}.${a3(to.get(rows[0].id).s)}–${v.u}.${a3(end)} · all free`];
      return {
        hint, change: !same, to, bad, way,
        line: template.replaceAll('{object}', object).replaceAll('{rows}', inOrder(rows.map(r => r.id))).replaceAll('{value}', `${v.u}.${a3(v.s)}`)
          + (t.layout === 'Keep spacing' ? ' /KeepSpacing' : every ? ` /Every ${every}` : ''),
      };
    },
    // The first free run that holds the whole selection, per universe in use (and the next one).
    free: () => freeAddresses(others, need, rows.map(r => r.u)),
  };
}
