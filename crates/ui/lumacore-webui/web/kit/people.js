// <luma-people> — the Users & Profiles window's content (decisions §30
// "12b2", the UsersProfiles board; 12c2: UserAsks + PhoneFixes boards). The
// users (avatar, rights level, profile, where logged in) and the profiles (who uses
// them; UserProfile 1 "Default" is the locked template nobody works in); the
// picked one's settings (an embedded <luma-settings>: a profile's tabs
// Look · Lock screen · Input & keys · Oops · Command line, a user's rights
// level / profile / colour / remote login) and its actions. Actions above the
// viewer's level stay visible, greyed with the reason ("Needs Admin — you are
// Program", decisions §30 "12d": users and profiles are Admin's; the station
// refuses the line anyway).
//
// The window has no dialogs of its own (12c2): every button sends one command
// line (event `run`) and every question comes back as a station ask (the 12b
// dialog) — bare `Store User`, bare `Store UserProfile`, `Delete User "x"`,
// `Delete UserProfile n`, `Set User "x" "Password" Ask`, `Set User "x" "Name" Ask`,
// `Label UserProfile n`, `Import UserProfile`, `Export UserProfile n`. Picking
// asks the station for the fields (event `pick` with the EditSetting line).
//
// Sized by its box (container queries): wide = list · settings · actions;
// medium = list · (actions over settings); narrow (a phone) = one column with
// a Users / Profiles switch, the list, New user… / Import… at the bottom, and
// a picked row opening its settings with ‹ back.
//
//   el.update({ users: [{no, name, rights, profile, profile_no, color, logged_in}],
//     profiles: [{no, name, locked, used_by, preset}], me: 'the viewer' })
//   el.setFields(title, fields)  — the station's answer to the pick

import './settings.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const q = s => `"${String(s).replace(/[";]/g, '')}"`;
/** Admin always exists (E-USER-005): it can't be renamed, deleted or lowered. */
const lockedUser = u => u?.name === 'Admin';
/** The rights ladder, lowest first (lumacore_tree::UserRights). */
const LEVELS = ['Guest', 'Playback', 'Presets', 'Program', 'Setup', 'Admin'];

class LumaPeople extends HTMLElement {
  constructor() {
    super();
    this.data = { users: [], profiles: [] };
    this.sel = null; this.title = ''; this.fields = [];
    this.tab = 'users';   // narrow: which list shows
    this.view = 'list';   // narrow: the list or the picked one
    // One settings element for the window's life: a redraw of the list (a
    // station event) never detaches it while its pick list or keypad is open.
    this.settings = document.createElement('luma-settings');
    this.settings.setAttribute('embedded', '');
    this.settings.addEventListener('run', e => { e.stopPropagation(); this.run(e.detail.line); });
  }
  connectedCallback() { this.dataset.kit = 'people'; this.render(); }
  update(data) {
    const before = JSON.stringify(this.sel);
    this.data = data;
    const gone = this.sel && !(this.sel.kind === 'user'
      ? data.users.some(u => u.name === this.sel.name)
      : data.profiles.some(p => p.no === this.sel.no));
    if (!this.sel || gone) {
      const me = data.users.find(u => u.logged_in?.length) || data.users[0];
      this.sel = me ? { kind: 'user', name: me.name } : null;
      this.title = ''; this.fields = [];
      if (gone) this.view = 'list';
    }
    this.render();
    // Ask for the fields only when the pick changed (a station event redraws the list only).
    if (JSON.stringify(this.sel) !== before || this.line() !== this.asked) this.ask();
  }
  setFields(title, fields) {
    // The window shows a profile's settings as on the board: the theme's
    // Paint colours are the theme editor's (12g).
    this.title = title;
    this.fields = fields.filter(f => !/ Theme Paint "/.test(f.line));
    this.settings.update(this.settingsSpec());
  }
  /** The picked profile when it is the locked template (Default). */
  lockedProfile() {
    return this.sel?.kind === 'profile' && this.data.profiles.find(p => p.no === this.sel.no)?.locked;
  }
  settingsSpec() {
    // A user's and a profile's settings are theirs (decisions §39: the User home).
    return { title: this.title, fields: this.fields, home: 'user',
      readonly: this.lockedProfile() ? 'Default is the template every new profile starts from — locked. Nobody works in it.' : false };
  }
  /** The EditSetting line of the picked user or profile. */
  line() {
    if (!this.sel) return null;
    return this.sel.kind === 'user' ? `EditSetting User ${q(this.sel.name)}` : `EditSetting UserProfile ${this.sel.no}`;
  }
  ask() { const l = this.line(); this.asked = l; if (l) this.emit('pick', { line: l }); }
  emit(name, detail) { this.dispatchEvent(new CustomEvent(name, { detail, bubbles: true })); }
  run(line) { if (line) this.emit('run', { line }); }

  /** The picked one's actions: each a command line; the station asks the rest. */
  actions() {
    const s = this.sel;
    if (!s) return [];
    if (s.kind === 'profile') {
      const p = this.data.profiles.find(x => x.no === s.no) || { no: s.no, name: '' };
      const lock = p.locked ? 'Default is locked' : '';
      const admin = this.needs('Admin');
      return [
        // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
        { label: 'New profile…', line: 'Store UserProfile', note: 'from Default, a copy of one, or a file', off: admin },
        { label: 'Rename…', line: `Label UserProfile ${p.no}`, off: lock || admin },
        // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
        { label: 'Export', line: `Export UserProfile ${p.no}`, note: `library/userprofiles/${p.name}.lumaprofile`, off: this.needs('Playback') },
        // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
        { label: 'Import…', line: 'Import UserProfile', off: admin },
        { label: 'Theme editor…', off: 'the theme editor (12g)' },
        // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
        { label: 'Delete…', line: `Delete UserProfile ${p.no}`, danger: true, off: lock || admin,
          note: p.used_by?.length ? `used by ${p.used_by.join(', ')} — move them first` : 'nobody uses it' },
      ];
    }
    const u = this.data.users.find(x => x.name === s.name) || { name: s.name };
    const lock = lockedUser(u) ? 'Admin always exists' : '';
    const own = u.name === this.data.me;
    return [
      // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
      { label: 'New user…', line: 'Store User', note: 'name, rights level, own or shared profile', off: this.needs('Admin') },
      { label: 'Password…', line: `Set User ${q(u.name)} "Password" Ask`, note: u.has_password ? 'has a password' : 'no password', off: own ? '' : this.needs('Admin') },
      { label: 'Rename…', line: `Set User ${q(u.name)} "Name" Ask`, off: lock || this.needs('Admin') },
      // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
      { label: 'Delete…', line: `Delete User ${q(u.name)}`, danger: true, off: lock || this.needs('Admin'), note: 'asks what goes with them' },
    ];
  }

  /** The viewer's level (from the station's users). */
  level() { return this.data.users.find(x => x.name === this.data.me)?.rights || 'Guest'; }
  /** '' when the viewer has `level`, else the reason the control is greyed. */
  needs(level) {
    // Until the page knows who views it nothing greys (no flash under a tap);
    // the station refuses the line anyway.
    if (!this.data.me) return '';
    const have = this.level();
    return LEVELS.indexOf(have) >= LEVELS.indexOf(level) ? '' : `Needs ${level} — you are ${have}`;
  }

  userRow(u, i) {
    const on = this.sel?.kind === 'user' && this.sel.name === u.name;
    const shared = (this.data.profiles.find(p => p.no === u.profile_no)?.used_by?.length || 0) > 1;
    const where = u.logged_in?.length ? ` · ${u.logged_in.join(', ')}` : '';
    return `<button type="button" class="lc-pp-row${on ? ' on' : ''}" data-user="${i}" data-sec="users">
      <luma-avatar name="${esc(u.name)}" color="${esc(u.color || '')}" size="40" selected></luma-avatar>
      <span class="lc-pp-text"><b>${esc(u.name)}</b>
        <small class="${u.logged_in?.length ? 'on' : ''}">${esc(u.rights)} · ${shared ? 'shares' : 'profile'} ${esc(u.profile || '—')}${esc(where)}</small></span>
      ${lockedUser(u) ? '<em class="lc-pp-lock">locked</em>' : ''}</button>`;
  }
  profileRow(p, i) {
    const on = this.sel?.kind === 'profile' && this.sel.no === p.no;
    const who = p.locked ? 'template · nobody works in it' : p.used_by?.length ? `used by ${p.used_by.join(', ')}` : 'not used';
    return `<button type="button" class="lc-pp-row${on ? ' on' : ''}" data-profile="${i}" data-sec="profiles">
      <span class="lc-pp-sw"><luma-icon name="${p.locked ? 'lock' : 'palette'}"></luma-icon></span>
      <span class="lc-pp-text"><b>${esc(p.no)} ${esc(p.name)}</b><small>${esc(who)} · ${esc(p.preset || 'Dark')}</small></span>
      ${p.locked ? '<em class="lc-pp-lock">locked</em>' : ''}</button>`;
  }

  render() {
    if (!this.isConnected) return;
    const { users, profiles } = this.data;
    const s = this.sel;
    const acts = this.actions();
    // A refresh of the list keeps the settings' tab (a fresh pick starts at the first).
    const keepTab = this.settings.tab ?? 0;
    const heading = s ? (s.kind === 'user' ? `User ${s.name}` : `Profile ${s.no} ${this.data.profiles.find(p => p.no === s.no)?.name || ''}`) : '';
    const foot = this.tab === 'users'
      // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
      ? [{ label: 'New user…', line: 'Store User', primary: true }, { label: 'Import…', line: 'Import UserProfile' }]
      // tidy: slab-ok (Users & Profiles moves onto <luma-grid> + <luma-slab> in 12d part 4)
      : [{ label: 'New profile…', line: 'Store UserProfile', primary: true }, { label: 'Import…', line: 'Import UserProfile' }];
    this.innerHTML = `
      <div class="lc-pp" data-tab="${this.tab}" data-view="${this.view}">
        <div class="lc-pp-switch" role="tablist">
          <button type="button" role="tab" data-switch="users" class="${this.tab === 'users' ? 'on' : ''}" aria-selected="${this.tab === 'users'}">Users</button>
          <button type="button" role="tab" data-switch="profiles" class="${this.tab === 'profiles' ? 'on' : ''}" aria-selected="${this.tab === 'profiles'}">Profiles</button>
        </div>
        <div class="lc-pp-list">
          <span class="lc-pp-head" data-sec="users">Users</span>
          ${users.map((u, i) => this.userRow(u, i)).join('')}
          <span class="lc-pp-head" data-sec="profiles">Profiles</span>
          ${profiles.map((p, i) => this.profileRow(p, i)).join('')}
        </div>
        <div class="lc-pp-foot">
          ${foot.map((b, i) => `<button type="button" class="lc-pp-fbtn${b.primary ? ' primary' : ''}" data-foot="${i}" ${this.needs('Admin') ? `disabled title="${esc(this.needs('Admin'))}"` : ''}>${esc(b.label)}</button>`).join('')}
        </div>
        <div class="lc-pp-detail">
          <div class="lc-pp-back"><button type="button" class="lc-pp-backbtn" data-back="1" aria-label="Back to the list"><luma-icon name="chevron-left"></luma-icon>${this.tab === 'users' ? 'Users' : 'Profiles'}</button>
            <b>${esc(heading)}</b></div>
          <span class="lc-pp-set"></span>
          <aside class="lc-pp-acts">
            <span class="lc-pp-head">${esc(heading)}</span>
            ${acts.map((a, i) => `<button type="button" class="lc-pp-act${a.danger ? ' danger' : ''}" data-act="${i}" ${a.off || !a.line ? 'disabled' : ''}>
              <b>${esc(a.label)}</b><small>${esc(a.off || a.note || a.line || '')}</small></button>`).join('')}
          </aside>
        </div>
      </div>`;
    const set = this.settings;
    this.querySelector('.lc-pp-set').replaceWith(set);
    set.tab = this.title ? keepTab : 0;
    set.update(this.title ? this.settingsSpec() : { title: '', fields: [] });
    const pick = sel => {
      this.sel = sel; this.title = ''; this.fields = []; this.view = 'detail';
      this.render(); this.ask();
    };
    for (const b of this.querySelectorAll('[data-user]')) b.addEventListener('click', () => pick({ kind: 'user', name: users[+b.dataset.user].name }));
    for (const b of this.querySelectorAll('[data-profile]')) b.addEventListener('click', () => pick({ kind: 'profile', no: profiles[+b.dataset.profile].no }));
    for (const b of this.querySelectorAll('[data-act]')) b.addEventListener('click', () => this.run(acts[+b.dataset.act].line));
    for (const b of this.querySelectorAll('[data-foot]')) b.addEventListener('click', () => this.run(foot[+b.dataset.foot].line));
    for (const b of this.querySelectorAll('[data-switch]')) b.addEventListener('click', () => {
      this.tab = b.dataset.switch; this.view = 'list'; this.render();
    });
    this.querySelector('[data-back]')?.addEventListener('click', () => { this.view = 'list'; this.render(); });
  }
}
customElements.define('luma-people', LumaPeople);
