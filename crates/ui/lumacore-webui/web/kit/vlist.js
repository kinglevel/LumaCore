// <luma-vlist> — a virtual list (decisions §30 "12b2 hardening": System
// Monitor). Thousands of items in memory, only the visible rows in the page;
// filter and search run over the items; rows are reused and keep their text
// selectable (`lc-selectable`). Follows the end like a log: scrolled up it
// pauses (`paused` class, `vl-follow` event), back at the end it follows
// again.
//
//   const vl = document.createElement('luma-vlist');
//   vl.renderRow = (item, el) => { el.className = 'row'; el.textContent = item.text; };
//   vl.filter = item => item.level !== 'info';   // null = every item
//   vl.cap = 5000;                               // oldest go first
//   vl.autoFollow = false;                       // new items never scroll it
//   vl.setItems(lines); vl.push(more); vl.refilter(); vl.toEnd();
//   vl.shown / vl.items.length                   // counts

const OVERSCAN = 6;

class LumaVlist extends HTMLElement {
  constructor() {
    super();
    this.items = [];
    this.view = [];
    this.filter = null;
    this.renderRow = (item, el) => { el.textContent = String(item); };
    this.cap = Infinity;
    this.following = true;
    /** False: new items never scroll the list (only filters / toEnd jump). */
    this.autoFollow = true;
    this.jump = true;
    this.rowH = 22;
    this.raf = 0;
    this.pool = [];
  }

  connectedCallback() {
    if (!this.spacer) {
      this.dataset.kit = 'vlist';
      this.spacer = document.createElement('div');
      this.spacer.className = 'lc-vl-spacer';
      this.layer = document.createElement('div');
      this.layer.className = 'lc-vl-rows';
      this.probe = document.createElement('div');
      this.probe.className = 'lc-vl-probe';
      this.empty = document.createElement('div');
      this.empty.className = 'lc-vl-empty';
      this.append(this.spacer, this.layer, this.probe, this.empty);
      this.addEventListener('scroll', () => this.onScroll(), { passive: true });
      new ResizeObserver(() => { this.measure(); this.schedule(); }).observe(this);
    }
    // A (re)attached list starts at the top; its scroll events until it has
    // settled aren't the user's (a window re-render keeps it following).
    this.settling = 2;
    this.jump = this.jump || this.following;
    this.measure();
    this.schedule();
  }

  /** Row height from the theme (`--lc-vl-row`); call after a font change. */
  measure() {
    const h = this.probe?.offsetHeight;
    if (h && Math.abs(h - this.rowH) > 0.1) { this.rowH = h; this.schedule(); }
  }

  get shown() { return this.view.length; }
  /** The text shown when no item passes (`placeholder` attribute). */
  get placeholder() { return this.getAttribute('placeholder') || ''; }

  atEnd() { return this.scrollHeight - this.scrollTop - this.clientHeight < Math.max(4, this.rowH / 2); }

  setItems(items) {
    this.items = items.slice(-this.cap);
    this.refilter();
  }

  push(items) {
    if (!items.length) return;
    for (const it of items) this.items.push(it);
    for (const it of items) if (!this.filter || this.filter(it)) this.view.push(it);
    const over = this.items.length - this.cap;
    if (over > 0) {
      const gone = this.items.splice(0, over);
      const goneShown = this.filter ? gone.filter(this.filter).length : gone.length;
      this.view.splice(0, goneShown);
      // Paused: the rows on screen stay where they are.
      if (!this.following) this.scrollTop -= goneShown * this.rowH;
    }
    this.schedule();
  }

  /** Filters changed: rebuild the view and follow the end again. */
  refilter() {
    this.view = this.filter ? this.items.filter(this.filter) : this.items.slice();
    for (const el of this.pool) el._item = null;
    this.toEnd();
  }

  /** Rows changed in place (a highlight): draw them again. */
  redraw() {
    for (const el of this.pool) el._item = null;
    this.schedule();
  }

  toEnd() { this.jump = true; this.setFollowing(true); this.schedule(); }

  setFollowing(on) {
    if (this.following === on) return;
    this.following = on;
    this.classList.toggle('paused', !on);
    this.dispatchEvent(new CustomEvent('vl-follow', { detail: { following: on } }));
  }

  onScroll() {
    if (this.settling) return;
    const end = this.atEnd();
    if (end !== this.following) this.setFollowing(end);
    this.schedule();
  }

  schedule() {
    if (!this.raf) this.raf = requestAnimationFrame(() => { this.raf = 0; this.paint(); });
  }

  paint() {
    if (!this.spacer) return;
    const h = this.rowH;
    this.spacer.style.height = `${this.view.length * h}px`;
    this.empty.textContent = this.view.length ? '' : this.placeholder;
    if (this.following && (this.autoFollow || this.jump)) {
      const bottom = Math.max(0, this.view.length * h - this.clientHeight);
      if (Math.abs(this.scrollTop - bottom) > 1) this.scrollTop = bottom;
    } else if (this.following && !this.atEnd()) {
      this.setFollowing(false);
    }
    this.jump = false;
    if (this.settling && --this.settling) this.schedule();
    const first = Math.max(0, Math.floor(this.scrollTop / h) - OVERSCAN);
    const count = Math.min(this.view.length - first, Math.ceil(this.clientHeight / h) + 2 * OVERSCAN);
    while (this.pool.length < count) {
      const el = document.createElement('div');
      el.style.height = `${h}px`;
      this.pool.push(el);
      this.layer.append(el);
    }
    while (this.pool.length > Math.max(count, 0)) this.pool.pop().remove();
    this.layer.style.transform = `translateY(${first * h}px)`;
    for (let i = 0; i < this.pool.length; i++) {
      const el = this.pool[i];
      const item = this.view[first + i];
      if (el._item === item && el._h === h) continue;
      el._item = item;
      el._h = h;
      el.style.height = `${h}px`;
      this.renderRow(item, el);
      el.classList.add('lc-vl-row', 'lc-selectable');
    }
  }
}
customElements.define('luma-vlist', LumaVlist);