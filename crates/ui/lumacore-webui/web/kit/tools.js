// Editor tools (decisions §30 "12e": the GroupTools board of
// mockups/2026-10-13-12e). The station's registry (`GET /api/tool-types`:
// groups, each tool's icon, question and status) drawn two ways:
//
//   toolbar(spec)       the desk: a toolbar docked at the bottom of an editor,
//                       every group side by side with its label above it — no
//                       drill-down; new tools and groups grow it sideways. It
//                       stays open after a tool (× closes it).
//   openToolSheet(spec) a phone: the same groups in a low sheet at the bottom
//                       (each group one row that scrolls sideways); it closes
//                       after a tool.
//
// A tool that asks (a choice or a number) carries the yellow `===` list mark
// and opens its question first (choices with their icons, or − n +, the line
// it will send, Cancel / Run); a planned tool is greyed and says so. Every
// tool is the caller's ONE command line: spec.line(tool, arg) → string,
// spec.run(line).
//
//   spec = { types: { groups, tools }, line: (tool, arg) => 'Grid Group 3 "…"',
//            run: line => …, onClose?: () => … }

import { icon } from './icon.js';
import { listMark } from './button.js';
import { toast } from './toast.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const words = s => String(s).replace(/([a-z])([A-Z])/g, '$1 $2');

let tools = null;
/** The registry (cached; static on the station). */
export async function toolTypes() {
  if (!tools) tools = fetch('/api/tool-types').then(r => r.json()).catch(() => { tools = null; return { groups: [], tools: [] }; });
  return tools;
}

function button(t) {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = 'lc-tool' + (t.planned ? ' planned' : '');
  b.dataset.tool = t.id;
  b.title = t.planned ? `${t.label} (planned)` : t.label;
  b.innerHTML = `${icon(t.icon)}<b>${esc(t.label)}</b>${t.planned ? '<small>planned</small>' : t.ask ? listMark() : '<small></small>'}`;
  return b;
}

function groupsInto(host, types, onTool) {
  for (const g of types.groups) {
    const list = types.tools.filter(t => t.group === g);
    if (!list.length) continue;
    const box = document.createElement('div');
    box.className = 'lc-tool-group';
    box.innerHTML = `<span class="lc-tool-glabel">${esc(g)}</span>`;
    const row = document.createElement('div');
    row.className = 'lc-tool-row';
    for (const t of list) {
      const b = button(t);
      b.addEventListener('click', () => onTool(t));
      row.appendChild(b);
    }
    box.appendChild(row);
    host.appendChild(box);
  }
}

/** Run `t`: a planned one refuses, one that asks asks first. */
function choose(t, spec, after) {
  if (t.planned) { toast(`${t.label} isn't built yet (planned)`); return; }
  if (!t.ask) { spec.run(spec.line(t, null)); after?.(); return; }
  ask(t, spec, after);
}

/** A tool's question (the board: choices with icons or − n +, the line, Cancel / Run). */
export function ask(t, spec, after) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-tool-scrim';
  scrim.dataset.kit = 'tool-ask';
  const box = document.createElement('div');
  box.className = 'lc-box lc-tool-ask';
  scrim.appendChild(box);
  let value = t.ask.kind === 'number' ? t.ask.default : t.ask.options[0].value;
  const close = () => { scrim.remove(); document.removeEventListener('keydown', onKey, true); };
  const onKey = e => { if (e.key === 'Escape') { e.stopPropagation(); close(); } };
  const draw = () => {
    const line = spec.line(t, value);
    box.innerHTML = `<div class="lc-box-title lc-tool-ask-title">${icon(t.icon)}<span>${esc(t.label)}</span>`
      + `<button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div>`
      + `<div class="lc-box-body lc-tool-ask-body"><div class="lc-tool-ask-label">${esc(t.ask.label)}</div>`
      + (t.ask.kind === 'number'
        ? `<div class="lc-tool-num"><button type="button" data-step="-1" aria-label="Less">−</button><span>${esc(value)}</span><button type="button" data-step="1" aria-label="More">+</button></div>`
        : `<div class="lc-tool-choices">${t.ask.options.map(o => `<button type="button" data-v="${esc(o.value)}" aria-pressed="${o.value === value}">${icon(o.icon)}<span>${esc(words(o.value))}</span></button>`).join('')}</div>`)
      + `<code class="lc-tool-line">${esc(line)}</code></div>`
      + '<div class="lc-tool-foot"><button type="button" data-cancel>Cancel</button><button type="button" class="run" data-run>Run</button></div>';
    box.querySelector('.lc-close').onclick = close;
    box.querySelector('[data-cancel]').onclick = close;
    box.querySelector('[data-run]').onclick = () => { close(); spec.run(line); after?.(); };
    for (const b of box.querySelectorAll('[data-v]')) b.onclick = () => { value = b.dataset.v; draw(); };
    for (const b of box.querySelectorAll('[data-step]')) b.onclick = () => {
      value = Math.max(t.ask.min, Math.min(t.ask.max, value + Number(b.dataset.step)));
      draw();
    };
  };
  draw();
  scrim.addEventListener('click', e => { if (e.target === scrim) close(); });
  document.addEventListener('keydown', onKey, true);
  document.body.appendChild(scrim);
}

/** The desk's docked toolbar (stays open after a tool). */
export function toolbar(spec) {
  const bar = document.createElement('div');
  bar.className = 'lc-toolbar';
  bar.dataset.kit = 'toolbar';
  const x = document.createElement('div');
  x.className = 'lc-toolbar-x';
  x.innerHTML = `<button type="button" class="lc-close" aria-label="Close the tools">${icon('x')}</button>`;
  x.querySelector('button').onclick = () => spec.onClose?.();
  bar.appendChild(x);
  groupsInto(bar, spec.types, t => choose(t, spec));
  return bar;
}

/** The phone's low sheet (closes after a tool). */
export function openToolSheet(spec) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-sheet-scrim lc-tool-sheet';
  scrim.dataset.kit = 'tool-sheet';
  const box = document.createElement('div');
  box.className = 'lc-box lc-sheet';
  box.innerHTML = `<div class="lc-box-title lc-sheet-title"><span>${esc(spec.title || 'Tools')}</span>`
    + `<button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div>`;
  const body = document.createElement('div');
  body.className = 'lc-box-body lc-tool-sheet-body';
  box.appendChild(body);
  scrim.appendChild(box);
  const close = () => { scrim.remove(); spec.onClose?.(); };
  box.querySelector('.lc-close').onclick = close;
  scrim.addEventListener('click', e => { if (e.target === scrim) close(); });
  groupsInto(body, spec.types, t => { scrim.remove(); choose(t, spec, spec.onClose); });
  document.body.appendChild(scrim);
  return close;
}
