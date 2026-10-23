// The one standard dialog (decisions §30, the Dialogs board; 12b: every
// question the station asks). MessageBox shape: a title row (a note or the
// countdown on the right), text, facts, a details list, inputs (text,
// password, number → keypad, link → pick list), toggles, selectors, a row of
// 60 px buttons, a note. The title row keeps a 50 px × (its Cancel) and the
// body scrolls inside a box never larger than the screen (12c2).
//
// Two uses:
//   openDialog(spec) → Promise   — a question the page asks itself (the
//     connect page, a Delete): resolves the button's `value` (null = Esc /
//     scrim), or { value, inputs, toggles, selectors } when it has fields.
//   showDialog(spec, { onAnswer(button, answers), onPause() }) → { update,
//     close } — a dialog the station holds (it sends one in the Status): the
//     station owns the countdown (`countdown: {deadline_ms, total_s, default,
//     paused}`), `update` refreshes it in place keeping what was typed, a
//     touch pauses it (onPause), a button answers (onAnswer). Every answer is
//     still a command line, built by the station.
//
// spec: { title, meta, meta_tone, text, rows: [{k, v, tone}], details: [{k,
//   tone}], inputs: [{name, label, value, kind: {kind: 'text'|'password'|
//   'number'|'link', min, max, unit, pick}, chips}], toggles: [{name, label,
//   on, flag}], selectors: [{name, label, choices, value, texts}], buttons: [{label,
//   value, kind: 'primary'|'danger'|'plain'|'please', disabled}], primary,
//   cancel, note, countdown, tree }
//
// A tree to tick (decisions 0293, the Update ask): `tree: { nodes: [{ id, depth, kind, no, name,
// badge, note, note_off, on, tickable, tone, when_on, when_off }] }`, drawn by <luma-tree> in pick
// mode with guides; a tick is answered like a toggle (by the node's id). The details list is the
// lines the answer will run: redrawn as you tick from the nodes' statements (`treeLines`, the
// station's `DialogTree::statements` and its line filling) — the station builds the real line.

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const kindOf = input => (typeof input.kind === 'string' ? input.kind : input.kind?.kind) || 'text';

// Dialogs stack: a newer one covers the older (a page's Delete over a
// station's ask); closing it shows the older again.
const stack = [];
let open = null;

function closeEntry(entry, value) {
  const at = stack.indexOf(entry);
  if (at < 0) return;
  stack.splice(at, 1);
  clearInterval(entry.timer);
  document.removeEventListener('keydown', entry.onKey, true);
  entry.host.remove();
  open = stack[stack.length - 1] || null;
  if (open) open.host.hidden = false;
  entry.resolve?.(value);
}

/** Close the top dialog with `value` (a local dialog's answer). */
export function closeDialog(value = null) {
  if (open) closeEntry(open, value);
}

export function dialogOpen() { return !!open; }

/** The page's own question; resolves when answered. */
export function openDialog(spec) {
  return new Promise(resolve => {
    const fields = !!(spec.inputs?.length || spec.toggles?.length || spec.selectors?.length);
    const answer = i => {
      const b = (spec.buttons || [])[i];
      const value = b ? b.value : null;
      closeEntry(entry, fields && b ? { value, ...collect() } : value);
    };
    const collect = mount(spec, { resolve, onAnswer: answer, local: true });
    const entry = open;
  });
}

/** A dialog the station holds (see the header). */
export function showDialog(spec, handlers) {
  const collect = mount(spec, { onAnswer: (i) => handlers.onAnswer?.(i, collect()), onPause: handlers.onPause, local: false });
  const mine = open;
  return {
    update(next) { if (stack.includes(mine)) mine.refresh(next); },
    close() { closeEntry(mine, null); },
    get open() { return stack.includes(mine); },
  };
}

function mount(spec, { resolve, onAnswer, onPause, local }) {
  if (open) open.host.hidden = true;
  const host = document.createElement('div');
  host.className = 'lc-dlg-scrim';
  host.dataset.kit = 'dialog';
  if (spec.id) host.dataset.id = spec.id;
  host.innerHTML = `<section class="lc-dlg" role="dialog" aria-modal="true"></section>`;
  const card = host.firstElementChild;
  const state = { spec, values: {}, toggles: {}, selectors: {}, ticks: {}, paused: false, keyboard: null };
  let me = null;
  const self = () => me;

  const collect = () => ({
    inputs: Object.fromEntries((state.spec.inputs || []).map(i => [i.name, state.values[i.name] ?? i.value ?? ''])),
    toggles: Object.fromEntries([
      ...(state.spec.toggles || []).map(t => [t.name, state.toggles[t.name] ?? !!t.on]),
      ...(state.spec.tree?.nodes || []).filter(n => n.tickable).map(n => [n.id, state.ticks[n.id] ?? !!n.on]),
    ]),
    selectors: Object.fromEntries((state.spec.selectors || []).map(s => [s.name, state.selectors[s.name] ?? s.value ?? s.choices?.[0] ?? ''])),
  });

  const render = () => {
    const s = state.spec;
    const buttons = s.buttons?.length ? s.buttons : [{ label: 'OK', value: true, kind: 'primary' }];
    card.setAttribute('aria-label', s.title || '');
    card.classList.toggle('files', (s.inputs || []).some(i => kindOf(i) === 'file'));
    // A File input's files element lives across the station's updates (its drive, folder and
    // selection are the viewer's; the station re-sends the dialog as it waits).
    const keptFiles = new Map([...card.querySelectorAll('.lc-dlg-files')].map(el => [el.dataset.field, el]));
    card.innerHTML = `
      <header class="lc-dlg-title"><span>${esc(s.title)}</span><span class="lc-dlg-meta"></span>${s.cancel != null || local ? '<button type="button" class="lc-close" data-kit="close" aria-label="Close"><luma-icon name="x"></luma-icon></button>' : ''}</header>
      <div class="lc-dlg-body">
        ${s.text ? `<p class="lc-dlg-text lc-selectable">${esc(s.text)}</p>` : ''}
        ${s.countdown ? '<div class="lc-dlg-bar"><i></i></div>' : ''}
        ${s.rows?.length ? `<div class="lc-dlg-rows">${s.rows.map(r => `<div><span>${esc(r.k)}</span><b class="${r.tone ? `tone-${esc(r.tone)}` : ''}">${esc(r.v)}</b></div>`).join('')}</div>` : ''}
        ${s.tree ? '<div class="lc-dlg-tree"><luma-tree></luma-tree></div>' : ''}
        ${s.details?.length && !s.tree ? detailsHtml(s.details) : ''}
        ${(s.inputs || []).map((inp, n) => fieldHtml(inp, n)).join('')}
        ${s.tree ? selectorsHtml(s) : ''}
        ${(s.toggles || []).map((t, n) => `<div class="lc-dlg-toggle"><span>${esc(t.label)}</span><button type="button" role="switch" data-toggle="${n}" aria-label="${esc(t.label)}"><i></i></button></div>`).join('')}
        ${s.tree ? '' : selectorsHtml(s)}
        ${s.tree ? `<ul class="lc-dlg-details lc-selectable" data-kit="lines"></ul>` : ''}
        <div class="lc-dlg-buttons" style="--lc-dlg-cols:${buttons.length}"></div>
        <span class="lc-dlg-note"></span>
      </div>`;
    for (const [n, el] of keptFiles) card.querySelector(`.lc-dlg-files[data-field="${n}"]`)?.replaceWith(el);
    // The × (12c2 overlay contract): the dialog's Cancel, or a page dialog's Esc.
    card.querySelector('.lc-close')?.addEventListener('click', () => {
      if (s.cancel != null) onAnswer(s.cancel); else closeEntry(self(), null);
    });
    const row = card.querySelector('.lc-dlg-buttons');
    buttons.forEach((b, i) => {
      const el = document.createElement('button');
      el.type = 'button';
      el.className = `lc-dlg-btn ${b.kind || 'plain'}`;
      el.textContent = b.label;
      el.dataset.index = i;
      el.disabled = !!b.disabled;
      if (typeof b.disabled === 'string') el.title = b.disabled;
      if (i === (s.primary ?? -1)) el.dataset.primary = '1';
      el.addEventListener('click', () => onAnswer(i));
      row.append(el);
    });
    const reasons = buttons.map(b => (typeof b.disabled === 'string' ? b.disabled : '')).filter(Boolean);
    const note = [s.note, ...reasons].filter(Boolean).join(' · ');
    card.querySelector('.lc-dlg-note').textContent = note;
    card.querySelector('.lc-dlg-note').hidden = !note;
    // Fields keep what was typed across updates.
    (s.inputs || []).forEach((inp, n) => wireField(inp, n));
    card.querySelectorAll('[data-toggle]').forEach(b => {
      const t = s.toggles[+b.dataset.toggle];
      const on = () => state.toggles[t.name] ?? !!t.on;
      const paint = () => b.setAttribute('aria-checked', String(on()));
      b.addEventListener('click', () => { state.toggles[t.name] = !on(); paint(); paintTree(); });
      paint();
    });
    card.querySelectorAll('[data-sel]').forEach(b => {
      const sel = s.selectors[+b.dataset.sel];
      const paint = () => card.querySelectorAll(`[data-sel="${b.dataset.sel}"]`).forEach(x =>
        x.classList.toggle('on', x.dataset.choice === (state.selectors[sel.name] ?? sel.value)));
      b.addEventListener('click', () => { state.selectors[sel.name] = b.dataset.choice; paint(); paintTree(); });
      paint();
    });
    const tree = card.querySelector('.lc-dlg-tree luma-tree');
    if (tree) tree.addEventListener('toggle', e => {
      const n = treePaths(s.tree.nodes).byPath.get(e.detail.path);
      if (!n || !n.tickable) return;
      state.ticks[n.id] = !(state.ticks[n.id] ?? !!n.on);
      paintTree();
    });
    paintTree();
    tick();
  };

  const detailsHtml = details => `<ul class="lc-dlg-details lc-selectable">${details.map(r => `<li class="${r.tone ? `tone-${esc(r.tone)}` : ''}"><i></i>${esc(r.k)}</li>`).join('')}</ul>`;
  const selectorsHtml = s => (s.selectors || []).map((sel, n) => `<div class="lc-dlg-select"><span>${esc(sel.label)}</span><span class="lc-dlg-seg">${(sel.choices || []).map(c => `<button type="button" data-sel="${n}" data-choice="${esc(c)}">${esc(c)}</button>`).join('')}</span></div>`).join('');

  // The tree's rows with the ticks as they stand, and the lines the answer will run.
  const paintTree = () => {
    const s = state.spec;
    if (!s.tree) return;
    const ticked = n => state.ticks[n.id] ?? !!n.on;
    const { paths } = treePaths(s.tree.nodes);
    card.querySelector('.lc-dlg-tree luma-tree')?.update({
      mode: 'pick', guides: true, empty: 'Nothing to update.',
      nodes: s.tree.nodes.map((n, i) => ({
        path: paths[i], no: n.no || '', name: n.name, id: n.badge || '', tickable: !!n.tickable, tone: n.tone || '',
        state: n.tickable ? (ticked(n) ? 'on' : 'off') : 'off',
        count: (n.tickable && !ticked(n) && n.note_off) ? n.note_off : (n.note || null),
      })),
    });
    const answers = collect();
    const lines = treeLines(s, answers);
    const list = card.querySelector('[data-kit="lines"]');
    if (list) list.innerHTML = lines.length ? lines.map(l => `<li class="tone-ok"><i></i>${esc(l)}</li>`).join('') : '<li><i></i>Nothing ticked</li>';
    const primary = card.querySelector('.lc-dlg-btn[data-primary]');
    if (primary) {
      const b = (s.buttons || [])[s.primary ?? 0];
      primary.disabled = !!b?.disabled || !lines.length;
      primary.textContent = lines.length ? `${b?.label || 'OK'} (${lines.length} line${lines.length === 1 ? '' : 's'})` : (b?.label || 'OK');
    }
  };

  const fieldHtml = (inp, n) => {
    const kind = kindOf(inp);
    const unit = inp.kind?.unit ? `<em>${esc(inp.kind.unit)}</em>` : '';
    const control = kind === 'file'
      ? `<div class="lc-dlg-files" data-field="${n}"><luma-files></luma-files></div>`
      : kind === 'link'
      ? `<button type="button" class="lc-dlg-input lc-dlg-link" data-field="${n}"><span></span><luma-icon name="chevron-down"></luma-icon></button>`
      : `<span class="lc-dlg-input"><input data-field="${n}" type="${kind === 'password' ? 'password' : 'text'}" autocomplete="off" spellcheck="false"
           ${kind === 'number' ? 'inputmode="decimal"' : ''} enterkeyhint="done">${unit}
           <button type="button" class="lc-dlg-kb" data-kb="${n}" aria-label="${kind === 'number' ? 'Keypad' : 'Keyboard'}"><luma-icon name="${kind === 'number' ? 'grid-3x3' : 'keyboard'}"></luma-icon></button></span>`;
    const chips = inp.chips?.length ? `<span class="lc-dlg-chips">${inp.chips.map(c => `<button type="button" data-chip="${n}">${esc(c)}</button>`).join('')}</span>` : '';
    // A File field is no <label>: a label activates its first button (WebKit: on any click
    // inside), which would be the files element's first drive.
    if (kind === 'file') return `<div class="lc-dlg-field file"><span>${esc(inp.label)}</span>${control}</div>`;
    return `<label class="lc-dlg-field"><span>${esc(inp.label)}</span>${control}${chips}</label>`;
  };

  const wireField = (inp, n) => {
    const kind = kindOf(inp);
    const value = () => state.values[inp.name] ?? inp.value ?? '';
    if (kind === 'file') {
      // The station's "which file?" ask (decisions §38 5): the files element on the kind's folder,
      // the drives on its rail; the value is `<drive>:<file>` (the file relative to that folder),
      // which the station renders as `"file" /Drive "usb1"`. A double-tap is the primary button.
      const box = card.querySelector(`.lc-dlg-files[data-field="${n}"]`);
      if (box.__wired) return;
      box.__wired = true;
      const lf = box.querySelector('luma-files');
      const kindOfFiles = inp.kind?.files || 'shows';
      lf.update({ feeds: window.feeds?.() || null, kind: kindOfFiles, level: state.spec.level || '' });
      const fileOf = (d, row) => {
        if (!row || row.dir) return '';
        const base = lf.folderOf?.(kindOfFiles) ?? kindOfFiles;
        const rel = d.folder && d.folder !== base && d.folder.startsWith(`${base}/`) ? `${d.folder.slice(base.length + 1)}/${row.file}` : row.file;
        return `${d.drive}:${rel}`;
      };
      lf.addEventListener('select', e => { const d = e.detail; state.values[inp.name] = d.rows?.length === 1 ? fileOf(d, d.rows[0]) : ''; });
      lf.addEventListener('pick', e => {
        const d = e.detail;
        state.values[inp.name] = fileOf(d, d.row);
        if (state.values[inp.name]) card.querySelector('.lc-dlg-btn[data-primary]')?.click();
      });
      return;
    }
    if (kind === 'link') {
      const b = card.querySelector(`.lc-dlg-link[data-field="${n}"]`);
      const pick = inp.kind.pick || { items: [] };
      const name = () => pick.items.find(i => String(i.no) === String(value()) || i.name === value())?.name ?? (value() || 'None');
      b.querySelector('span').textContent = name();
      b.addEventListener('click', async () => {
        const got = await window.LumaKit.popup.openPick({ title: inp.label, pick: { ...pick, add: null }, value: value() });
        if (!got) return;
        state.values[inp.name] = got.none ? 'none' : String(got.no);
        b.querySelector('span').textContent = name();
      });
      return;
    }
    const field = card.querySelector(`input[data-field="${n}"]`);
    field.value = value();
    field.addEventListener('input', () => { state.values[inp.name] = field.value; });
    card.querySelector(`[data-kb="${n}"]`).addEventListener('click', async () => {
      if (kind === 'number') {
        const got = await window.LumaKit.keypad.openKeypad({ title: inp.label, value: field.value, min: inp.kind.min, max: inp.kind.max, unit: inp.kind.unit });
        if (got != null) { field.value = got.text; state.values[inp.name] = got.text; }
        reopenFocus();
      } else {
        toggleKeyboard(field, kind === 'password');
      }
    });
    if (kind === 'number' && matchMedia('(any-pointer: coarse)').matches) {
      field.readOnly = true;
      field.addEventListener('click', () => card.querySelector(`[data-kb="${n}"]`).click());
    }
    card.querySelectorAll(`[data-chip="${n}"]`).forEach(c => c.addEventListener('click', () => {
      field.value = c.textContent; state.values[inp.name] = field.value; field.focus();
    }));
    if (kind !== 'number' && window.LumaKit?.keyboard?.useBuiltIn?.()) {
      field.inputMode = 'none';
      if (n === 0) queueMicrotask(() => toggleKeyboard(field, kind === 'password', true));
    }
  };

  // The built-in keyboard under the card (per the screen's Keyboard setting).
  const toggleKeyboard = (field, password, onlyOpen = false) => {
    if (state.keyboard) {
      if (onlyOpen) return;
      state.keyboard.remove(); state.keyboard = null; field.inputMode = ''; keyboardRoom(); field.focus(); return;
    }
    const kb = document.createElement('luma-keyboard');
    kb.setAttribute('mode', password ? 'password' : 'text');
    kb.className = 'lc-dlg-keyboard';
    host.append(kb);
    // The card stays above the keyboard (12c2 overlay contract): its body scrolls.
    new ResizeObserver(keyboardRoom).observe(kb);
    kb.target = field;
    kb.addEventListener('please', () => onAnswer(state.spec.primary ?? 0));
    kb.addEventListener('escape', () => { const c = state.spec.cancel; if (c != null) onAnswer(c); else if (local) closeEntry(self(), null); });
    kb.addEventListener('hide', () => { kb.remove(); state.keyboard = null; keyboardRoom(); });
    state.keyboard = kb;
    keyboardRoom();
  };
  const keyboardRoom = () => {
    const h = state.keyboard?.isConnected ? state.keyboard.offsetHeight : 0;
    host.style.paddingBottom = h ? `${h + 8}px` : '';
    host.style.alignItems = h ? 'flex-end' : '';
  };
  const reopenFocus = () => card.querySelector('input[data-field]')?.focus();

  // The countdown: the station's deadline, or a local one in seconds.
  const localEnd = local && spec.countdown?.seconds ? Date.now() + spec.countdown.seconds * 1000 : 0;
  const tick = () => {
    const s = state.spec, meta = card.querySelector('.lc-dlg-meta');
    if (!meta) return;
    meta.className = `lc-dlg-meta${s.meta_tone ? ` tone-${s.meta_tone}` : (s.metaWarn ? ' tone-warn' : '')}`;
    meta.textContent = s.meta || '';
    const c = s.countdown, bar = card.querySelector('.lc-dlg-bar i');
    // A station's question without a countdown waits (the 12b Asks board).
    if (!c && !s.meta && !local) meta.textContent = 'waits for you';
    if (!c) return;
    if (c.paused || state.paused) {
      meta.textContent = 'paused'; meta.className = 'lc-dlg-meta';
      if (bar) bar.parentElement.hidden = true;
      return;
    }
    const end = local ? localEnd : c.deadline_ms;
    const total = local ? c.seconds * 1000 : Math.max(1, (c.total_s || 0) * 1000 || end - (state.started ??= Date.now()));
    const left = Math.max(0, end - Date.now());
    const what = (s.buttons || [])[local ? -1 : c.default]?.label;
    meta.textContent = `${what ? `${what} in ` : ''}${Math.ceil(left / 1000)} s`;
    meta.className = 'lc-dlg-meta tone-warn';
    if (bar) bar.style.width = `${Math.min(1, left / total) * 100}%`;
    if (local && left <= 0) closeEntry(self(), spec.countdown.value);
  };

  // A touch on the dialog stops a running countdown.
  card.addEventListener('pointerdown', () => {
    const c = state.spec.countdown;
    if (!c || c.paused || state.paused) return;
    state.paused = true;
    tick();
    onPause?.();
  });
  if (local) host.addEventListener('pointerdown', e => { if (e.target === host) closeEntry(self(), null); });
  const onKey = e => {
    if (open?.host !== host) return;
    if (e.target?.closest?.('luma-keyboard, luma-keypad, luma-popup')) return;
    if (e.key === 'Escape') {
      e.preventDefault(); e.stopPropagation();
      const c = state.spec.cancel;
      if (c != null) onAnswer(c); else if (local) closeEntry(self(), null);
    } else if (e.key === 'Enter' && !e.isComposing) {
      const target = e.target?.closest?.('.lc-dlg-btn');
      if (target) return; // the focused button clicks itself
      e.preventDefault(); e.stopPropagation();
      const p = state.spec.primary ?? 0;
      if (!(state.spec.buttons || [])[p]?.disabled) onAnswer(p);
    }
  };
  document.addEventListener('keydown', onKey, true);
  document.body.append(host);
  open = {
    host, resolve, onKey,
    timer: setInterval(tick, 200),
    refresh(next) {
      // Keep the caret where it was while the station updates the dialog.
      const active = document.activeElement;
      const field = active?.dataset?.field;
      const at = [active?.selectionStart, active?.selectionEnd];
      state.spec = next;
      render();
      if (field != null) {
        const again = card.querySelector(`input[data-field="${field}"]`);
        again?.focus();
        if (again && at[0] != null) again.setSelectionRange(at[0], at[1]);
      }
    },
  };
  me = open;
  stack.push(open);
  render();
  (card.querySelector('input[data-field]') || card.querySelector('.lc-dlg-btn[data-primary="1"]:not(:disabled)') || card.querySelector('.lc-dlg-btn:not(:disabled)'))?.focus();
  return collect;
}

/** A tree's nodes as <luma-tree> paths (`1`, `1.2` …, from each node's depth). */
function treePaths(nodes) {
  const paths = [], byPath = new Map(), counters = [];
  for (const n of nodes) {
    const d = Math.min(n.depth || 0, counters.length);
    counters.length = d + 1;
    counters[d] = (counters[d] || 0) + 1;
    const path = counters.join('.');
    paths.push(path);
    byPath.set(path, n);
  }
  return { paths, byPath };
}

/** The lines a tree dialog's answer runs (decisions 0293): its nodes' statements lane by lane,
 *  `{?name}` / `{?name!=text}` guards worked out, then the selectors' texts and the toggles'
 *  flags filled in — the station's `DialogTree::statements` and `Ask::line_for`, for the preview. */
export function treeLines(spec, answers) {
  const nodes = spec.tree?.nodes || [];
  const sels = spec.selectors || [];
  const text = name => {
    const s = sels.find(x => x.name === name);
    if (!s) return null;
    const v = answers.selectors?.[name] ?? s.value;
    return (s.texts || [])[(s.choices || []).indexOf(v)] ?? null;
  };
  const on = id => {
    if (id.includes('!=')) { const [name, not] = id.split('!='); const t = text(name); return t != null && t !== not; }
    if (answers.toggles && id in answers.toggles) return !!answers.toggles[id];
    return !!nodes.find(n => n.id === id)?.on;
  };
  const picked = [];
  nodes.forEach((n, i) => {
    for (const st of (!n.tickable || on(n.id)) ? (n.when_on || []) : (n.when_off || [])) {
      let t = st.text, keep = true;
      for (let m; (m = /^\{\?([^}]*)\}/.exec(t));) { keep = keep && on(m[1]); t = t.slice(m[0].length); }
      if (keep && t.trim()) picked.push([st.lane || 0, i, t.trim()]);
    }
  });
  picked.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  return picked.map(([, , t]) => {
    for (const s of sels) t = t.split(`{${s.name}}`).join(text(s.name) ?? '');
    for (const tg of spec.toggles || []) t = t.split(`{${tg.name}}`).join((answers.toggles?.[tg.name] ?? tg.on) ? (tg.flag || '') : '');
    return t.split(/\s+/).filter(Boolean).join(' ');
  });
}
