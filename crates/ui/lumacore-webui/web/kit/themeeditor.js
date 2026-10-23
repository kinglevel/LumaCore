// The theme editor (decisions theme-on-profile; archive §30 "12g"; the approved ThemeBg board,
// 2026-10-12-12g-test, option A "Tint replaced"). A profile's Theme as one dialog:
//
//   header    the base (Dark · Daylight · Night), A | B (the theme as opened ↔ now), Import…,
//             Export… (JSON), Reset all
//   Adjust    Brightness, Contrast, Saturation, Background hue, Background sat., Accent hue —
//             a drag previews on the whole page, the release is one line (one Oops step); ↺ resets;
//             the contrast readout warns when text or text-dim gets too low
//   Colours   the 14 palette colours: swatch, hex, ↺ when changed; a tap edits (a colour pick)
//   Pick      tap an element on screen to find which colour it uses
//   Preview   the kit's look in the colours shown; Hold: normal theme shows the factory preset
//   Paint     (a tab, decisions 0294) the theme's paint slots on the `paint` feed as a <luma-grid>:
//             each row tinted with its colour, Your colour opens the paint picker (the slot's
//             own line), Add a slot (`Store … Theme Slot "<name>"`), Reset to LumaCore
//
// Every change is a line from the station's own fields (`Set [UserProfile n] Theme "Brightness"
// 20`, `… Theme Slot "Accent" "#9a7a08"`, `Reset … Theme [Slot]`); the station answers with
// fresh fields (`ctl.update(fields)`).
//
//   const ctl = openThemeEditor({ title, fields }, { run(line), preview(look | 'normal' | null) })
//   ctl.update(fields) · ctl.close()

import { token } from './theme.js';
import { openPaint } from './paint.js';
import { openDialog } from './dialog.js';
import './grid.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const FILE_BUTTONS = [
  // tidy: slab-ok (the editor's own header: one theme to and from a JSON file, not a list of objects)
  '<button type="button" class="lc-te-btn" data-import>Import…</button>',
  // tidy: slab-ok (the editor's own header: one theme to and from a JSON file, not a list of objects)
  '<button type="button" class="lc-te-btn" data-export>Export…</button>',
].join('');

/** The adjusters: (field name, label, the theme key, min, max, unit). */
export const ADJUST = [
  ['Brightness', 'Brightness', 'brightness', -100, 100, ''],
  ['Contrast', 'Contrast', 'contrast', -100, 100, ''],
  ['Saturation', 'Saturation', 'saturation', -100, 100, ''],
  ['BackgroundHue', 'Background hue', 'background_hue', 0, 360, '°'],
  ['BackgroundSaturation', 'Background sat.', 'background_saturation', 0, 100, ''],
  ['AccentHue', 'Accent hue', 'accent_hue', -180, 180, '°'],
];
/** The 14 colours: (field name, label, the palette key, the token). */
export const COLOURS = [
  ['Background', 'Background', 'background', 'bg'], ['Surface', 'Surface', 'surface', 'surface'],
  ['Raised', 'Raised', 'raised', 'raised'], ['Line', 'Line', 'line', 'line'],
  ['Text', 'Text', 'text', 'text'], ['TextDim', 'Text dim', 'text_dim', 'text-dim'],
  ['Accent', 'Accent', 'accent', 'accent'], ['Info', 'Info', 'info', 'info'],
  ['Ok', 'OK', 'ok', 'ok'], ['Warn', 'Warn', 'warn', 'warn'],
  ['Alert', 'Alert', 'alert', 'alert'], ['Programmer', 'Programmer', 'programmer', 'programmer'],
  ['Blind', 'Blind', 'blind', 'blind'], ['Park', 'Park', 'park', 'park'],
];
const BASES = ['Dark', 'Daylight', 'Night'];
/** The editor's tabs: the look (base, adjusters, colours) and the paint slots (decisions 0294). */
const TABS = ['Look', 'Paint'];

/** WCAG contrast of two `rgb(…)` / `#rrggbb` colours. */
export function contrast(a, b) {
  const lum = c => {
    const m = String(c).match(/\d+(\.\d+)?/g)?.map(Number) || [];
    const [r, g, bl] = String(c).startsWith('#') ? [1, 3, 5].map(i => parseInt(String(c).slice(i, i + 2), 16)) : m;
    return [r, g, bl].map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; })
      .reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
  };
  const x = lum(a), y = lum(b);
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

/** The look the fields describe (`{ theme: { preset, adjust, palette } }`, as /api/me sends it). */
export function lookOf(fields) {
  const by = new Map((fields || []).map(f => [f.name, f]));
  const adjust = {};
  for (const [name, , key] of ADJUST) adjust[key] = Number(by.get(name)?.value) || 0;
  const palette = {};
  for (const [name, , key] of COLOURS) {
    const v = by.get(name)?.value || '';
    if (/^#[0-9a-f]{6}$/i.test(v)) palette[key] = v.toLowerCase();
  }
  return { theme: { preset: by.get('Base')?.value || 'Dark', adjust, palette } };
}

let open = null;

export function openThemeEditor(spec, cb = {}) {
  open?.close();
  const scrim = document.createElement('div');
  scrim.className = 'lc-te-scrim lc-scrim';
  scrim.innerHTML = '<div class="lc-te lc-box" data-kit="theme-editor" role="dialog" aria-label="Theme"></div>';
  document.body.append(scrim);
  const box = scrim.firstElementChild;
  let fields = spec.fields || [];
  const opened = lookOf(fields); // A: the theme as it was when the editor opened
  let shown = null;              // what the page previews now (null = the station's)
  let showA = false;
  let drag = null;               // { key, value } while a slider moves
  let picking = false;

  const field = name => fields.find(f => f.name === name);
  const line = (name, value) => field(name)?.line?.replace('{value}', value) ?? null;
  // `Set UserProfile 3 Theme "Base" {value}` → `UserProfile 3 Theme`: the owner for Reset.
  const owner = () => (field('Base')?.line.match(/^Set (.*?Theme) "Base"/)?.[1] || 'Theme');
  const run = l => { if (l) cb.run?.(l); };
  // A field's own reset line, else the decided forms: an adjuster to 0, a colour `Reset … Theme Slot "<Colour>"`.
  const resetLine = name => field(name)?.reset || (COLOURS.some(c => c[0] === name) ? `Reset ${owner()} Slot "${name}"` : line(name, 0));
  const preview = look => { shown = look; cb.preview?.(look); paintReadout(); };
  let tab = 'Look';

  // ── The Paint tab (decisions 0294, round 4's second board): the theme's paint slots on the
  // `paint` feed of this profile, as a <luma-grid> — each row tinted with its colour, a tap on
  // Your colour opens the paint picker (Palette / Free: a slot holds a colour) and runs the slot's
  // own line; Add a slot asks a name; Reset to LumaCore runs the feed's `reset_all`. ──
  const paintTab = (() => {
    let sub = null, data = null, grid = null, picking = false;
    const profile = () => Number(/^UserProfile (\d+)\b/.exec(owner())?.[1]) || null;
    const slots = () => data?.slots || [];
    const COLS = [
      { id: 'name', label: 'Slot', width: 150, frozen: true },
      { id: 'colour', label: 'Your colour', kind: 'paint', prop: 'Paint', width: 170 },
      { id: 'default', label: 'LumaCore default', width: 150 },
      { id: 'kind', label: 'Kind', width: 100 },
    ];
    const rows = () => slots().map((s, i) => ({
      key: s.name, no: i + 1, name: s.name, colour: s.colour, default: s.default || '—',
      kind: s.added ? 'yours' : 'built in', paint: s.colour, changed: s.changed ? ['colour'] : [],
    }));
    const count = () => {
      const n = slots().length, ch = slots().filter(s => s.changed || s.added).length;
      return `${n} slot${n === 1 ? '' : 's'}${ch ? ` · ${ch} changed` : ''}`;
    };
    const draw = () => {
      grid?.update({ key: 'ThemePaint', object: 'Theme Slot', id: 'no', row_key: 'key', bar: false, columns: COLS, rows: rows(), level: 'Admin' });
      const c = box.querySelector('[data-paint-count]');
      if (c) c.textContent = data ? count() : 'waiting for the station…';
      const r = box.querySelector('[data-paint-reset]');
      if (r) r.disabled = !data?.reset_all;
    };
    const pick = async key => {
      const s = slots().find(x => x.name === key);
      if (!s || picking) return;
      picking = true;
      await openPaint({ object: 'Theme Slot', name: s.name, line: s.line, value: s.colour, slot: true, profile: profile() }, { run });
      picking = false;
      grid?.none();
    };
    const addSlot = async () => {
      const got = await openDialog({
        title: 'Add a paint slot', text: 'A slot of your own: objects link to it with Paint Theme "<name>".',
        inputs: [{ name: 'v', label: 'Name', value: '', kind: 'text' }],
        buttons: [{ label: 'Please', value: 'set', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1,
      });
      const name = got?.value && got.inputs?.v?.trim().replace(/[";]/g, '');
      if (!name || !data?.add) return;
      run(data.add.replace('{name}', `"${name}"`).replace('{value}', '').trim());
    };
    return {
      html: () => `<section class="lc-te-card lc-te-paint" data-kit="theme-paint">
          <div class="lc-te-paintbar"><span class="lc-te-cap"><span>Paint slots — objects link to them with <b>Paint Theme "Position"</b></span><span data-paint-count></span></span>
            <button type="button" class="lc-te-btn" data-paint-add><luma-icon name="plus"></luma-icon>Add a slot</button>
            <button type="button" class="lc-te-btn danger" data-paint-reset>Reset to LumaCore</button></div>
          <div class="lc-te-paintgrid" data-paint-grid></div>
        </section>`,
      mount() {
        if (!grid) {
          grid = document.createElement('luma-grid');
          // A tap on Your colour is the picker; the grid's edit card and menu would build an object line.
          grid.addEventListener('select', e => { const d = e.detail; if (d.count === 1 && d.cells === 1 && d.column === 'colour') pick(d.rows[0]); });
          grid.addEventListener('edit', e => { grid.card?.close(); if (e.detail.rows?.length === 1) pick(e.detail.rows[0]); });
          grid.addEventListener('context', e => { e.preventDefault(); });
          grid.addEventListener('line', e => { e.stopPropagation(); if (/^(Set|Reset) GridColumns /.test(e.detail.line)) run(e.detail.line); });
        }
        box.querySelector('[data-paint-grid]').append(grid);
        if (!sub) {
          const p = profile();
          sub = window.feeds?.()?.sub('paint', p ? { profile: p } : {}, { batch: m => { if (m.data) { data = m.data; draw(); } } }) || null;
        }
        draw();
      },
      add: addSlot,
      reset: () => { if (data?.reset_all) run(data.reset_all); },
      close() { sub?.close(); sub = null; },
    };
  })();

  const current = () => {
    const look = lookOf(fields);
    if (drag) look.theme.adjust[drag.key] = drag.value;
    return look;
  };
  const exportJson = () => {
    const t = lookOf(fields).theme;
    const blob = new Blob([JSON.stringify({ lumacore: 'theme', ...t }, null, 2)], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${(spec.title || 'theme').replace(/[^\w.-]+/g, '_')}.json`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  };
  const importJson = () => {
    const inp = document.createElement('input');
    inp.type = 'file';
    inp.accept = 'application/json,.json';
    inp.onchange = async () => {
      const f = inp.files?.[0];
      if (!f) return;
      let t;
      try { t = JSON.parse(await f.text()); } catch { return; }
      const lines = [];
      if (BASES.includes(t.preset)) lines.push(line('Base', t.preset));
      for (const [name, , key, min, max] of ADJUST) if (Number.isFinite(t.adjust?.[key])) lines.push(line(name, Math.max(min, Math.min(max, Math.round(t.adjust[key])))));
      for (const [name, , key] of COLOURS) {
        const v = t.palette?.[key];
        lines.push(/^#[0-9a-f]{6}$/i.test(v || '') ? line(name, `"${v}"`) : resetLine(name));
      }
      for (const l of lines.filter(Boolean)) run(l);
    };
    inp.click();
  };

  function paintReadout() {
    const cs = getComputedStyle(document.documentElement);
    const v = t => cs.getPropertyValue(`--lc-${t}`).trim();
    const cr = contrast(v('text'), v('bg')), cd = contrast(v('text-dim'), v('surface'));
    const r = box.querySelector('[data-readout]');
    if (!r) return;
    r.textContent = `text ${cr.toFixed(1)}:1 · dim ${cd.toFixed(1)}:1${cr < 4.5 || cd < 3 ? ' — low contrast' : ''}`;
    r.classList.toggle('low', cr < 4.5 || cd < 3);
    // The swatches show what the page shows.
    for (const [name, , , tok] of COLOURS) {
      const sw = box.querySelector(`[data-colour="${name}"] .lc-te-sw`);
      if (sw) sw.style.background = `var(--lc-${tok})`;
    }
  }

  function render() {
    const look = lookOf(fields).theme;
    const base = look.preset;
    const tabs = `<span class="lc-te-seg" role="group" aria-label="Tab">${TABS.map(t => `<button type="button" data-te-tab="${t}" class="${t === tab ? 'on' : ''}" aria-pressed="${t === tab}">${t}</button>`).join('')}</span>`;
    if (tab === 'Paint') {
      box.innerHTML = `
        <div class="lc-te-head">
          <h2>${esc(spec.title || 'Theme')}</h2>${tabs}
          <button type="button" class="lc-close" data-close aria-label="Close"><luma-icon name="x"></luma-icon></button>
        </div>
        ${paintTab.html()}`;
      paintTab.mount();
      return;
    }
    box.innerHTML = `
      <div class="lc-te-head">
        <h2>${esc(spec.title || 'Theme')}</h2>${tabs}
        <span class="lc-te-seg" role="group" aria-label="Base">${BASES.map(b => `<button type="button" data-base="${b}" class="${b === base ? 'on' : ''}" aria-pressed="${b === base}">${b}</button>`).join('')}</span>
        <button type="button" class="lc-te-btn${showA ? ' on' : ''}" data-ab aria-pressed="${showA}" title="A = the theme as it was when you opened this · B = now">A | B</button>
        ${FILE_BUTTONS}
        <button type="button" class="lc-te-btn danger" data-reset-all>Reset all</button>
        <button type="button" class="lc-close" data-close aria-label="Close"><luma-icon name="x"></luma-icon></button>
      </div>
      <div class="lc-te-body">
        <div class="lc-te-left">
          <section class="lc-te-card">
            <div class="lc-te-cap"><span>Adjust — neutral = no change</span><span data-readout></span></div>
            ${ADJUST.map(([name, label, key, min, max, unit]) => {
              const f = field(name);
              const v = Number(f?.value) || 0;
              return `<div class="lc-te-adj" data-adj="${name}">
                <label for="te-${name}">${esc(label)}</label>
                <input id="te-${name}" type="range" min="${min}" max="${max}" step="1" value="${v}" data-adj-key="${key}">
                <span class="lc-te-val">${v}${unit}</span>
                <button type="button" class="lc-te-reset" data-reset="${name}" ${v !== 0 ? '' : 'disabled'} aria-label="Reset ${esc(label)}"><luma-icon name="rotate-ccw"></luma-icon></button>
              </div>`;
            }).join('')}
          </section>
          <section class="lc-te-card">
            <div class="lc-te-cap"><span>Colours (14)</span><span>tap to edit · <i class="lc-te-dot"></i> = changed</span></div>
            <div class="lc-te-colours">
              ${COLOURS.map(([name, label]) => {
                const f = field(name);
                const own = /^#[0-9a-f]{6}$/i.test(f?.value || '');
                return `<button type="button" class="lc-te-colour" data-colour="${name}">
                  <span class="lc-te-sw"></span><span class="lc-te-name">${esc(label)}${own ? ' <i class="lc-te-dot"></i>' : ''}</span>
                  <span class="lc-te-hex">${own ? esc(f.value) : 'base'}</span>
                  <span class="lc-te-reset" data-reset="${name}" role="button" aria-label="Reset ${esc(label)}" ${own ? '' : 'hidden'}><luma-icon name="rotate-ccw"></luma-icon></span>
                </button>`;
              }).join('')}
            </div>
          </section>
        </div>
        <div class="lc-te-right">
          <div class="lc-te-pickrow">
            <button type="button" class="lc-te-btn${picking ? ' on' : ''}" data-pick><luma-icon name="crosshair"></luma-icon>Pick on screen</button>
            <span class="lc-te-note">Tap an element on screen to find which colour it uses.</span>
          </div>
          <section class="lc-te-card lc-te-preview" aria-label="Preview">
            <div class="lc-te-tabs"><span class="on"><i></i>Groups</span><span><i></i>Sequences</span></div>
            <div class="lc-te-cells">${['All', 'Odd', 'Even', 'Front', 'Back', 'Spots', 'Wash L', 'Wash R'].map((n, i) => `<div class="lc-te-cell${i === 2 ? ' sel' : ''}"><small>${i + 1}</small>${n}</div>`).join('')}</div>
            <div class="lc-te-chips"><span class="store">Store</span><span>Edit</span><span class="del">Delete</span><span class="prg">Dimmer 47 %</span><span class="bl">Blind</span><span class="pk">Parked</span></div>
            <div class="lc-te-term"><div><b class="w">WARN</b> network "FOH" not mapped</div><div><b class="o">OK</b> Store Group 3 "Even"</div><div class="d">Root&gt; Group 3 At 50</div></div>
          </section>
          <div class="lc-te-normal">
            <span>The station's normal theme never writes yours: anyone at this desk can hold to see it.</span>
            <button type="button" class="lc-te-btn" data-normal>Hold: normal theme</button>
          </div>
        </div>
      </div>`;
    paintReadout();
  }

  // ── Gestures ──
  box.addEventListener('click', e => {
    const t = e.target.closest('button, [data-reset]');
    if (!t) return;
    if (t.dataset.close != null) return ctl.close();
    if (t.dataset.teTab) { tab = t.dataset.teTab; return render(); }
    if (t.dataset.paintAdd != null) return paintTab.add();
    if (t.dataset.paintReset != null) return paintTab.reset();
    if (t.dataset.base) return run(line('Base', t.dataset.base));
    if (t.dataset.ab != null) { showA = !showA; preview(showA ? opened : null); return render(); }
    if (t.dataset.import != null) return importJson();
    if (t.dataset.export != null) return exportJson();
    if (t.dataset.resetAll != null) { run(`Reset ${owner()}`); run(`Reset ${owner()} Slot`); return; }
    if (t.dataset.reset) { e.stopPropagation(); return run(resetLine(t.dataset.reset)); }
    if (t.dataset.pick != null) return startPick();
    if (t.dataset.colour) return editColour(t.dataset.colour);
  });
  box.addEventListener('input', e => {
    const r = e.target.closest('input[type="range"]');
    if (!r) return;
    drag = { key: r.dataset.adjKey, value: Number(r.value) };
    const row = r.closest('[data-adj]');
    const unit = ADJUST.find(a => a[0] === row.dataset.adj)?.[5] || '';
    row.querySelector('.lc-te-val').textContent = `${r.value}${unit}`;
    preview(current());
  });
  box.addEventListener('change', e => {
    const r = e.target.closest('input[type="range"]');
    if (!r) return;
    drag = null;
    // The release is the one line (one Oops step); the station's answer redraws the page.
    run(line(r.closest('[data-adj]').dataset.adj, r.value));
  });
  // Hold: normal theme — only while held.
  box.addEventListener('pointerdown', e => {
    if (!e.target.closest('[data-normal]')) return;
    cb.preview?.('normal');
    const up = () => { cb.preview?.(shown); document.removeEventListener('pointerup', up, true); document.removeEventListener('pointercancel', up, true); };
    document.addEventListener('pointerup', up, true);
    document.addEventListener('pointercancel', up, true);
  });

  function editColour(name) {
    const f = field(name);
    const tok = COLOURS.find(c => c[0] === name)[3];
    const now = getComputedStyle(document.documentElement).getPropertyValue(`--lc-${tok}`).trim();
    const inp = document.createElement('input');
    inp.type = 'color';
    inp.value = /^#[0-9a-f]{6}$/i.test(f?.value || '') ? f.value : (token(tok) || now);
    inp.className = 'lc-te-colorin';
    box.append(inp);
    inp.addEventListener('input', () => {
      const look = current();
      look.theme.palette[COLOURS.find(c => c[0] === name)[2]] = inp.value;
      preview(look);
    });
    inp.addEventListener('change', () => { run(line(name, `"${inp.value}"`)); inp.remove(); });
    inp.addEventListener('blur', () => setTimeout(() => inp.remove(), 300));
    inp.click();
  }

  // Pick on screen: the editor steps aside, a hint says what to do; the next tap anywhere names
  // the colour under it (taken before the page's keys see it), Esc cancels.
  function startPick() {
    picking = true;
    scrim.classList.add('picking');
    const hint = document.createElement('div');
    hint.className = 'lc-te-pickhint';
    hint.textContent = 'Tap an element to find its colour · Esc cancels';
    document.body.append(hint);
    const toks = COLOURS.map(([name, , , tok]) => [name, getComputedStyle(document.documentElement).getPropertyValue(`--lc-${tok}`).trim()]);
    const rgbOf = c => { const d = document.createElement('i'); d.style.color = c; document.body.append(d); const v = getComputedStyle(d).color; d.remove(); return v; };
    const want = toks.map(([name, c]) => [name, rgbOf(c)]);
    const swallow = ev => { ev.preventDefault(); ev.stopPropagation(); };
    const done = () => {
      window.removeEventListener('click', onPick, true);
      window.removeEventListener('pointerdown', swallow, true);
      window.removeEventListener('keydown', onEsc, true);
      hint.remove();
      scrim.classList.remove('picking');
      picking = false;
    };
    const onEsc = ev => { if (ev.key === 'Escape') { swallow(ev); done(); render(); } };
    const onPick = ev => {
      swallow(ev);
      done();
      const el = document.elementFromPoint(ev.clientX, ev.clientY) || ev.target;
      const cs = getComputedStyle(el);
      const seen = [cs.backgroundColor, cs.color, cs.borderTopColor].filter(c => c && !/,\s*0\)$/.test(c)); // a colour with no alpha draws nothing
      const hit = want.find(([, c]) => seen.includes(c))?.[0];
      render();
      if (hit) {
        const row = box.querySelector(`[data-colour="${hit}"]`);
        row?.classList.add('found');
        row?.scrollIntoView({ block: 'nearest' });
      }
      box.querySelector('.lc-te-note').textContent = hit ? `That is ${COLOURS.find(c => c[0] === hit)[1]}.` : 'That colour is derived (hover, focus, a mix): change the colour it comes from.';
    };
    window.addEventListener('pointerdown', swallow, true);
    window.addEventListener('click', onPick, true);
    window.addEventListener('keydown', onEsc, true);
  }

  // Esc closes the editor, not while a dialog or the paint picker over it has the keys.
  const onKey = e => { if (e.key === 'Escape' && !picking && !document.querySelector('[data-kit="dialog"], [data-kit="paint-picker"]')) { e.stopPropagation(); ctl.close(); } };
  document.addEventListener('keydown', onKey, true);

  const ctl = {
    el: box,
    update(next) {
      fields = next || fields;
      if (!drag) { if (!showA) preview(null); render(); }
    },
    close() {
      paintTab.close();
      document.removeEventListener('keydown', onKey, true);
      scrim.remove();
      cb.preview?.(null);
      cb.closed?.();
      if (open === ctl) open = null;
    },
  };
  open = ctl;
  render();
  return ctl;
}

/** Is a theme editor open? */
export const themeEditorOpen = () => !!open;
/** The open editor, to feed it the station's fresh fields. */
export const openEditor = () => open;
