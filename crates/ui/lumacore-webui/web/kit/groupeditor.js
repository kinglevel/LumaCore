// The group editor (decisions §22, §30 "12e": the GroupTools board of
// mockups/2026-10-13-12e). `Edit Group n` claims the group; this draws it from
// the station's `pool_editor` feed and turns every gesture into the caller's
// command line:
//
//   the member list  <luma-grid reorder>: a grip (44 px) drags a member, ↑ ↓
//                    step it → `Set Group g Fixture f "Order" n`; the rows
//                    scroll freely; a tap selects (the canvas follows)
//   the slab         Remove (the edit slab's Delete: `Remove Fixture …`), Oops,
//                    Rotate 90°, Mirror X / Y on the one selected member, and
//                    Tools (the yellow `===`: it opens the toolbar)
//   the canvas       members on their grid spots; a drag (pointer core,
//                    capture, cancel) → `Set Group g Fixture f "X" x; … "Y" y`
//   the tools        desk: the grouped toolbar docked at the bottom (stays
//                    open); phone: Remove · Tools at the bottom, Tools opens
//                    the low sheet. Each tool: `Grid Group g "<Tool>" [arg]`.
//
//   const ed = openGroupEditor({ line: text => …, exit: () => …, types })
//   ed.update(data)   data = { no, name, mode, preserve_grid, members: [{ fixture,
//                     name, x, y, rotation, mirror_x, mirror_y, missing }] }
//   ed.close()

import { icon } from './icon.js';
import { drag } from './pointer.js';
import { toolbar, openToolSheet } from './tools.js';
import { listMark } from './button.js';
import './grid.js';
import './slab.js';

const CELL = 44;
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

const COLUMNS = [
  // Grip 48 + No 56 + these + ↑ ↓ 96 fit the editor's 460 px column.
  { id: 'fixture', label: 'Fix', kind: 'int', width: 64, frozen: true },
  { id: 'name', label: 'Name', width: 120 },
  { id: 'spot', label: 'X / Y', width: 64 },
  { id: 'turn', label: 'Turn', width: 70, priority: 2 },
];

export function openGroupEditor({ line, exit, types }) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ged-scrim';
  scrim.dataset.kit = 'group-editor';
  const card = document.createElement('div');
  card.className = 'lc-card lc-ged';
  card.innerHTML = `<div class="lc-box-title lc-ged-title"><span class="lc-ged-name"></span>`
    + `<button type="button" class="lc-close" aria-label="Close (exit)">${icon('x')}</button></div>`
    + '<div class="lc-ged-body"><div class="lc-ged-left"><luma-slab></luma-slab><luma-grid></luma-grid></div>'
    + '<div class="lc-ged-canvas"><div class="lc-ged-plane"></div></div></div>'
    + '<div class="lc-ged-dock"></div>'
    + `<div class="lc-ged-bottom"><button type="button" class="lc-ged-btn danger" data-remove>${icon('trash-2')}<span>Remove</span></button>`
    + `<button type="button" class="lc-ged-btn" data-tools>${icon('wrench')}<span>Tools</span>${listMark()}</button></div>`;
  scrim.appendChild(card);
  document.body.appendChild(scrim);
  const slab = card.querySelector('luma-slab');
  const grid = card.querySelector('luma-grid');
  const plane = card.querySelector('.lc-ged-plane');
  const dock = card.querySelector('.lc-ged-dock');
  let data = null, sel = [], barOpen = false;
  // Never redraw the canvas under a finger: an update waits for the drag's end.
  let dragging = false, stale = false;

  const phone = () => card.clientWidth < 600;
  const g = () => data?.no;
  const remove = () => { if (sel.length) line(`Remove Fixture ${sel.join(' + ')}`); };
  const toolSpec = () => ({
    types,
    title: `Tools · Group ${g()}`,
    line: (t, arg) => `Grid Group ${g()} "${t.id}"${arg == null ? '' : typeof arg === 'number' ? ` ${arg}` : ` "${arg}"`}`,
    run: line,
  });
  const openTools = () => {
    if (phone()) { openToolSheet(toolSpec()); return; }
    barOpen = !barOpen;
    drawDock();
    drawSlab();
  };
  const drawDock = () => {
    dock.innerHTML = '';
    if (barOpen && !phone()) dock.appendChild(toolbar({ ...toolSpec(), onClose: () => { barOpen = false; drawDock(); drawSlab(); } }));
  };
  const one = () => data?.members.find(m => sel.length === 1 && m.fixture === sel[0]);
  const drawSlab = () => {
    slab.update({
      edit: { delete: sel.length ? { line: 'remove' } : { off: 'Select members first' }, oops: { line: 'Oops' } },
      count: sel.length,
      // The slab sits in a narrow column: its phone layout shows these first.
      phone: ['tools', 'rotate', 'mirrorx', 'mirrory'],
      phone_edit: ['delete', 'oops'],
      actions: [
        { id: 'rotate', label: 'Rotate 90°', icon: 'rotate-cw', needs: 'one' },
        { id: 'mirrorx', label: 'Mirror X', icon: 'flip-horizontal-2', needs: 'one' },
        { id: 'mirrory', label: 'Mirror Y', icon: 'flip-vertical-2', needs: 'one' },
        { id: 'tools', label: 'Tools', icon: 'wrench', list: true },
      ],
    });
  };
  slab.addEventListener('act', e => {
    const { id } = e.detail;
    const m = one();
    const base = m && `Set Group ${g()} Fixture ${m.fixture}`;
    if (id === 'delete') remove();
    else if (id === 'oops') line('Oops');
    else if (id === 'tools') openTools();
    else if (id === 'rotate' && m) line(`${base} "Rotation" ${(m.rotation + 90) % 360}`);
    else if (id === 'mirrorx' && m) line(`${base} "MirrorX" ${m.mirror_x ? 'Off' : 'On'}`);
    else if (id === 'mirrory' && m) line(`${base} "MirrorY" ${m.mirror_y ? 'Off' : 'On'}`);
  });
  grid.addEventListener('select', e => { sel = e.detail.rows.map(Number); drawSlab(); drawPlane(); });
  grid.addEventListener('reorder', e => line(`Set Group ${g()} Fixture ${e.detail.key} "Order" ${e.detail.to + 1}`));
  card.querySelector('[data-remove]').onclick = remove;
  card.querySelector('[data-tools]').onclick = openTools;
  const close = () => { scrim.remove(); };
  card.querySelector('.lc-close').onclick = () => { close(); exit(); };
  new ResizeObserver(() => { card.classList.toggle('phone', phone()); drawDock(); }).observe(card);

  const drawGrid = () => {
    const rows = data.members.filter(m => !m.missing).map((m, i) => ({
      fixture: m.fixture, gutter: i + 1, name: m.name ?? '', spot: `${m.x} / ${m.y}`,
      turn: `${m.rotation ? `${m.rotation}°` : ''}${m.mirror_x ? ' ↔' : ''}${m.mirror_y ? ' ↕' : ''}`.trim(),
    }));
    grid.update({ key: `group-${g()}`, object: 'Fixture', id: 'fixture', row_key: 'fixture', columns: COLUMNS, rows, reorder: true,
      state: { set: 'Full' }, empty: 'No members — select fixtures and Store Group /Merge' });
    if (sel.length) grid.select(sel.map(String));
  };
  const drawPlane = () => {
    if (dragging) { stale = true; return; }
    const live = data.members.filter(m => !m.missing);
    const maxX = Math.max(8, ...live.map(m => m.x + 2)), maxY = Math.max(4, ...live.map(m => m.y + 2));
    plane.style.width = `${maxX * CELL}px`;
    plane.style.height = `${maxY * CELL}px`;
    plane.innerHTML = '';
    live.forEach((m, i) => {
      const c = document.createElement('div');
      c.className = 'lc-ged-cell' + (sel.includes(m.fixture) ? ' on' : '');
      c.style.left = `${m.x * CELL}px`;
      c.style.top = `${m.y * CELL}px`;
      c.innerHTML = `<b style="transform:rotate(${m.rotation}deg) scale(${m.mirror_x ? -1 : 1}, ${m.mirror_y ? -1 : 1})">${esc(m.fixture)}</b><span>${i + 1}</span>`;
      let moved = false;
      drag(c, {
        onStart: () => { moved = false; dragging = true; c.classList.add('lifted'); },
        onMove: ({ dx, dy }) => {
          if (Math.hypot(dx, dy) > 4) moved = true;
          c.style.transform = `translate(${dx}px, ${dy}px)`;
        },
        onEnd: ({ dx, dy }, cancelled) => {
          dragging = false;
          c.classList.remove('lifted');
          c.style.transform = '';
          if (stale) { stale = false; queueMicrotask(drawPlane); }
          if (cancelled) return;
          if (!moved) { sel = [m.fixture]; grid.select([String(m.fixture)]); drawSlab(); drawPlane(); return; }
          const x = Math.max(0, m.x + Math.round(dx / CELL)), y = Math.max(0, m.y + Math.round(dy / CELL));
          if (x !== m.x || y !== m.y) line(`Set Group ${g()} Fixture ${m.fixture} "X" ${x}; Set Group ${g()} Fixture ${m.fixture} "Y" ${y}`);
        },
      });
      plane.appendChild(c);
    });
  };

  return {
    update(d) {
      data = d;
      const live = new Set(d.members.map(m => m.fixture));
      sel = sel.filter(f => live.has(f));
      card.querySelector('.lc-ged-name').textContent =
        `Edit Group ${d.no}${d.name ? ` "${d.name}"` : ''} · ${d.members.length} fixtures · Master ${d.mode}`;
      drawGrid();
      drawPlane();
      drawSlab();
    },
    close,
    isOpen: () => scrim.isConnected,
  };
}
