// <luma-encoder-bars> — your encoder bar layouts (decisions §30 "12f-a", the
// Encoder bar tab of mockups/2026-10-14-12f/Attr-Round01): the Users &
// Profiles menu's tab, opened by `Edit EncoderBar`. Drawn from the
// `attributes` feed of your profile (`profile: "mine"`); every gesture is a
// command line:
//
//   layouts         the profile's pool (each screen shows one); New →
//                   `Store UserProfile EncoderBar` (the station asks the name)
//   strip buttons   your groups in bar order; drag the grip →
//                   `Move UserProfile EncoderBar "L" FeatureGroup "G" At n`;
//                   tap one to see its pages
//   pages           4 slots each; drag a slot onto another (swap or a free
//                   slot) → `Move UserProfile EncoderBar "L" Attribute "a" At n`;
//                   onto a strip button → `Set UserProfile Attribute "a"
//                   FeatureGroup "G"` (your grouping: the bottom on every layout)
//
//   el.update(data)   data = the `attributes` feed (with `encoder_bars`)
//   el.addEventListener('run', e => e.detail.line)

import { icon } from './icon.js';
import { claim } from './pointer.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const q = n => `"${String(n).replace(/"/g, '')}"`;
const SLOP = 6;

class LumaEncoderBars extends HTMLElement {
  constructor() { super(); this.data = null; this.layout = null; this.group = null; this.busy = false; }
  connectedCallback() { this.dataset.kit = 'encoder-bars'; this.render(); }
  update(d) { this.data = d; if (!this.busy) this.render(); }
  run(line) { this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }

  render() {
    if (!this.isConnected) return;
    const d = this.data;
    if (!d) { this.innerHTML = '<div class="lc-eb-note">Loading…</div>'; return; }
    const bars = d.encoder_bars || [];
    const bar = bars.find(b => b.no === this.layout) || bars[0];
    if (!bar) { this.innerHTML = '<div class="lc-eb-note">No encoder bar layout.</div>'; return; }
    this.layout = bar.no;
    const groups = new Map(d.groups.map(g => [g.no, g]));
    const attr = n => d.attrs.find(a => a.name === n);
    const btn = bar.buttons.find(b => b.group === this.group) || bar.buttons[0];
    this.group = btn?.group ?? null;
    const slots = btn ? [...btn.slots] : [];
    // One free slot past the last: pages grow as slots fill.
    const pages = Math.max(1, Math.ceil((slots.length + 1) / 4));
    const g = btn && groups.get(btn.group);
    this.innerHTML = `<div class="lc-eb">
      <div class="lc-eb-left">
        <h5>Layouts · one per screen</h5>
        <div class="lc-eb-lays">${bars.map(b => `<button type="button" data-lay="${b.no}" class="${b.no === bar.no ? 'on' : ''}">${esc(b.name)}</button>`).join('')}
          <button type="button" class="add" data-addlay>${icon('plus')}New</button></div>
        <h5>Strip buttons · drag to reorder</h5>
        <div class="lc-eb-strip">${bar.buttons.map(b => {
          const gr = groups.get(b.group);
          const fresh = b.slots.some(n => n && attr(n)?.new);
          return `<div class="lc-eb-sb${b.group === this.group ? ' on' : ''}" data-sb="${b.group}" style="--g:${gr?.color || 'var(--lc-text-dim)'}">`
            + `<span class="lc-eb-grip" data-grip>${icon('grip-vertical')}</span>${icon(gr?.icon || 'square')}<span>${esc(gr?.name ?? '?')}</span>`
            + `${fresh ? '<em>new</em>' : ''}<small>${b.slots.filter(Boolean).length}</small></div>`;
        }).join('')}</div>
      </div>
      <div class="lc-eb-pages">
        <div class="lc-eb-hint">${g ? `Strip button <b>${esc(g.name)}</b> · ${slots.filter(Boolean).length} attributes · drag a slot onto another (swap / free slot), or onto a strip button to move it into that group (at the bottom).` : 'No strip buttons.'}</div>
        ${btn ? Array.from({ length: pages }, (_, p) => `<div class="lc-eb-pg"><h6>Page ${p + 1}</h6><div class="lc-eb-slots">${[0, 1, 2, 3].map(k => {
          const i = p * 4 + k, n = slots[i], a = n && attr(n);
          return n ? `<div class="lc-eb-slot full${a?.new ? ' new' : ''}${a && !a.used ? ' unused' : ''}" data-i="${i}" data-att="${esc(n)}"><b>${esc(n)}</b><small>encoder ${k + 1}${a && !a.used ? ' · unused' : ''}</small></div>`
            : `<div class="lc-eb-slot" data-i="${i}">empty</div>`;
        }).join('')}</div></div>`).join('') : ''}
      </div>
    </div>`;
    this.wire(bar, groups);
  }

  wire(bar, groups) {
    const prefix = this.data.owner ? `${this.data.owner} ` : 'UserProfile ';
    const L = q(bar.name);
    for (const b of this.querySelectorAll('[data-lay]')) b.onclick = () => { this.layout = Number(b.dataset.lay); this.render(); };
    this.querySelector('[data-addlay]').onclick = () => this.run(`Store ${prefix}EncoderBar`);
    for (const sb of this.querySelectorAll('[data-sb]')) sb.onclick = e => {
      if (e.target.closest('[data-grip]')) return;
      this.group = Number(sb.dataset.sb); this.render();
    };
    // One drag: a strip button's grip (reorder) or a full slot (move / swap / regroup).
    const start = (e, el, kind) => {
      if ((e.pointerType === 'mouse' && e.button !== 0) || !claim(e)) return;
      e.preventDefault();
      const x0 = e.clientX, y0 = e.clientY;
      let moved = false, over = null, ghost = null;
      this.busy = true;
      el.setPointerCapture(e.pointerId);
      const move = ev => {
        if (!moved && Math.hypot(ev.clientX - x0, ev.clientY - y0) < SLOP) return;
        if (!moved) {
          moved = true;
          el.classList.add('dragging');
          ghost = document.createElement('div');
          ghost.className = 'lc-ab-ghost';
          ghost.textContent = kind === 'slot' ? el.dataset.att : groups.get(Number(el.dataset.sb))?.name ?? '';
          document.body.appendChild(ghost);
        }
        ghost.style.left = `${ev.clientX}px`; ghost.style.top = `${ev.clientY}px`;
        const t = document.elementFromPoint(ev.clientX, ev.clientY);
        const hit = kind === 'slot' ? t?.closest('.lc-eb-slot, .lc-eb-sb') : t?.closest('.lc-eb-sb');
        if (hit !== over) { over?.classList.remove('over'); over = hit && hit !== el && this.contains(hit) ? hit : null; over?.classList.add('over'); }
      };
      const end = () => {
        el.removeEventListener('pointermove', move);
        el.removeEventListener('pointerup', end);
        el.removeEventListener('pointercancel', end);
        ghost?.remove();
        over?.classList.remove('over');
        this.busy = false;
        if (moved && over) {
          if (kind === 'grip') {
            const to = bar.buttons.findIndex(b => String(b.group) === over.dataset.sb) + 1;
            const g = groups.get(Number(el.dataset.sb));
            if (g && to > 0) this.run(`Move ${prefix}EncoderBar ${L} FeatureGroup ${q(g.name)} At ${to}`);
          } else if (over.dataset.sb) {
            const g = groups.get(Number(over.dataset.sb));
            if (g && Number(over.dataset.sb) !== this.group) this.run(`Set ${prefix}Attribute ${q(el.dataset.att)} FeatureGroup ${q(g.name)}`);
          } else if (over.dataset.i != null) {
            this.run(`Move ${prefix}EncoderBar ${L} Attribute ${q(el.dataset.att)} At ${Number(over.dataset.i) + 1}`);
          }
        }
        this.render();
      };
      el.addEventListener('pointermove', move);
      el.addEventListener('pointerup', end);
      el.addEventListener('pointercancel', end);
    };
    for (const gp of this.querySelectorAll('[data-grip]')) gp.addEventListener('pointerdown', e => start(e, gp.closest('.lc-eb-sb'), 'grip'));
    for (const s of this.querySelectorAll('.lc-eb-slot.full')) s.addEventListener('pointerdown', e => start(e, s, 'slot'));
  }
}
customElements.define('luma-encoder-bars', LumaEncoderBars);
