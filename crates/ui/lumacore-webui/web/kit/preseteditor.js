// The preset editor (decisions 0288): one preset's values in a <luma-grid> — a row per fixture
// (a Universal preset: one row, "every fixture"), a column per attribute — fed by the
// `pool_editor` feed, so it follows every Store / Merge live. The first build only shows the
// values; editing them comes with Update.
//
//   const ed = openPresetEditor({ onClose() });  ed.update(poolEditorData);  ed.close()

import { icon } from './icon.js';
import './grid.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const MODES = { S: 'Selective', U: 'Universal', G: 'Global' };

export function openPresetEditor({ onClose }) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ped-scrim';
  scrim.dataset.kit = 'preset-editor';
  const card = document.createElement('div');
  card.className = 'lc-card lc-ped';
  card.innerHTML = `<div class="lc-box-title"><span class="lc-ped-name"></span>`
    + `<button type="button" class="lc-close" aria-label="Close">${icon('x')}</button></div>`
    + '<luma-grid></luma-grid>';
  scrim.appendChild(card);
  document.body.appendChild(scrim);
  const grid = card.querySelector('luma-grid');
  const close = () => { scrim.remove(); };
  card.querySelector('.lc-close').onclick = () => { close(); onClose?.(); };
  scrim.addEventListener('click', e => { if (e.target === scrim) { close(); onClose?.(); } });

  return {
    update(d) {
      card.querySelector('.lc-ped-name').innerHTML =
        `Preset ${esc(d.no)}${d.name ? ` "${esc(d.name)}"` : ''} · ${esc(MODES[d.mode] || d.mode)} · ${esc(d.rows.length)} row${d.rows.length === 1 ? '' : 's'}`;
      const columns = [{ id: 'fixture', label: 'Fixture', width: 120, frozen: true },
        ...d.attributes.map((a, i) => ({ id: `a${i}`, label: a, width: 96 }))];
      const rows = d.rows.map((r, i) => ({ key: String(i), fixture: r.fixture, ...Object.fromEntries(r.values.map((v, j) => [`a${j}`, v ?? ''])) }));
      grid.update({ key: `preset-${d.no}`, object: 'Preset', id: 'key', row_key: 'key', columns, rows, bar: false,
        state: { set: 'Full' }, empty: 'No values: Store Preset with values in your programmer' });
    },
    close,
    isOpen: () => scrim.isConnected,
  };
}
