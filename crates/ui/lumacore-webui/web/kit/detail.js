// <luma-detail> — a row's detail in a setup menu (decisions §30 "Users &
// Profiles follows storage", mockups/2026-10-14-users-profiles): the object's
// data where it is stored. Declared by the grid (`MenuDef` Grid.detail):
// groups (Settings · Content) of parts; each part is the object's settings
// cells of one section, a read-only list from the `detail` feed, or a page
// element (the encoder bar editor).
//
//   desk   `‹ Profiles › Busking` and the stored path, the group tabs, the
//          part tabs, the part
//   phone  (box < 600 px, P1) a screen per level: the groups as two big
//          tiles → the group's parts as rows → the part; ‹ one level up,
//          the crumbs jump further
//
//   el.update({ back, title, path, groups, owner, level, fields, data,
//     custom: { [id]: node }, at: { group, part } })
//   events: back · nav { group, part } · run { line }

import { icon } from './icon.js';
import './settings.js';
import './grid.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaDetail extends HTMLElement {
  constructor() {
    super();
    this.spec = { groups: [] };
    this.g = 0;        // group index
    this.p = 0;        // part index
    this.level = 'groups'; // phone: groups | parts | part
  }
  connectedCallback() {
    this.dataset.kit = 'detail';
    this.ro = new ResizeObserver(() => { if (this.phone !== this.isPhone()) this.render(); });
    this.ro.observe(this);
    this.render();
  }
  disconnectedCallback() { this.ro?.disconnect(); }
  isPhone() { return this.clientWidth > 0 && this.clientWidth < 600; }
  emit(name, detail = {}) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  /** Jump to a group / part by name (`/Open "3 Content EncoderBars"`). */
  goTo(group, part) {
    const gi = this.spec.groups.findIndex(g => g.name.toLowerCase() === String(group || '').toLowerCase());
    if (gi >= 0) { this.g = gi; this.level = 'parts'; }
    const pi = this.spec.groups[this.g]?.parts.findIndex(p => p.name.toLowerCase() === String(part || '').toLowerCase()) ?? -1;
    if (pi >= 0) { this.p = pi; this.level = 'part'; }
    this.render();
  }

  update(spec) {
    const first = !this.spec.title;
    this.spec = spec;
    if (first && spec.at) { this.goTo(spec.at.group, spec.at.part); return; }
    // A data / fields update redraws only the part (scroll and focus stay).
    if (this.built === this.key()) this.drawPart(); else this.render();
  }
  key() { return `${this.spec.title}|${this.g}|${this.p}|${this.level}|${this.isPhone()}`; }

  group() { return this.spec.groups[this.g] || { parts: [] }; }
  part() { return this.group().parts[this.p]; }
  path(withPart = true) {
    const g = this.group(), p = this.part();
    return [this.spec.path, g.path, withPart && p ? p.path : ''].filter(Boolean).join('.');
  }
  oneGroup() { return this.spec.groups.length === 1; }

  render() {
    if (!this.isConnected) return;
    this.phone = this.isPhone();
    const s = this.spec;
    if (!s.title) { this.innerHTML = ''; return; }
    if (this.oneGroup() && this.level === 'groups') this.level = 'parts';
    const g = this.group(), p = this.part();
    let head, nav = '';
    if (this.phone) {
      // P1: ‹ one level up; the crumbs name every level.
      const up = this.level === 'part' ? (this.oneGroup() ? s.title : g.label) : this.level === 'parts' && !this.oneGroup() ? s.title : s.back;
      const title = this.level === 'part' ? p?.label : this.level === 'parts' && !this.oneGroup() ? g.label : s.title;
      const crumbs = [[s.back, 'back'], [s.title, 'top']];
      if (this.level !== 'groups' && !this.oneGroup()) crumbs.push([g.label, 'group']);
      if (this.level === 'part') crumbs.push([p?.label, 'part']);
      head = `<div class="lc-dt-head"><button type="button" class="lc-dt-back" data-up>${icon('chevron-left')}${esc(up)}</button><b>${esc(title)}</b></div>`
        + `<div class="lc-dt-crumbs">${crumbs.map(([l, k]) => `<button type="button" data-crumb="${k}">${esc(l)}</button>`).join('')}</div>`;
    } else {
      head = `<div class="lc-dt-head"><button type="button" class="lc-dt-back" data-crumb="back">${icon('chevron-left')}${esc(s.back)}</button>`
        + `<span class="lc-dt-sep">›</span><b>${esc(s.title)}</b><span class="lc-dt-path">${esc(this.path())}</span></div>`;
      if (!this.oneGroup()) nav += `<nav class="lc-dt-tabs">${s.groups.map((x, i) => `<button type="button" data-g="${i}" class="${i === this.g ? 'on' : ''}">${esc(x.label)}</button>`).join('')}</nav>`;
      nav += `<nav class="lc-dt-tabs second">${g.parts.map((x, i) => `<button type="button" data-p="${i}" class="${i === this.p ? 'on' : ''}">${esc(x.label)}</button>`).join('')}</nav>`;
    }
    this.innerHTML = `<div class="lc-dt${this.phone ? ' phone' : ''}">${head}${nav}<div class="lc-dt-body" data-body></div></div>`;
    this.built = this.key();
    this.wire();
    this.drawPart();
  }

  drawPart() {
    const body = this.querySelector('[data-body]');
    if (!body) return;
    const s = this.spec, g = this.group();
    if (this.phone && this.level === 'groups') {
      body.innerHTML = `<span class="lc-dt-stored">${esc(s.path)}</span><div class="lc-dt-tiles">${s.groups.map((x, i) =>
        `<button type="button" class="lc-dt-tile" data-g="${i}"><b>${esc(x.label)}</b><small>${esc(x.sub)}</small></button>`).join('')}</div>`;
      return this.wireBody(body);
    }
    if (this.phone && this.level === 'parts') {
      body.innerHTML = `<span class="lc-dt-stored">${esc(this.path(false))}</span><div class="lc-dt-rows">${g.parts.map((x, i) =>
        `<button type="button" class="lc-dt-row" data-p="${i}"><span>${esc(x.label)}</span>${this.badge(x)}${icon('chevron-right')}</button>`).join('')}</div>`;
      return this.wireBody(body);
    }
    const p = this.part();
    if (!p) { body.innerHTML = ''; return; }
    const owner = s.owner;
    const acts = (p.actions || []).map(a => `<button type="button" class="lc-dt-act" data-line="${esc(a.line.replace(/\{owner\}/g, owner))}">${esc(a.label)}</button>`).join('');
    const keep = body.querySelector(':scope > [data-part]');
    // The same part again: update it in place.
    if (keep?.dataset.part === `${this.g}.${this.p}`) return this.fill(keep, p);
    body.innerHTML = `${this.phone ? `<span class="lc-dt-stored">${esc(this.path())}</span>` : ''}${acts ? `<div class="lc-dt-acts">${acts}</div>` : ''}<div class="lc-dt-part" data-part="${this.g}.${this.p}"></div>`;
    this.wireBody(body);
    this.fill(body.querySelector('[data-part]'), p);
  }

  /** A part's content: settings cells, a read-only list, or the host's element. */
  fill(host, p) {
    const s = this.spec, c = p.content;
    if (c.kind === 'settings') {
      let el = host.querySelector('luma-settings');
      if (!el) {
        el = document.createElement('luma-settings');
        el.setAttribute('embedded', '');
        el.addEventListener('run', e => { e.stopPropagation(); this.emit('run', { line: e.detail.line }); });
        host.replaceChildren(el);
      }
      // A section's cells, or the one field the section names (a user's Profile).
      const fields = (s.fields || []).filter(f => !c.section || (f.section || '') === c.section || f.name === c.section).map(f => ({ ...f, section: '' }));
      el.update({ title: s.title, fields, level: s.level, note: fields.length ? '' : (s.fields ? 'Nothing to set here.' : 'Loading…') });
    } else if (c.kind === 'list') {
      let el = host.querySelector('luma-grid');
      if (!el) { el = document.createElement('luma-grid'); host.replaceChildren(el); }
      const rows = s.data?.[c.rows] || [];
      el.update({ key: `detail.${p.name.toLowerCase()}`, object: null, id: c.row_key, row_key: c.row_key, columns: c.columns, rows, state: { set: 'Full' }, level: s.level, empty: s.data ? 'Nothing here yet' : 'Loading…' });
    } else {
      const node = s.custom?.[c.id];
      if (node && node.parentElement !== host) host.replaceChildren(node);
      else if (!node) host.innerHTML = `<div class="lc-dt-note">${esc(c.id)}</div>`;
    }
  }

  /** What a phone row says beside a part (new attributes waiting). */
  badge(p) {
    const n = p.name === 'Attributes' ? this.spec.data?.waiting : 0;
    return n ? `<em>${n} new</em>` : '';
  }

  wire() {
    for (const b of this.querySelectorAll('.lc-dt-head [data-crumb], .lc-dt-crumbs [data-crumb]')) b.onclick = () => this.crumb(b.dataset.crumb);
    this.querySelector('[data-up]')?.addEventListener('click', () => this.up());
    for (const b of this.querySelectorAll('nav [data-g]')) b.onclick = () => { this.g = +b.dataset.g; this.p = 0; this.nav(); };
    for (const b of this.querySelectorAll('nav [data-p]')) b.onclick = () => { this.p = +b.dataset.p; this.nav(); };
  }
  wireBody(body) {
    for (const b of body.querySelectorAll('.lc-dt-tile[data-g]')) b.onclick = () => { this.g = +b.dataset.g; this.p = 0; this.level = 'parts'; this.nav(); };
    for (const b of body.querySelectorAll('.lc-dt-row[data-p]')) b.onclick = () => { this.p = +b.dataset.p; this.level = 'part'; this.nav(); };
    for (const b of body.querySelectorAll('[data-line]')) b.onclick = () => this.emit('run', { line: b.dataset.line });
  }
  nav() { this.render(); this.emit('nav', { group: this.group().name, part: this.part()?.name }); }
  up() {
    if (this.level === 'part') this.level = 'parts';
    else if (this.level === 'parts' && !this.oneGroup()) this.level = 'groups';
    else return this.emit('back');
    this.render();
  }
  crumb(k) {
    if (k === 'back') return this.emit('back');
    this.level = k === 'top' ? 'groups' : k === 'group' ? 'parts' : 'part';
    this.render();
  }
}
customElements.define('luma-detail', LumaDetail);
