// The Showroom window (decisions §30): every kit element in every state,
// drawn by the kit itself. It grows with each part of step 12 and is the
// reference for windows and plugins — and, through
// `./scripts/unix/screenshots`, the kit's picture set. A preset switched here
// previews on this page only (the user's theme is stored on the user, 12a2).

import { PRESETS, preset, setPreset, applyLook } from './theme.js';
import { openThemeEditor, ADJUST, COLOURS } from './themeeditor.js';
import { openPaint } from './paint.js';
import { iconNames } from './icon.js';
import { hold } from './pointer.js';
import { openMenu } from './menu.js';
import { openDialog, showDialog } from './dialog.js';
import { openKeypad } from './keypad.js';
import { openPick } from './popup.js';
import { SampleRing } from './feeds.js';
import { openSettings } from './settings.js';
import './windowbar.js';
import './cuesheet.js';
import './tiles.js';
import './maxbar.js';
import { toast } from './toast.js';
import { miniature } from './minipane.js';
import { handle as splitHandle } from './split.js';
import './pool.js';
import { objectMenus, openContext } from './context.js';
import { openValue, renumberKind } from './value.js';
import { toolbar, toolTypes } from './tools.js';
import { build12d } from './showroom12d.js';
import { openSheet } from './sheet.js';
import { openAttrBoard } from './attrboard.js';
import './detail.js';
import * as marks from './marks.js';
import './devicegrid.js';
import './treewindow.js';
import './ruleswindow.js';
import './keymap.js';

const PALETTE = [
  ['Background', 'bg'], ['Surface', 'surface'], ['Raised', 'raised'], ['Line', 'line'], ['Text', 'text'],
  ['TextDim', 'text-dim'], ['Accent', 'accent'], ['Info', 'info'], ['Ok', 'ok'], ['Warn', 'warn'],
  ['Alert', 'alert'], ['Programmer', 'programmer'], ['Blind', 'blind'], ['Park', 'park'],
];
const DERIVED = ['gap', 'hover', 'pressed', 'alt-row', 'selected', 'lane', 'focus', 'disabled', 'dot-off', 'on-accent', 'strong'];
const SIZES = [['hit', 'Smallest control'], ['row', 'Tab row, list rows'], ['bar', 'Window bar'], ['action', 'Action rows'], ['cmdbar', 'Command line bar']];

function section(title, body) {
  const s = document.createElement('section');
  s.className = 'lc-sr-sec';
  s.innerHTML = `<h3>${title}</h3>`;
  s.append(body);
  return s;
}
function div(cls, html = '') { const d = document.createElement('div'); d.className = cls; d.innerHTML = html; return d; }

/** The Showroom's content for window `win`. */
export function buildShowroom() {
  const root = div('lc-showroom');
  root.dataset.kit = 'showroom';

  // Presets
  const presets = div('lc-sr-row');
  const paint = () => presets.querySelectorAll('luma-button').forEach(b =>
    b.setAttribute('variant', b.getAttribute('label') === preset() ? 'primary' : ''));
  for (const p of PRESETS) {
    const b = document.createElement('luma-button');
    b.setAttribute('label', p);
    b.addEventListener('click', () => { setPreset(p); paint(); });
    presets.append(b);
  }
  paint();
  root.append(section('Theme preset (preview on this page)', presets));

  // Palette + derived
  const pal = div('lc-sr-swatches');
  for (const [name, tok] of PALETTE)
    pal.append(div('lc-sr-swatch', `<i style="background:var(--lc-${tok})"></i><b>${name}</b><small>--lc-${tok}</small>`));
  root.append(section('Palette — 14 editable colours', pal));
  const der = div('lc-sr-swatches');
  for (const tok of DERIVED) der.append(div('lc-sr-swatch', `<i style="background:var(--lc-${tok})"></i><b>${tok}</b><small>derived</small>`));
  root.append(section('Derived roles — never edited', der));

  // Type and sizes
  root.append(section('Type', div('lc-sr-type',
    `<div style="font-size:var(--lc-fs-title);font-weight:600">Title · Source Sans 3 semibold</div>
     <div>Body · Source Sans 3 regular</div>
     <div style="font-size:var(--lc-fs-small);color:var(--lc-text-dim)">Caption · never smaller than 12 px</div>
     <div style="font-family:var(--lc-mono);color:var(--lc-strong)">[Admin@desk1] Root&gt; Group 3 At 50 · JetBrains Mono</div>`)));
  const sizes = div('lc-sr-sizes');
  for (const [tok, what] of SIZES)
    sizes.append(div('lc-sr-size', `<span>${what}</span><i style="width:var(--lc-${tok})"></i><small>--lc-${tok}</small>`));
  root.append(section('Sizes (larger on touch screens)', sizes));

  // Icons
  const icons = div('lc-sr-icons');
  iconNames().then(names => {
    for (const n of names) icons.append(div('lc-sr-icon', `<luma-icon name="${n}"></luma-icon><small>${n}</small>`));
  });
  root.append(section('Icons — Lucide + LumaCore glyphs', icons));

  // Buttons
  const buttons = div('lc-sr-row');
  buttons.innerHTML = `
    <luma-button variant="primary" icon="save" label="Primary"></luma-button>
    <luma-button icon="settings" label="Default"></luma-button>
    <luma-button variant="danger" icon="trash-2" label="Delete"></luma-button>
    <luma-button variant="quiet" label="Quiet"></luma-button>
    <luma-button disabled icon="lock" label="Disabled (Guest)"></luma-button>
    <luma-button icon="menu" title="Icon only"></luma-button>`;
  root.append(section('luma-button', buttons));

  // Hold ring + menu
  const holdTile = div('lc-sr-hold', '<luma-icon name="ellipsis"></luma-icon><span>Hold me — or right-click</span>');
  hold(holdTile, {
    onHold: ({ x, y }) => openMenu({ x, y }, [
      { header: 'Hold = show me more' },
      { icon: 'pencil', label: 'Edit', fn: () => {} },
      { icon: 'tag', label: 'Label', hint: 'Label Self', fn: () => {} },
      null,
      { icon: 'trash-2', label: 'Delete', danger: true, fn: () => {} },
      { icon: 'lock', label: 'Disabled item', off: true },
    ]),
  });
  root.append(section('Hold ring → menu', holdTile));

  // Toasts: a line that goes by itself, and the copy fallback (the NAV inspector's copy when no
  // clipboard can be reached): the text selected in a field, ready for Ctrl+C.
  const toasts = div('lc-sr-row');
  toasts.innerHTML = `
    <luma-button label="A toast"></luma-button>
    <luma-button icon="copy" label="Hold a text to copy"></luma-button>`;
  const [plainBtn, holdBtn] = toasts.querySelectorAll('luma-button');
  plainBtn.addEventListener('click', () => toast('Too small — hold to open Groups'));
  holdBtn.addEventListener('click', () => toast('Copy GUID: Ctrl+C', { hold: '6f1d0c52-7a1e-4c0b-9d3e-000000000101', onCopy: () => toast('Copied GUID') }));
  root.append(section('Toast — a line; one that holds a text to copy', toasts));

  // Split handle (12e): a mouse drags the line, a finger only the grip pill;
  // double-tap = 50 / 50; arrows / Home / End / Enter with the keyboard.
  const splitDemo = div('lc-sr-split');
  splitDemo.style.cssText = 'display:flex;height:140px;border:1px solid var(--lc-line);background:var(--lc-gap)';
  const paneA = div('lc-sr-split-pane', 'A'), paneB = div('lc-sr-split-pane', 'B');
  for (const p of [paneA, paneB]) p.style.cssText = 'display:grid;place-items:center;background:var(--lc-bg);color:var(--lc-text-dim);min-width:0';
  let share = 0.5;
  const out = div('lc-sr-note', 'ratio 0.50');
  const h = splitHandle('h', {
    size: () => splitDemo.offsetWidth - 4,
    ratio: () => share,
    set: r => { share = r; paneA.style.flex = `${r} 1 0%`; paneB.style.flex = `${1 - r} 1 0%`; out.textContent = `ratio ${r.toFixed(2)}`; },
    commit: () => toast(`One layout step: ${share.toFixed(2)}`),
    minPx: 60,
  });
  splitDemo.append(paneA, h, paneB);
  const splitWrap = div('');
  splitWrap.append(splitDemo, out);
  root.append(section('Split handle — the grip pill on touch, the line with a mouse', splitWrap));

  // <luma-pool> (12e): NodeMeta colours, the pool type's colour, a hold opens
  // the one context menu (§34 round 3, 18); Move picks its target with a tap.
  const poolBox = div('lc-sr-pool');
  poolBox.style.cssText = 'display:flex;height:220px;border:1px solid var(--lc-line)';
  const pool = document.createElement('luma-pool');
  poolBox.append(pool);
  const demo = [[1, 'All', 'var(--lc-ok)'], [2, 'Odd', 'var(--lc-alert)'], [3, 'Even', 'var(--lc-blind)'], [4, 'Front', null], [5, 'Back', null], [9, 'Truss', 'var(--lc-info)']];
  queueMicrotask(() => {
    pool.setup({
      kind: 'groups', settings: { columns: 0, size: 80, rows: 0 },
      onTap: n => toast(`Group ${n}`),
      onHold: async (n, c, at) => c && openContext({
        at, menu: (await objectMenus()).Group, object: 'Group', rows: String(n), count: 1, note: 'DataPool 1',
        value: p => (p === 'Name' ? [c.name] : null), level: 'Admin',
        run: line => toast(line), edit: () => toast(`EditSetting Group ${n}`),
        valueDialog: k => k === 'number' && (openValue({
          title: 'Renumber', sub: `Group ${n} · ${c.name}`,
          kind: renumberKind({ object: 'Group', rows: [n], grid: [n], taken: demo.map(d => d[0]).filter(x => x !== n) }), apply: line => toast(line),
        }), true),
        target: (item, go) => {
          if (item.id !== 'move' && item.id !== 'copy') return false;
          pool.armPick(`${item.label} Group ${n}: tap the target`, n, to => go(String(to)));
          return true;
        },
      }),
    });
    pool.cells = demo.map(([no, name, color]) => ({ no, name, color }));
  });
  root.append(section('luma-pool — hold an object for the one context menu', poolBox));

  // The list button and the tools (12e): the yellow `===` = opens a list.
  const toolsBox = div('lc-sr-tools');
  toolsBox.style.cssText = 'display:flex;flex-direction:column;gap:8px;min-width:0';
  const lb = document.createElement('luma-button');
  lb.setAttribute('icon', 'wrench'); lb.setAttribute('label', 'Tools'); lb.setAttribute('list', '');
  toolsBox.append(lb);
  toolTypes().then(types => toolsBox.append(toolbar({ types, line: (t, a) => `Grid Group 1 "${t.id}"${a == null ? '' : ` "${a}"`}`, run: l => toast(l) })));
  root.append(section('List button (===) and the tools toolbar', toolsBox));

  // Miniature lists (12e round 5): the sequence in dense text, the fixture
  // sheet as the hybrid — each column's mini renderer.
  const miniBox = div('lc-sr-mini');
  miniBox.style.cssText = 'display:flex;gap:12px;flex-wrap:wrap';
  const seqG = document.createElement('luma-grid'), fixG = document.createElement('luma-grid');
  for (const g of [seqG, fixG]) { g.style.cssText = 'width:190px;height:300px;border:3px solid var(--lc-gap)'; miniBox.append(g); }
  const cues = ['Preset look', 'Walk-in', 'House to half', 'Opening', 'Verse 1', 'Chorus', 'Verse 2', 'Bridge', 'Drop', 'Outro', 'Blackout', 'Bows'];
  queueMicrotask(() => {
    seqG.update({ key: 'sr-seq', view: 'mini', mini_row: 14, rows: cues.map((name, i) => ({ guid: `c${i}`, no: i + 1, name, fade: [2, 3, 1, 5][i % 4], delay: i % 5 ? 0 : 1, trig: i === 3 ? 'Follow' : 'Go', mini: i === 5 ? 'run' : i === 6 ? 'next' : i < 5 ? 'done' : '', progress: i === 5 ? 0.4 : null })),
      columns: [{ id: 'cue', field: 'no', label: 'Cue', mini: 'num', mini_min: 26, mini_w: 0.9 }, { id: 'name', label: 'Name', frozen: true, mini_min: 64, mini_w: 3 },
        { id: 'fade', label: 'Fade', mini: 'num', mini_min: 30 }, { id: 'delay', label: 'Delay', mini: 'num', mini_min: 32 }, { id: 'trig', label: 'Trig', mini_min: 36 }] });
    fixG.update({ key: 'sr-fix', view: 'mini', mini_row: 14, rows: Array.from({ length: 16 }, (_, i) => ({ guid: `f${i}`, no: 101 + i, name: `Spot ${i + 1}`, dim: [100, 80, 0, 45][i % 4], prg: i > 3 && i < 8, col: ['var(--lc-white)', 'var(--lc-alert)', 'var(--lc-info)', 'var(--lc-warn)'][i % 4], pan: (i * 17) % 90, mini: i > 3 && i < 8 ? 'sel' : '' })),
      columns: [{ id: 'fix', field: 'no', label: 'Fix', mini: 'num', mini_min: 28 }, { id: 'name', label: 'Name', frozen: true, mini_min: 50, mini_w: 2 },
        { id: 'dim', label: 'Dim', mini: 'level', tone: 'prg', mini_min: 46, mini_w: 1.8 }, { id: 'col', label: 'Col', mini: 'swatch', mini_min: 26, mini_w: 0.8 }, { id: 'pan', label: 'Pan', mini: 'num', mini_min: 30 }] });
  });
  root.append(section('Miniature lists — the sequence (dense text) and the fixture sheet (hybrid)', miniBox));

  // The value marks and the device grid (decisions device-grid-and-value-marks).
  const marksBox = div('lc-sr-marks');
  marksBox.style.cssText = 'display:flex;flex-direction:column;gap:10px;min-width:0';
  const states = div('lc-sr-row');
  states.style.cssText = 'display:flex;flex-wrap:wrap;gap:6px';
  states.innerHTML = Object.keys(marks.STATES).filter(s => s !== 'cue').map(s => `<span style="display:inline-flex;flex-direction:column;gap:3px;width:118px">`
    + marks.valueCell({ t: s === 'park' ? '30°' : '75', s, l: s === 'prgA' ? ['abs'] : s === 'prgI' ? ['deact'] : s === 'park' ? ['park'] : [] })
    + `<small style="color:var(--lc-text-dim)">${s}</small></span>`).join('')
    + `<span style="display:inline-flex;flex-direction:column;gap:3px;width:118px">${marks.valueCell({ t: '62', s: 'new', f: 0.4 })}<small style="color:var(--lc-text-dim)">fading</small></span>`;
  const layers = div('lc-sr-row');
  layers.style.cssText = 'display:flex;flex-wrap:wrap;gap:6px';
  layers.innerHTML = [...marks.LAYERS, 'deact', 'tester'].map(l => `<span style="display:inline-flex;flex-direction:column;gap:3px;width:118px">`
    + marks.valueCell({ t: '100', s: 'def', l: [l] })
    + `<small style="color:var(--lc-text-dim)">bar: ${l}</small></span>`).join('');
  const dg = document.createElement('luma-device-grid');
  dg.style.cssText = 'height:260px;border:3px solid var(--lc-gap)';
  dg.addEventListener('line', e => { e.stopPropagation(); toast(e.detail.line); });
  marksBox.append(states, layers, dg);
  queueMicrotask(() => {
    const row = (key, name, fid, depth, kind = 'fixture', extra = {}) => ({ key, kind, id: kind === 'folder' ? `Fixture Media ${fid}` : `Fixture ${fid}`, fid: kind === 'folder' ? null : fid, name, type: 'ACME Icarus 620', id_type: kind === 'folder' ? 'Media' : 'Fixture', cid: kind === 'folder' ? fid : null, depth, count: kind === 'folder' ? 3 : 0, sel: null, bar: null, ...extra });
    const v = (t, s, n, l) => ({ t, s, n, ...(l ? { l } : {}) });
    dg.show({
      layer: 'Absolute', total: 4, from: 0, selected: 1,
      columns: [{ id: 'Dimmer', label: 'Dim', group: 'Dimmer', priority: 1 }, { id: 'Pan', label: 'Pan', group: 'Position', priority: 1 }, { id: 'Tilt', label: 'Tilt', group: 'Position', priority: 2 }, { id: 'ColorSub_M', label: 'M', group: 'Color', priority: 1 }],
      rows: [row('f', 'ACME Icarus', 7003, 0, 'folder', { bar: 'abs' }), row('a', 'Icarus', 785, 1, 'fixture', { sel: 0, bar: 'abs' }), row('b', 'Icarus', 837, 1), row('c', 'Icarus', 245, 1, 'fixture', { bar: 'deact' })],
      cells: {
        f: { Dimmer: { t: '·', s: 'mixed', l: ['abs'] }, Pan: { t: '·', s: 'mixed' }, Tilt: { t: '30°', s: 'def', n: 0.6 }, ColorSub_M: { t: '0', s: 'def', n: 0 } },
        a: { Dimmer: v('100', 'prgA', 1, ['abs', 'fade']), Pan: v('-45°', 'prgA', 0.4, ['abs']), Tilt: v('30°', 'track', 0.6), ColorSub_M: v('80', 'prgA', 0.8, ['abs', 'preset']), '#mix': 'var(--lc-v-delay)' },
        b: { Dimmer: v('75', 'new', 0.75), Pan: v('10°', 'other', 0.5), Tilt: v('30°', 'block', 0.6), ColorSub_M: v('0', 'def', 0), '#mix': 'var(--lc-white)' },
        c: { Dimmer: v('60', 'prgI', 0.6, ['deact']), Pan: v('0°', 'park', 0.5, ['park']), Tilt: v('30°', 'otherT', 0.6), ColorSub_M: v('35', 'lower', 0.35), '#mix': 'var(--lc-v-track)' },
      },
    }, { mode: 'Fixture' });
  });
  root.append(section('Value marks and <luma-device-grid> — source as text, state as ground, layers as bars', marksBox));

  // The top bar (decisions top-bar-status-strip): the strip in four states of its mark.
  const stripBox = div('lc-sr-strips');
  stripBox.style.cssText = 'display:flex;flex-direction:column;gap:6px;min-width:0';
  const base = { user: 'Admin', show: { name: 'demo', dirty: true }, modes: ['Highlight'], strip: { station: 'A', profile: 'Default', datapool: '1 Main', sequence: '1 Main', oops: 12 } };
  for (const [conn, ls, extra] of [['up', null, {}], ['up', { status: 'Member', health: 'ok', latency_ms: 12, master: 'A' }, { modes: ['Blind', 'Solo'] }], ['retry', null, {}], ['lost', null, {}]]) {
    const s = document.createElement('luma-status-strip');
    s.style.cssText = 'height:30px;padding:0 10px;background:var(--lc-surface);border:1px solid var(--lc-gap)';
    s.addEventListener('line', e => { e.stopPropagation(); toast(e.detail.line); });
    stripBox.append(s);
    queueMicrotask(() => s.update({ conn, status: { ...base, ...extra, lumasession: ls, strip: { ...base.strip, station: ls ? 'B' : 'A' } } }));
  }
  root.append(section('<luma-status-strip> — the top bar: the mark, the Show tab, the facts, the modes, the clock', stripBox));

  // The theme editor (archive §30 "12g", the ThemeBg board): sample fields; its lines are toasts.
  const teBox = div('lc-sr-row');
  const teBtn = document.createElement('luma-button');
  teBtn.setAttribute('icon', 'palette');
  teBtn.setAttribute('label', 'Theme editor');
  teBtn.addEventListener('click', () => {
    const f = (name, value, line) => ({ name, value: String(value), line: `Set Theme ${line} {value}`, reset: `Set Theme ${line} Default`, changed: !!value && value !== 'Dark' });
    openThemeEditor({ title: 'Theme · Showroom', fields: [f('Base', 'Dark', '"Base"'), ...ADJUST.map(([n]) => f(n, n === 'BackgroundHue' ? 210 : 0, `"${n}"`)), ...COLOURS.map(([n]) => f(n, '', `Paint "${n}"`))] }, {
      run: line => toast(line),
      preview: look => (look && look !== 'normal' ? applyLook(look) : window.applyMyLook?.()),
    });
  });
  teBox.append(teBtn);
  root.append(section('The theme editor — base, adjusters, the 14 colours, pick on screen, A | B, import / export', teBox));

  // <luma-paint> (decisions 0294): the paint picker on the live `paint` feed (the station's palette,
  // your slots and recent colours — never a palette of the page's own), opened in each mode; its
  // Please and mode lines are toasts here.
  const pkBox = div('lc-sr-row');
  for (const [mode, value] of [['Palette', ''], ['Free', ''], ['Theme', 'Theme "Position"']]) {
    const b = document.createElement('luma-button');
    b.setAttribute('icon', 'palette');
    b.setAttribute('label', `Paint · ${mode}`);
    b.addEventListener('click', () => openPaint({ object: 'Group 3', name: 'Blinder', no: 3, mode, line: 'Set Group 3 "Paint" {value}', value },
      { run: line => toast(line) }));
    pkBox.append(b);
  }
  const slotBtn = document.createElement('luma-button');
  slotBtn.setAttribute('icon', 'palette');
  slotBtn.setAttribute('label', 'Paint · a theme slot');
  slotBtn.addEventListener('click', () => openPaint({ object: 'Theme Slot', name: 'Position', slot: true, line: 'Set Theme Slot "Position" {value}', value: '' }, { run: line => toast(line) }));
  pkBox.append(slotBtn);
  root.append(section('luma-paint — the paint picker on the live paint feed: Palette (the station\'s hues and tones, greys, none), Free (square + hue strip), Theme (your slots), Recent, Examples, Hex, the Line', pkBox));

  // Avatars (12a3)
  const avatars = div('lc-sr-row lc-sr-dark');
  avatars.innerHTML = `
    <luma-avatar name="Admin" color="var(--lc-accent)" badge="Admin" state="this device" selected></luma-avatar>
    <luma-avatar name="Bob" color="var(--lc-info)" badge="LD" state="on FOH"></luma-avatar>
    <luma-avatar name="Ann" color="var(--lc-programmer)" badge="LD" state="only on desk1" dim></luma-avatar>
    <luma-avatar name="Guest" color="var(--lc-text-dim)" badge="Guest" state="logged out"></luma-avatar>`;
  root.append(section('luma-avatar — own colour, profile badge, where logged in', avatars));

  // Keyboard (12a3; text mode with the command-word row)
  const kbWrap = div('lc-sr-kb');
  const field = document.createElement('input');
  field.className = 'lc-sr-field';
  field.value = 'Label Group 3 "Front wa';
  const kb = document.createElement('luma-keyboard');
  kb.setAttribute('mode', 'text');
  kbWrap.append(field, kb);
  kb.target = field;
  kb.setWords(['At', 'Thru', '+', '-', 'Group', 'Fixture', 'Store', 'Oops']);
  root.append(section('luma-keyboard — built-in keyboard (password mode hides the word row)', kbWrap));

  // Dialog (12a3)
  const dlg = div('lc-sr-row');
  const open = (spec) => () => openDialog(spec);
  const b1 = document.createElement('luma-button');
  b1.setAttribute('label', 'Dialog: Clean start');
  b1.addEventListener('click', open({ title: 'Clean start', meta: 'local server only',
    text: 'desk1 starts with an empty show and its station settings reset. The show has unsaved changes.',
    buttons: [{ label: 'Save, then clean start', value: 'save', kind: 'primary' }, { label: "Don't save", value: 'nosave', kind: 'danger' }, { label: 'Cancel', value: null }],
    note: 'Save → CleanStart /Save' }));
  const b2 = document.createElement('luma-button');
  b2.setAttribute('label', 'Dialog: countdown');
  b2.addEventListener('click', open({ title: 'Shutdown', text: 'Save before shutting down?', countdown: { seconds: 8, value: null },
    buttons: [{ label: 'Save', value: 'save', kind: 'primary' }, { label: "Don't save", value: 'nosave', kind: 'danger' }, { label: 'Cancel', value: null }] }));
  const btn = (label, fn) => { const b = document.createElement('luma-button'); b.setAttribute('label', label); b.addEventListener('click', fn); return b; };
  // 12b: what the station's dialogs carry — inputs, toggles, selectors,
  // details, a disabled button with its reason, a countdown it owns.
  const b3 = btn('Dialog: inputs', open({ title: 'Copy Group 4', text: 'Copy Group 4 "Front wash" to another number.',
    inputs: [
      { name: 'to', label: 'To (Group number · 1–9999)', value: '12', kind: { kind: 'number', min: 1, max: 9999 } },
      { name: 'pool', label: 'Into data pool (link)', value: '1', kind: { kind: 'link', pick: { kind: 'DataPool', items: [{ no: 1, name: 'Default' }, { no: 2, name: 'Festival' }] } } },
    ],
    toggles: [{ name: 'name', label: 'Keep its name', on: true }],
    selectors: [{ name: 'used', label: 'If the number is used', choices: ['Merge', 'Overwrite', 'Cancel'], value: 'Merge' }],
    buttons: [{ label: 'Please', value: 'go', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1 }));
  const b4 = btn('Dialog: Label + chips', open({ title: 'Label Group 4', text: 'A new name. Empty clears the label.',
    inputs: [{ name: 'name', label: 'Name', value: 'Front wash', kind: 'text', chips: ['Front warm', 'Back', 'Spots'] }],
    buttons: [{ label: 'Please', value: 'go', kind: 'please' }, { label: 'Cancel', value: null }], primary: 0, cancel: 1 }));
  const b5 = btn('Dialog: merge (station)', () => {
    const ctl = showDialog({ id: 'merge', title: 'Rejoin LumaSession "Festival"', text: 'desk3 was apart for 12 min. The master is desk1.',
      details: [{ k: 'Group 3 "Even" · changed on both', tone: 'warn' }, { k: 'Group 7 "Floor" · new here', tone: 'ok' }, { k: 'Sequence 1 Cue 4 · deleted on master', tone: 'alert' }],
      buttons: [{ label: 'Merge', kind: 'primary', disabled: 'the patch changed on both sides' }, { label: 'Use Master' }, { label: 'Use Mine', kind: 'danger' }, { label: 'Cancel' }],
      primary: 1, cancel: 3, countdown: { deadline_ms: Date.now() + 41000, total_s: 60, default: 1, paused: false } },
    { onAnswer: () => ctl.close(), onPause: () => {} });
  });
  dlg.append(b1, b2, b3, b4, b5);
  root.append(section('The standard dialog — every question the station asks', dlg));

  // Keypad and pick list (12b)
  const kp = div('lc-sr-row');
  kp.append(
    btn('Keypad: Autosave minutes', () => openKeypad({ title: 'Autosave · minutes', value: '15', min: 0, max: 1440, unit: 'min' })),
    btn('Keypad: units', () => openKeypad({ title: 'Cue 3 Fade', value: '2.5', min: 0, max: 3600, units: ['Seconds', '25 fps', '30 fps'] })),
  );
  root.append(section('luma-keypad — the old value preselected, relative input (+ 3, * 2) worked out by the station', kp));
  const pk = div('lc-sr-row');
  pk.append(
    btn('Pick list: required + missing', () => openPick({ title: 'Profile of Bob', value: 'LD',
      pick: { kind: 'UserProfile', missing: 'missing · 7c2e1f41', add: 'Store UserProfile 6', items: [
        { no: 1, name: 'Admin', note: '2 users' }, { no: 2, name: 'LD', note: '3 users' }, { no: 3, name: 'Programmer' }, { no: 4, name: 'Guest' }] } })),
    btn('Pick list: optional', () => openPick({ title: 'LockScreen image · LD', value: '1',
      pick: { kind: 'Image', optional: true, items: [{ no: 1, name: 'Stage left', note: '1920×1080' }, { no: 2, name: 'Truss night', note: '2560×1440' }] } })),
  );
  root.append(section('luma-popup — the pick list for every link (filter, None, missing, Add New)', pk));

  // Settings window (12b2): tabs from sections, typed cells, applies at once
  const st = div('lc-sr-row');
  const demoFields = () => [
    { name: 'Tap action', section: 'Pool', input: { type: 'choice', options: ['Select', 'Edit', 'None'] }, value: 'Select', line: 'Set Window 2 "Action" {value}' },
    { name: 'DataPool', section: 'Pool', input: { type: 'link' }, value: 'Selected DataPool', line: 'Set Window 2 "Datapool" {value}', hint: 'Selected DataPool follows your Select DataPool',
      pick: { kind: 'DataPool', follow: 'Selected DataPool', items: [{ no: 1, name: 'Default', note: '3 groups' }, { no: 2, name: 'Busk', note: '12 groups' }] } },
    { name: 'First index', section: 'Pool', input: { type: 'number', min: 1, max: 9999 }, value: '1', line: 'Set Window 2 "FirstIndex" {value}' },
    { name: 'Columns', section: 'Layout', input: { type: 'number', min: 0, max: 64, named: ['Fit'] }, value: 'Fit', line: 'Set Window 2 "Columns" {value}' },
    { name: 'Show title', section: 'Layout', input: { type: 'toggle' }, value: 'Yes', line: 'Set Window 2 "ShowTitle" {value}' },
    { name: 'Empty', section: 'Colours', input: { type: 'colour' }, value: 'Theme', line: 'Set Window 2 "EmptyPaint" {value}' },
  ];
  st.append(btn('Settings: Group pool window', () => openSettings({ title: 'Window 2 · Groups', fields: demoFields(), delete_line: 'Delete Window 2' }, { run: () => {} })));
  // 12c: a window's settings with its bar — pins, ↺, the Window bar tab, presets.
  let dmxBar = ['universe', 'prev', 'next', 'readout'];
  let dmxReadout = 'Decimal';
  const dmxFields = () => [
    { name: 'Universe', key: 'universe', section: 'Display', input: { type: 'number', min: 1, max: 63999 }, value: '1 Front', line: 'Set Window 1.4 universe {value}' },
    { name: 'View', key: 'view', section: 'Display', input: { type: 'choice', options: ['Output (merged)', 'Input (raw)'] }, value: 'Output (merged)', line: 'Set Window 1.4 view {value}' },
    { name: 'Readout', key: 'readout', section: 'Display', input: { type: 'choice', options: ['Percent', 'Decimal', 'Hex'] }, value: dmxReadout,
      changed: dmxReadout !== 'Percent', reset: 'Set Window 1.4 readout Default', line: 'Set Window 1.4 readout {value}' },
    { name: 'Columns', key: 'columns', section: 'Display', input: { type: 'number', min: 0, max: 64, named: ['Fit'] }, value: 'Fit', line: 'Set Window 1.4 columns {value}' },
    { name: 'ID', key: 'show_id', section: 'Mask', input: { type: 'toggle' }, value: 'No', line: 'Set Window 1.4 show_id {value}' },
    { name: 'Window bar', key: 'window_bar', section: 'Window bar', input: { type: 'toggle' }, value: 'Yes', line: 'Set Window 1.4 window_bar {value}' },
    { name: 'Bar items', key: 'window_bar_items', section: 'Window bar', value: dmxBar.join(','), line: 'Set Window 1.4 window_bar_items {value}',
      input: { type: 'list', items: [
        { id: 'universe', label: 'Universe', kind: 'setting' }, { id: 'view', label: 'View', kind: 'setting' }, { id: 'readout', label: 'Readout', kind: 'setting' },
        { id: 'columns', label: 'Columns', kind: 'setting' }, { id: 'show_id', label: 'ID', kind: 'setting' },
        { id: 'prev', label: 'Universe −', kind: 'action', line: 'Set Self "Universe" - 1' }, { id: 'next', label: 'Universe +', kind: 'action', line: 'Set Self "Universe" + 1' },
        { id: 'first', label: 'First universe', kind: 'action', line: 'Set Self "Universe" 1' }] } },
  ];
  const dmxSpec = () => ({ title: 'Window 1.4 · DMX', subtitle: 'from preset 2 “Big DMX” · changed', icon: 'app-window', fields: dmxFields(),
    actions: [{ label: 'Load preset', line: 'Call WindowPreset' }, { label: 'Save preset', line: 'Store WindowPreset' },
      { label: 'Reset window', line: 'Reset Self', variant: 'warn' }, { label: 'Delete window', line: 'Delete Self', variant: 'danger' }] });
  st.append(btn('Settings: DMX window with its bar (12c)', () => {
    const w = openSettings(dmxSpec(), { run: line => {
      const m = /^Set Self WindowBarItems "(.*)"$/.exec(line);
      if (m) dmxBar = m[1].split(',').filter(Boolean);
      if (/WindowBarItems Default|^Reset Self/.test(line)) dmxBar = ['universe', 'prev', 'next', 'readout'];
      if (/readout Default|^Reset Self/.test(line)) dmxReadout = 'Percent';
      w.update(dmxSpec());
    } });
  }));
  root.append(section('luma-settings — the one settings window (tabs from sections, a cell per input, every change a command line)', st));

  // luma-tree (decisions §12 "Tags hardened"): the tag tree, browsed (the pool window) and as a
  // three-way picker (EditSetting's Tags tab). Demo nodes; every tap shows the line it would run.
  const treeNodes = [
    { path: '1', name: 'All', colour: 'var(--lc-accent)', count: 0 }, { path: '1.1', name: 'Lx1', colour: 'var(--lc-ok)', count: 12, state: 'part' },
    { path: '1.1.1', name: 'Right', colour: 'var(--lc-ok)', count: 4, state: 'on' }, { path: '1.1.2', name: 'Left', colour: 'var(--lc-ok)', count: 4 },
    { path: '1.2', name: 'Lx2', colour: 'var(--lc-info)', count: 12 }, { path: '1.2.1', name: 'Right', count: 4 },
    { path: '2', name: 'Busking', colour: 'var(--lc-alert)', count: 0 }, { path: '2.1', name: 'Encore', count: 3, badge: 'Go+ · planned' },
  ];
  const trees = div('lc-sr-row');
  const treeOut = div('lc-sr-note lc-selectable', 'Tap a row, a box, ≡ or New tag here: the line it runs shows here.');
  for (const mode of ['browse', 'pick']) {
    const frame = div('lc-sr-tree');
    frame.style.cssText = 'width:min(420px,100%);height:360px;display:flex;flex-direction:column';
    const t = document.createElement('luma-tree');
    frame.append(t);
    trees.append(frame);
    let selected = mode === 'browse' ? '1.1' : null;
    const paint = () => t.update({ nodes: treeNodes.map(n => ({ ...n, state: mode === 'pick' ? n.state || 'off' : undefined })), mode, selected, search: mode === 'pick', newHere: mode === 'pick' });
    t.addEventListener('select', e => { selected = e.detail.path; treeOut.textContent = mode === 'browse' ? `› Set Self "Tag" "${e.detail.path}"` : `› (highlight ${e.detail.path})`; paint(); });
    t.addEventListener('toggle', e => {
      const n = treeNodes.find(x => x.path === e.detail.path);
      n.state = e.detail.on ? 'on' : 'off';
      treeOut.textContent = `› ${e.detail.on ? 'Assign' : 'Unassign'} Tag ${e.detail.path} At Fixture 103`;
      paint();
    });
    t.addEventListener('menu', e => { treeOut.textContent = `› ≡ Tag ${e.detail.path}`; });
    t.addEventListener('new', e => { treeOut.textContent = e.detail.parent ? `› Store Tag ${e.detail.parent} Tag "…"` : '› Store Tag "…"'; });
    paint();
  }
  trees.append(treeOut);
  root.append(section('luma-tree — the tag tree: browse (the Tags pool window) and pick (EditSetting\'s Tags tab, three-way boxes)', trees));

  // luma-dmxmap (decisions §34 round 3, 12, 18; `dmx-map.html` v2): the DMX map zoomed out, on
  // the mockup's data — six Auras on universe 1, three BMFLs on 2 with 303 over 302 (both parked).
  const mapBox = div('');
  mapBox.style.cssText = 'display:flex;flex-direction:column;gap:8px';
  const mapFrame = div('');
  mapFrame.style.cssText = 'height:420px;display:flex;flex-direction:column;border:1px solid var(--lc-line)';
  const dm = document.createElement('luma-dmxmap');
  mapFrame.append(dm);
  const mapOut = div('lc-sr-note lc-selectable', 'Tap a block, then a free run; tap a fix — the line it runs shows here.');
  mapBox.append(mapFrame, mapOut);
  const auras = [1, 15, 29, 85, 105, 125].map((start, i) => ({ no: 101 + i, copy: null, break: 1, name: `Aura ${i + 1}`, universe: 1, start, count: 14, fixture_type: 1, parked: false }));
  dm.update({
    universes: [{ no: 1, name: 'Stage', used: 84 }, { no: 2, name: 'Truss', used: 128 }, { no: 3, name: '', used: 0 }],
    runs: [...auras, { no: 101, copy: 1, break: 1, name: 'Aura 1', universe: 3, start: 1, count: 14, fixture_type: 1, parked: false },
      ...[[301, 1, false], [302, 49, true], [303, 81, true]].map(([no, start, parked]) => ({ no, copy: null, break: 1, name: `BMFL ${no - 300}`, universe: 2, start, count: 48, fixture_type: 2, parked }))],
    conflicts: [{ kind: 'overlap', a: { no: 302, word: 'Fixture 302', range: '2.049–2.096' }, b: { no: 303, word: 'Fixture 303', range: '2.081–2.128' },
      fixes: [{ label: '303 to next free', line: 'Assign Fixture 303 At DMX', primary: true }, { label: 'Swap 302 ⇄ 303', line: 'Assign Fixture 302 At Fixture 303 DMX /Swap' }, { label: 'Unpatch 303', line: 'Unassign Fixture 303 At DMX' }] }],
    level: 'Admin',
  });
  dm.addEventListener('line', e => { mapOut.textContent = `› ${e.detail.line}`; });
  dm.addEventListener('select', e => { mapOut.textContent = `selected: Fixture ${e.detail.fixtures.join(' + ')}`; });
  root.append(section('luma-dmxmap — the DMX map zoomed out: usage, blocks by type (copies hatched), free runs, parked overlaps and their fixes', mapBox));
  // Zoomed in (Edit Patch, Insert fixtures' sheet): universe 2's slots, 302 current, 303 over it.
  const slotsBox = div('');
  slotsBox.style.cssText = 'height:300px;display:flex;flex-direction:column;border:1px solid var(--lc-line)';
  const ds = document.createElement('luma-dmxmap');
  slotsBox.append(ds);
  ds.update({ zoom: 'slots', universe: 2, runs: dm.spec.runs, current: 302, picked: [302, 303], flash: [120, 121, 122] });
  ds.addEventListener('slot', e => { mapOut.textContent = `› (tap) universe ${e.detail.universe} slot ${e.detail.slot}${e.detail.no ? ` · Fixture ${e.detail.no}` : ''}`; });
  root.append(section('luma-dmxmap zoomed in — one universe\'s slots in rows of 20 (10 on an upright phone): the current fixture gold, the picked outlined, overlaps striped, a probe flashing', slotsBox));

  // luma-files (decisions §38 5): the one way to load and save files — this station's drives on
  // the rail, the folder's files by kind, This device, a name field for a save.
  const filesBox = div('');
  filesBox.style.cssText = 'display:flex;flex-direction:column;gap:8px';
  const filesFrame = div('');
  filesFrame.style.cssText = 'height:360px;display:flex;flex-direction:column;border:1px solid var(--lc-line)';
  const lf = document.createElement('luma-files');
  filesFrame.append(lf);
  const filesOut = div('lc-sr-note lc-selectable', 'Tap a drive, a folder; double-tap a file — what it picks shows here.');
  filesBox.append(filesFrame, filesOut);
  lf.update({ feeds: window.feeds?.() || null, kind: 'shows', save: true, level: 'Admin' });
  lf.addEventListener('pick', e => { filesOut.textContent = `› pick ${e.detail.drive} · ${e.detail.folder}/${e.detail.name}`; });
  lf.addEventListener('save', e => { filesOut.textContent = `› save ${e.detail.drive} · ${e.detail.folder}/${e.detail.name}`; });
  lf.addEventListener('where', e => { filesOut.textContent = `at ${e.detail.drive} · ${e.detail.folder || 'library'}`; });
  lf.addEventListener('upload', e => { filesOut.textContent = `uploaded ${e.detail.name} to ${e.detail.drive} · ${e.detail.folder}`; });
  root.append(section('luma-files — the one way to load and save files: the drives rail (this station\'s, live), crumbs, the folder\'s files by kind, This device, a name for a save', filesBox));

  // The phone's one sheet (decisions §37 7, sheet.js): every phone pop-up's look — from the bottom
  // upright, from the right on its side; the grab handle's swipe, a tap outside, ✕ or Esc close it.
  const sheetRow = div('lc-sr-row');
  for (const side of [false, true]) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'lc-btn';
    b.textContent = side ? 'Open from the right (on its side)' : 'Open from the bottom (upright)';
    b.onclick = () => openSheet({ title: '1 Fixture · 104', sub: 'Live patch — live columns only', side,
      body: '<p class="lc-psheet-note">At most about two-thirds of the screen, so the rows it acts on stay in sight; it scrolls inside.</p>' });
    sheetRow.append(b);
  }
  root.append(section('The phone sheet — one look for the value dialog, the context menu and the setup frame\'s tree / More / lock', sheetRow));

  // luma-tags-pool (decisions §12 "Tags hardened"): the Tags pool window — the tree and the
  // picked tag's members. Demo feed data; every gesture shows the line it would run.
  const tpFrame = div('lc-sr-tags-pool');
  tpFrame.style.cssText = 'height:420px;display:flex;flex-direction:column';
  const tp = document.createElement('luma-tags-pool');
  const tpOut = div('lc-sr-note lc-selectable', 'Tap a tag, Direct | Deep, Run action or ≡: the line it runs shows here.');
  const col = (id, label, width) => ({ id, field: id, label, kind: 'text', width, priority: 1 });
  const tpData = deep => ({
    deep, missing: false,
    nodes: treeNodes.map(n => ({ path: n.path, name: n.name, colour: n.colour, count: n.count, badge: n.badge })),
    tag: { path: '1.1', name: 'Lx1', guid: 'demo', own_action: false, lines: { select: `Tag 1.1${deep ? ' /R' : ''}`, settings: 'EditSetting Tag 1.1', label: 'Label Tag 1.1 {name}', child: 'Store Tag 1.1 Tag {name}', top: 'Move Tag 1.1 At Root', delete: 'Delete Tag 1.1' } },
    columns: [{ ...col('no', 'No', 64), frozen: true }, col('kind', 'Kind', 120), col('name', 'Name', 200), col('via', 'Tagged', 140)],
    kinds: [{ kind: 'Fixture', count: deep ? 6 : 2 }, { kind: 'Sequence', count: 1 }],
    members: [
      { guid: 'f101', no: '101', kind: 'Fixture', name: 'MegaPointe SL 1', via: 'direct' }, { guid: 'f102', no: '102', kind: 'Fixture', name: 'MegaPointe SL 2', via: 'direct' },
      { guid: 's31', no: '31', kind: 'Sequence', name: 'Lx1 chase', via: 'direct' },
      ...(deep ? [{ guid: 'f117', no: '117', kind: 'Fixture', name: 'MegaPointe SL 17', via: 'from Right' }, { guid: 'f118', no: '118', kind: 'Fixture', name: 'MegaPointe SL 18', via: 'from Right' },
        { guid: 'f121', no: '121', kind: 'Fixture', name: 'MegaPointe SL 21', via: 'from Left' }, { guid: 'f122', no: '122', kind: 'Fixture', name: 'MegaPointe SL 22', via: 'from Left' }] : []),
    ],
  });
  let tpDeep = true;
  tp.addEventListener('run', e => {
    tpOut.textContent = `› ${e.detail.line}`;
    const m = /^Set Self "Members" (\w+)$/.exec(e.detail.line);
    if (m) { tpDeep = m[1] === 'Deep'; tp.update(tpData(tpDeep), { action: 'Select', counts: true }); }
  });
  tpFrame.append(tp);
  const tpWrap = div('lc-sr-col');
  tpWrap.append(tpFrame, tpOut);
  root.append(section('luma-tags-pool — the Tags pool window: the tree, the picked tag\'s members Direct | Deep, Run action, the ≡ menu', tpWrap));
  requestAnimationFrame(() => tp.update(tpData(tpDeep), { action: 'Select', counts: true }));

  // luma-pathbar and luma-tree-window (decisions 0292): NAV's tree and inspector as a window —
  // the path bar's crumbs and its field, the tree in host mode, the inspector's copy buttons and
  // settings cells. Demo feed data; every gesture shows the line it would run (or the copy).
  const twOut = div('lc-sr-note lc-selectable', 'Tap a crumb, the bar\'s empty space, a row, a value, ↺ or a copy button: what it does shows here.');
  const pb = document.createElement('luma-pathbar');
  pb.addEventListener('jump', e => { twOut.textContent = `› Set Self "Path" "${e.detail.path}"`; pb.update({ path: e.detail.path }); });
  pb.addEventListener('go', e => { twOut.textContent = `› Set Self "Path" "${e.detail.text}"`; });
  const pbFrame = div('lc-sr-pathbar');
  pbFrame.style.cssText = 'width:min(560px,100%)';
  pbFrame.append(pb);
  const G1 = 'Root.ShowData.DataPools.1.Groups.1';
  const twRow = (path, depth, extra = {}) => ({ path, depth, kind: 'node', name: path.split('.').pop(), branch: true, open: true, meta: '', ...extra });
  const twGroup = (no, name, extra = {}) => ({ path: `Root.ShowData.DataPools.1.Groups.${no}`, depth: 4, kind: 'object', no, name, quoted: true, type: 'Group', object: 'Group', target: `Group ${no}`, branch: false, guid: `demo-${no}`, ...extra });
  const twTree = {
    path: G1, hidden: 0, cut: 0, follow: null,
    rows: [twRow('Root.ShowData', 0, { meta: '1371 fixtures · 4 users' }), twRow('Root.ShowData.Devices', 1, { open: false, meta: '1371 fixtures' }),
      twRow('Root.ShowData.DataPools', 1, { meta: '77 groups' }),
      { path: 'Root.ShowData.DataPools.1', depth: 2, kind: 'object', no: 1, name: 'Default', quoted: true, type: 'DataPool', object: 'DataPool', target: 'DataPool 1', branch: true, open: true },
      twRow('Root.ShowData.DataPools.1.Groups', 3, { meta: '77 groups' }),
      twGroup(1, 'Photon', { marks: 'a' }), twGroup(2, 'Icarus'), twGroup(3, 'Blinder'), twGroup(4, 'LedBar')],
  };
  const twField = (name, value, input, changed) => ({ name, key: name, value, input, line: `Set Group 1 "${name}" {value}`, changed, ...(changed ? { reset: `Reset Group 1 "${name}"` } : {}) });
  const twInsp = {
    title: 'Group 1 "Photon"', path: G1, target: 'Group 1', guid: '2fe94d0c-f976-429d-ad96-95e2134e4a18', name: 'Photon', marks: 'a',
    rows: [
      { section: 'identity', name: 'Slot', value: '1' }, { section: 'identity', name: 'Kind', value: 'Group' },
      { section: 'identity', name: 'GUID', value: '2fe94d0c-f976-429d-ad96-95e2134e4a18' }, { section: 'identity', name: 'Path', value: G1 },
      { section: 'info', name: 'Fixtures', value: '6' },
      { section: 'property', name: 'Mode', value: 'Scaling', field: twField('Mode', 'Scaling', { type: 'choice', options: ['None', 'Positive', 'Negative', 'Scaling', 'Additive'] }, true) },
      { section: 'property', name: 'PreserveGrid', value: 'On', field: twField('PreserveGrid', 'On', { type: 'toggle' }, false) },
      { section: 'property', name: 'Name', value: 'Photon', field: twField('Name', 'Photon', { type: 'text', max_len: 64 }, true) },
      { section: 'link', name: 'Links to', value: 'Fixture 6 "ACME PHOTON HYBRID"', link: 'Root.ShowData.Devices.Dmx.Patch.Fixtures.6' },
      { section: 'link', name: 'Links to', value: 'missing · 7c1f0e2a', missing: true },
    ],
  };
  const twFrame = div('lc-sr-tree-window');
  twFrame.style.cssText = 'height:440px;display:flex;flex-direction:column;border:1px solid var(--lc-line)';
  const tw = document.createElement('luma-tree-window');
  tw.addEventListener('run', e => { e.stopPropagation(); twOut.textContent = `› ${e.detail.line}`; });
  twFrame.append(tw);
  const twWrap = div('lc-sr-col');
  twWrap.append(pbFrame, twFrame, twOut);
  root.append(section('luma-pathbar and luma-tree-window — the Tree window: crumbs or a typed path / GUID, the tree, the inspector (copy, settings cells, links)', twWrap));
  requestAnimationFrame(() => {
    pb.update({ path: 'Root.ShowData.DataPools.1.Groups.1' });
    tw.configure({ follow: 'Selected', show: 'All', split: 45, path: G1, open: '', search: '', level: 'Admin' });
    tw.showTree(twTree);
    tw.showInspector(twInsp);
  });

  // luma-rules-window (decisions 0294): the attribute rules in order, a row tinted in its feature
  // group's colour (here the theme's pool tokens stand in for the station's resolved paints),
  // Which rule? framing the row that places a name. Every gesture shows the line it would run.
  const rwOut = div('lc-sr-note lc-selectable', 'Ask which rule places a name, drag a row onto another, edit a pattern, Delete: the line shows here.');
  const tok = n => getComputedStyle(document.documentElement).getPropertyValue(n).trim();
  const rwRow = (no, pattern, group, family, kind, paint, hit = false) => ({ no, guid: `rule-${no}`, pattern, group, family, feature: '', match: 'family', kind, paint, colour: paint, hit });
  const rwFrame = div('lc-sr-rules-window');
  rwFrame.style.cssText = 'height:420px;display:flex;flex-direction:column;border:1px solid var(--lc-line)';
  const rw = document.createElement('luma-rules-window');
  rw.addEventListener('run', e => { e.stopPropagation(); rwOut.textContent = `› ${e.detail.line}`; });
  rwFrame.append(rw);
  const rwWrap = div('lc-sr-col');
  rwWrap.append(rwFrame, rwOut);
  requestAnimationFrame(() => rw.show({
    title: 'Show', count: { standard: 4, learned: 1 }, groups: ['Dimmer', 'Position', 'Control', 'Shutter'],
    rows: [rwRow(1, 'Dimmer*', 'Dimmer', 'dimmer', 'standard', tok('--lc-pool-presets') || ''), rwRow(2, 'Pan*', 'Position', 'pantilt', 'standard', tok('--lc-pool-groups')),
      rwRow(3, 'Tilt*', 'Position', 'pantilt', 'standard', tok('--lc-pool-groups')), rwRow(4, '*Reset*', 'Control', 'control', 'standard', null),
      rwRow(5, 'Shutter{n}StrobeBurst', 'Shutter', '—', 'learned', tok('--lc-pool-sequences'), true)],
    which: { name: 'Shutter2StrobeBurst', rule: 5, group: 'Shutter', why: 'the learned rule Shutter{n}StrobeBurst', how: 'learned' },
    lines: { move: 'Move AttributeRule {from} At {to}', pattern: 'Set AttributeRule {no} "Pattern" {value}', group: 'Set AttributeRule {no} "FeatureGroup" {value}', delete: 'Delete AttributeRule {no}', store: 'Store AttributeRule {pattern} FeatureGroup {group}', info: 'Info AttributeRule {name}' },
  }, { level: 'Admin' }));

  // Window bar (12c): pinned settings + the type's actions, chips, `…` overflow.
  const wbTypes = {
    sysmon: { name: 'sysmon', bar_min: { width_px: 200, height_px: 100 },
      actions: [{ id: 'pause', label: 'Pause', line: 'Set Self AutoScroll Toggle', state: '!Self.AutoScroll' }, { id: 'clear', label: 'Clear', line: 'Set Self Since Now' }],
      settings: [{ key: 'wrap', label: 'Wrap', type: 'bool' }, { key: 'level', label: 'Level', type: 'enum', options: [['info', 'All'], ['warn', 'Warnings + errors'], ['error', 'Errors']] },
        { key: 'time', label: 'Time', type: 'enum', options: [['session', 'Session'], ['wall', 'Wall clock']] }] },
    dmx: { name: 'dmx', bar_min: { width_px: 200, height_px: 100 },
      actions: [{ id: 'prev', label: 'Universe −', icon: 'chevron-left', line: 'Set Self "Universe" - 1' }, { id: 'next', label: 'Universe +', icon: 'chevron-right', line: 'Set Self "Universe" + 1' }],
      settings: [{ key: 'universe', label: 'Universe', type: 'int', min: 1, max: 63999 }, { key: 'readout', label: 'Values', type: 'enum', options: [['percent', '%'], ['decimal', 'Dec'], ['hex', 'Hex']] },
        { key: 'view', label: 'Show input', type: 'bool' }] },
  };
  const wbState = { auto: true, wrap: false, chip: true, uni: 1 };
  const wbOut = div('lc-sr-note lc-selectable', '› tap an item');
  const wbBars = [];
  const wbViews = () => [
    ['sysmon', { bar: true, items: ['pause', 'wrap', 'level', 'time', 'clear'], states: { pause: !wbState.auto, wrap: wbState.wrap },
      values: { level: 'All', time: 'Session' }, chips: wbState.chip ? [{ id: 'search', label: '× Search “act 2”', line: 'Set Self Search ""' }] : [] }],
    ['dmx', { bar: true, items: ['universe', 'prev', 'next', 'readout', 'view'], states: { view: false }, values: { universe: String(wbState.uni), readout: '%' }, chips: [] }],
  ];
  const wbPaint = () => wbViews().forEach(([t, v], i) => wbBars[i]?.forEach(b => b.update({ type: wbTypes[t], view: v })));
  const wbRun = line => {
    wbOut.textContent = `› ${line}`;
    if (/AutoScroll/.test(line)) wbState.auto = !wbState.auto;
    if (/Wrap/.test(line)) wbState.wrap = !wbState.wrap;
    if (/Search ""/.test(line)) wbState.chip = false;
    const m = /"?Universe"? ([-+]) 1/.exec(line);
    if (m) wbState.uni = Math.max(1, wbState.uni + (m[1] === '+' ? 1 : -1));
    wbPaint();
  };
  const wb = div('lc-sr-vl');
  for (const [t, width] of [['sysmon', '100%'], ['dmx', '100%'], ['sysmon', '240px']]) {
    const frame = div('lc-sr-wb');
    frame.style.width = width;
    const bar = document.createElement('luma-window-bar');
    bar.addEventListener('run', e => wbRun(e.detail.line));
    frame.append(bar, div('lc-sr-wb-body', t === 'sysmon' ? (width === '100%' ? 'System Monitor' : 'narrow: the rest is in …') : 'DMX'));
    wb.append(frame);
    const i = t === 'sysmon' ? 0 : 1;
    (wbBars[i] ||= []).push(bar);
  }
  wb.append(wbOut);
  requestAnimationFrame(wbPaint);
  root.append(section('luma-window-bar — pinned settings, the type’s actions, chips, … when narrow (every tap a Set Self line)', wb));

  // Corner handle (12b2): brings hidden bars back (F12 full screen)
  const ch = div('lc-sr-row');
  const handle = document.createElement('button');
  handle.className = 'lc-corner-handle';
  handle.style.position = 'relative';
  handle.setAttribute('aria-label', 'Show the bars (F12)');
  handle.innerHTML = '<luma-icon name="minimize"></luma-icon>';
  ch.append(handle);
  root.append(section('Corner handle — F12 full screen: back to the bars (faded on desktop until the pointer is near)', ch));

  // Tile grid (12c2, MenuTiles board): columns auto-fill; a narrow box = rows
  const tilesRow = div('lc-sr-tiles');
  const TILES = [
    { id: 'patch', icon: 'plug', label: 'Patch', disabled: true }, { id: 'net', icon: 'network', label: 'Network' },
    { id: 'proto', icon: 'radio', label: 'DMX Protocols' }, { id: 'backup', icon: 'save', label: 'Backup' },
    { id: 'quick', icon: 'save-all', label: 'Quick Save', tone: 'please' }, { id: 'cfg', icon: 'monitor', label: 'Screen config', value: 'Follow user' },
  ];
  for (const w of ['wide', 'narrow']) {
    const box = div(`lc-sr-tilebox ${w}`);
    const tl = document.createElement('luma-tiles');
    box.append(tl);
    tilesRow.append(box);
    tl.update(TILES);
    tl.addEventListener('tile', e => toast(`Tile ${e.detail.id}`));
  }
  root.append(section('luma-tiles — Menu, Shutdown, Backup: icon 24 over one line, auto-fill columns, icon + label rows when narrow', tilesRow));

  // Toast (12c2)
  const toastRow = div('lc-sr-row');
  toastRow.append(btn('Show a toast', () => toast('Too small — hold to open Groups')));
  root.append(section('Toast — one short line that goes by itself (a quick tap on a miniature)', toastRow));

  // Held full screen + miniature (12c2, HoldFullscreen board)
  const mxWrap = div('lc-sr-mx');
  const mx = document.createElement('luma-maxbar');
  const mxBody = div('lc-sr-mx-body', 'Groups at real size — ‹ › the neighbour pane, × (or hold the title) back');
  mxWrap.append(mx, mxBody);
  const panes = ['Groups', 'DMX', 'System Monitor'];
  let at = 0;
  mx.update({ title: panes[at], sub: 'Default', count: panes.length });
  mx.addEventListener('maxbar', e => {
    if (e.detail.go === 'close') return toast('Back to the whole screen');
    at = (at + (e.detail.go === 'next' ? 1 : panes.length - 1)) % panes.length;
    mx.update({ title: panes[at] });
  });
  const miniPane = div('lc-sr-mini');
  const miniInner = div('lc-mini-inner lc-sr-mini-inner', `<div class="lc-sr-mini-cells">${Array.from({ length: 12 }, (_, i) =>
    `<button type="button" class="lc-btn">${i + 1}</button>`).join('')}</div>`);
  miniPane.append(miniInner);
  miniature(miniPane, miniInner, { natural: () => ({ w: 720, h: 300 }), name: () => 'Groups', onHold: () => toast('Held: Groups full screen'), enabled: () => true });
  root.append(section('luma-maxbar + miniature — a pane too small draws the desk smaller; tap = "Too small", hold ≈ 0.4 s = full screen', div('lc-sr-row')));
  root.lastElementChild.querySelector('.lc-sr-row').append(mxWrap, miniPane);

  // Surface (12a3): a preview that closes on Esc or the corner taps
  const sf = div('lc-sr-row');
  const preview = mode => () => {
    const s = document.createElement('luma-surface');
    const close = () => { s.remove(); document.removeEventListener('keydown', onKey, true); };
    const onKey = e => { if (e.key === 'Escape') close(); };
    document.addEventListener('keydown', onKey, true);
    for (const ev of ['surface-unlock', 'surface-corners', 'surface-login', 'surface-focus', 'surface-stop']) s.addEventListener(ev, close);
    document.body.append(s);
    s.update({
      mode, user: mode === 'login' ? null : 'Admin', waiting: mode === 'locked' ? 2 : 0,
      station: { name: 'desk1', show: 'Gala', status: { dmx: 'running', role: 'Master', lumasession: 'Festival', unsaved: true },
        users: [
          { name: 'Admin', color: 'var(--lc-accent)', rights: 'Admin', profile: 'Admin', has_password: true, remote_login: true, logged_in: ['desk1'] },
          { name: 'LD', color: 'var(--lc-park)', rights: 'Program', profile: 'LD', has_password: false, remote_login: true, logged_in: ['desk1', 'FOH'] },
          { name: 'Bob', color: 'var(--lc-info)', rights: 'Playback', profile: 'LD', has_password: true, remote_login: true, logged_in: ['FOH'] },
          { name: 'Guest', color: 'var(--lc-text-dim)', rights: 'Guest', profile: 'Guest', has_password: false, remote_login: true, logged_in: [] },
        ] },
      servers: { rows: [
        { name: 'desk1', info: '192.0.2.12:47656 · Gala', note: 'this station', current: true, dot: 'ok' },
        { name: 'FOH', info: '192.0.2.40:47656', note: 'not answering 2 s', warn: true, dot: 'warn' },
      ] },
    });
  };
  for (const [label, mode] of [['Surface: login', 'login'], ['Surface: locked', 'locked'], ['Surface: blurred screen', 'blurred']]) {
    const b = document.createElement('luma-button');
    b.setAttribute('label', label);
    b.addEventListener('click', preview(mode));
    sf.append(b);
  }
  root.append(section('luma-surface — lock · login · connect (Esc closes the preview)', sf));

  // Suggestions (12b2): the bottom bar's list and the palette, from a reply
  // as the station sends it (`core::suggest`).
  const sg = div('lc-sr-suggest');
  const item = (kind, title, subtitle, icon, extra = {}) =>
    ({ kind, title, subtitle, icon, insert: `${title} `, section: extra.section || 'Commands', status: 'ready', score: 0, actions: [], ...extra });
  const sgBar = document.createElement('luma-suggest');
  sgBar.setAttribute('mode', 'inline');
  const sgPal = document.createElement('luma-suggest');
  sgPal.setAttribute('mode', 'palette');
  sg.append(sgBar, sgPal);
  root.append(section('luma-suggest — inline above the command line, palette in full screen (↑ ↓ move here)', sg));
  sgBar.update({ line: 'Gr', replace: [0, 2], total: 6, items: [
    item('keyword', 'Group', 'Group <n> — select, At, Store, Label …', 'command'),
    item('object', 'Group 3', '"Even" · used 12×', 'box', { section: 'Objects' }),
    item('object', 'Group 1', '"All"', 'box', { section: 'Objects' }),
    item('history', 'Group 3 At 50', 'last: Group 3 At 50', 'history', { section: 'History' }),
    item('verb', 'GrandMaster', 'GrandMaster At <0–100>', 'play'),
    item('keyword', 'Grid', "isn't built yet (planned)", 'command', { status: 'planned' }),
    item('verb', 'Restart', 'Guest cannot run \'Restart\': it needs Admin.', 'play', { disabled: 'E-RIGHTS-001' }),
  ] });
  const acts = n => [{ label: 'Select', line: `Group ${n}` }, { label: 'At', line: `At Group ${n}` }, { label: 'Edit', line: `Edit Group ${n}` }, { label: 'Info', line: `Info Group ${n}` }];
  sgPal.update({ line: 'front', replace: [0, 5], total: 214, syntax: 'Group [<sel>] [At <value>]', items: [
    item('object', 'Group 4', '"Front wash" · used 12×', 'box', { section: 'Objects', actions: acts(4) }),
    item('object', 'Preset 4.2', '"Front blue"', 'box', { section: 'Objects' }),
    item('history', 'Group 4 At 80', 'last: Group 4 At 80', 'history', { section: 'History' }),
    item('verb', 'FrontView', "isn't built yet (planned)", 'play', { status: 'planned' }),
  ] });
  for (const el of [sgBar, sgPal]) {
    el.tabIndex = 0;
    el.addEventListener('keydown', e => {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); e.stopPropagation(); el.move(e.key === 'ArrowDown' ? 1 : -1); }
    });
  }

  // Virtual list (12b2): 10 000 lines, only the visible rows exist.
  const vlWrap = div('lc-sr-vl');
  const vlSearch = document.createElement('input');
  vlSearch.className = 'lc-sr-field';
  vlSearch.placeholder = 'Filter';
  const vl = document.createElement('luma-vlist');
  vl.className = 'lc-sr-vlist';
  vl.setAttribute('placeholder', 'No line matches.');
  const subsystems = ['Cmd', 'Server', 'Session', 'Render', 'Http'];
  const words = ['Group 3 → ok', 'shell resize applied', 'clock locked, offset 0.12 ms', 'frame 2.1 ms (budget 16.7)', 'GET /api/station 200', 'purrrrrrrrr'];
  const sample = n => ({ n, sub: subsystems[n % 5], text: `#${n} ${words[n % 6]}`, warn: n % 37 === 0 });
  vl.renderRow = (item, el) => {
    el.className = `lc-sr-vrow${item.warn ? ' warn' : ''}`;
    el.textContent = `${item.sub.padEnd(8)} ${item.text}`;
  };
  vl.cap = 10000;
  vl.setItems(Array.from({ length: 10000 }, (_, i) => sample(i + 1)));
  const vlState = div('lc-sr-vstate', 'LIVE');
  vl.addEventListener('vl-follow', e => { vlState.textContent = e.detail.following ? 'LIVE' : 'PAUSED'; });
  vlSearch.addEventListener('input', () => {
    const q = vlSearch.value.toLowerCase();
    vl.filter = q ? item => item.text.toLowerCase().includes(q) : null;
    vl.refilter();
  });
  let vlNext = 10001;
  const vlAdd = btn('Add 50 lines', () => vl.push(Array.from({ length: 50 }, () => sample(vlNext++))));
  vlWrap.append(div('lc-sr-row'), vl);
  vlWrap.firstChild.append(vlSearch, vlAdd, vlState);
  root.append(section('luma-vlist — a virtual list: filter in memory, scroll up pauses, rows selectable', vlWrap));

  // Graph (12b2): one point per sample, budget, late marks, a gap.
  const ring = new SampleRing(600, ['v', 'late']);
  const graph = document.createElement('luma-graph');
  graph.className = 'lc-sr-graph';
  graph.setAttribute('tone', 'accent');
  graph.budget = 16.67;
  let t = 0, lateNext = false;
  const feed = () => {
    // Waits until the Showroom shows; stops when it is gone.
    if (!graph.isConnected) { if (!graph.seen) requestAnimationFrame(feed); return; }
    graph.seen = true;
    ring.pushRow({ v: 2 + Math.sin(t++ / 23) * 0.6 + Math.random() * 0.5 + (lateNext ? 15.5 : 0), late: lateNext ? 1 : 0 });
    lateNext = false;
    graph.draw(ring, 'v', 'late');
    requestAnimationFrame(feed);
  };
  const graphRow = div('lc-sr-row');
  graphRow.append(btn('A late frame', () => { lateNext = true; }), btn('A gap (75 frames)', () => ring.gap(75)));
  const graphWrap = div('lc-sr-vl');
  graphWrap.append(graphRow, graph);
  requestAnimationFrame(feed);
  root.append(section('luma-graph — 10 s, one point per frame, dashed budget, red dot = late, shaded break = gap', graphWrap));

  // 12f-a: the attribute board (sample data; its lines show as toasts).
  const ab = div('lc-sr-row');
  for (const [label, filter, view] of [['New-attribute board', 'new', 'group'], ['All attributes', 'all', 'group'], ['By fixture type', 'new', 'type']]) {
    const b = document.createElement('luma-button');
    b.setAttribute('label', label);
    b.addEventListener('click', () => openAttrBoard({ line: t => toast(t), close: () => toast('Done'), filter, view }).update(sampleBoard()));
    ab.append(b);
  }
  const ebar = document.createElement('luma-encoder-bars');
  ebar.style.height = '420px';
  ebar.addEventListener('run', e => toast(e.detail.line));
  const sb = sampleBoard();
  sb.owner = 'UserProfile';
  sb.encoder_bars = [{ no: 1, name: 'Default', buttons: sb.groups.map(g => ({ group: g.no, slots: [...g.members] })) },
    { no: 2, name: 'Busking', buttons: sb.groups.slice(0, 4).map(g => ({ group: g.no, slots: [...g.members] })) }];
  ebar.update(sb);
  // §30 "Users & Profiles follows storage": a row's detail (desk; a phone box shows P1).
  const dt = document.createElement('luma-detail');
  dt.style.cssText = 'display:flex;height:420px';
  dt.addEventListener('run', e => toast(e.detail.line));
  dt.addEventListener('back', () => toast('‹ back to the grid'));
  root.append(section('luma-detail — a row opened from its grid: ‹ back, Settings / Content tabs, the stored path; under 600 px a screen per level', dt));
  dt.update({
    back: 'Profiles', title: 'Busking', path: 'ShowData.UserProfiles.3', owner: 'UserProfile 3', level: 'Admin',
    groups: [
      { name: 'Settings', label: 'Settings', path: 'Settings', sub: 'Look · Lock screen · Input & keys', parts: [
        { name: 'Input', label: 'Input & keys', path: 'Preferences', content: { kind: 'settings', section: '' }, actions: [] },
      ] },
      { name: 'Content', label: 'Content', path: 'Content', sub: 'Keys · Images', parts: [
        { name: 'Keys', label: 'Keys', path: 'Keybindings', content: { kind: 'list', rows: 'keys', row_key: 'key', columns: [{ id: 'key', label: 'Key', field: 'key', width: 120, priority: 1, frozen: true }, { id: 'ui_key', label: 'UI key', field: 'ui_key', width: 160, priority: 1 }] }, actions: [] },
      ] },
    ],
    fields: [{ name: 'MeowKey', key: 'meow_key', section: '', value: 'Shift', input: { type: 'choice', options: ['Shift', 'Alt', 'Ctrl'] }, line: 'Set UserProfile 3 "MeowKey" {value}' }],
    data: { keys: [{ key: 'F1', ui_key: 'user.go' }, { key: 'F2', ui_key: 'user.back' }] },
    custom: {},
  });
  // 12f: the masters as faders — every drag is a real line on the gesture channel.
  const bank = div('lc-fader-bank');
  for (const [label, sub, line, value] of [['Grand', 'Master 2.1', 'Master 2.1 At {value}', 100], ['Highlight', 'Master 2.2', 'Master 2.2 At {value}', 100],
    ['Solo', 'Master 2.4', 'Master 2.4 At {value}', 100], ['My HL', 'yours', 'Master "Highlight" At {value}', 70]]) {
    const f = document.createElement('luma-fader');
    bank.append(f);
    f.update({ label, sub, line, value });
  }
  root.append(section('luma-fader — drag (relative, a tap never jumps), a second finger = fine ×0.1, snaps at the ends; a drag is one gesture', bank));
  // 12f: the sequence sheet (window A) on sample data — its lines stop here (they'd edit the show).
  const cs = document.createElement('luma-cue-sheet');
  const csBox = div('lc-sr-cuesheet');
  csBox.style.height = '520px';
  csBox.append(cs);
  for (const ev of ['run', 'gesture']) cs.addEventListener(ev, e => e.stopPropagation());
  cs.open = '2';
  cs.update(sampleSheet());
  root.append(section('luma-cue-sheet — sequence window A: + opens a cue (feature groups, curves, handles), tap a time = slider panel, press and drag = live', csBox));
  const tl = document.createElement('luma-cue-sheet');
  const tlBox = div('lc-sr-cuesheet');
  tlBox.style.height = '420px';
  tlBox.append(tl);
  for (const ev of ['run', 'gesture']) tl.addEventListener(ev, e => e.stopPropagation());
  tl.layout = 'Timeline';
  tl.update(sampleSheet());
  root.append(section('luma-cue-sheet layout Timeline — sequence window B: a column per cue, spans over the seconds (fades in their Path), tap a head = options, + add timing, tap / drag a time = live', tlBox));
  root.append(section('luma-encoder-bars — the Encoder bar tab: layouts, strip buttons (grip reorders), 4-slot pages (drag a slot)', ebar));
  root.append(section('luma-attr-board — tap selects, drag moves (drop line / header = bottom), lasso, hold = details, hold a header = menu', ab));

  // 12d: the grid, the edit card, the slab, the power toggle, kind badges, the setup frame.
  build12d(root, { section, div, btn });
  // After 12d: its slab stays the first one in the Showroom (kit-12d reads the first).
  root.append(section('luma-rules-window — the Rules window: the attribute rules in order (first match wins), Which rule?, drag to reorder, the slab', rwWrap));
  // The key editor (decisions 0295) on sample data — its lines show here instead of running.
  const km = document.createElement('luma-keymap');
  const kmOut = div('lc-sr-note lc-selectable', 'Pick a key, edit its presses, Make it mine, Unbind: the line shows here.');
  km.addEventListener('run', e => { e.stopPropagation(); kmOut.textContent = `› ${e.detail.line}`; });
  const kmWrap = div('lc-sr-col');
  kmWrap.append(km, kmOut);
  km.show(sampleKeys(), { layer: 'PRG' });
  root.append(section('luma-keymap — the key editor: a keyboard per shortcut mode, modifier layers, a key\'s layers, its press cycle (types / runs), Make it mine, Unbind', kmWrap));
  return root;
}

// A few keys as the `keys` feed sends them (lumacore_views::keys).
export function sampleKeys() {
  const lines = (press, meow = []) => ({ press, meow, Release: '', Hold: '', ReleaseAfterHold: '', DoublePress: '' });
  const t = line => ({ line, types: true });
  const r = line => ({ line, types: false });
  const core = (key, text) => ({ key, from: 'All', source: 'core', ui_key: 'core', text, locked: true, lines: lines([r(text)]) });
  const all = [core('F9', 'DeskLock'), core('F10', 'Shortcuts'), core('F11', 'Screenshot'), core('F12', 'Sidebars'),
    { key: 'F5', from: 'All', source: 'profile', ui_key: 'user.f5', text: 'F5', locked: false, lines: lines([r('List Fixture')], [r('Info Fixture')]) },
    { key: 'Enter', from: 'All', source: 'line', text: 'Run the line', locked: true }];
  const prg = [...all,
    { key: 'E', from: 'PRG', source: 'lumacore', ui_key: 'prg.e', text: 'Edit', locked: true, lines: lines([t('Edit'), t('EditSetting')]) },
    { key: 'Alt-A', from: 'PRG', source: 'lumacore', ui_key: 'prg.alt-a', text: 'Assign', locked: true, lines: lines([t('Assign'), t('Label')], [t('Set')]) },
    { key: 'F', from: 'PRG', source: 'lumacore', ui_key: 'prg.f', text: 'Fixture', locked: true, lines: lines([t('Fixture'), t('Selection')], [t('FixtureType')]) },
    { key: '.', from: 'PRG', source: 'lumacore', ui_key: 'prg.dot', text: '.', locked: true, lines: lines([t('.'), r('Zero')]) },
    { key: 'L', from: 'PRG', source: 'off', ui_key: '', text: '', locked: true, lines: null }];
  const nav = [...all, { key: 'Y', from: 'NAV', source: 'nav', text: 'Copy…', locked: true }, { key: 'Up', from: 'NAV', source: 'nav', text: 'Move', locked: true }];
  return {
    profile: 2, name: 'Admin', meow: 'Shift', core: ['F9', 'F10', 'F11', 'F12', 'Pause', 'PrintScreen'],
    layers: [{ layer: 'All', own: 'user.', keys: all }, { layer: 'KBD', own: 'user.kbd.', keys: all }, { layer: 'PRG', own: 'user.prg.', keys: prg }, { layer: 'NAV', own: 'user.nav.', keys: nav }],
    lines: { bind: 'Bind "{key}" {lines}{mode}', mode: ' /Shortcuts "{layer}"', unbind: 'Unbind "{key}"{mode}', set: 'Set UIKey "{id}" "{prop}" {value}' },
  };
}

// A small sequence as the `sequence` feed sends it (lumacore_views::sequence).
export function sampleSheet() {
  const FG = [['Dimmer', 'var(--lc-programmer)'], ['Position', 'var(--lc-sel-line)'], ['Color', 'var(--lc-alert)'], ['Beam', 'var(--lc-text-dim)']];
  const g = (o = {}) => FG.map((_, i) => ({ fade: null, delay: null, outfade: null, outdelay: null, path: null, ...(o[i] || {}) }));
  const cue = (cue, name, fade, delay, extra = {}) => ({ cue, key: null, name, trig: 'Go', fade, outfade: 0, delay, outdelay: 0, values: 12, parts: 1,
    current: false, run: false, line: `Set Sequence 1 Cue ${cue}`, groups: g(), ...extra });
  return {
    sequence: 1, datapool: 1, name: 'Main', running: true, current: '2',
    feature_groups: FG.map(([name, color]) => ({ name, word: `FeatureGroup "${name}"`, color })),
    paths: ['Linear', 'SCurve', 'EaseIn', 'EaseOut', 'Snap'],
    cues: [
      cue('0', 'CueZero', 0, 0, { fixed: 'CueZero', values: 0 }),
      cue('1', 'Preset', 2, 0),
      cue('2', 'Verse', 3, 0, { current: true, run: true, groups: g({ 0: { delay: 1.5, path: 'SCurve' }, 1: { fade: 5 }, 2: { path: 'EaseOut' } }) }),
      cue('3', 'Chorus', 0.5, 0),
      cue('OffCue', 'OffCue', 0, 0, { fixed: 'OffCue', values: 0 }),
    ],
  };
}

// A small board as the `attributes` feed sends it (lumacore_views::attribute_board).
export function sampleBoard() {
  const G = [['Dimmer', 'var(--lc-programmer)', 'sun', ['Dimmer', 'BackgroundDimmer']], ['Position', 'var(--lc-sel-line)', 'move', ['Pan', 'Tilt', 'PanRotate', 'TiltRotate', 'XYZ_X']],
    ['Color', 'var(--lc-alert)', 'palette', ['ColorAdd_R', 'ColorAdd_G', 'ColorAdd_B', 'ColorAdd_W', 'CTO', 'ColorAdd_RY', 'ColorRGB_Red']],
    ['Shutter', 'var(--lc-white)', 'zap', ['Shutter1', 'Shutter1Strobe', 'Shutter2StrobeBurst']], ['FX', 'var(--lc-blind)', 'sparkles', ['Gobo1', 'Gobo1Pos', 'Gobo2', 'Gobo2PosShake', 'Zoom', 'Focus1', 'Iris', 'Pixel Pattern']],
    ['Control', 'var(--lc-text-dim)', 'sliders-horizontal', ['LampControl', 'ColorMSpeed', 'PixelMSpeed', 'Pan Invert', 'CrowdLED']], ['Shapers', 'var(--lc-warn)', 'crop', ['Blade1A', 'Blade1B']], ['Video', 'var(--lc-ok)', 'video', []]];
  const NEW = { BackgroundDimmer: ['hint', '"Background" + Dimmer'], ColorAdd_RY: ['family', 'alias of the RGB family'], ColorRGB_Red: ['family', 'Color · Indirect'],
    Shutter2StrobeBurst: ['family', 'family Shutter{n}Strobe*'], Gobo2PosShake: ['parent', 'its main attribute Gobo2'], 'Pixel Pattern': ['hint', 'keyword "pattern"'],
    PixelMSpeed: ['family', 'family *MSpeed'], 'Pan Invert': ['hint', 'keyword "invert"'], CrowdLED: ['guess', 'nothing matched'] };
  const TYPE = n => (/Pixel|ColorAdd|ColorRGB|Background/.test(n) ? 'ACME Pixel Line' : /Pan Invert|CrowdLED/.test(n) ? 'Cyclone Fan' : 'Robe MegaPointe 2');
  const attrs = G.flatMap(([, , , m]) => m).map(n => ({ name: n, pretty: n, new: !!NEW[n], used: n !== 'Blade1B', how: NEW[n]?.[0] ?? 'exact',
    why: NEW[n]?.[1] ?? 'standard name', file: NEW[n] ? 'Beam.Beam' : null, main: n === 'Gobo2PosShake' ? 'Gobo2' : null, feature: null, show_group: null,
    types: [TYPE(n)], readout: 'Percent', resolution: 'Coarse' }));
  const types = ['ACME Pixel Line', 'Cyclone Fan', 'Robe MegaPointe 2'].map(t => ({ name: t, attrs: attrs.filter(a => a.types[0] === t || a.name === 'Dimmer').map(a => a.name) }));
  return { target: 'show', title: 'Show', owner: '', locked: false, template: 'LumaCore FX', templates: ['LumaCore FX', 'GDTF standard'],
    groups: G.map(([name, color, ic, members], i) => ({ no: i + 1, name, color, icon: ic, members })), attrs, types };
}
