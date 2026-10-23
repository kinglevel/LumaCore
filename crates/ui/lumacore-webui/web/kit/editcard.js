// The edit card (decisions §30 "12d hardening": "when i multi select rows in
// the grid and press edit, it should open a dialog so i can see a unified
// card with tabs and <different values>"). One kit dialog for a grid's
// selection: tabs from the columns' sections, one cell per field drawn like
// the <luma-settings> cells (kind badge, the value by kind), each showing the
// shared value or ‹different values› with the range ("25–44 Hz", "Full /
// Changes"). A change sets the field on every selected row that has it as ONE
// line (`Set Output 1 Thru 3 "Rate" 44`; a number may be typed relative `+ 5`
// or spread `21 Thru 30`); rows without the field are skipped and named.
// Per-row fields (Name) are read-only with several rows; fields above the
// viewer's level are greyed with the reason. No OK — × (or Esc) closes. On a
// phone (container < 600 px) the card is full screen, tabs on top, one
// column; the pickers are the kit's own (keypad, pick list, dialog; a `paint` column the
// paint picker, kit/paint.js, whose Please runs `Set Group 1 Thru 3 + 5 "Paint" Blue`).
//
//   openEditCard({ object, columns, rows, focus, level, id, row_key, hidden,
//     open }, { run(line), closed() }) → { update(rows), close(), typed(ch),
//     isOpen() }

import { openKeypad } from './keypad.js';
import { openPick } from './popup.js';
import { openDialog } from './dialog.js';
import { kindOf, kindBadge, homeMark } from './settings.js';
import { needs, rowsWord, valueText, cellText } from './grid.js';
import { toast } from './toast.js';
import { openPaint } from './paint.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const isOn = v => v === true || /^(yes|on|true)$/i.test(String(v));

let current = null;
/** A paint cell's swatch: the row's colour as the station resolved it (decisions 0294). */
const paintSwatch = c => (/^#[0-9a-f]{6}$/i.test(c || '') ? `<i class="lc-set-sw" style="--c:${esc(c)}"></i>` : '<i class="lc-set-sw none"></i>');

/** What the selected rows hold in `col`: the rows that have it, the shared
 *  value or the mixed range. */
export function fieldState(col, rows) {
  const key = col.field ?? col.id;
  const have = rows.filter(r => r[key] !== undefined);
  const vals = have.map(r => r[key]);
  const uniq = [...new Set(vals.map(v => (col.kind === 'bool' ? String(isOn(v)) : String(v ?? ''))))];
  let shown = '—', mixed = false, range = '';
  if (uniq.length === 1) shown = cellText(col, vals[0]);
  else if (uniq.length > 1) {
    mixed = true;
    const nums = vals.map(Number);
    if (col.kind === 'int' && nums.every(n => Number.isFinite(n))) range = `${Math.min(...nums)}–${Math.max(...nums)}${col.unit ? ` ${col.unit}` : ''}`;
    else range = [...new Set(vals.map(v => cellText(col, v)))].join(' / ');
    shown = '‹different values›';
  }
  return { have, mixed, shown, range, value: uniq.length === 1 ? vals[0] : null };
}

export function openEditCard(spec, { run, closed } = {}) {
  current?.close();
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim';
  scrim.dataset.kit = 'edit-card';
  document.body.append(scrim);
  let rows = spec.rows || [];
  let last = '';
  const editable = (spec.columns || []).filter(c => (c.prop || c.per_row) && c.id !== 'no');
  const sections = [];
  for (const c of editable) if (!sections.includes(c.section || 'General')) sections.push(c.section || 'General');
  let tab = Math.max(0, sections.indexOf((editable.find(c => c.id === spec.focus)?.section) || sections[0]));
  let focus = spec.focus;
  const objectOf = r => r.object || spec.object;
  const idOf = r => r[spec.id || 'no'];
  const title = () => {
    const words = [...new Set(rows.map(objectOf))];
    const word = words.length === 1 ? words[0] : 'Object';
    return rows.length === 1 ? `${word} ${idOf(rows[0])}` : `${rows.length} ${word}s`;
  };

  /** Why `col` can't change here ('' = it can). */
  const why = col => {
    if (col.per_row && rows.length > 1) return `${col.label} is per row — select one row to change it`;
    if (!col.prop) return `${col.label} is read-only`;
    return needs(col.rights, spec.level);
  };

  const draw = () => {
    const shown = editable.filter(c => (c.section || 'General') === sections[tab]);
    const subtitle = `${rows.length > 1 ? 'Edit · ' : ''}${[...new Set(rows.map(objectOf))].map(o => `${o} ${rowsWord(rows.filter(r => objectOf(r) === o).map(idOf))}`).join(' + ')}${spec.hidden ? ` (${spec.hidden} hidden)` : ''}`;
    scrim.innerHTML = `<div class="lc-card" role="dialog" aria-label="${esc(title())}">
      <header class="lc-set-title lc-card-title"><luma-icon name="pencil"></luma-icon>
        <span class="lc-set-name">${esc(title())}${rows.length > 1 || spec.hidden ? ` <small class="lc-set-sub">${esc(subtitle)}</small>` : ''}</span>
        <button type="button" class="lc-set-x" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
      <div class="lc-set-body lc-card-body">
        ${sections.length > 1 ? `<nav class="lc-set-tabs">${sections.map((n, i) => `<button type="button" data-tab="${i}" class="${i === tab ? 'on' : ''}">${esc(n)}</button>`).join('')}</nav>` : ''}
        <div class="lc-set-main">
          <div class="lc-set-grid">${shown.map((c, i) => {
            const st = fieldState(c, rows);
            const w = why(c);
            const kind = kindOf(c.kind);
            const skipped = rows.length - st.have.length;
            return `<div class="lc-set-cell k-${kind}${w ? ' ro' : ''}${c.id === focus ? ' focus' : ''}${st.mixed ? ' mixed' : ''}" data-field="${esc(c.id)}" data-kind="${kind}">
              <button type="button" class="lc-set-tap" data-i="${i}" title="${esc(w)}">
                <span class="lc-set-label">${kindBadge(kind, !!w)}<span class="lc-set-fname">${esc(c.label)}</span>${homeMark(c.home, spec.home)}${w && c.prop ? '<luma-icon name="lock" class="lc-set-lock"></luma-icon>' : ''}</span>
                <span class="lc-set-row">${kind === 'paint' && !st.mixed ? paintSwatch(st.have[0]?.paint) : ''}${kind === 'toggle' && !st.mixed ? `<i class="lc-switch${isOn(st.value) ? ' on' : ''}" aria-hidden="true"></i>` : ''}<span class="lc-set-value${st.mixed ? ' lc-mixed' : ''}">${esc(st.shown)}</span>${st.mixed ? `<small class="lc-set-range">${esc(st.range)}</small>` : ''}${kind === 'list' && !w ? '<luma-icon name="chevron-down" class="lc-set-drop"></luma-icon>' : ''}</span>
              </button>
              ${skipped && st.have.length ? `<small class="lc-card-skip">${st.have.length} of ${rows.length} have it</small>` : ''}
            </div>`;
          }).join('') || '<div class="lc-set-note">Nothing to change here.</div>'}</div>
          <div class="lc-set-last lc-selectable" data-last>${last ? `› ${esc(last)}` : 'one change = one line for every selected row'}</div>
        </div>
      </div>
    </div>`;
    scrim.querySelector('[data-x]').onclick = () => close();
    for (const b of scrim.querySelectorAll('[data-tab]')) b.onclick = () => { tab = +b.dataset.tab; draw(); };
    for (const b of scrim.querySelectorAll('.lc-set-tap')) b.onclick = () => change(shown[+b.dataset.i]);
  };

  /** Ask for the new value by kind; null = cancelled. */
  const ask = async (col, st, typed) => {
    const name = `${col.label} · ${rows.length} ${rows.length === 1 ? 'row' : 'rows'}`;
    if (col.kind === 'bool') {
      const [on, off] = col.options?.length === 2 ? col.options : ['Yes', 'No'];
      return st.mixed || !isOn(st.value) ? on : off;
    }
    if (col.kind === 'list') {
      const items = (col.options || []).map((o, i) => ({ no: i + 1, name: o }));
      const got = await openPick({ title: name, value: st.mixed ? '' : cellText(col, st.value), pick: { kind: 'Choice', items, plain: true, note: st.mixed ? `now: ${st.range}` : ' ' } });
      return got?.no ? col.options[got.no - 1] : null;
    }
    if (col.kind === 'int') {
      const got = await openKeypad({ title: name, value: typed ?? (st.mixed ? '' : String(st.value ?? '')), min: col.min, max: col.max, unit: col.unit });
      let text = got?.text?.trim() || '';
      // `25 + 5` on the old value is `+ 5` for every row (the station works
      // relative input out per row).
      const old = st.mixed ? null : String(st.value ?? '');
      const rel = old && text.startsWith(old) ? /^\s*([+\-*/])\s*(\S.*)$/.exec(text.slice(old.length)) : null;
      if (rel) text = `${rel[1]} ${rel[2]}`;
      return text || null;
    }
    const got = await openDialog({
      title: name, text: st.mixed ? `Now: ${st.range}` : '',
      inputs: [{ name: 'v', label: col.label, value: typed ?? (st.mixed ? '' : String(st.value ?? '')), kind: 'text' }],
      buttons: [{ label: 'Please', value: 'set', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
    });
    return got?.value ? String(got.inputs?.v ?? '') : null;
  };

  const change = async (col, typed) => {
    focus = col.id;
    const w = why(col);
    if (w) { toast(w); draw(); return; }
    const st = fieldState(col, rows);
    if (!st.have.length) { toast(`None of these rows has ${col.label}`); return; }
    const words = [...new Set(st.have.map(objectOf))];
    // The property is quoted (§33 58); a keyed path (`Network "Rig" "Interface"`) comes quoted already.
    const prop = /^[A-Za-z0-9_]+$/.test(col.prop) ? `"${col.prop}"` : col.prop;
    const skipped = rows.filter(r => !st.have.includes(r));
    const skipNote = skipped.length ? `   (skipped ${skipped.map(r => `${objectOf(r)} ${idOf(r)}`).join(', ')} — no ${col.label})` : '';
    if (col.kind === 'paint') {
      if (!scrim.isConnected) return; // closed before its editor opened (a host took the edit)
      // A Paint (decisions 0294): the kit's picker; its Please runs the one line for every row.
      const targets = words.map(o => `${o} ${rowsWord(st.have.filter(r => objectOf(r) === o).map(idOf))}`);
      const template = targets.map(t => `Set ${t} ${prop} {value}`).join('; ');
      const one = st.have.length === 1 ? st.have[0] : null;
      const word = await openPaint({ object: targets.join(' + '), name: one?.name || '', no: one ? idOf(one) : '', line: template, value: st.mixed ? '' : String(st.value ?? '') },
        { run: l => run?.(l) });
      if (word == null || !current) return;
      last = `${template.split('{value}').join(word)}${skipNote}`;
      draw();
      return;
    }
    const v = await ask(col, st, typed);
    if (v == null || !current) return;
    // One line for every row that has the field (grouped by object word when
    // a grid mixes them); the others are named.
    const line = words.map(o => `Set ${o} ${rowsWord(st.have.filter(r => objectOf(r) === o).map(idOf))} ${prop} ${valueText(col, v)}`).join('; ');
    last = `${line}${skipNote}`;
    run?.(line);
    draw();
  };

  const onKey = e => {
    if (e.key === 'Escape' && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, [data-kit="dialog"], [data-kit="paint-picker"]')) { e.preventDefault(); e.stopPropagation(); close(); }
  };
  document.addEventListener('keydown', onKey, true);
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });

  const close = () => {
    if (!scrim.isConnected) return;
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    if (current?.scrim === scrim) current = null;
    closed?.();
  };
  draw();
  current = {
    scrim, close,
    isOpen: () => scrim.isConnected,
    update(next) { rows = next.filter(Boolean); if (!rows.length) return close(); draw(); },
    typed(ch) { const col = editable.find(c => c.id === focus); if (col) change(col, ch); },
  };
  if (spec.open && focus) {
    const col = editable.find(c => c.id === focus);
    // A toggle never flips by itself: it only takes the focus.
    if (col && !why(col) && col.kind !== 'bool') setTimeout(() => change(col), 0);
  }
  return current;
}

export function editCardOpen() { return !!current; }
