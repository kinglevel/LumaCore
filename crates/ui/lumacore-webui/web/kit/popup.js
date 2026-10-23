// <luma-popup> — the pick list for every link (decisions §30 "12b", one
// pop-up for every list; the 12b PickList board): a filter (Enter picks
// the first match), None for an optional link, a "missing …" top row while
// the link points at something gone, the items (number, name, note), Add New
// when the user may store one.
//
//   openPick({ title, pick: { kind, items: [{no, name, note}], optional,
//     follow, missing, add, add_label }, value }) → Promise<{ no } | { none:
//     true } | { follow: true } | { add: true } | null>
// `follow` (12b2): a first row that follows the user's selection ("Selected
// DataPool"); `add_label` names the add row (default "Add New…").
// The page turns the answer into the setting's command line.

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaPopup extends HTMLElement {
  constructor() { super(); this.spec = { pick: { items: [] } }; this.filter = ''; }
  connectedCallback() { this.dataset.kit = 'popup'; this.render(); }
  update(spec) { this.spec = spec; this.render(); }
  emit(name, detail = {}) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  shown() {
    const f = this.filter.trim().toLowerCase();
    return (this.spec.pick.items || []).filter(i => !f || String(i.no) === f || i.name.toLowerCase().includes(f));
  }

  render() {
    if (!this.isConnected) return;
    const { title, pick, value } = this.spec;
    const items = this.shown();
    const current = i => String(i.no) === String(value) || i.name === value || `${i.no} ${i.name}` === value;
    const firstPick = this.filter.trim() ? 0 : -1;
    this.innerHTML = `
      <div class="lc-pick">
        <header class="lc-pick-title"><span>${esc(title)}</span><button type="button" class="lc-pick-x" aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
        <label class="lc-pick-filter"><luma-icon name="search"></luma-icon><input type="text" placeholder="Filter" aria-label="Filter" value="${esc(this.filter)}" autocomplete="off" spellcheck="false"></label>
        <div class="lc-pick-list">
          ${pick.missing ? `<div class="lc-pick-row missing"><i></i><span>${esc(pick.missing)}</span></div>` : ''}
          ${pick.follow ? `<button type="button" class="lc-pick-row follow${value === pick.follow ? ' on' : ''}" data-follow="1"><em></em><span>${esc(pick.follow)}</span><small>follows you</small></button>` : ''}
          ${pick.optional ? `<button type="button" class="lc-pick-row none" data-none="1"><span>None</span></button>` : ''}
          ${items.map((i, n) => `<button type="button" class="lc-pick-row${current(i) ? ' on' : ''}${n === firstPick ? ' first' : ''}" data-no="${i.no}"><em>${pick.plain ? '' : i.no}</em><span>${esc(i.name)}</span><small>${esc(i.note || '')}</small></button>`).join('')}
          ${items.length ? '' : `<div class="lc-pick-empty">${this.filter ? 'No match' : 'Nothing here yet'}</div>`}
          ${pick.add ? `<button type="button" class="lc-pick-row add" data-add="1"><luma-icon name="plus"></luma-icon><span>${esc(pick.add_label || 'Add New…')}</span></button>` : ''}
        </div>
        <span class="lc-pick-note">${pick.note ? esc(pick.note) : pick.optional ? 'None clears the link' : `A ${esc(pick.kind || 'link')} is required`}</span>
      </div>`;
    const input = this.querySelector('input');
    input.addEventListener('input', () => {
      this.filter = input.value;
      const at = input.selectionStart;
      this.render();
      const next = this.querySelector('input');
      next.focus();
      next.setSelectionRange(at, at);
    });
    input.addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); const first = this.shown()[0]; if (first) this.emit('pick', { no: first.no }); }
      if (e.key === 'Escape') { e.preventDefault(); this.emit('close'); }
    });
    this.querySelector('.lc-pick-x').addEventListener('click', () => this.emit('close'));
    for (const b of this.querySelectorAll('[data-no]')) b.addEventListener('click', () => this.emit('pick', { no: Number(b.dataset.no) }));
    this.querySelector('[data-none]')?.addEventListener('click', () => this.emit('pick', { none: true }));
    this.querySelector('[data-follow]')?.addEventListener('click', () => this.emit('pick', { follow: true }));
    this.querySelector('[data-add]')?.addEventListener('click', () => this.emit('pick', { add: true }));
  }
}
customElements.define('luma-popup', LumaPopup);

/** The pick list as a pop-up; resolves the choice. */
export function openPick(spec) {
  return new Promise(resolve => {
    const scrim = document.createElement('div');
    scrim.className = 'lc-pick-scrim';
    const pop = document.createElement('luma-popup');
    scrim.append(pop);
    document.body.append(scrim);
    pop.update(spec);
    const done = v => { scrim.remove(); resolve(v); };
    pop.addEventListener('pick', e => done(e.detail));
    pop.addEventListener('close', () => done(null));
    scrim.addEventListener('pointerdown', e => { if (e.target === scrim) done(null); });
    pop.querySelector('input')?.focus();
  });
}
