// <luma-surface> — the one surface for connect, login and lock (decisions §30
// "12a3", boards Lock / LockPhone / Connect / Surface / Screens). Full screen
// over the user's lock picture (or the built-in background): date and clock,
// the station line, the user, a password field (with the built-in keyboard)
// or "tap to unlock", the other users as avatars, `Servers…` and the status
// line. Driven by `update(data)`; every answer is an event the page turns
// into a login, an unlock or a command line:
//
//   surface-pick {user} · surface-login {user, password} · surface-unlock
//   {password} · surface-corners · surface-servers {open} · surface-server
//   {index} · surface-address {text} · surface-local {action} ·
//   surface-toggle {name, on} · surface-now · surface-stop · surface-focus
//
// data: { mode: 'connect'|'login'|'locked'|'blurred', station, user, error,
//   countdown: {seconds, total, user?, to?}, busy, waiting, kiosk, remote,
//   showServers, servers: {rows, local, address, toggles}, hint }
// A server row: {name, info, note, dot: 'ok'|'warn'|'alert'|'off', warn, ok,
//   off, current} — `warn` / `ok` colour the note (§30 "12b2 hardening":
//   green fresh, amber "not answering N s", grey wrong version).

import './avatar.js';
import './keyboard.js';

const BUILT_IN = new URL('./lockbg.jpg', import.meta.url).href;
const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const CORNERS = ['tl', 'br', 'tr', 'bl'];
const CORNER_PX = 64, CORNER_MS = 3000;

class LumaSurface extends HTMLElement {
  constructor() {
    super();
    this.data = { mode: 'login' };
    this.picked = null;      // a user picked for switch / login
    this.keyboardOpen = false;
    this.corners = [];
  }

  connectedCallback() {
    if (this.built) return;
    this.built = true;
    this.dataset.kit = 'surface';
    this.innerHTML = `
      <div class="lc-sf-bg"></div><div class="lc-sf-shade"></div>
      <div class="lc-sf-top"><span class="lc-sf-date"></span><span class="lc-sf-clock"></span>
        <span class="lc-sf-station"></span><span class="lc-sf-text"></span></div>
      <div class="lc-sf-center"></div>
      <div class="lc-sf-row"></div>
      <div class="lc-sf-kb" hidden></div>
      <div class="lc-sf-bottom"><button type="button" class="lc-sf-pill lc-sf-servers">Servers…</button>
        <span class="lc-sf-status"></span><span class="lc-sf-pill lc-sf-hint"></span></div>`;
    this.pw = document.createElement('input');
    this.pw.type = 'password';
    this.pw.className = 'lc-sf-pw';
    this.pw.placeholder = 'Password';
    this.pw.autocomplete = 'current-password';
    this.pw.addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); this.submit(); }
      e.stopPropagation();
    });
    this.kb = document.createElement('luma-keyboard');
    this.kb.setAttribute('mode', 'password');
    this.kb.target = this.pw;
    this.kb.addEventListener('please', () => this.submit());
    this.kb.addEventListener('hide', () => { this.keyboardOpen = false; this.render(); });
    this.kb.addEventListener('escape', () => { this.keyboardOpen = false; this.pw.value = ''; this.render(); });
    this.querySelector('.lc-sf-kb').append(this.kb);
    this.querySelector('.lc-sf-servers').addEventListener('click', () => this.emit('surface-servers', { open: !this.data.showServers }));
    this.addEventListener('pointerdown', e => this.corner(e), true);
    this.clock = setInterval(() => this.tick(), 1000);
    this.tick();
    this.render();
  }

  disconnectedCallback() { clearInterval(this.clock); this.built = false; }

  emit(name, detail = {}) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }

  /** New data (merged); renders. */
  update(data) {
    const userChanged = data.user !== undefined && data.user !== this.data.user;
    const modeChanged = data.mode !== undefined && data.mode !== this.data.mode;
    const before = JSON.stringify(this.data);
    this.data = { ...this.data, ...data };
    // A status that changes nothing redraws nothing: the centre (and the password
    // field someone is typing into) stays as it is.
    if (JSON.stringify(this.data) === before) return;
    if (userChanged || modeChanged) { this.picked = null; this.pw.value = ''; }
    if (this.data.kiosk && (userChanged || modeChanged)) this.keyboardOpen = true;
    this.render();
  }

  /** The password field (the page focuses it, tests type into it). */
  get password() { return this.pw; }

  users() { return this.data.station?.users || []; }
  user(name) { return this.users().find(u => u.name.toLowerCase() === String(name ?? '').toLowerCase()); }

  /** The user the centre shows: picked, else the current one. */
  shown() { return this.user(this.picked) || this.user(this.data.user); }

  submit() {
    const u = this.shown();
    if (!u) return;
    const pw = this.pw.value;
    if (this.data.mode === 'locked' && !this.picked) this.emit('surface-unlock', { password: pw });
    else this.emit('surface-login', { user: u.name, password: u.has_password ? pw : null });
  }

  tick() {
    const now = new Date();
    this.querySelector('.lc-sf-date').textContent = now.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' });
    this.querySelector('.lc-sf-clock').textContent = now.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' });
  }

  /** The four-corner tap pattern unlocks a user without a password. */
  corner(e) {
    if (this.data.mode !== 'locked') return;
    const r = this.getBoundingClientRect();
    const x = e.clientX - r.left, y = e.clientY - r.top;
    const left = x < CORNER_PX, right = x > r.width - CORNER_PX, top = y < CORNER_PX, bottom = y > r.height - CORNER_PX;
    const at = top && left ? 'tl' : bottom && right ? 'br' : top && right ? 'tr' : bottom && left ? 'bl' : null;
    const now = Date.now();
    this.corners = this.corners.filter(c => now - c.t < CORNER_MS);
    if (!at) return;
    this.corners.push({ at, t: now });
    const seq = this.corners.map(c => c.at).join(',');
    if (seq.endsWith(CORNERS.join(','))) { this.corners = []; this.emit('surface-corners'); }
  }

  look(u) { return u?.look?.lock_screen || null; }

  render() {
    if (!this.built) return;
    // The centre is rebuilt: the password field keeps its focus and caret
    // across it (a status landing mid-typing must not drop keystrokes, and
    // WebKit's fill inserts only into the focused field).
    const refocus = document.activeElement === this.pw;
    const caret = refocus ? [this.pw.selectionStart, this.pw.selectionEnd] : null;
    const d = this.data;
    const blurred = d.mode === 'blurred';
    this.classList.toggle('blurred', blurred);
    const u = this.shown();
    const lock = this.look(u || this.user(d.user));
    const img = lock?.image?.sha256 ? `/api/media/${lock.image.sha256}` : BUILT_IN;
    this.querySelector('.lc-sf-bg').style.backgroundImage = `url("${img}")`;
    const st = d.station;
    this.querySelector('.lc-sf-clock').hidden = lock?.show_clock === false && d.mode !== 'connect';
    this.querySelector('.lc-sf-date').hidden = this.querySelector('.lc-sf-clock').hidden;
    const line = d.mode === 'connect'
      ? (d.subtitle || 'LumaCore · this computer')
      : [st?.name, st?.status?.lumasession ? `LumaSession "${st.status.lumasession}"` : null, st?.show ? `show ${st.show}` : null].filter(Boolean).join(' · ');
    this.querySelector('.lc-sf-station').innerHTML = `<i class="lc-sf-dot"></i>${esc(line)}`;
    this.querySelector('.lc-sf-text').textContent = d.mode === 'connect' ? '' : (lock?.text || '');
    const center = this.querySelector('.lc-sf-center');
    const row = this.querySelector('.lc-sf-row');
    const bottom = this.querySelector('.lc-sf-bottom');
    const kbHost = this.querySelector('.lc-sf-kb');
    center.innerHTML = ''; row.innerHTML = '';
    bottom.hidden = blurred;
    if (blurred) {
      center.innerHTML = '<span class="lc-sf-note">Locked · tap to show the surface here</span>';
      center.onclick = () => this.emit('surface-focus');
      kbHost.hidden = true;
      return;
    }
    center.onclick = null;
    if (d.mode === 'connect') this.renderConnect(center);
    else if (d.showServers) this.renderServers(center);
    else if (!u) this.renderWho(center);
    else this.renderUser(center, u);
    if (!d.showServers && d.mode !== 'connect' && (u || d.mode === 'locked')) this.renderRow(row, u);
    kbHost.hidden = !(this.keyboardOpen && this.pw.isConnected);
    this.classList.toggle('kb-open', !kbHost.hidden);
    this.renderBottom();
    if (refocus && this.pw.isConnected && document.activeElement !== this.pw) {
      this.pw.focus({ preventScroll: true });
      if (caret && caret[0] != null) try { this.pw.setSelectionRange(caret[0], caret[1]); } catch (_) { /* not selectable */ }
    }
  }

  avatar(u, { size = 68, big = false, onTap } = {}) {
    const a = document.createElement('luma-avatar');
    a.setAttribute('name', u.name);
    a.setAttribute('color', u.color || '');
    if (u.rights) a.setAttribute('level', u.rights);
    if (u.profile) a.setAttribute('badge', u.profile);
    a.setAttribute('size', String(size));
    if (big) a.setAttribute('big', '');
    if (this.blocked(u)) a.setAttribute('dim', '');
    a.setAttribute('state', this.presence(u));
    a.dataset.user = u.name;
    if (onTap) a.addEventListener('click', onTap);
    return a;
  }

  /** Can't log in from here: Remote Login off on a remote device. */
  blocked(u) { return !!this.data.remote && u.remote_login === false; }

  presence(u) {
    if (this.blocked(u)) return `only on ${this.data.station?.name ?? 'its station'}`;
    const here = this.data.station?.name;
    if (u.name === this.data.user) return 'this device';
    const at = u.logged_in || [];
    if (!at.length) return 'logged out';
    return 'on ' + at.map(s => (s === here ? s : s)).join(', ');
  }

  renderWho(center) {
    center.innerHTML = '<span class="lc-sf-ask">Who\'s working here?</span><div class="lc-sf-who"></div>';
    const who = center.querySelector('.lc-sf-who');
    for (const u of this.users()) who.append(this.avatar(u, { size: 96, onTap: () => { this.picked = u.name; this.pw.value = ''; this.emit('surface-pick', { user: u.name }); this.render(); } }));
  }

  renderUser(center, u) {
    const d = this.data;
    const switching = !!this.picked && d.mode === 'locked';
    center.append(this.avatar(u, { size: 128, big: true }));
    const name = center.querySelector('luma-avatar');
    name.classList.add('lc-sf-me');
    const counting = d.countdown && (!d.countdown.user || d.countdown.user === u.name) && !u.has_password;
    if (counting) {
      const left = Math.max(0, d.countdown.seconds);
      const total = d.countdown.total || 5;
      center.insertAdjacentHTML('beforeend', `<div class="lc-sf-bar"><i style="width:${(left / total) * 100}%"></i></div>
        <span class="lc-sf-note">Logging in in ${Math.ceil(left)} s — tap, Enter or Space now · Esc stops</span>`);
      name.addEventListener('click', () => this.emit('surface-now'));
    } else if (u.has_password) {
      // A <div>, not a <label>: a label hands its clicks on (WebKit toggled
      // the keyboard button twice).
      const field = document.createElement('div');
      field.className = 'lc-sf-field' + (d.error ? ' error' : '');
      field.append(this.pw);
      const kbBtn = document.createElement('button');
      kbBtn.type = 'button'; kbBtn.className = 'lc-sf-round quiet'; kbBtn.setAttribute('aria-label', 'Keyboard');
      kbBtn.innerHTML = '<luma-icon name="keyboard"></luma-icon>';
      kbBtn.addEventListener('click', () => { this.keyboardOpen = !this.keyboardOpen; this.render(); this.pw.focus(); });
      const go = document.createElement('button');
      go.type = 'button'; go.className = 'lc-sf-round go'; go.setAttribute('aria-label', switching ? 'Switch user' : 'Log in');
      go.innerHTML = '<luma-icon name="chevron-right"></luma-icon>';
      go.addEventListener('click', () => this.submit());
      field.append(kbBtn, go);
      center.append(field);
      center.insertAdjacentHTML('beforeend', `<span class="lc-sf-error">${esc(d.error || '')}</span>`);
      if (!this.keyboardOpen) setTimeout(() => this.pw.isConnected && document.activeElement !== this.pw && this.pw.focus({ preventScroll: true }), 0);
    } else if (d.mode === 'locked' && !switching) {
      const pill = document.createElement('button');
      pill.type = 'button'; pill.className = 'lc-sf-pill lc-sf-unlock';
      pill.textContent = 'Tap or press F9 to unlock';
      pill.addEventListener('click', () => this.emit('surface-unlock', { password: null }));
      name.addEventListener('click', () => this.emit('surface-unlock', { password: null }));
      center.append(pill);
    } else {
      const btn = document.createElement('button');
      btn.type = 'button'; btn.className = 'lc-sf-pill go';
      btn.textContent = switching ? `Switch to ${u.name}` : `Log in as ${u.name}`;
      btn.addEventListener('click', () => this.submit());
      center.append(btn);
    }
    if (switching || (d.mode === 'login' && this.picked)) {
      const cancel = document.createElement('button');
      cancel.type = 'button'; cancel.className = 'lc-sf-link'; cancel.textContent = 'Cancel';
      cancel.addEventListener('click', () => { this.picked = null; this.pw.value = ''; this.emit('surface-pick', { user: null }); this.render(); });
      center.append(cancel);
      if (switching) center.insertAdjacentHTML('beforeend', `<span class="lc-sf-note">Every window of this device logs in as ${esc(u.name)}. ${esc(d.user)}'s programmer, selection and views stay on ${esc(d.user)}.</span>`);
    }
    if (d.mode === 'locked' && d.waiting > 0) {
      center.insertAdjacentHTML('beforeend', `<span class="lc-sf-waiting"><i></i>${d.waiting} waiting — shown after unlocking</span>`);
    }
  }

  renderRow(row, current) {
    const d = this.data;
    for (const u of this.users()) {
      if (current && u.name === current.name) continue;
      row.append(this.avatar(u, {
        onTap: () => {
          if (this.blocked(u)) return;
          this.picked = u.name; this.pw.value = ''; this.emit('surface-pick', { user: u.name }); this.render();
        },
      }));
    }
    if (d.mode === 'login' && current && !this.picked) {
      const other = document.createElement('button');
      other.type = 'button'; other.className = 'lc-sf-link'; other.textContent = 'Someone else…';
      other.addEventListener('click', () => { this.picked = null; this.update({ user: null }); this.emit('surface-pick', { user: null }); });
      row.append(other);
    }
  }

  renderConnect(center) {
    const d = this.data;
    if (d.busy) {
      center.innerHTML = `<span class="lc-sf-ask">${esc(d.busy)}</span><div class="lc-sf-bar busy"><i></i></div>`;
      const c = document.createElement('button');
      c.type = 'button'; c.className = 'lc-sf-pill'; c.textContent = 'Cancel';
      c.addEventListener('click', () => this.emit('surface-stop'));
      center.append(c);
      return;
    }
    if (d.countdown && !d.showServers) {
      const left = Math.max(0, d.countdown.seconds), total = d.countdown.total || 5;
      center.innerHTML = `<span class="lc-sf-ask">${esc(d.countdown.to || '')}</span>
        <span class="lc-sf-mono">${esc(d.countdown.info || '')}</span>
        <div class="lc-sf-bar"><i style="width:${(left / total) * 100}%"></i></div>
        <span class="lc-sf-note">in ${Math.ceil(left)} s</span><div class="lc-sf-actions"></div>`;
      const a = center.querySelector('.lc-sf-actions');
      const now = document.createElement('button'); now.type = 'button'; now.className = 'lc-sf-pill go'; now.textContent = 'Connect now';
      now.addEventListener('click', () => this.emit('surface-now'));
      const srv = document.createElement('button'); srv.type = 'button'; srv.className = 'lc-sf-pill'; srv.textContent = 'Servers… (Esc)';
      srv.addEventListener('click', () => this.emit('surface-stop'));
      a.append(now, srv);
      return;
    }
    this.renderServers(center);
  }

  renderServers(center) {
    const s = this.data.servers || {};
    const panel = document.createElement('div');
    panel.className = 'lc-sf-panel';
    panel.innerHTML = `<div class="lc-sf-panel-head"><span>Servers</span><span class="lc-sf-note">${esc(s.searching || '')}</span></div>`
      + (this.data.error ? `<span class="lc-sf-error">${esc(this.data.error)}</span>` : '');
    if (this.data.mode !== 'connect') {
      const close = document.createElement('button');
      close.type = 'button'; close.className = 'lc-sf-round quiet'; close.setAttribute('aria-label', 'Close');
      close.innerHTML = '<luma-icon name="x"></luma-icon>';
      close.addEventListener('click', () => this.emit('surface-servers', { open: false }));
      panel.querySelector('.lc-sf-panel-head').append(close);
    }
    if (s.local) {
      const l = document.createElement('div');
      l.className = 'lc-sf-local';
      l.innerHTML = `<span><b>This computer</b><small>${esc(s.local.text)}</small></span>`;
      for (const act of s.local.actions || []) {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'lc-sf-btn' + (act.primary ? ' go' : '');
        b.textContent = act.label;
        b.addEventListener('click', () => this.emit('surface-local', { action: act.action }));
        l.append(b);
      }
      panel.append(l);
    }
    (s.rows || []).forEach((r, i) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'lc-sf-server' + (r.current ? ' current' : '') + (r.off ? ' off' : '');
      b.innerHTML = `<i class="lc-sf-dot ${r.dot || ''}"></i><span><b>${esc(r.name)}</b><small>${esc(r.info)}</small></span><em class="${r.warn ? 'warn' : r.ok ? 'ok' : ''}">${esc(r.note || '')}</em>`;
      b.addEventListener('click', () => this.emit('surface-server', { index: i }));
      panel.append(b);
    });
    if (s.address !== false) {
      const a = document.createElement('div');
      a.className = 'lc-sf-addr';
      a.innerHTML = '<input type="text" placeholder="Address or host:port" spellcheck="false"><button type="button" class="lc-sf-btn">Connect</button>';
      const input = a.querySelector('input');
      const go = () => input.value.trim() && this.emit('surface-address', { text: input.value.trim() });
      input.addEventListener('keydown', e => { e.stopPropagation(); if (e.key === 'Enter') go(); });
      a.querySelector('button').addEventListener('click', go);
      panel.append(a);
    }
    if (s.toggles?.length) {
      const t = document.createElement('div');
      t.className = 'lc-sf-toggles';
      for (const tg of s.toggles) {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'lc-sf-check' + (tg.on ? ' on' : '');
        b.innerHTML = `<i>${tg.on ? '<luma-icon name="check"></luma-icon>' : ''}</i>${esc(tg.label)}`;
        b.addEventListener('click', () => this.emit('surface-toggle', { name: tg.name, on: !tg.on }));
        t.append(b);
      }
      panel.append(t);
    }
    center.append(panel);
  }

  renderBottom() {
    const d = this.data;
    this.querySelector('.lc-sf-servers').hidden = d.mode === 'connect' || d.servers?.hidden;
    const st = d.station?.status;
    const items = [];
    if (st && d.mode !== 'connect') {
      if (st.dmx && st.dmx !== 'no output') items.push(['ok', st.dmx === 'running' ? 'DMX output running' : `DMX ${st.dmx}`]);
      if (st.role) items.push(['accent', st.role]);
      if (st.unsaved) items.push(['alert', 'unsaved changes']);
    }
    this.querySelector('.lc-sf-status').innerHTML = items.map(([c, t]) => `<span><i class="lc-sf-dot ${c}"></i>${esc(t)}</span>`).join('');
    const hint = this.querySelector('.lc-sf-hint');
    hint.textContent = d.hint ?? (d.mode === 'connect' ? '' : 'Normal theme · Meow + F9');
    hint.hidden = !hint.textContent;
  }
}
customElements.define('luma-surface', LumaSurface);
