// The value marks (decisions device-grid-and-value-marks; system colours and coloured
// indicators in the kit's paint): one vocabulary for every place a value shows — the device grid,
// the encoder bar, the feature-group strip, the attribute board, the programmer. Three parts,
// each in its own place so they never fight:
//
//   text colour   where the value comes from   (data-s: def · prgI · new · lower · track ·
//                                                block · other · otherT · park · cue · out · mixed)
//   ground        what happens to it            (prgA = your programmer, active; fading)
//   bars on top   which layers hold data        (abs · rel · fade · delay · phaser · preset ·
//                                                park; deact white on the left, the DMX
//                                                tester white on the right)
//
// The paint is the theme's `--lc-v-*` tokens (kit/tokens.css); this module only names them.
//
//   valueCell(cell, { layer, attrs }) → HTML of one value cell (`cell` as the devices feed sends
//     it: { t, s, n, l, f, src }; null = the device has no such attribute; undefined = not loaded)
//   bars(layers) → HTML of the bar strip      nameBar(bar) → a device's bar ('abs' | 'deact')
//   STATES, LAYERS, describe(state)

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

/** Every state a value cell can be in, with the words its hover and the Showroom use. */
export const STATES = {
  def: 'At its default',
  prgA: 'Your programmer, active: a Store takes it',
  prgI: 'Your programmer, not active: kept, not stored',
  new: 'New in the current cue of the selected sequence (a dimmer: higher)',
  lower: "A dimmer's new, lower value in the current cue",
  track: 'Tracked into the current cue of the selected sequence',
  block: 'Blocked in the current cue of the selected sequence',
  other: 'From a sequence that is not selected',
  otherT: 'Tracked, from a sequence that is not selected',
  park: 'Parked',
  cue: 'A time of the current cue',
  out: 'What leaves the station',
  mixed: 'The devices below differ',
};

/** The bar layers, in the order they draw. */
export const LAYERS = ['abs', 'rel', 'fade', 'delay', 'phaser', 'preset', 'park'];

/** The words for a state (the hover's fallback). */
export const describe = s => STATES[s] || '';

/** The bar strip over a value: deactivated data on the left, the layers, the DMX tester right. */
export function bars(layers) {
  const l = layers || [];
  if (!l.length) return '';
  const own = LAYERS.filter(k => l.includes(k)).map(k => `<i data-l="${k}"></i>`).join('');
  return `<span class="lc-v-bars" aria-hidden="true">${l.includes('deact') ? '<i data-l="off"></i>' : ''}${own}<b></b>${l.includes('tester') ? '<i data-l="off"></i>' : ''}</span>`;
}

/** A device's own bar (its name cell, a tile, a programmer row): 'abs' = active values in your
 *  programmer, 'deact' = only deactivated ones. */
export function nameBar(bar) {
  return bar ? `<span class="lc-v-bars lc-v-name" aria-hidden="true"><i data-l="${bar === 'abs' ? 'abs' : 'off'}"></i></span>` : '';
}

/** One value cell's inner parts and attributes: `{ cls, attrs, html }` for a host's element. */
export function valueParts(cell, { layer = 'Absolute' } = {}) {
  if (cell === null) return { cls: 'lc-v none', attrs: 'data-s="none"', html: '', title: 'No such attribute' };
  if (cell === undefined) return { cls: 'lc-v wait', attrs: 'data-s="wait"', html: '', title: '' };
  const fading = cell.f != null;
  const sym = cell.sym ? `<span class="lc-v-sym" style="background:${esc(cell.sym)}"></span>` : '';
  const prog = fading ? `<span class="lc-v-prog" style="width:${Math.round(Math.min(1, Math.max(0, cell.f)) * 100)}%"></span>` : '';
  return {
    cls: `lc-v${fading ? ' fading' : ''}`,
    attrs: `data-s="${esc(cell.s || 'def')}" data-layer="${esc(layer)}"`,
    html: `${bars(cell.l)}${sym}<span class="lc-v-t">${esc(cell.t ?? '')}</span>${prog}`,
    title: cell.src || describe(cell.s),
  };
}

/** One value cell as an element of its own (the encoder bar, the Showroom). */
export function valueCell(cell, opts = {}) {
  const p = valueParts(cell, opts);
  return `<span class="${p.cls}" ${p.attrs} title="${esc(p.title)}">${p.html}</span>`;
}
