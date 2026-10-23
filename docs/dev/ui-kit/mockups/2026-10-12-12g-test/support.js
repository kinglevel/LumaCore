// Minimal offline runtime for the mockup boards (*.dc.html).
//
// The boards were drawn on a claude.ai Design canvas, whose runtime renders the
// `<x-dc>` template format. This file lets the same sources open straight from
// the repo in any browser: `{{path}}` holes, `<sc-for list as>`, `<sc-if value>`,
// `<helmet>`, `onX="{{fn}}"` handlers and `class Component extends DCLogic`
// with props / state / setState. Re-renders patch the DOM in place, so pointer
// capture (fader drags) survives. Not part of the LumaCore frontend.
(function () {
  'use strict';

  class DCLogic {
    constructor(props) { this.props = props || {}; this.state = {}; }
    setState(patch) { Object.assign(this.state, typeof patch === 'function' ? patch(this.state) : patch); this.__render(); }
    forceUpdate() { this.__render(); }
  }
  window.DCLogic = DCLogic;

  const WHOLE = /^\s*\{\{\s*([^}]+?)\s*\}\}\s*$/;
  const HOLE = /\{\{\s*([^}]+?)\s*\}\}/g;
  const EVENTS = { change: 'input', doubleclick: 'dblclick' };

  function lookup(path, scope) {
    if (path === 'true') return true;
    if (path === 'false') return false;
    if (path === 'null') return null;
    if (/^-?\d+(\.\d+)?$/.test(path)) return Number(path);
    if (/^(['"]).*\1$/.test(path)) return path.slice(1, -1);
    let v = scope;
    for (const k of path.split('.')) { if (v == null) return undefined; v = v[k]; }
    return v;
  }
  const text = (s, scope) => s.replace(HOLE, (_, p) => { const v = lookup(p, scope); return v == null ? '' : String(v); });

  // Template node → array of fresh nodes for this scope.
  function build(node, scope) {
    if (node.nodeType === 3) return [document.createTextNode(text(node.data, scope))];
    if (node.nodeType !== 1) return [];
    const tag = node.localName;
    if (tag === 'helmet') return [];
    if (tag === 'sc-if') {
      const m = WHOLE.exec(node.getAttribute('value') || '');
      return (m ? lookup(m[1], scope) : false) ? kids(node, scope) : [];
    }
    if (tag === 'sc-for') {
      const m = WHOLE.exec(node.getAttribute('list') || '');
      const list = (m ? lookup(m[1], scope) : null) || [];
      const as = node.getAttribute('as') || 'item';
      const out = [];
      list.forEach((item, i) => out.push(...kids(node, Object.assign(Object.create(scope), { [as]: item, $index: i }))));
      return out;
    }
    const el = document.createElementNS(node.namespaceURI, node.localName);
    el.__dcl = {};
    for (const a of node.attributes) {
      if (a.name.startsWith('hint-')) continue;
      const m = WHOLE.exec(a.value);
      if (m) {
        const v = lookup(m[1], scope);
        if (/^on[a-z]/i.test(a.name)) {
          const ev = a.name.slice(2).toLowerCase();
          if (typeof v === 'function') el.__dcl[EVENTS[ev] || ev] = v;
          continue;
        }
        if (v === false || v == null) continue;
        el.setAttribute(a.name, v === true ? '' : String(v));
      } else {
        el.setAttribute(a.name, text(a.value, scope));
      }
    }
    for (const c of kids(node, scope)) el.appendChild(c);
    return [el];
  }
  function kids(node, scope) {
    const src = node.localName === 'template' ? node.content : node;
    const out = [];
    for (const c of src.childNodes) out.push(...build(c, scope));
    return out;
  }

  function bind(el) {
    el.__bound = el.__bound || new Set();
    for (const ev of Object.keys(el.__dcl || {})) {
      if (el.__bound.has(ev)) continue;
      el.__bound.add(ev);
      el.addEventListener(ev, e => { const f = el.__dcl[ev]; if (f) f(e); });
    }
  }

  // Patch `parent`'s children to match `fresh` without replacing equal nodes.
  function morph(parent, fresh) {
    const old = Array.from(parent.childNodes);
    fresh.forEach((n, i) => {
      const o = old[i];
      if (o && o.nodeType === n.nodeType && (n.nodeType === 3 || o.tagName === n.tagName)) {
        if (n.nodeType === 3) { if (o.data !== n.data) o.data = n.data; return; }
        for (const a of Array.from(o.attributes)) if (!n.hasAttribute(a.name)) o.removeAttribute(a.name);
        for (const a of n.attributes) if (o.getAttribute(a.name) !== a.value) o.setAttribute(a.name, a.value);
        if ('value' in o && n.hasAttribute('value') && o.value !== n.getAttribute('value') && document.activeElement !== o) o.value = n.getAttribute('value');
        o.__dcl = n.__dcl; bind(o);
        morph(o, Array.from(n.childNodes));
      } else {
        bindTree(n);
        if (o) parent.replaceChild(n, o); else parent.appendChild(n);
      }
    });
    for (let i = fresh.length; i < old.length; i++) old[i].remove();
  }
  function bindTree(n) { if (n.nodeType !== 1) return; bind(n); for (const c of n.children) bindTree(c); }

  function start() {
    const host = document.querySelector('x-dc');
    const script = document.querySelector('script[data-dc-script]');
    if (!host || !script) return;
    const tpl = document.createElement('template');
    tpl.innerHTML = host.innerHTML;
    for (const h of tpl.content.querySelectorAll('helmet')) for (const c of Array.from(h.children)) document.head.appendChild(c.cloneNode(true));
    let defs = {};
    try { defs = JSON.parse(script.getAttribute('data-props') || '{}'); } catch (_) {}
    const props = {};
    for (const [k, d] of Object.entries(defs)) if (!k.startsWith('$') && d && 'default' in d) props[k] = d.default;
    const Component = new Function('DCLogic', script.textContent + '\n;return Component;')(DCLogic);
    const comp = new Component(props);
    comp.__render = () => morph(host, kids(tpl, comp.renderVals() || {}));
    host.innerHTML = '';
    host.style.display = 'contents';
    comp.__render();
    if (comp.componentDidMount) comp.componentDidMount();
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start); else start();
})();
