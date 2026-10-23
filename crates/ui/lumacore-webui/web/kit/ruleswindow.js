// <luma-rules-window> — the Rules window (decisions 0294, the approved mockup
// docs/dev/ui-kit/mockups/2026-10-23-paint/04-paint.html "Feature groups and their rules"): one
// layer's AttributeRule list (the show's, or the viewer's profile's own) in order — the first match
// wins, learned rules beat the standard table — drawn from the `rules` feed, which builds every row
// and every line (lumacore_views::rules). Nothing is resolved here.
//
//   the head          "Attribute rules · first match wins", the counts, a "Which rule?" field: Enter
//                     (or its button) is the window's line `Set Self "Which" "<name>"`; the station's
//                     answer (rule n · group · why) shows under it and its row is framed and scrolled to
//   the grid          <luma-grid>: No, Pattern (a text cell → the feed's `pattern` line), Feature group
//                     (a pick list of the layer's groups → the `group` line; the row tinted in that
//                     group's colour as this viewer sees it), Family, Feature, Kind; the edit card edits
//                     like any grid (the object word comes from the feed's lines: `UserProfile …` on a
//                     profile's layer)
//   drag a row        the grid's lift (`lift = 'any'`): drop it on another row → the `move` line
//                     `Move AttributeRule {from} At {to}`
//   the slab          Delete (the `delete` line naming the selected rules) · Oops
//
//   el.show(data, { level })  ·  events: run { line } (bubbles; the host sends it with the window's
//   number, `Self`)

import './grid.js';
import './slab.js';
import { rowsWord } from './grid.js';
import { phoneMode, onPhoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
/** A template from the feed with its `{name}`s filled. */
const fill = (tpl, words) => String(tpl || '').replace(/\{(\w+)\}/g, (m, k) => (k in words ? String(words[k]) : m));
/** Text as one quoted word (quotes and `;` can't ride inside it). */
const quoted = s => `"${String(s ?? '').replace(/[";]/g, '')}"`;

const COLUMNS = [
  { id: 'pattern', label: 'Pattern', kind: 'text', prop: 'Pattern', width: 210, priority: 1 },
  { id: 'group', label: 'Feature group', kind: 'list', prop: 'FeatureGroup', width: 160, priority: 1 },
  { id: 'family', label: 'Family', width: 120, priority: 2 },
  { id: 'feature', label: 'Feature', width: 120, priority: 2 },
  { id: 'kind', label: 'Kind', width: 100, priority: 2 },
];

class LumaRulesWindow extends HTMLElement {
  constructor() { super(); this.data = null; this.cfg = {}; this.sel = []; }

  connectedCallback() {
    this.dataset.kit = 'rules-window';
    if (this.built) return;
    this.built = true;
    this.innerHTML = `<div class="lc-rules">
      <div class="lc-rules-head">
        <span class="lc-rules-cap">Attribute rules · first match wins</span>
        <span class="lc-rules-count" data-count></span>
        <span class="lc-rules-which">
          <label class="lc-dlg-input"><luma-icon name="search"></luma-icon><input data-which type="text" autocomplete="off" spellcheck="false" enterkeyhint="search" placeholder="An attribute name" aria-label="Which rule places this attribute"></label>
          <luma-button icon="crosshair" data-ask>Which rule?</luma-button>
        </span>
      </div>
      <div class="lc-rules-answer" data-answer hidden></div>
      <div class="lc-rules-body"><luma-slab></luma-slab><luma-grid></luma-grid></div>
    </div>`;
    this.input = this.querySelector('[data-which]');
    this.answer = this.querySelector('[data-answer]');
    this.slab = this.querySelector('luma-slab');
    this.grid = this.querySelector('luma-grid');
    const ask = () => this.run(`Set Self "Which" ${quoted(this.input.value.trim())}`);
    this.input.addEventListener('keydown', e => { if (e.key === 'Enter') { e.preventDefault(); e.stopPropagation(); ask(); } });
    this.querySelector('[data-ask]').addEventListener('click', ask);
    // The grid's lines (the edit card's Set lines) are the window's lines.
    this.grid.addEventListener('line', e => { e.stopPropagation(); this.run(e.detail.line); });
    this.grid.addEventListener('select', e => { this.sel = e.detail.rows; this.drawSlab(); });
    this.wireLift();
    this.offPhone = onPhoneMode(() => { if (this.data) this.render(); });
    this.slab.addEventListener('act', e => {
      const { id } = e.detail;
      if (id === 'delete') { const nos = this.selectedNos(); if (nos.length) this.run(fill(this.lines().delete, { no: rowsWord(nos) })); }
      else if (id === 'oops') this.run('Oops');
    });
    if (this.data) this.render();
  }


  run(line) { if (line) this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }
  lines() { return this.data?.lines || {}; }
  rowByKey(k) { return (this.data?.rows || []).find(r => String(r.guid) === String(k)); }
  selectedNos() { return this.sel.map(k => this.rowByKey(k)?.no).filter(n => n != null); }

  show(data, cfg = {}) {
    this.data = data || { rows: [] };
    this.cfg = cfg;
    if (this.built) this.render();
  }

  render() {
    const d = this.data;
    const count = d.count || {};
    this.querySelector('[data-count]').textContent = d.missing ? 'No such profile'
      : `${count.standard ?? 0} standard · ${count.learned ?? 0} learned${d.title ? ` · ${d.title}` : ''}`;
    // The field shows what the window asks (its setting) unless it is being typed in.
    const asked = d.which?.name ?? this.cfg.which ?? '';
    if (document.activeElement !== this.input && this.input.value !== asked) this.input.value = asked;
    const w = d.which;
    this.answer.hidden = !w;
    if (w) {
      const head = w.rule != null ? `Rule ${w.rule}` : 'No rule';
      this.answer.innerHTML = `<b>${esc(w.name)}</b><span>${esc(head)}</span>${w.group ? `<span>${esc(w.group)}</span>` : ''}`
        + `${w.why ? `<span class="why">${esc(w.why)}</span>` : ''}${w.how ? `<small>${esc(w.how)}</small>` : ''}`;
    }
    const lines = this.lines();
    // The object word as the feed's lines write it (`AttributeRule`, `UserProfile 2 AttributeRule`).
    const object = String(lines.pattern || 'Set AttributeRule {no}').replace(/^Set /, '').split(' {no}')[0];
    const marks = {};
    for (const r of d.rows || []) if (r.hit) marks[r.guid] = { hit: true };
    this.grid.lift = 'any';
    this.grid.update({
      key: 'rules', object, id: 'no', row_key: 'guid',
      columns: COLUMNS.map(c => (c.id === 'group' ? { ...c, options: d.groups || [] } : c)),
      rows: d.rows || [], marks, level: this.cfg.level || '', state: { set: 'Full' },
      // The selection bar folds into the header's No cell (decisions §37 1): the list keeps the window's height.
      bar: 'head',
      empty: d.missing ? 'No such profile.' : 'No rules in this layer.',
    });
    this.drawSlab();
    // A new answer brings its row into sight, once.
    const hit = (d.rows || []).find(r => r.hit);
    const key = hit ? `${w?.name}|${hit.guid}` : '';
    if (key && key !== this.revealed) {
      this.revealed = key;
      requestAnimationFrame(() => this.grid.querySelector('.lc-grid-row.lc-hit')?.scrollIntoView({ block: 'center' }));
    } else if (!key) this.revealed = '';
  }

  drawSlab() {
    const nos = this.selectedNos();
    this.slab.update({
      edit: {
        delete: nos.length ? { line: this.lines().delete } : { off: 'Select rules first' },
        oops: { line: 'Oops' },
      },
      count: nos.length, level: this.cfg.level || '', phone_edit: ['delete', 'oops'],
      // One bar of the cells that work now (decisions §37 1–2): the list keeps the window's height.
      bar: phoneMode() === 'side' ? 'side' : 'up',
    });
  }

  /** A row dragged onto another: the pointer core's lift through the grid; the drop is one line. */
  wireLift() {
    const g = this.grid;
    let from = null, over = null;
    const rowAt = (x, y) => {
      const el = document.elementFromPoint(x, y)?.closest?.('.lc-grid-row[data-row]');
      if (el && g.contains(el)) return el;
      // Above the first row or below the last: the end it is nearest.
      const all = [...g.querySelectorAll('.lc-grid-row[data-row]')];
      if (!all.length) return null;
      return y < all[0].getBoundingClientRect().top ? all[0] : y > all[all.length - 1].getBoundingClientRect().bottom ? all[all.length - 1] : null;
    };
    const mark = el => { if (over === el) return; over?.classList.remove('lc-drop'); over = el; over?.classList.add('lc-drop'); };
    const done = () => { mark(null); g.querySelector('.lc-grid-row.lc-lifted')?.classList.remove('lc-lifted'); from = null; };
    g.addEventListener('lift', e => {
      from = this.rowByKey(e.detail.key);
      g.querySelector(`.lc-grid-row[data-row="${CSS.escape(String(e.detail.key))}"]`)?.classList.add('lc-lifted');
    });
    g.addEventListener('liftmove', e => { if (from) mark(rowAt(e.detail.x, e.detail.y)); });
    g.addEventListener('drop', e => {
      const target = !e.detail.cancelled && from ? this.rowByKey(rowAt(e.detail.x, e.detail.y)?.dataset.row) : null;
      const no = from?.no;
      done();
      if (target && no != null && target.no !== no) this.run(fill(this.lines().move, { from: no, to: target.no }));
    });
  }
}
customElements.define('luma-rules-window', LumaRulesWindow);

export { LumaRulesWindow };
