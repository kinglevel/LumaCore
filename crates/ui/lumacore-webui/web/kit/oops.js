// The Oops window (decisions §30 "12f", the round 7 board): your Oops list as a pop-up — held Oops
// key or `Menu "Oops"`. Newest at the bottom; a tap on a step selects it and every newer one and
// the button becomes "Oops n steps" (a second tap runs it — never a one-tap undo; the profile's
// OopsConfirmation still asks); undone steps stay dimmed below a line, a tap selects them for
// "Redo n"; the chips are the profile's Oops filters (each a `Set UserProfile` line). A tap outside
// closes it; PopupDim dims behind it. Every gesture is a command line (`Oops 3`, `Redo 2`).
//
//   const w = openOops({ run(line), onClose() });  w.update(oopsFeedData);  w.close()

const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const clock = ms => (ms ? new Date(ms).toTimeString().slice(0, 8) : '');

export function openOops({ run, onClose }) {
  const scrim = document.createElement('div');
  scrim.className = 'lc-oops-scrim';
  scrim.dataset.kit = 'oops';
  scrim.innerHTML = `<div class="lc-oops lc-box" role="dialog" aria-label="Oops">
      <header class="lc-oops-head"><b>Oops</b><span class="lc-oops-chips"></span>
        <button type="button" class="lc-close" data-kit="close" aria-label="Close"><luma-icon name="x"></luma-icon></button></header>
      <div class="lc-oops-list lc-box-body"></div>
      <footer class="lc-oops-foot"><button type="button" class="oops" data-act="oops">Oops</button><button type="button" class="redo" data-act="redo">Redo</button></footer>
    </div>`;
  document.body.append(scrim);
  let d = { steps: [], undone: [] };
  let pick = null; // { kind: 'oops', from } | { kind: 'redo', to }
  const list = scrim.querySelector('.lc-oops-list');

  const draw = () => {
    const steps = d.steps || [], undone = d.undone || [];
    scrim.classList.toggle('dim', !!d.popup_dim);
    scrim.querySelector('.lc-oops-chips').innerHTML = (d.filters || []).map(f =>
      `<button type="button" class="lc-oops-chip" aria-pressed="${!!f.on}" data-filter="${esc(f.name)}" ${d.locked ? 'disabled' : ''}>${esc(f.label)}</button>`).join('');
    const row = (s, n, cls, attr) => `<div class="lc-oops-row ${cls}" ${attr}><span class="n">${n}</span><span class="ln">${esc(s.line)}</span>`
      + `<span class="cat ${esc(s.category)}">${esc(s.category)}</span><span class="tm">${esc(s.station)} · ${clock(s.time_ms)}</span></div>`;
    let html = steps.length ? '' : '<div class="lc-oops-empty">Nothing to undo.</div>';
    steps.forEach((s, i) => {
      const on = pick?.kind === 'oops' && i >= pick.from;
      html += row(s, steps.length - i, on ? 'pick' : '', `data-i="${i}"`);
    });
    if (undone.length) {
      html += `<div class="lc-oops-div">Undone · Redo brings back from the top</div>`;
      undone.forEach((s, j) => {
        const on = pick?.kind === 'redo' && j <= pick.to;
        html += row(s, '↺', `undone${on ? ' pick' : ''}`, `data-j="${j}"`);
      });
    }
    list.innerHTML = html;
    list.scrollTop = list.scrollHeight;
    const nOops = pick?.kind === 'oops' ? steps.length - pick.from : 1;
    const nRedo = pick?.kind === 'redo' ? pick.to + 1 : 1;
    const ob = scrim.querySelector('[data-act="oops"]'), rb = scrim.querySelector('[data-act="redo"]');
    ob.textContent = pick?.kind === 'oops' ? `Oops ${nOops} step${nOops > 1 ? 's' : ''}` : 'Oops last step';
    ob.disabled = !steps.length;
    rb.textContent = !undone.length ? 'Redo' : pick?.kind === 'redo' ? `Redo ${nRedo} step${nRedo > 1 ? 's' : ''}` : 'Redo 1 step';
    rb.disabled = !undone.length;
  };

  list.addEventListener('click', e => {
    const r = e.target.closest('.lc-oops-row');
    if (!r) return;
    if (r.dataset.i != null) {
      const i = Number(r.dataset.i);
      pick = pick?.kind === 'oops' && pick.from === i ? null : { kind: 'oops', from: i };
    } else {
      const j = Number(r.dataset.j);
      pick = pick?.kind === 'redo' && pick.to === j ? null : { kind: 'redo', to: j };
    }
    draw();
  });
  scrim.querySelector('.lc-oops-chips').addEventListener('click', e => {
    const b = e.target.closest('[data-filter]');
    if (!b || d.profile == null) return;
    run(`Set UserProfile ${d.profile} "${b.dataset.filter}" ${b.getAttribute('aria-pressed') === 'true' ? 'No' : 'Yes'}`);
  });
  scrim.querySelector('[data-act="oops"]').addEventListener('click', () => {
    const n = pick?.kind === 'oops' ? (d.steps || []).length - pick.from : 1;
    pick = null;
    run(n > 1 ? `Oops ${n}` : 'Oops');
  });
  scrim.querySelector('[data-act="redo"]').addEventListener('click', () => {
    const n = pick?.kind === 'redo' ? pick.to + 1 : 1;
    pick = null;
    run(n > 1 ? `Redo ${n}` : 'Redo');
  });
  const close = () => {
    document.removeEventListener('keydown', onKey, true);
    scrim.remove();
    onClose?.();
  };
  const onKey = e => { if (e.key === 'Escape') { e.stopPropagation(); close(); } };
  document.addEventListener('keydown', onKey, true);
  scrim.querySelector('[data-kit="close"]').addEventListener('click', close);
  // A tap anywhere outside closes it.
  scrim.addEventListener('pointerdown', e => { if (e.target === scrim) close(); });
  draw();
  return {
    update(data) {
      // A list that changed (an Oops ran) drops a selection that no longer fits.
      if (pick && JSON.stringify([data.steps?.length, data.undone?.length]) !== JSON.stringify([d.steps?.length, d.undone?.length])) pick = null;
      d = data || d;
      draw();
    },
    close,
    get open() { return scrim.isConnected; },
  };
}
