// <luma-attr-board> — the attribute board (decisions §30 "12f-a", board D of
// mockups/2026-10-14-12f/Attr-Round07): one element for every attribute
// editor — the show's feature groups (the patch's definitions) and a
// profile's own groups (your encoder bar), opened by `Edit [UserProfile]
// Attribute [/New|/Hint|/Unused] [/By FixtureType]`. It draws the station's
// `attributes` feed and turns every gesture into the caller's command line:
//
//   tap a tile          selects it (again: deselects); a lasso on empty space
//                       (mouse / pen) selects many
//   drag a tile         moves the selection: onto a header or below the last
//                       tile → `Assign … Attribute "a" + "b" At … FeatureGroup "G"`
//                       (the bottom), between tiles (the gold drop line) →
//                       `Move … Attribute … At FeatureGroup "G" <n>`; the
//                       board scrolls near its edges; a single digit-named
//                       attribute offers "Apply to similar names?" →
//                       `Store … AttributeRule "<pattern>" FeatureGroup "G"`
//   hold a tile         details (popover; a sheet on a phone): its group
//                       (a pick list), why it is there, what the file says,
//                       fixture types, readout / resolution, Select all
//                       <family>, Make a rule, Info
//   hold a header       (or right-click) the group menu: Sort by name /
//                       family / main first, Merge into …, Split by family,
//                       Move left / right (an empty group is removed by
//                       its command line)
//   "+" column          `Store … FeatureGroup` (the station asks the name)
//   chips               All · New n · Hints & guesses n · Unused n; By group /
//                       By fixture type; search highlights
//   bottom              Assign n to group (pick list) · Deselect · Accept n
//                       (`Default … Attribute /New`) · Done
//
//   const b = openAttrBoard({ line, close, filter, view })
//   b.update(data)   data = the `attributes` feed (lumacore_views::attribute_board)
//   b.close()

import { icon } from './icon.js';
import { claim, hold, holdTime } from './pointer.js';
import { openMenu } from './menu.js';
import { openPick } from './popup.js';
import { listMark } from './button.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const q = n => `"${String(n).replace(/"/g, '')}"`;
const SLOP = 8;
const EDGE = 48; // px near an edge that scrolls while dragging
const FILTERS = [['all', 'All'], ['new', 'New'], ['doubt', 'Hints & guesses'], ['unused', 'Unused']];
const doubt = a => a.new && (a.how === 'hint' || a.how === 'guess');
// `Shutter2StrobeBurst` → `Shutter{n}StrobeBurst`: the family a rule names.
const family = n => n.replace(/\d+/g, '#');
const pattern = n => n.replace(/\d+/g, '*');
const READOUTS = ['Percent', 'Physical'];
const RESOLUTIONS = ['Coarse', 'Fine', 'Ultra'];

export function openAttrBoard({ line, close: onClose, filter = 'all', view = 'group' }) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ab-scrim';
  scrim.dataset.kit = 'attr-board';
  const card = document.createElement('div');
  card.className = 'lc-card lc-ab';
  card.innerHTML = `<div class="lc-box-title lc-ab-title"><b></b><small></small><span class="lc-ab-prog"></span>`
    + `<button type="button" class="lc-ab-more" aria-label="Grouping">${icon('ellipsis')}</button>`
    + `<button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div>`
    + '<div class="lc-ab-tools"></div><div class="lc-ab-scrub" hidden></div>'
    + '<div class="lc-ab-board"></div><div class="lc-ab-bottom"></div>';
  scrim.appendChild(card);
  document.body.appendChild(scrim);
  const $ = s => card.querySelector(s);
  const board = $('.lc-ab-board');
  let data = null, sel = new Set(), open = new Set(), search = '', pop = null, toast = null;
  // Never redraw under a finger: an update waits for the gesture's end.
  let busy = false, stale = false;

  const pre = () => (data?.owner ? `${data.owner} ` : '');
  const attr = n => data.attrs.find(a => a.name === n);
  const groupOf = n => data.groups.find(g => g.members.includes(n));
  const names = list => list.map(q).join(' + ');
  const phone = () => card.clientWidth < 600;
  const counts = () => ({
    new: data.attrs.filter(a => a.new).length,
    doubt: data.attrs.filter(doubt).length,
    unused: data.attrs.filter(a => !a.used).length,
  });
  const gColor = g => g.color || 'var(--lc-text-dim)';
  const gIcon = g => icon(g.icon || 'square');

  // ── Lines ──
  const moveTo = (list, g, at) => {
    if (!list.length) return;
    if (at == null) line(`Assign ${pre()}Attribute ${names(list)} At ${pre()}FeatureGroup ${q(g.name)}`);
    else line(`Move ${pre()}Attribute ${names(list)} At FeatureGroup ${q(g.name)} ${at}`);
    sel.clear();
    if (list.length === 1 && /\d/.test(list[0])) toast = { pat: pattern(list[0]), g: g.name };
  };
  const pickGroup = async (title, except) => {
    const items = data.groups.filter(g => g.name !== except).map(g => ({ no: g.no, name: g.name }));
    const r = await openPick({ title, pick: { kind: 'FeatureGroup', items } });
    return r?.no != null ? data.groups.find(g => g.no === r.no) : null;
  };
  const assign = async list => {
    const g = await pickGroup(`Assign ${list.length} to group`);
    if (g) { moveTo(list, g, null); draw(); }
  };

  // ── Drawing ──
  function tileHtml(a, { dest = false } = {}) {
    const g = groupOf(a.name);
    const cls = ['lc-ab-tile', `how-${a.how}`];
    if (a.new) cls.push('new');
    else if (a.how === 'user' && filter !== 'all') cls.push('placed');
    if (!a.used) cls.push('unused');
    if (sel.has(a.name)) cls.push('sel');
    const s = search.toLowerCase();
    if (s && !a.name.toLowerCase().includes(s)) cls.push('dim');
    else if (s) cls.push('hit');
    if (filter === 'doubt' && !doubt(a)) cls.push('dim');
    const note = !a.used ? 'no patched fixture' : a.new ? a.why : a.how === 'user' ? 'placed by you' : a.how;
    const cmp = a.show_group && g && a.show_group !== g.name ? `<span class="lc-ab-cmp">show: ${esc(a.show_group)}</span>` : '';
    const d = dest && g ? `<span class="lc-ab-dest" data-dest="${esc(a.name)}" style="--g:${gColor(g)}">→ ${gIcon(g)}${esc(g.name)}${listMark()}</span>` : '';
    return `<div class="${cls.join(' ')}" data-att="${esc(a.name)}"><b title="${esc(a.name)}">${esc(a.name)}</b>`
      + `<small>${esc(note)}</small>${d}${dest ? '' : cmp}</div>`;
  }
  function colHtml(g) {
    const all = g.members.map(attr).filter(Boolean);
    const list = filter === 'unused' ? all : all.filter(a => a.used);
    const featured = filter === 'unused' ? all.filter(a => !a.used) : filter === 'doubt' ? list.filter(doubt) : list.filter(a => a.new);
    const old = list.filter(a => !featured.includes(a));
    const fresh = all.filter(a => a.new).length;
    const pl = (a, i) => (i && i % 4 === 0 ? `<div class="lc-ab-pline">page ${i / 4 + 1}</div>` : '') + tileHtml(a);
    let body;
    if (filter === 'all') body = list.map(pl).join('');
    else if (!open.has(g.no) && !search) {
      body = (old.length ? `<button type="button" class="lc-ab-fold" data-unfold="${g.no}">+${old.length} existing ${icon('chevron-down')}</button>` : '')
        + featured.map(a => tileHtml(a)).join('');
    } else {
      body = old.map(pl).join('') + (old.length ? `<button type="button" class="lc-ab-fold" data-refold="${g.no}">fold ${icon('chevron-up')}</button>` : '')
        + featured.map(a => tileHtml(a)).join('');
    }
    return `<div class="lc-ab-col" data-drop="${g.no}" style="--g:${gColor(g)}"><div class="lc-ab-head" data-head="${g.no}">`
      + `${gIcon(g)}<span>${esc(g.name)}</span><em>${all.filter(a => a.used).length}${fresh ? ` <b>+${fresh}</b>` : ''}</em></div>`
      + `<div class="lc-ab-body" data-body="${g.no}">${body}</div></div>`;
  }
  function typeView() {
    const shown = data.types.filter(t => filter === 'all' || t.attrs.some(n => attr(n)?.new));
    if (!shown.length) return '<div class="lc-ab-empty">No fixture type brings new attributes.</div>';
    return `<div class="lc-ab-types">${shown.map(t => {
      const mine = new Set(t.attrs);
      const fresh = t.attrs.filter(n => attr(n)?.new).length;
      const cols = data.groups.map(g => {
        const list = g.members.filter(n => mine.has(n)).map(attr).filter(Boolean);
        const old = list.filter(a => !a.new && a.how !== 'user');
        const tiles = list.filter(a => a.new || a.how === 'user');
        const minis = old.length > 2 ? [old[0].name, `+${old.length - 1}`] : old.map(a => a.name);
        return `<div class="lc-ab-col mini${list.length ? '' : ' empty'}" data-drop="${g.no}" style="--g:${gColor(g)}">`
          + `<div class="lc-ab-head" data-head="${g.no}" title="${esc(g.name)}">${gIcon(g)}<span>${esc(g.name)}</span></div>`
          + `<div class="lc-ab-body">${minis.length ? `<div class="lc-ab-old" title="${esc(old.map(a => a.name).join(', '))}">${minis.map(n => `<span>${esc(n)}</span>`).join('')}</div>` : ''}`
          + `${tiles.map(a => tileHtml(a, { dest: true })).join('')}</div></div>`;
      }).join('');
      return `<section class="lc-ab-type"><h5>${esc(t.name)} <small>${fresh} new</small></h5>`
        + `<div class="lc-ab-mb" style="--cols:${data.groups.length}">${cols}<div class="lc-ab-col plus mini" data-plus="1"><button type="button" aria-label="New group">${icon('plus')}</button></div></div></section>`;
    }).join('')}</div>`;
  }
  function draw() {
    if (!data) return;
    if (busy) { stale = true; return; }
    stale = false;
    card.classList.toggle('phone', phone());
    const c = counts();
    const show = data.target === 'show';
    $('.lc-ab-title b').textContent = filter === 'new'
      ? (show ? 'New attributes · show groups' : 'New attributes for your encoder bar')
      : (show ? 'Attribute definitions · show' : `Attributes · ${data.owner === 'UserProfile' ? 'your groups' : data.title}`);
    $('.lc-ab-title small').textContent = show ? `Template ${data.template}` : `UserProfile ${q(data.title)} · ${data.template}`;
    $('.lc-ab-prog').textContent = c.new ? `${c.new} new` : '';
    $('.lc-ab-tools').innerHTML = FILTERS.map(([id, label]) =>
      `<button type="button" class="lc-ab-chip" data-filter="${id}" aria-pressed="${filter === id}">${label}${id === 'all' ? '' : ` <span>${c[id]}</span>`}</button>`).join('')
      + `<span class="lc-ab-seg"><button type="button" data-view="group" aria-pressed="${view === 'group'}">By group</button>`
      + `<button type="button" data-view="type" aria-pressed="${view === 'type'}">By fixture type</button></span>`
      + `<input class="lc-ab-search" type="search" placeholder="Search…" spellcheck="false" value="${esc(search)}">`;
    const keep = board.scrollLeft;
    board.classList.toggle('types', view === 'type');
    board.style.setProperty('--cols', data.groups.length);
    board.innerHTML = view === 'type' ? typeView()
      : data.groups.map(colHtml).join('') + `<div class="lc-ab-col plus" data-plus="1"><button type="button" aria-label="New group">${icon('plus')}</button></div>`;
    board.scrollLeft = keep;
    // By fixture type on a phone: every type's row scrolls together.
    const rows = [...board.querySelectorAll('.lc-ab-mb')];
    rows.forEach(r => r.addEventListener('scroll', () => {
      rows.forEach(o => { if (o !== r && o.scrollLeft !== r.scrollLeft) o.scrollLeft = r.scrollLeft; });
    }, { passive: true }));
    const scrub = $('.lc-ab-scrub');
    scrub.hidden = !phone() || view === 'type';
    if (!scrub.hidden) {
      scrub.style.setProperty('--n', data.groups.length);
      scrub.innerHTML = `<div class="lc-ab-segs">${data.groups.map(g => `<span style="--g:${gColor(g)}" title="${esc(g.name)}">${gIcon(g)}${g.members.some(n => attr(n)?.new) ? '<b></b>' : ''}</span>`).join('')}</div><i></i><small></small>`;
      scrubShow();
    }
    $('.lc-ab-bottom').innerHTML = `<button type="button" class="lc-ab-btn" data-assign ${sel.size ? '' : 'disabled'}>Assign ${sel.size || ''} to group${listMark()}</button>`
      + `<button type="button" class="lc-ab-btn" data-deselect ${sel.size ? '' : 'disabled'}>Deselect</button>`
      + (c.new ? `<button type="button" class="lc-ab-btn" data-accept>Accept ${c.new}</button>` : '')
      + '<span class="lc-ab-sp"></span><button type="button" class="lc-ab-btn ok" data-done>Done</button>'
      + (toast ? `<div class="lc-ab-toast"><span>Apply to similar names?</span><input value="${esc(toast.pat)}" spellcheck="false"><span>→ <b>${esc(toast.g)}</b></span>`
        + '<button type="button" class="lc-ab-btn ok" data-learn>Apply + learn</button><button type="button" class="lc-ab-btn" data-only>Only these</button></div>' : '');
  }

  // ── The phone's group scrubber: a segment per group, the visible window ──
  function scrubShow() {
    const scrub = $('.lc-ab-scrub');
    const w = board.scrollWidth || 1;
    const win = scrub.querySelector('i');
    if (!win) return;
    win.style.left = `${(board.scrollLeft / w) * 100}%`;
    win.style.width = `${(board.clientWidth / w) * 100}%`;
    const cols = [...board.querySelectorAll('.lc-ab-col[data-drop]')];
    const seen = cols.filter(c => c.offsetLeft + c.offsetWidth / 2 >= board.scrollLeft && c.offsetLeft + c.offsetWidth / 2 <= board.scrollLeft + board.clientWidth);
    scrub.querySelector('small').textContent = seen.map(c => data.groups.find(g => String(g.no) === c.dataset.drop)?.name).join(' · ');
  }
  board.addEventListener('scroll', () => { if (!$('.lc-ab-scrub').hidden) scrubShow(); }, { passive: true });
  const scrubTo = e => {
    const r = $('.lc-ab-scrub').getBoundingClientRect();
    board.scrollLeft = ((e.clientX - r.left) / r.width) * board.scrollWidth - board.clientWidth / 2;
  };
  $('.lc-ab-scrub').addEventListener('pointerdown', e => {
    if (!claim(e)) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    scrubTo(e);
    const move = ev => scrubTo(ev);
    const up = () => { e.currentTarget.removeEventListener('pointermove', move); };
    e.currentTarget.addEventListener('pointermove', move);
    e.currentTarget.addEventListener('pointerup', up, { once: true });
    e.currentTarget.addEventListener('pointercancel', up, { once: true });
  });

  // ── Details (hold a tile) ──
  function details(name, at) {
    closePop();
    const a = attr(name);
    if (!a) return;
    const g = groupOf(name);
    const fam = data.attrs.filter(x => family(x.name) === family(name)).length;
    const seg = (prop, list, cur) => `<div class="lc-ab-segs2">${list.map(v =>
      `<button type="button" data-prop="${prop}" data-v="${v}" class="${String(cur) === v ? 'on' : ''}">${v}</button>`).join('')}</div>`;
    pop = document.createElement('div');
    pop.className = 'lc-ab-pop' + (phone() ? ' sheet' : '');
    pop.innerHTML = `<div class="lc-ab-poph"><b>${esc(name)}</b><button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div>`
      + '<div class="lc-ab-popb">'
      + `<div><h6>Group</h6><button type="button" class="lc-ab-pill" data-repick style="--g:${g ? gColor(g) : ''}">${g ? gIcon(g) : ''}${esc(g?.name ?? '—')}${listMark()}</button>`
      + `${a.show_group ? ` <small>show: ${esc(a.show_group)}</small>` : ''}</div>`
      + `<dl><dt>Why here</dt><dd><span class="lc-ab-conf how-${a.how}">${a.how}</span> ${esc(a.why)}</dd>`
      + `${a.file ? `<dt>File says</dt><dd>${esc(a.file)}</dd>` : ''}${a.pretty && a.pretty !== name ? `<dt>Pretty</dt><dd>${esc(a.pretty)}</dd>` : ''}`
      + `${a.main ? `<dt>Main</dt><dd>${esc(a.main)}</dd>` : ''}${a.feature ? `<dt>Feature</dt><dd>${esc(a.feature)}</dd>` : ''}</dl>`
      + `<div><h6>Fixture types</h6><div class="lc-ab-typelist">${a.types.length ? a.types.map(t => `<span>${esc(t)}</span>`).join('') : '<small>none patched</small>'}</div></div>`
      + `<div><h6>Readout</h6>${seg('Readout', READOUTS, a.readout)}</div>`
      + `<div><h6>Resolution</h6>${seg('Resolution', RESOLUTIONS, a.resolution)}</div>`
      + `<div class="lc-ab-dact">${fam > 1 ? `<button type="button" class="lc-ab-btn" data-family>Select all ${esc(pattern(name))} (${fam})</button>` : ''}`
      + `<button type="button" class="lc-ab-btn" data-rule>Make a rule</button><button type="button" class="lc-ab-btn" data-info>Info</button></div></div>`;
    card.appendChild(pop);
    if (!phone()) {
      const r = card.getBoundingClientRect();
      pop.style.left = `${Math.max(8, Math.min(at.x - r.left + 12, r.width - pop.offsetWidth - 8))}px`;
      pop.style.top = `${Math.max(8, Math.min(at.y - r.top - 20, r.height - pop.offsetHeight - 8))}px`;
    }
    pop.addEventListener('click', async e => {
      const b = e.target.closest('button');
      if (!b) return;
      if (b.classList.contains('lc-close')) return closePop();
      if (b.dataset.prop) line(`Set ${pre()}Attribute ${q(name)} "${b.dataset.prop}" ${b.dataset.v}`);
      else if (b.hasAttribute('data-repick')) {
        const to = await pickGroup(`${name} → group`, g?.name);
        if (to) { closePop(); moveTo([name], to, null); draw(); }
      } else if (b.hasAttribute('data-family')) {
        data.attrs.filter(x => family(x.name) === family(name)).forEach(x => sel.add(x.name));
        closePop(); draw();
      } else if (b.hasAttribute('data-rule') && g) { closePop(); toast = { pat: pattern(name), g: g.name }; draw(); }
      else if (b.hasAttribute('data-info')) line(`Info ${pre()}Attribute ${q(name)}`);
    });
  }
  function closePop() { pop?.remove(); pop = null; }

  // ── The group menu (hold / right-click a header) ──
  function groupMenu(g, at) {
    const i = data.groups.indexOf(g);
    const G = `${pre()}FeatureGroup ${q(g.name)}`;
    openMenu(at, [
      { header: g.name },
      { icon: 'arrow-down-a-z', label: 'Sort by name', fn: () => line(`Sort ${G} /By Name`) },
      { icon: 'list-tree', label: 'Sort by family', fn: () => line(`Sort ${G} /By Family`) },
      { icon: 'list-start', label: 'Main attributes first', fn: () => line(`Sort ${G} /By MainFirst`) },
      null,
      { icon: 'merge', label: 'Merge into …', fn: async () => { const to = await pickGroup(`Merge ${g.name} into`, g.name); if (to) line(`Assign ${G} At ${pre()}FeatureGroup ${q(to.name)}`); } },
      { icon: 'split', label: 'Split by family', fn: () => line(`Split ${G} /By Family`) },
      null,
      { icon: 'chevron-left', label: 'Move left', off: i === 0, fn: () => line(`Move ${G} At ${i}`) },
      { icon: 'chevron-right', label: 'Move right', off: i === data.groups.length - 1, fn: () => line(`Move ${G} At ${i + 2}`) },
    ]);
  }
  hold(board, {
    filter: e => !!e.target.closest('[data-head]'),
    onHold: ({ x, y, target }) => {
      const g = data?.groups.find(gr => String(gr.no) === target.closest('[data-head]')?.dataset.head);
      if (g) groupMenu(g, { x, y });
    },
  });
  card.querySelector('.lc-ab-more').onclick = e => {
    const r = e.currentTarget.getBoundingClientRect();
    const show = data?.target === 'show';
    const own = data?.owner === 'UserProfile';
    openMenu({ x: r.left, y: r.bottom }, [
      { header: 'Grouping' },
      { icon: 'wand-sparkles', label: 'Accept every new one', off: !counts().new, fn: () => line(`Default ${pre()}Attribute /New`) },
      ...(data?.templates || []).map(t => ({ icon: 'layout-template', label: `Apply template ${t}`, hint: t === data.template ? 'now' : '',
        fn: () => line(`Call AttributeTemplate ${q(t)} At ${show ? 'ShowData' : data.owner}`) })),
      null,
      { icon: 'list', label: 'Rules', fn: () => line(`List ${pre()}AttributeRule`) },
      ...(own ? [{ icon: 'sliders-horizontal', label: 'Encoder bars', fn: () => line('List UserProfile EncoderBar') }] : []),
    ]);
  };

  // ── Tiles: tap / hold / drag, and the lasso ──
  let ghost = null, dropLine = null, over = null, scrollTimer = 0, lastPt = null;
  const target = (x, y) => {
    const el = document.elementFromPoint(x, y);
    const col = el?.closest('.lc-ab-col');
    if (!col || !board.contains(col)) return null;
    if (col.dataset.plus) return { plus: true, col };
    const g = data.groups.find(gr => String(gr.no) === col.dataset.drop);
    if (!g || el.closest('[data-head]') || view === 'type') return { g, col, at: null };
    // Between tiles: the tile whose middle is below the finger.
    const moving = [...sel];
    const tiles = [...col.querySelectorAll('.lc-ab-tile')].filter(t => !moving.includes(t.dataset.att));
    const next = tiles.find(t => { const r = t.getBoundingClientRect(); return y < r.top + r.height / 2; });
    if (!next) return { g, col, at: null, line: tiles.at(-1) ?? null, after: true };
    const order = g.members.filter(n => !moving.includes(n));
    return { g, col, at: order.indexOf(next.dataset.att) + 1, line: next };
  };
  const showDrop = t => {
    board.querySelectorAll('.lc-ab-col.over').forEach(c => c.classList.remove('over'));
    dropLine?.remove(); dropLine = null;
    if (!t) return;
    t.col.classList.add('over');
    if (t.line) {
      const body = t.col.querySelector('.lc-ab-body');
      const r = t.line.getBoundingClientRect(), b = body.getBoundingClientRect();
      dropLine = document.createElement('div');
      dropLine.className = 'lc-ab-dropline';
      dropLine.style.top = `${(t.after ? r.bottom + 2 : r.top - 3) - b.top + body.scrollTop}px`;
      body.appendChild(dropLine);
    }
  };
  // While dragging, the board and the column under the finger scroll near their edges.
  const autoScroll = () => {
    if (!ghost || !lastPt) return;
    const r = board.getBoundingClientRect();
    if (lastPt.x < r.left + EDGE) board.scrollLeft -= 12;
    else if (lastPt.x > r.right - EDGE) board.scrollLeft += 12;
    const body = document.elementFromPoint(lastPt.x, lastPt.y)?.closest('.lc-ab-body');
    if (body) {
      const b = body.getBoundingClientRect();
      if (lastPt.y < b.top + EDGE / 2) body.scrollTop -= 10;
      else if (lastPt.y > b.bottom - EDGE / 2) body.scrollTop += 10;
    }
    scrollTimer = requestAnimationFrame(autoScroll);
  };

  board.addEventListener('pointerdown', e => {
    if ((e.pointerType === 'mouse' && e.button !== 0) || !e.isPrimary || !data) return;
    if (e.target.closest('[data-head], [data-unfold], [data-refold], [data-plus], [data-dest]')) return;
    const tile = e.target.closest('.lc-ab-tile');
    const body = e.target.closest('.lc-ab-body, .lc-ab-board');
    if (!tile && (!body || e.pointerType === 'touch')) return;
    if (!claim(e)) return;
    const x0 = e.clientX, y0 = e.clientY;
    let mode = null, timer = 0, lasso = null;
    busy = true;
    if (tile) {
      timer = setTimeout(() => {
        if (mode) return;
        mode = 'held';
        details(tile.dataset.att, { x: x0, y: y0 });
      }, holdTime());
    }
    const move = ev => {
      lastPt = { x: ev.clientX, y: ev.clientY };
      const far = Math.hypot(ev.clientX - x0, ev.clientY - y0) > SLOP;
      if (!mode && far) {
        clearTimeout(timer);
        if (tile) {
          mode = 'drag';
          if (!sel.has(tile.dataset.att)) { sel.clear(); sel.add(tile.dataset.att); }
          card.querySelectorAll('.lc-ab-tile').forEach(t => t.classList.toggle('dragging', sel.has(t.dataset.att)));
          ghost = document.createElement('div');
          ghost.className = 'lc-ab-ghost';
          ghost.innerHTML = `${esc(tile.dataset.att)}${sel.size > 1 ? `<em>+${sel.size - 1}</em>` : ''}`;
          document.body.appendChild(ghost);
          scrollTimer = requestAnimationFrame(autoScroll);
        } else {
          mode = 'lasso';
          lasso = document.createElement('div');
          lasso.className = 'lc-ab-lasso';
          card.appendChild(lasso);
          if (!ev.shiftKey) sel.clear();
        }
        try { board.setPointerCapture(e.pointerId); } catch (_) {}
      }
      if (mode === 'drag') {
        ghost.style.left = `${ev.clientX}px`;
        ghost.style.top = `${ev.clientY}px`;
        over = target(ev.clientX, ev.clientY);
        showDrop(over);
      } else if (mode === 'lasso') {
        const c = card.getBoundingClientRect();
        const L = Math.min(x0, ev.clientX), T = Math.min(y0, ev.clientY), R = Math.max(x0, ev.clientX), B = Math.max(y0, ev.clientY);
        Object.assign(lasso.style, { left: `${L - c.left}px`, top: `${T - c.top}px`, width: `${R - L}px`, height: `${B - T}px` });
        card.querySelectorAll('.lc-ab-tile').forEach(t => {
          const r = t.getBoundingClientRect();
          const hit = r.right > L && r.left < R && r.bottom > T && r.top < B;
          t.classList.toggle('sel', hit || sel.has(t.dataset.att));
        });
      }
    };
    const end = (ev, cancelled) => {
      board.removeEventListener('pointermove', move);
      board.removeEventListener('pointerup', up);
      board.removeEventListener('pointercancel', cancel);
      clearTimeout(timer);
      cancelAnimationFrame(scrollTimer);
      busy = false;
      if (mode === 'drag') {
        ghost?.remove(); ghost = null;
        showDrop(null);
        const t = cancelled ? null : over;
        over = null;
        if (t?.plus) line(`Store ${pre()}FeatureGroup`);
        else if (t?.g) moveTo([...sel], t.g, t.at);
      } else if (mode === 'lasso') {
        lasso.remove();
        card.querySelectorAll('.lc-ab-tile.sel').forEach(t => sel.add(t.dataset.att));
      } else if (!mode && tile && !cancelled) {
        const n = tile.dataset.att;
        if (sel.has(n)) sel.delete(n); else sel.add(n);
      }
      draw();
    };
    const up = ev => end(ev, false);
    const cancel = ev => end(ev, true);
    board.addEventListener('pointermove', move);
    board.addEventListener('pointerup', up);
    board.addEventListener('pointercancel', cancel);
  });

  // ── Buttons ──
  card.addEventListener('click', async e => {
    const b = e.target.closest('button, [data-dest]');
    if (!b || !data || pop?.contains(b)) return;
    if (b.classList.contains('lc-close') && b.closest('.lc-ab-title')) return close(true);
    if (b.dataset.filter) { filter = b.dataset.filter; open.clear(); return draw(); }
    if (b.dataset.view) { view = b.dataset.view; return draw(); }
    if (b.dataset.unfold) { open.add(Number(b.dataset.unfold)); return draw(); }
    if (b.dataset.refold) { open.delete(Number(b.dataset.refold)); return draw(); }
    if (b.closest('[data-plus]')) return line(`Store ${pre()}FeatureGroup`);
    if (b.dataset.dest) {
      const to = await pickGroup(`${b.dataset.dest} → group`, groupOf(b.dataset.dest)?.name);
      if (to) { moveTo([b.dataset.dest], to, null); draw(); }
      return;
    }
    if (b.hasAttribute('data-assign')) return assign([...sel]);
    if (b.hasAttribute('data-deselect')) { sel.clear(); return draw(); }
    if (b.hasAttribute('data-accept')) return line(`Default ${pre()}Attribute /New`);
    if (b.hasAttribute('data-done')) return close(true);
    if (b.hasAttribute('data-learn') && toast) {
      const pat = card.querySelector('.lc-ab-toast input').value.trim();
      if (pat) line(`Store ${pre()}AttributeRule ${q(pat)} FeatureGroup ${q(toast.g)}`);
      toast = null; return draw();
    }
    if (b.hasAttribute('data-only')) { toast = null; return draw(); }
  });
  card.addEventListener('input', e => {
    if (!e.target.classList.contains('lc-ab-search')) return;
    search = e.target.value;
    const pos = e.target.selectionStart;
    draw();
    const s = card.querySelector('.lc-ab-search');
    s.focus(); s.setSelectionRange(pos, pos);
  });
  // Right-click a tile: its details (as a hold).
  board.addEventListener('contextmenu', e => {
    const tile = e.target.closest('.lc-ab-tile');
    if (!tile) return;
    e.preventDefault();
    details(tile.dataset.att, { x: e.clientX, y: e.clientY });
  });
  // Tap outside the details closes them.
  card.addEventListener('pointerdown', e => { if (pop && !pop.contains(e.target)) closePop(); }, true);
  new ResizeObserver(() => { if (data && card.classList.contains('phone') !== phone()) draw(); }).observe(card);

  const close = user => {
    closePop();
    scrim.remove();
    if (user) onClose?.();
  };
  return {
    update(d) {
      data = d;
      // Selected attributes that went away leave the selection.
      sel = new Set([...sel].filter(n => d.attrs.some(a => a.name === n)));
      draw();
      if (stale && !busy) draw();
    },
    close: () => close(false),
  };
}
