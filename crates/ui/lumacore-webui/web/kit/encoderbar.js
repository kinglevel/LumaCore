// <luma-encoder-bar> — a screen's encoder bar above the command line (decisions §30 "12f", the
// approved rounds 9–12: concept A, four encoders, Mode above Layer). Drawn from the `encoder`
// feed; every gesture is a command line:
//   · a strip button       → `Set Screen "EncoderGroup" <n>`
//   · MODE (a list)        → `Set Screen "EncoderMode" <mode>`
//   · LAYER / a narrow fader → `Set Screen "EncoderLayer" Next | <layer>`
//   · Page ‹ ›             → `Set Screen "EncoderPage" Previous | Next`
//   · an encoder's fader   → its line on the gesture channel (`Attribute "Dimmer" At 48.5`,
//     `… Fade 2`): press and drag adjusts at once, a tap opens the slider panel (− / +,
//     Keypad), the mouse wheel steps.
// Masters mode is a bank of <luma-fader>s. Cue timing (vertical) gives each encoder F · D · Out
// faders in seconds (a feature group's unset time shows the cue's in brackets); Cue timing
// (horizontal) gives each an In and an Out timeline on one shared scale with the delay-end and
// fade-end handles dragged sideways (the scale holds still while dragging). A narrow bar (a phone) has no pages: it swipes
// through every attribute of the feature.
//
//   el.update(feedData)  ·  events: run { line } · gesture { … } (bubbles)

import './fader.js';
import { bars } from './marks.js';
import { openMenu } from './menu.js';
import { openSliderPanel } from './sliderpanel.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const SECONDS = 10;      // a timing fader's travel spans 0–10 s (more by the keypad)
const EVERY_MS = 33;
const WHEEL_REST_MS = 400;
const STEP = { Coarse: 1, Fine: 0.1, Ultra: 0.01 };
let gestures = 0;
const newId = () => `e${Date.now().toString(36)}.${++gestures}`;
const fmt = v => String(Math.round(v * 100) / 100);
const CUE_MODES = ['CueTimingVertical', 'CueTimingHorizontal'];

class LumaEncoderBar extends HTMLElement {
  constructor() {
    super();
    this.d = null;
    this.wheel = null; // { id, key, value, timer }
  }

  connectedCallback() {
    this.dataset.kit = 'encoder-bar';
    this.ro = new ResizeObserver(() => { if (this.d && this.phone !== this.isPhone()) this.render(); });
    this.ro.observe(this);
  }
  disconnectedCallback() { this.ro?.disconnect(); }
  isPhone() { return this.clientWidth > 0 && this.clientWidth < 600; }

  run(line) { this.dispatchEvent(new CustomEvent('run', { detail: { line }, bubbles: true })); }
  gesture(detail) { this.dispatchEvent(new CustomEvent('gesture', { detail, bubbles: true })); }

  update(d) {
    this.d = d;
    this.hidden = !d?.on;
    // A drag under way keeps its fader; the rest redraws.
    if (this.dragging) { this.pending = true; return; }
    this.render();
  }

  render() {
    const d = this.d;
    if (!d || !d.on) { this.innerHTML = ''; return; }
    this.phone = this.isPhone();
    this.classList.toggle('phone', this.phone);
    const mode = (d.modes || []).find(m => m.word === d.mode) || { label: d.mode };
    const cueMode = CUE_MODES.includes(d.mode);
    const mini = m => m?.kind === 'timing'
      ? `<span class="lc-eb-tmini"><i style="flex:${Math.max(0.05, m.delay)}"></i><b style="flex:${Math.max(0.05, m.fade)}"></b></span>`
      : `<span class="lc-eb-mini">${(m?.values || []).slice(0, 8).map(v => `<b style="height:${Math.max(8, v)}%"></b>`).join('')}</span>`;
    const strip = (d.strip || []).map(b => `<button type="button" class="lc-eb-sb${b.no === d.group ? ' on' : ''}${b.current ? ' current' : ''}" data-group="${b.no}"
        style="--g:${esc(b.color || 'var(--lc-text-dim)')}">
        <span class="lc-eb-sb-name"><small>${b.no}</small> ${esc(b.name)}</span>${b.programmed ? bars(['abs']) : ''}
        ${mini(b.mini)}</button>`).join('');
    const left = `<div class="lc-eb-left">
        <div class="lc-eb-feature">${esc(d.feature || mode.label)}</div>
        <button type="button" class="lc-eb-mode" data-act="mode"><small>MODE</small>${esc(mode.label)}</button>
        ${d.mode === 'Attributes' ? `<button type="button" class="lc-eb-layer" data-act="layer"><small>LAYER</small>${esc(d.layer)}<small>${(d.layers || []).indexOf(d.layer) + 1} of ${(d.layers || []).length}</small></button>` : ''}
      </div>`;
    let body;
    if (d.mode === 'Masters') {
      body = '<div class="lc-eb-masters lc-fader-bank"></div>';
    } else if (d.planned) {
      body = `<div class="lc-eb-note">${esc(d.planned)}</div>`;
    } else if (d.note) {
      body = `<div class="lc-eb-note">${esc(d.note)}</div>`;
    } else {
      const list = this.phone ? d.all || [] : d.encoders || [];
      const draw = d.mode === 'CueTimingHorizontal' ? (e, i) => this.timelineHtml(e, i, list) : cueMode ? (e, i) => this.cueEncoderHtml(e, i) : (e, i) => this.encoderHtml(e, i);
      body = `<div class="lc-eb-encs${this.phone ? ' swipe' : ''}">${list.map(draw).join('')}</div>`;
    }
    const right = (d.mode === 'Attributes' || cueMode) && !this.phone
      ? `<div class="lc-eb-right"><small>Page</small><b>${d.page}/${d.pages || 1}</b>
          <div class="lc-eb-pagebtns"><button type="button" data-act="prev" aria-label="Previous page">‹</button><button type="button" data-act="next" aria-label="Next page">›</button></div></div>`
      : '';
    this.innerHTML = `${d.mode === 'Attributes' || cueMode ? `<div class="lc-eb-strip">${strip}</div>` : ''}<div class="lc-eb-row">${left}${body}${right}</div>`;
    this.wire();
    if (d.mode === 'Masters') {
      const bank = this.querySelector('.lc-eb-masters');
      for (const m of d.masters || []) {
        const f = document.createElement('luma-fader');
        bank.append(f);
        f.update({ label: m.label, sub: m.sub, line: m.line, value: m.value });
      }
    }
  }

  encoderHtml(e, i) {
    if (!e) return `<div class="lc-eb-enc empty" data-i="${i}"><span>—</span></div>`;
    const layer = this.d.layer;
    const ticks = (e.levels || []).slice(0, 16).map(v => `<i style="bottom:${v}%"></i>`).join('');
    const fader = (key, label, text, frac, extra = '') => `<div class="lc-eb-fd${key === layer ? ' wide' : ''}${key === 'Value' && e.programmed ? ' prog' : ''}" data-i="${i}" data-key="${key}">
        ${key === 'Value' ? bars([...(e.programmed ? ['abs'] : []), ...(e.fade?.value != null ? ['fade'] : []), ...(e.delay?.value != null ? ['delay'] : [])]) : ''}<small>${label}</small><span class="lc-eb-text">${esc(text)}</span>
        <div class="lc-eb-track"><div class="lc-eb-fill" style="height:${Math.max(0, Math.min(1, frac)) * 100}%"></div>${extra}</div></div>`;
    const sec = t => (t?.value == null ? 0 : t.value / SECONDS);
    return `<div class="lc-eb-enc" data-i="${i}">
        <div class="lc-eb-fds">${fader('Value', 'VALUE', e.readout, (e.value ?? 0) / 100, ticks)}${fader('Fade', 'F', e.fade?.text, sec(e.fade))}${fader('Delay', 'D', e.delay?.text, sec(e.delay))}</div>
        <div class="lc-eb-attr"><b>${esc(e.pretty)}</b>${e.fade?.value != null || e.delay?.value != null ? '<span class="lc-eb-badge" title="Programmer timing"><luma-icon name="timer"></luma-icon></span>' : ''}
          <small>${esc(e.attr)}</small><span class="lc-eb-res" title="Encoder resolution">${esc((e.resolution || 'Coarse')[0])}</span></div></div>`;
  }

  /** Cue timing · vertical: F · D · Out of the cue or a feature group (inherited in brackets). */
  cueEncoderHtml(e, i) {
    if (!e) return `<div class="lc-eb-enc empty" data-i="${i}"><span>—</span></div>`;
    const fader = (key, label, f) => `<div class="lc-eb-fd${f?.inherited ? ' inherited' : ''}" data-i="${i}" data-key="${key}">
        <small>${label}</small><span class="lc-eb-text">${esc(f?.text)}</span>
        <div class="lc-eb-track"><div class="lc-eb-fill" style="height:${Math.min(1, (f?.value ?? 0) / SECONDS) * 100}%"></div></div></div>`;
    return `<div class="lc-eb-enc" data-i="${i}" style="--g:${esc(e.color || 'transparent')}">
        <div class="lc-eb-fds">${fader('in_fade', 'F', e.in?.fade)}${fader('in_delay', 'D', e.in?.delay)}${fader('out_fade', 'OUT', e.out?.fade)}</div>
        <div class="lc-eb-attr"><b>${esc(e.label)}</b><small>${esc(e.sub)}</small></div></div>`;
  }

  /** Cue timing · horizontal: an In and an Out timeline (delay, then the fade ramp) on the scale
   *  every drawn encoder shares: at least 2 s past the longest delay + fade, +20 %. */
  timelineHtml(e, i, list) {
    if (!e) return `<div class="lc-eb-enc empty" data-i="${i}"><span>—</span></div>`;
    const scale = this.scale ?? this.scaleOf(list);
    const row = (dir, t) => {
      const delay = t?.delay?.value ?? 0, fade = t?.fade?.value ?? 0;
      const pc = s => `${Math.min(100, (s / scale) * 100)}%`;
      return `<div class="lc-eb-tl" data-i="${i}" data-dir="${dir}">
          <small>${dir === 'in' ? 'IN' : 'OUT'}</small>
          <div class="lc-eb-tl-track">
            <i class="lc-eb-tl-delay" style="width:${pc(delay)}"></i>
            <b class="lc-eb-tl-fade${dir}" style="left:${pc(delay)};width:${pc(fade)}"></b>
            <span class="lc-eb-tl-h" data-h="delay" style="left:${pc(delay)}" title="End of delay"></span>
            <span class="lc-eb-tl-h fade" data-h="fade" style="left:${pc(delay + fade)}" title="End of fade"></span>
          </div>
          <span class="lc-eb-tl-text${t?.delay?.inherited && t?.fade?.inherited ? ' inherited' : ''}">${esc(t?.delay?.text)} + ${esc(t?.fade?.text)}</span>
        </div>`;
    };
    return `<div class="lc-eb-enc" data-i="${i}">${row('in', e.in)}${row('out', e.out)}
        <div class="lc-eb-attr"><b>${esc(e.label)}</b><small>${esc(e.sub)} · 0–${fmt(scale)} s</small></div></div>`;
  }

  scaleOf(list) {
    let longest = 0;
    for (const e of list || []) {
      for (const t of [e?.in, e?.out]) longest = Math.max(longest, (t?.delay?.value ?? 0) + (t?.fade?.value ?? 0));
    }
    return Math.max(2, Math.ceil(longest * 1.2));
  }

  /** The encoder `i` of what is drawn, and what a fader of it sends. */
  target(i, key) {
    const e = (this.phone ? this.d.all : this.d.encoders)?.[i];
    if (!e) return null;
    if (CUE_MODES.includes(this.d.mode)) {
      const [dir, what] = key.split('_');
      const f = e[dir]?.[what];
      return { e, seconds: true, line: e.lines?.[key], value: f?.value ?? 0, max: SECONDS, step: 0.1,
        label: `${e.label} ${dir === 'in' ? '' : 'Out '}${what === 'fade' ? 'Fade' : 'Delay'}`, unit: ' s' };
    }
    const seconds = key !== 'Value';
    const t = key === 'Fade' ? e.fade : key === 'Delay' ? e.delay : null;
    return {
      e,
      seconds,
      line: e.lines?.[key.toLowerCase()],
      value: seconds ? (t?.value ?? 0) : (e.value ?? 0),
      max: seconds ? SECONDS : 100,
      step: seconds ? 0.1 : STEP[e.resolution] || 1,
      label: `${e.pretty}${seconds ? ` ${key}` : ''}`,
      unit: seconds ? ' s' : '%',
    };
  }

  send(t, id, phase, v, from) {
    const detail = { id, line: t.line.replace('{value}', fmt(v)), phase, value: fmt(v) };
    if (from != null) Object.assign(detail, { label: t.label, from: fmt(from) });
    this.gesture(detail);
  }

  wire() {
    for (const b of this.querySelectorAll('[data-group]')) b.onclick = () => this.run(`Set Screen "EncoderGroup" ${b.dataset.group}`);
    this.querySelector('[data-act="prev"]')?.addEventListener('click', () => this.run('Set Screen "EncoderPage" Previous'));
    this.querySelector('[data-act="next"]')?.addEventListener('click', () => this.run('Set Screen "EncoderPage" Next'));
    this.querySelector('[data-act="layer"]')?.addEventListener('click', () => this.run('Set Screen "EncoderLayer" Next'));
    this.querySelector('[data-act="mode"]')?.addEventListener('click', e => {
      // The page closes its menu on a click elsewhere: this click opens it.
      e.stopPropagation();
      const r = e.currentTarget.getBoundingClientRect();
      openMenu({ x: r.left, y: r.top - 8 }, [
        { header: 'Encoder bar mode' },
        ...(this.d.modes || []).map(m => ({
          label: m.label,
          hint: m.planned ? 'planned' : '',
          check: m.word === this.d.mode,
          fn: () => this.run(`Set Screen "EncoderMode" ${m.word}`),
        })),
      ]);
    });
    for (const fd of this.querySelectorAll('.lc-eb-fd')) this.wireFader(fd);
    for (const h of this.querySelectorAll('.lc-eb-tl-h')) this.wireHandle(h);
  }

  /** A timeline handle: dragged sideways, the delay-end changes the delay, the fade-end the fade;
   *  the scale holds still while dragging (round 7). */
  wireHandle(h) {
    const row = h.closest('.lc-eb-tl');
    const i = Number(row.dataset.i), dir = row.dataset.dir, which = h.dataset.h;
    let g = null;
    h.addEventListener('pointerdown', e => {
      const t = this.target(i, `${dir}_${which}`);
      const other = this.target(i, `${dir}_${which === 'fade' ? 'delay' : 'fade'}`);
      if (!t?.line) return;
      e.preventDefault();
      e.stopPropagation();
      try { h.setPointerCapture(e.pointerId); } catch { /* synthetic */ }
      this.scale = this.scaleOf(this.phone ? this.d.all : this.d.encoders);
      const track = row.querySelector('.lc-eb-tl-track').getBoundingClientRect();
      g = { t, id: newId(), x0: e.clientX, v0: t.value, v: t.value, delay: which === 'fade' ? other.value : 0, width: track.width, moved: false, last: 0 };
      this.dragging = true;
    });
    h.addEventListener('pointermove', e => {
      if (!g) return;
      const dx = e.clientX - g.x0;
      if (!g.moved && Math.abs(dx) < 3) return;
      g.moved = true;
      const v = Math.max(0, g.v0 + (dx / g.width) * this.scale);
      g.v = Math.round(v * 10) / 10;
      h.style.left = `${Math.min(100, ((g.delay + g.v) / this.scale) * 100)}%`;
      const now = performance.now();
      if (now - g.last >= EVERY_MS) {
        g.last = now;
        this.send(g.t, g.id, 'move', g.v, g.sent ? null : g.v0);
        g.sent = true;
      }
    });
    const up = () => {
      if (!g) return;
      const was = g;
      g = null;
      this.dragging = false;
      if (was.moved) this.send(was.t, was.id, 'end', was.v, was.sent ? null : was.v0);
      this.scale = null;
      this.pending = false;
      this.render();
    };
    h.addEventListener('pointerup', up);
    h.addEventListener('pointercancel', up);
  }

  wireFader(fd) {
    const i = Number(fd.dataset.i), key = fd.dataset.key;
    const track = fd.querySelector('.lc-eb-track');
    let g = null;
    fd.addEventListener('pointerdown', e => {
      const t = this.target(i, key);
      if (!t?.line) return;
      e.preventDefault();
      try { fd.setPointerCapture(e.pointerId); } catch { /* synthetic */ }
      g = { t, id: newId(), y0: e.clientY, v0: t.value, v: t.value, moved: false, last: 0, x: e.clientX, y: e.clientY };
    });
    fd.addEventListener('pointermove', e => {
      if (!g) return;
      const dy = e.clientY - g.y0;
      if (!g.moved && Math.abs(dy) < 3) return;
      if (!g.moved) {
        g.moved = true;
        this.dragging = true;
        fd.classList.add('active');
      }
      const h = Math.max(60, track.clientHeight);
      const v = Math.min(g.t.max, Math.max(0, g.v0 - (dy / h) * g.t.max));
      g.v = Math.round(v / g.t.step) * g.t.step;
      fd.querySelector('.lc-eb-fill').style.height = `${(g.v / g.t.max) * 100}%`;
      fd.querySelector('.lc-eb-text').textContent = `${fmt(g.v)}${g.t.seconds ? 's' : '%'}`;
      const now = performance.now();
      if (now - g.last >= EVERY_MS) {
        g.last = now;
        this.send(g.t, g.id, 'move', g.v, g.sentFrom ? null : g.v0);
        g.sentFrom = true;
      }
    });
    const up = () => {
      if (!g) return;
      const was = g;
      g = null;
      fd.classList.remove('active');
      this.dragging = false;
      if (was.moved) {
        this.send(was.t, was.id, 'end', was.v, was.sentFrom ? null : was.v0);
      } else if (!CUE_MODES.includes(this.d.mode) && key !== this.d.layer) {
        // A tap on a narrow fader picks its layer for every encoder.
        this.run(`Set Screen "EncoderLayer" ${key}`);
      } else {
        this.panel(was.t, was.x, was.y);
      }
      if (this.pending) { this.pending = false; this.render(); }
    };
    fd.addEventListener('pointerup', up);
    fd.addEventListener('pointercancel', up);
    fd.addEventListener('wheel', e => {
      const t = this.target(i, key);
      if (!t?.line) return;
      e.preventDefault();
      const k = `${i}.${key}`;
      if (!this.wheel || this.wheel.key !== k) this.wheel = { id: newId(), key: k, value: t.value, from: t.value };
      const w = this.wheel;
      w.value = Math.min(t.max, Math.max(0, w.value + (e.deltaY < 0 ? 1 : -1) * t.step));
      this.send(t, w.id, 'move', w.value, w.sent ? null : w.from);
      w.sent = true;
      clearTimeout(w.timer);
      w.timer = setTimeout(() => {
        this.send(t, w.id, 'end', w.value);
        this.wheel = null;
      }, WHEEL_REST_MS);
    }, { passive: false });
  }

  panel(t, x, y) {
    let id = newId(), first = true;
    const from = t.value;
    openSliderPanel({
      at: { x, y: y - 140 },
      title: t.label,
      value: t.value,
      min: 0,
      max: t.max,
      step: t.step,
      unit: t.unit,
      dim: this.d?.popup_dim,
      onMove: v => { this.send(t, id, 'move', v, first ? from : null); first = false; },
      onEnd: v => { this.send(t, id, 'end', v, first ? from : null); id = newId(); first = true; },
      onKeypad: text => this.run(t.line.replace('{value}', text)),
    });
  }
}
customElements.define('luma-encoder-bar', LumaEncoderBar);
