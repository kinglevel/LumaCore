// Import MVR — the two-step dialog of the Patch dialog's Import cell (decisions §34 "Import MVR
// hardened" 1): 1 · File is the files element on library/mvr/ (the drives rail, the MVR facts,
// This device, a drop); 2 · Options is the profile's MVR import preset as a settings form — every
// change one `Set UserProfile "Mvr…"` line, no OK — with the scope picked from the draft's MVR
// tree or classes. The footer shows the line as it grows; Review runs it, the dialog closes and
// the Patch grid is the review (the `patch` feed's review mode, drawn by <luma-setup>).
//
//   openMvrImport({ feeds, level, groups: [{ at, name }], classes: [{ at, name }], run, closed })
//     → close()
//   The preset's fields come from `EditSetting UserProfile`: the page hands fields no setup menu
//   uses to `document` as `lc-fields` ({ detail: { title, fields } }); the dialog takes the
//   "MVR import" section and calls preventDefault.
//
// On a phone each step is one screen (decisions §37 4). Nothing is written before Review; the
// settings lines go to the profile as they are changed, like every settings cell.

import './files.js';
import './settings.js';
import { openPick } from './popup.js';
import { phoneMode, onPhoneMode } from './phone.js';

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const quoted = s => `"${String(s).replace(/[";]/g, '')}"`;

let current = null;
export function closeMvrImport() { current?.(); }

export function openMvrImport({ feeds = null, level = '', groups = [], classes = [], run, closed } = {}) {
  closeMvrImport();
  const st = { step: 1, file: null, scope: null, fields: null, lineOpen: false };
  const scrim = document.createElement('div');
  scrim.className = 'lc-card-scrim lc-ins-scrim';
  scrim.dataset.kit = 'mvrimport';
  document.body.append(scrim);
  const box = document.createElement('div');
  box.className = 'lc-box lc-ins lc-mvr';
  box.setAttribute('role', 'dialog');
  box.setAttribute('aria-label', 'Import MVR');
  scrim.append(box);

  const line = () => {
    if (!st.file) return '';
    const drive = st.file.drive && st.file.drive !== 'local-library' ? ` /Drive ${quoted(st.file.drive)}` : '';
    const scope = st.scope ? ` /Scope ${st.scope.at}` : '';
    return `Import Mvr ${quoted(st.file.name)}${drive}${scope}`;
  };
  const head = () => `<header class="lc-box-title lc-ins-title"><luma-icon name="plug"></luma-icon>
      <span class="lc-ins-crumb"><span>Patch</span><i>›</i><b>Import MVR</b></span>
      <span class="lc-ins-steps" role="group" aria-label="Steps">
        <button type="button" data-step="1" aria-pressed="${st.step === 1}"><b>1</b> File</button>
        <button type="button" data-step="2" aria-pressed="${st.step === 2}"${st.file ? '' : ' disabled'}><b>2</b> Options</button></span>
      <button type="button" class="lc-close" data-x aria-label="Close"><luma-icon name="x"></luma-icon></button></header>`;
  const fileStep = () => `<div class="lc-mvr-step"><luma-files class="lc-mvr-files"></luma-files></div>`;
  const optionsStep = () => `<div class="lc-mvr-step lc-mvr-options">
      <div class="lc-mvr-scope"><span>Scope</span><button type="button" class="lc-ins-v lc-ins-pickgroup" data-scope>${esc(st.scope ? `${st.scope.at} · ${st.scope.name}` : 'Whole patch')}</button>
        <span class="lc-mvr-hint">An MVR layer or group, a class (the file side by uuid), or the whole patch</span>
        <button type="button" class="lc-ins-chip" data-reset>Reset to Offer all</button></div>
      <luma-settings embedded class="lc-mvr-settings"></luma-settings>
      <p class="lc-ins-note">Your MVR import preset: each change is one Set UserProfile line, kept for the next import. The review takes what the file may change: Take = ticked, Offer = shown unticked, None = hidden.</p>
    </div>`;
  const draw = () => {
    const phone = phoneMode();
    box.classList.toggle('phone', !!phone);
    scrim.classList.toggle('phone', !!phone);
    const l = line();
    box.innerHTML = `${head()}<div class="lc-box-body lc-ins-body">${st.step === 1 ? fileStep() : optionsStep()}</div>
      <footer class="lc-ins-foot"><div class="lc-ins-line lc-selectable">${l ? `<code>${esc(l)}</code>` : 'Pick an .mvr of the library (This device uploads one)'}</div>
        <button type="button" class="lc-btn" data-cancel>Cancel</button>
        ${st.step === 1 ? `<button type="button" class="lc-btn primary" data-next${st.file ? '' : ' disabled'}>Next · Options</button>`
          : `<button type="button" class="lc-btn" data-step="1">‹ File</button><button type="button" class="lc-btn primary" data-review${st.file ? '' : ' disabled'}>Review</button>`}</footer>`;
    const lf = box.querySelector('luma-files');
    if (lf) {
      lf.update({ feeds, kind: 'mvr', drive: st.file?.drive || 'local-library', level });
      lf.addEventListener('select', e => { const d = e.detail; const row = d.rows?.length === 1 ? d.rows[0] : null; st.file = row && !row.dir ? { drive: d.drive, name: row.file } : null; refresh(); });
      lf.addEventListener('pick', e => { const d = e.detail; if (d.row && !d.row.dir) { st.file = { drive: d.drive, name: d.row.file }; st.step = 2; draw(); } });
      lf.addEventListener('line', e => e.stopPropagation());
    }
    const ls = box.querySelector('luma-settings');
    if (ls) {
      ls.update({ title: 'MVR import', fields: st.fields || [], level, note: st.fields ? '' : 'Asking the station for your preset…', home: 'user' });
      ls.addEventListener('run', e => { e.stopPropagation(); run?.(e.detail.line); });
      if (!st.fields) run?.('EditSetting UserProfile');
    }
  };
  /** The footer and the step buttons follow the pick without redrawing the element. */
  const refresh = () => {
    const l = line();
    const foot = box.querySelector('.lc-ins-line');
    if (foot) foot.innerHTML = l ? `<code>${esc(l)}</code>` : 'Pick an .mvr of the library (This device uploads one)';
    for (const b of box.querySelectorAll('[data-next], [data-review], [data-step="2"]')) b.disabled = !st.file;
  };
  const onFields = e => {
    const d = e.detail || {};
    if (!/^userprofile\b/i.test(String(d.title || ''))) return; // "UserProfile 2"
    e.preventDefault();
    st.fields = (d.fields || []).filter(f => (f.section || '') === 'MVR import');
    const ls = box.querySelector('luma-settings');
    if (ls) ls.update({ title: 'MVR import', fields: st.fields, level, note: '', home: 'user' });
  };
  document.addEventListener('lc-fields', onFields);
  const offPhone = onPhoneMode(() => draw());
  const close = () => {
    if (!scrim.isConnected) return;
    offPhone();
    document.removeEventListener('lc-fields', onFields);
    scrim.remove();
    window.removeEventListener('keydown', onEsc, true);
    current = null;
    closed?.();
  };
  const onEsc = e => { if (e.key === 'Escape' && !document.querySelector('.lc-kp-scrim, .lc-pick-scrim, .lc-dlg-scrim')) { e.preventDefault(); e.stopPropagation(); close(); } };
  window.addEventListener('keydown', onEsc, true);
  current = close;

  box.addEventListener('click', async e => {
    const b = e.target.closest('button');
    if (!b) return;
    const d = b.dataset;
    if (d.x != null || d.cancel != null) return close();
    if (d.step) { st.step = +d.step; return draw(); }
    if (d.next != null) { if (st.file) { st.step = 2; draw(); } return; }
    if (d.scope != null) {
      const items = [{ no: 1, name: 'Whole patch' }, ...groups.map((g, i) => ({ no: i + 2, name: `${g.at} · ${g.name || 'no name'}` })), ...classes.map((c, i) => ({ no: groups.length + i + 2, name: `${c.at} · ${c.name || 'no name'}` }))];
      const got = await openPick({ title: 'Scope', pick: { kind: 'Scope', items, plain: true } });
      if (!got?.no) return;
      st.scope = got.no === 1 ? null : got.no <= groups.length + 1 ? groups[got.no - 2] : classes[got.no - groups.length - 2];
      return draw();
    }
    if (d.reset != null) {
      const lines = ['MvrAddresses', 'MvrIds', 'MvrNames', 'MvrPosition', 'MvrRevisions'].map(p => `Set UserProfile "${p}" Offer`);
      lines.push('Set UserProfile "MvrNewFixtures" Take', 'Set UserProfile "MvrMatch" Auto', 'Set UserProfile "MvrLost" Mark', 'Set UserProfile "MvrRotation" Take', 'Set UserProfile "MvrMissing" Ask');
      run?.(lines.join('; '));
      return;
    }
    if (d.review != null) {
      const l = line();
      if (!l) return;
      close();
      run?.(l);
    }
  });
  draw();
  return close;
}
