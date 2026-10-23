// The kit's pop-up menu (window menu, context menus): drawn into the page's
// #ctx element. Items: { icon, label, fn, off, danger, warn, hint, keep,
// value, check, id } · null = separator · { header } = a heading. `value`
// (a setting's value, with ▾) and `check` (an on/off box) draw a window bar
// item in a menu (decisions §30 "12c"); `id` becomes the row's data-id. Opened by tap (the tab dot), hold or
// right-click — the same menu for mouse and touch. Row height follows
// --lc-row (48 px on touch screens).

const host = () => document.getElementById('ctx');
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

/** Open `items` at {x, y} (client px). */
export function openMenu(at, items) {
  const el = host();
  el.dataset.kit = 'menu';
  el.innerHTML = '';
  for (const item of items) {
    if (!item) { el.insertAdjacentHTML('beforeend', '<div class="csep"></div>'); continue; }
    if (item.header) { el.insertAdjacentHTML('beforeend', `<div class="chdr">${esc(item.header)}</div>`); continue; }
    const row = document.createElement('div');
    row.className = 'ci' + (item.danger ? ' danger' : '') + (item.warn ? ' warn' : '') + (item.off ? ' off' : '');
    row.setAttribute('role', 'menuitem');
    if (item.id) row.dataset.id = item.id;
    row.innerHTML = `<span class="cion">${item.icon ? `<luma-icon name="${item.icon}"></luma-icon>` : ''}</span>`
      + `<span class="clbl">${esc(item.label)}</span>${item.hint ? `<span class="chint">${esc(item.hint)}</span>` : ''}`
      + (item.value != null ? `<span class="cval">${esc(item.value)}<luma-icon name="chevron-down"></luma-icon></span>` : '')
      + (item.check != null ? `<span class="cbox${item.check ? ' on' : ''}" aria-checked="${!!item.check}">${item.check ? '<luma-icon name="check"></luma-icon>' : ''}</span>` : '');
    row.onclick = ev => {
      ev.stopPropagation();
      if (item.off) return;
      if (!item.keep) closeMenu();
      item.fn?.();
    };
    el.appendChild(row);
  }
  el.hidden = false;
  const vw = window.innerWidth, vh = window.innerHeight;
  el.style.left = `${Math.max(4, Math.min(at.x, vw - el.offsetWidth - 4))}px`;
  el.style.top = `${Math.max(4, Math.min(at.y, vh - el.offsetHeight - 4))}px`;
}

export function closeMenu() { const el = host(); if (el) el.hidden = true; }
export function menuOpen() { return !!host() && !host().hidden; }
