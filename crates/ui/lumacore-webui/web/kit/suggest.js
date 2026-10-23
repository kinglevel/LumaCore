// <luma-suggest mode="inline|palette"> — the command line's suggestion list
// (decisions §30 "12b2", the Suggest board): rows with the kind's icon, the
// title, the subtitle and a shortcut chip on the selected row; the palette
// adds sections, a breadcrumb (= the command line), a preview pane and the
// key legend. The station makes and ranks the items (`core::suggest`); the
// list only shows them. Picking never runs code here: the host writes the
// command line with the item's `insert` (or runs one of its action lines).
//
//   const ctl = LumaKit.suggest.attach({ el, surface: 'bar' | 'palette',
//     send: obj => ws.send(JSON.stringify(obj)),   // the page's shell socket
//     anchor: () => DOMRect,                       // inline: the line to sit above
//     onTake: (item, reply) => …,                  // Tab / tap
//     onReply: reply => … });                      // every fresh reply
//   ctl.query({ line?, cursor? })  // no line = the session's own command line
//   ctl.open() · ctl.close() · ctl.isOpen() · ctl.move(±1) · ctl.take() · ctl.selected()
//   LumaKit.suggest.receive(msg)   // route `{"type":"suggestions",…}` here
//   LumaKit.suggest.apply(reply, item) → { line, cursor }  // for a local input

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

class LumaSuggest extends HTMLElement {
  constructor() {
    super();
    this.reply = null;
    this.sel = 0;
    this.view = [];
  }
  connectedCallback() {
    this.dataset.kit = 'suggest';
    // A press on the list never takes focus from the command line.
    this.addEventListener('pointerdown', e => e.preventDefault());
    this.render();
  }
  get mode() { return this.getAttribute('mode') === 'palette' ? 'palette' : 'inline'; }

  /** Show a reply (keeps the selected item when it is still there). */
  update(reply) {
    const was = this.selected()?.insert;
    this.reply = reply;
    this.order();
    const keep = this.view.findIndex(i => i.insert === was);
    this.sel = keep >= 0 ? keep : 0;
    this.render();
  }
  /** Items in display order: the palette groups them by section. */
  order() {
    const items = this.reply?.items || [];
    if (this.mode !== 'palette') { this.view = items; return; }
    const sections = [];
    for (const i of items) if (!sections.includes(i.section)) sections.push(i.section);
    this.view = sections.flatMap(s => items.filter(i => i.section === s));
  }
  selected() { return this.view[this.sel] || null; }
  move(step) {
    if (!this.view.length) return;
    this.sel = (this.sel + step + this.view.length) % this.view.length;
    this.render();
    this.querySelector('.lc-sg-row.sel')?.scrollIntoView({ block: 'nearest' });
  }
  emit(name, detail) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  row(item, n) {
    const off = item.disabled || item.status === 'planned';
    const chip = n === this.sel ? `<kbd class="lc-sg-chip">${this.mode === 'palette' ? 'Enter' : 'Tab'}</kbd>` : '';
    const right = this.mode === 'palette' && n !== this.sel && item.disabled ? `<small class="lc-sg-code">${esc(item.disabled)}</small>` : '';
    return `<button type="button" class="lc-sg-row k-${item.kind}${n === this.sel ? ' sel' : ''}${off ? ' off' : ''}" data-i="${n}">`
      + `<luma-icon name="${esc(item.icon)}"></luma-icon>`
      + (this.mode === 'palette'
        ? `<span class="lc-sg-text"><span class="lc-sg-name">${esc(item.title)}</span><small>${esc(item.subtitle)}</small></span>`
        : `<span class="lc-sg-title">${esc(item.title)}</span><small class="lc-sg-sub">${esc(item.subtitle)}</small>`)
      + right + chip + '</button>';
  }

  render() {
    if (!this.isConnected) return;
    const r = this.reply || { items: [], line: '', total: 0 };
    let rows = '';
    let section = null;
    this.view.forEach((item, n) => {
      if (this.mode === 'palette' && item.section !== section) {
        section = item.section;
        rows += `<div class="lc-sg-sec">${esc(section)}</div>`;
      }
      rows += this.row(item, n);
    });
    if (this.mode === 'inline') {
      this.innerHTML = `<div class="lc-sg lc-sg-inline">${rows}`
        + `<span class="lc-sg-legend">Tab takes · ↑ ↓ move · Enter runs the line as typed · → takes the grey text · Esc closes</span></div>`;
    } else {
      const it = this.selected();
      const actions = (it?.actions || []).map((a, n) => `<li><span>${n === 0 ? 'Enter' : n === 1 ? 'Meow + Enter' : ''}</span><b>${esc(a.label)}</b><code>${esc(a.line)}</code></li>`).join('');
      this.innerHTML = `<div class="lc-sg lc-sg-palette">
        <header class="lc-sg-head"><luma-icon name="search"></luma-icon>
          <span class="lc-sg-crumb">${esc(r.line)}</span>
          <small class="lc-sg-count">${this.view.length} of ${r.total}</small></header>
        <div class="lc-sg-body">
          <div class="lc-sg-list">${rows || '<div class="lc-sg-empty">Nothing fits here</div>'}</div>
          <aside class="lc-sg-preview">${it ? `<b>${esc(it.title)}</b><p>${esc(it.subtitle)}</p>${r.syntax ? `<code>${esc(r.syntax)}</code>` : ''}<ul>${actions}</ul>` : ''}</aside>
        </div>
        <footer class="lc-sg-foot"><span>Enter ${esc(it?.actions?.[0]?.label || 'takes')}</span><span>Meow + Enter ${esc(it?.actions?.[1]?.label || '—')}</span><span>Tab open</span><span>Esc back</span>
          <code>line: ${esc(it?.actions?.[0]?.line || r.line)}</code></footer>
      </div>`;
    }
    for (const b of this.querySelectorAll('.lc-sg-row')) {
      b.addEventListener('click', () => {
        this.sel = Number(b.dataset.i);
        this.render();
        this.emit('take', { item: this.selected() });
      });
    }
  }
}
customElements.define('luma-suggest', LumaSuggest);

// ── The query / reply on the page's shell socket ─────────────────────────
// One sequence number for the page: a reply older than its controller's
// newest query is dropped.
let seq = 0;
const waiting = new Map(); // seq → controller

/** Route a `suggestions` message to the list that asked. */
export function receive(msg) {
  const ctl = waiting.get(msg.seq);
  if (!ctl) return;
  waiting.delete(msg.seq);
  ctl.receive(msg);
}

/** The command line after taking `item` from `reply` (for a local input). */
export function apply(reply, item) {
  const chars = [...(reply.line || '')];
  const [start, end] = reply.replace || [chars.length, chars.length];
  const line = chars.slice(0, start).join('') + item.insert + chars.slice(end).join('');
  return { line, cursor: start + [...item.insert].length };
}

/** Wire a `<luma-suggest>` to the station (see the top of this file). */
export function attach({ el, surface = 'bar', send, anchor, onTake, onReply }) {
  let latest = 0;
  let shown = false;
  const place = () => {
    if (!anchor || el.mode !== 'inline') return;
    const r = anchor();
    // Clamped to the screen (12c2): never wider than it, never off its left
    // or right edge, and never taller than the room above the line.
    const vw = window.innerWidth;
    const width = Math.min(640, vw - 16);
    el.style.width = `${width}px`;
    el.style.left = `${Math.max(8, Math.min(r.left, vw - width - 8))}px`;
    el.style.bottom = `${Math.max(0, window.innerHeight - r.top + 6)}px`;
    el.style.maxHeight = `${Math.max(120, r.top - 14)}px`;
  };
  const ctl = {
    reply: null,
    query({ line, cursor } = {}) {
      latest = ++seq;
      waiting.set(latest, ctl);
      const msg = { type: 'suggest', seq: latest, surface };
      if (line != null) { msg.line = line; msg.cursor = cursor ?? [...line].length; }
      send(msg);
      return latest;
    },
    receive(reply) {
      if (reply.seq !== latest) return; // stale
      ctl.reply = reply;
      el.update(reply);
      if (!reply.items.length) ctl.close();
      else if (shown) place();
      onReply?.(reply);
    },
    open() {
      if (!ctl.reply?.items?.length) return;
      shown = true;
      el.hidden = false;
      place();
    },
    close() { shown = false; el.hidden = true; },
    isOpen: () => shown && !el.hidden,
    move: step => el.move(step),
    selected: () => el.selected(),
    take() {
      const item = el.selected();
      if (!item || item.disabled) return false;
      onTake?.(item, ctl.reply);
      return true;
    },
  };
  el.hidden = true;
  el.addEventListener('take', e => { if (e.detail.item && !e.detail.item.disabled) onTake?.(e.detail.item, ctl.reply); });
  return ctl;
}
