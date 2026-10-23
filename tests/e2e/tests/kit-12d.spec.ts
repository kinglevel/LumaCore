// Step 12d part 3 + 4 (decisions §30 "12d hardening", the locked boards in
// docs/dev/ui-kit/mockups/2026-10-13-12d/): the kit's <luma-grid> (selection
// with latched + / −, Thru, header columns, lasso by mouse and by finger, the
// edit card with ‹different values›), <luma-slab> (greyed actions say why),
// the power toggle's looks, the settings cells' kind badges and home marks,
// and <luma-setup> drawing the station's MenuDef registry: `Menu "Settings"`
// opens / closes / switches tabs by command line, and a grid edit on the
// Network menu is ONE line on the station. On the desktop, a touch tablet, a
// phone and WebKit.

import path from 'node:path';
import { expect, test, type Page, type TestInfo, type Locator } from '@playwright/test';
import { REPO, Server, connected, dialog, kbdMode } from '../support/server';
import { fitIssues } from '../support/fit';
import { slabCell as sheetCell, closeSheet } from '../support/slab';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const touch = (info: TestInfo) => info.project.name === 'phone' || info.project.name === 'tablet';
const phone = (info: TestInfo) => info.project.name === 'phone';
const cdp = (info: TestInfo) => touch(info); // chromium projects with touch: CDP touch events

async function open(page: Page, user = 'Admin') {
  await server.open(page, '/', user);
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction(() => (0, eval)('typeof openSetupMenu === "function" && window.LumaKit'));
  await page.evaluate(() => document.getElementById('fb-cards')?.replaceChildren());
}

/** The kit's Showroom over the page (its 12d sections). */
async function showroom(page: Page) {
  await page.evaluate(() => {
    const host = document.createElement('div');
    host.id = 'sr12d';
    host.style.cssText = 'position:fixed;inset:0;z-index:99999;overflow:auto;background:var(--lc-bg);padding:12px';
    host.append((window as any).LumaKit.showroom());
    document.body.append(host);
  });
  await expect(page.locator('#sr-grid [data-col]').first()).toBeAttached();
}

/** A finger tap on touch projects, a click elsewhere. A live grid may
 *  redraw under the press (a feed update): then it is pressed again. */
async function press(page: Page, info: TestInfo, el: Locator, opts: { modifiers?: ('Control' | 'Alt' | 'Shift')[]; force?: boolean } = {}) {
  // The scroll is bounded (a node redrawn under it never settles: look it up again); the
  // press itself is retried only when the node went — a press that landed but timed out
  // must not be sent twice (two taps are a double-tap: Edit).
  for (let i = 0; ; i++) {
    try {
      await el.scrollIntoViewIfNeeded({ timeout: 8_000 });
    } catch (e) {
      if (i < 2 && /not attached|detached|Timeout/.test(String(e))) continue;
      throw e;
    }
    try {
      if (touch(info) && !opts.modifiers) await el.tap({ force: opts.force });
      else await el.click(opts);
      return;
    } catch (e) {
      if (i >= 2 || !/not attached|detached/.test(String(e))) throw e;
    }
  }
}

/** Raw touch points through CDP (Chromium): start, moves, end. */
async function fingers(page: Page) {
  const s = await page.context().newCDPSession(page);
  return (type: string, pts: { x: number; y: number }[]) =>
    s.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map((p, i) => ({ x: p.x, y: p.y, id: i + 1 })) });
}

const grid = (page: Page) => page.locator('#sr-grid');
const cell = (g: Locator, k: string, c: string) => g.locator(`[data-k="${k}"][data-col="${c}"]`);
async function count(g: Locator) {
  // The bar's count, or the No cell's on a phone (the bar folds into it, §37 1).
  const c = g.locator('.lc-grid-count, .lc-gh-sel').first();
  return `${await c.getAttribute('data-count')}/${await c.getAttribute('data-cells')}`;
}
const noteOf = (page: Page, g: Locator) => g.locator('xpath=../..').locator('.lc-sr-note');
/** The + / − latch: a bar button, or the No cell's menu on a phone (§37 1). */
async function setMode(page: Page, info: TestInfo, g: Locator, m: '+' | '-') {
  if (!phone(info)) return press(page, info, g.locator(`[data-mode="${m}"]`));
  await g.locator('[data-selmenu]').click();
  await page.locator('#ctx .ci', { hasText: m === '-' ? '− removes' : '+ adds' }).click();
}

test('grid selection: a tap replaces, + and − latch, Thru makes a range, a header twice is the whole column', async ({ page }, info) => {
  await open(page);
  await showroom(page);
  const g = grid(page);
  await press(page, info, cell(g, 'o1', 'rate'));
  expect(await count(g)).toBe('1/1');
  // + latches: two more taps add, until + is pressed again.
  await press(page, info, g.locator('[data-mode="+"]'));
  await expect(g.locator('[data-mode="+"]')).toHaveAttribute('aria-pressed', 'true');
  await press(page, info, cell(g, 'o2', 'rate'));
  await press(page, info, cell(g, 'o3', 'rate'));
  expect(await count(g)).toBe('3/3');
  // − takes over from + (the other latch goes off) and removes.
  await press(page, info, g.locator('[data-mode="-"]'));
  await expect(g.locator('[data-mode="+"]')).toHaveAttribute('aria-pressed', 'false');
  await press(page, info, cell(g, 'o2', 'rate'));
  expect(await count(g)).toBe('2/2');
  await press(page, info, g.locator('[data-mode="-"]'));
  // Without a latch a tap replaces.
  await press(page, info, cell(g, 'o4', 'rate'));
  expect(await count(g)).toBe('1/1');
  // Thru is one-shot: A … Thru … B is the rectangle.
  await press(page, info, g.locator('[data-thru]'));
  await expect(g.locator('[data-thru]')).toHaveAttribute('aria-pressed', 'true');
  await press(page, info, cell(g, 'o1', 'protocol'));
  await expect(g.locator('[data-thru]')).toHaveAttribute('aria-pressed', 'false');
  // Protocol … Rate: three columns (on a phone Universes folds into the row).
  expect(await count(g)).toBe(phone(info) ? '4/8' : '4/12');
  // The row-number gutter selects whole rows.
  await press(page, info, g.locator('[data-gutter="o2"]'));
  expect(await count(g)).toBe('1/0');
  await expect(g.locator('[data-row="o2"]')).toHaveClass(/\bwhole\b/);
  // A header: the column on the selected rows; twice: the whole column.
  await press(page, info, g.locator('[data-htap="rate"]'));
  expect(await count(g)).toBe('1/1');
  await press(page, info, g.locator('[data-htap="rate"]'));
  expect(await count(g)).toBe('5/5');
  await expect(g.locator('[data-hcol="rate"]')).toHaveClass(/\bon\b/);
  if (!phone(info)) {
    // All / None / Invert (a phone keeps them off the bar).
    await press(page, info, g.locator('[data-none]'));
    expect(await count(g)).toBe('0/0');
    await press(page, info, g.locator('[data-gutter="o1"]'));
    await press(page, info, g.locator('[data-invert]'));
    expect(await count(g)).toBe('4/0');
    await press(page, info, g.locator('[data-all]'));
    expect(await count(g)).toBe('5/0');
  }
  if (!touch(info)) {
    // Mouse: Ctrl = +, Alt = −, Shift = Thru.
    await cell(g, 'o1', 'send').click();
    await cell(g, 'o3', 'send').click({ modifiers: ['Control'] });
    expect(await count(g)).toBe('2/2');
    await cell(g, 'o3', 'send').click({ modifiers: ['Alt'] });
    expect(await count(g)).toBe('1/1');
    // Shift: the range from the last tapped (o3) to here is added.
    await cell(g, 'o4', 'send').click({ modifiers: ['Shift'] });
    expect(await count(g)).toBe('3/3');
  }
});

test('lasso: a mouse drag, one finger on a tablet, hold-then-drag on a phone — the cells land on finger-up', async ({ page }, info) => {
  await open(page);
  await showroom(page);
  const g = grid(page);
  await cell(g, 'o3', 'rate').scrollIntoViewIfNeeded();
  const a = (await cell(g, 'o1', 'protocol').boundingBox())!;
  const z = (await cell(g, 'o3', 'rate').boundingBox())!;
  const A = { x: a.x + 8, y: a.y + 8 }, Z = { x: z.x + 10, y: z.y + 10 };
  if (!touch(info)) {
    await page.mouse.move(A.x, A.y);
    await page.mouse.down();
    await page.mouse.move(A.x + 20, A.y + 20, { steps: 3 });
    await expect(g.locator('[data-lasso]')).toBeVisible();
    await page.mouse.move(Z.x, Z.y, { steps: 6 });
    // Nothing lands before the button comes up.
    expect(await count(g)).toBe('0/0');
    await page.mouse.up();
  } else {
    const t = await fingers(page);
    // On a phone one finger scrolls: a quick drag selects nothing.
    if (phone(info)) {
      await t('touchStart', [A]);
      for (let i = 1; i <= 6; i++) await t('touchMove', [{ x: A.x + 4 * i, y: A.y + 12 * i }]);
      await t('touchEnd', []);
      expect(await count(g)).toBe('0/0');
      await page.waitForTimeout(100);
    }
    await t('touchStart', [A]);
    await page.waitForTimeout(phone(info) ? 650 : 60); // phone: hold first
    for (let i = 1; i <= 8; i++) {
      await t('touchMove', [{ x: A.x + (Z.x - A.x) * i / 8, y: A.y + (Z.y - A.y) * i / 8 }]);
      await page.waitForTimeout(16);
    }
    expect(await count(g)).toBe('0/0');
    await t('touchEnd', []);
  }
  await expect(g.locator('[data-lasso]')).toBeHidden();
  // Three rows × Protocol … Rate. On a phone Universes folds into the row and
  // the frozen Name may sit over the start (scrolled sideways): what is seen
  // under the lasso is what lands.
  if (phone(info)) expect(await count(g)).toMatch(/^3\/(6|9)$/);
  else expect(await count(g)).toBe('3/9');
  await expect(cell(g, 'o2', 'protocol')).toHaveClass(/\bon\b/);
  // A lasso that starts on a selected cell replaces (it never drags).
  if (!touch(info)) {
    const b = (await cell(g, 'o2', 'rate').boundingBox())!;
    await page.mouse.move(b.x + 5, b.y + 5);
    await page.mouse.down();
    await page.mouse.move(b.x + 30, b.y + 50, { steps: 5 });
    await page.mouse.up();
    expect(await count(g)).toBe('2/2');
  }
});

test('edit: a two-finger tap / double-click / the Edit chip opens the card with ‹different values›; one change = one line', async ({ page }, info) => {
  await open(page);
  await showroom(page);
  const g = grid(page);
  // Rows 1 … 3 by the gutter and Thru.
  await press(page, info, g.locator('[data-gutter="o1"]'));
  await press(page, info, g.locator('[data-thru]'));
  await press(page, info, g.locator('[data-gutter="o3"]'));
  expect(await count(g)).toBe('3/0');
  if (cdp(info)) {
    await cell(g, 'o2', 'rate').scrollIntoViewIfNeeded();
    const r = (await cell(g, 'o2', 'rate').boundingBox())!;
    const t = await fingers(page);
    await t('touchStart', [{ x: r.x + 8, y: r.y + 8 }, { x: r.x + r.width - 8, y: r.y + r.height - 8 }]);
    await page.waitForTimeout(80);
    await t('touchEnd', []);
  } else if (info.project.name === 'webkit') {
    // The Edit chip shows beside the selection after a pause.
    await expect(g.locator('[data-chip]')).toBeVisible({ timeout: 3000 });
    await g.locator('[data-chip]').click();
  } else {
    // Desktop: Enter (or F2) on the focused grid; a double-click edits one cell (below).
    await page.keyboard.press('Enter');
  }
  const card = page.locator('.lc-card');
  await expect(card).toBeVisible();
  await expect(card.locator('.lc-set-name')).toContainText('3 Sacns');
  await card.locator('[data-tab]', { hasText: 'Timing' }).click();
  const rate = card.locator('[data-field="rate"]');
  await expect(rate.locator('.lc-set-value')).toHaveText('‹different values›');
  await expect(rate.locator('.lc-set-range')).toHaveText('30–44 Hz');
  await expect(card.locator('[data-field="send"] .lc-set-range')).toHaveText('Full / Changes');
  await expect(rate.locator('.lc-kind')).toHaveText('123');
  // Per-row fields are read-only with several rows.
  await card.locator('[data-tab]', { hasText: 'General' }).click();
  await expect(card.locator('[data-field="name"]')).toHaveClass(/\bro\b/);
  await card.locator('[data-tab]', { hasText: 'Timing' }).click();
  await rate.locator('.lc-set-tap').click();
  await expect(page.locator('.lc-kp-scrim')).toBeVisible();
  await page.keyboard.type('40');
  await page.keyboard.press('Enter');
  await expect(noteOf(page, g)).toHaveText('› Set Sacn 1 Thru 3 "Rate" 40');
  await expect(rate.locator('.lc-set-value')).toHaveText('40 Hz');
  await card.locator('[data-x]').click();
  await expect(card).toHaveCount(0);
  for (const k of ['o1', 'o2', 'o3']) await expect(cell(g, k, 'rate')).toHaveText('40 Hz');
  if (info.project.name === 'desktop') {
    // A double-click edits the cell under it: the card on that field, its keypad at once.
    await cell(g, 'o4', 'rate').dblclick();
    await expect(card.locator('.lc-set-name')).toContainText('Sacn 4');
    await expect(page.locator('.lc-kp-scrim')).toBeVisible();
    await page.keyboard.type('+ 5');
    await page.keyboard.press('Enter');
    await expect(noteOf(page, g)).toHaveText('› Set Sacn 4 "Rate" + 5');
    await expect(cell(g, 'o4', 'rate')).toHaveText('30 Hz');
    await page.keyboard.press('Escape');
    await expect(card).toHaveCount(0);
  }
});

test('Network on <luma-setup>: the Shell column, − one interface, Edit → Shell No is ONE line on the station', async ({ page }, info) => {
  await open(page);
  await page.evaluate(() => (0, eval)('openSetupMenu("Network")'));
  const setup = page.locator('luma-setup');
  const g = setup.locator('luma-grid');
  await expect(g.locator('[data-col="shell"]').first()).toBeVisible();
  const names = await g.locator('[data-col="shell"]').evaluateAll(els => els.map(e => (e as HTMLElement).dataset.k!));
  expect(names.length).toBeGreaterThan(0);
  // The power toggle is the station's own state (look A).
  await expect(setup.locator('luma-power .lc-power').first()).toHaveAttribute('data-state', 'on');
  await expect(setup.locator('.lc-power-word').filter({ visible: true })).toHaveText('ONLINE');
  // The header with nothing selected: the whole column.
  await press(page, info, g.locator('[data-htap="shell"]'));
  expect(await count(g)).toBe(`${names.length}/${names.length}`);
  // − on the last one (when there is more than one).
  const kept = names.length > 1 ? names.slice(0, -1) : names;
  if (names.length > 1) {
    await setMode(page, info, g, '-');
    await press(page, info, cell(g, names[names.length - 1], 'shell'));
    await setMode(page, info, g, '-');
  }
  expect(await count(g)).toBe(`${kept.length}/${kept.length}`);
  // The slab: labels carry the count; New is greyed with the reason. A phone keeps
  // both in the More sheet (§37 1), where the full names show.
  const slab = setup.locator('luma-slab');
  const editCell = phone(info) ? await sheetCell(setup, '[data-act="edit"]') : slab.locator('[data-act="edit"]').filter({ visible: true });
  await expect(editCell).toContainText(`Edit ${kept.length}`);
  const newCell = phone(info) ? await sheetCell(setup, '[data-act="new"]') : slab.locator('[data-act="new"]').filter({ visible: true });
  await expect(newCell).toHaveAttribute('data-why', 'Interfaces come from the hardware');
  await press(page, info, newCell, { force: true }); // greyed (aria-disabled): it says why
  await expect(page.locator('.lc-toast')).toHaveText('Interfaces come from the hardware');
  // Edit: cells in one column open the card on that field (a switch never flips by itself).
  // A phone: the More sheet that showed New covers the bar; close it and find Edit again.
  if (phone(info)) await closeSheet(page);
  await press(page, info, phone(info) ? await sheetCell(setup, '[data-act="edit"]') : editCell);
  const card = page.locator('.lc-card');
  await expect(card).toBeVisible();
  const shell = card.locator('[data-field="shell"]');
  await expect(shell).toHaveClass(/\bfocus\b/);
  await expect(shell.locator('.lc-kind')).toHaveText('I/O');
  const from = server.mark();
  await shell.locator('.lc-set-tap').click();
  // The card quotes every name (a quoted word is a value, §15) and the property (§33 58).
  const line = `Set Interface ${kept.map(n => `"${n}"`).join(' + ')} "Shell" No`;
  await server.waitSysmon(new RegExp(`\\] ${line.replace(/[+.]/g, m => `\\${m}`)} →`), from);
  // One line, not one per row.
  expect(server.sysmonSince(/\] Set Interface .* "Shell" No →/, from)).toHaveLength(1);
  await card.locator('[data-x]').click();
  // The feed brings the change back into the grid.
  for (const n of kept.filter(n => n !== 'lo0')) await expect(cell(g, n, 'shell')).toHaveText('No');
  await server.command(`Set Interface ${kept.map(n => `"${n}"`).join(' + ')} "Shell" Yes`);
});

test('Settings: Menu "Settings" opens and the same line closes it; /Tab switches; cells show their kind and home; a change is one Set line', async ({ page }, info) => {
  await open(page);
  const run = (l: string) => page.evaluate(line => (window as any).wsCommand(line), l);
  await run('Menu "Settings"');
  const setup = page.locator('luma-setup');
  await expect(setup).toBeVisible();
  await expect(setup.locator('.lc-setup-tabs [data-tab="Station"]')).toHaveClass(/\bon\b/);
  const cells = setup.locator('luma-settings .lc-set-cell');
  await expect(cells.first()).toBeVisible();
  // Kinds: Aa text, ≡ list, I/O switch — neutral badges; home marks.
  await expect(setup.locator('.lc-set-cell[data-kind="text"] .lc-kind').first()).toHaveText('Aa');
  await expect(setup.locator('.lc-set-cell[data-kind="list"] .lc-kind').first()).toHaveText('≡');
  await expect(setup.locator('.lc-set-cell[data-kind="toggle"] .lc-kind').first()).toHaveText('I/O');
  await expect(setup.locator('.lc-set-cell .lc-home[data-home="station"]').first()).toBeAttached();
  // The same line again closes it (never a stacked copy).
  await run('Menu "Settings"');
  await expect(setup).toHaveCount(0);
  // /Tab opens on that tab.
  await run('Menu "Settings" /Tab Show');
  await expect(setup.locator('.lc-setup-tabs [data-tab="Show"]')).toHaveClass(/\bon\b/);
  const autosave = setup.locator('.lc-set-cell', { hasText: 'Autosave' });
  await expect(autosave.locator('.lc-kind')).toHaveText('123');
  await expect(setup.locator('.lc-set-cell .lc-home[data-home="show"]').first()).toBeAttached();
  // A tab tap is a Menu line (the profile keeps it, no Oops step).
  let from = server.mark();
  await press(page, info, setup.locator('.lc-setup-tabs [data-tab="Station"]'));
  await server.waitSysmon(/\] Menu "Settings" \/Tab Station → ok/, from);
  await expect(setup.locator('.lc-setup-tabs [data-tab="Station"]')).toHaveClass(/\bon\b/);
  await expect(setup).toBeVisible();
  // A cell change: one Set line.
  from = server.mark();
  await setup.locator('.lc-set-cell', { hasText: 'Location' }).locator('.lc-set-tap').click();
  const d = dialog(page);
  await d.locator('input').first().fill('Booth');
  await d.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(/\] Set Station "Location" "Booth" → ok/, from);
  await expect(setup.locator('.lc-set-cell', { hasText: 'Location' }).locator('.lc-set-value')).toHaveText('Booth');
  // × closes; reopened it starts on the last tab.
  await setup.locator('[data-x]').click();
  await expect(setup).toHaveCount(0);
  await run('Menu "Settings"');
  await expect(setup.locator('.lc-setup-tabs [data-tab="Station"]')).toHaveClass(/\bon\b/);
  const issues = await fitIssues(page, 'settings menu', { touch: touch(info) });
  expect(issues).toEqual([]);
});

test('rights: a Program user sees the station cells greyed with the reason', async ({ page }, info) => {
  const name = `Prog12d${info.project.name.slice(0, 1).toUpperCase()}`;
  const made = await server.command(`Store User "${name}" /Rights Program`);
  expect(made.ok, made.message).toBe(true);
  await open(page, name);
  await page.evaluate(() => (window as any).wsCommand('Menu "Settings"'));
  const setup = page.locator('luma-setup');
  const loc = setup.locator('.lc-set-cell', { hasText: 'Location' });
  await expect(loc).toHaveAttribute('data-why', 'Needs Setup — you are Program');
  await expect(loc.locator('.lc-kind')).toHaveClass(/\bro\b/);
  await loc.locator('.lc-set-tap').click();
  await expect(page.locator('.lc-toast')).toHaveText('Needs Setup — you are Program');
  await expect(dialog(page)).toHaveCount(0);
});

test('power toggle look A and the slab in the Showroom: on, off, failing; greyed cells say why', async ({ page }, info) => {
  await open(page);
  await showroom(page);
  const powers = page.locator('#sr12d .lc-sr-powers .lc-power');
  await expect(powers.nth(0)).toHaveAttribute('data-state', 'on');
  await expect(powers.nth(1)).toHaveAttribute('data-state', 'off');
  await expect(powers.nth(2)).toHaveAttribute('data-state', 'failing');
  await expect(powers.nth(2).locator('.lc-power-press')).toHaveText('en1 has no link — DMX not sent');
  for (const i of [0, 1, 2]) await expect(powers.nth(i).locator('.lc-power-bar')).toBeAttached();
  const bar = await powers.nth(0).locator('.lc-power-bar').evaluate(e => getComputedStyle(e).height);
  expect(bar).toBe('5px');
  // The slab: the standard actions in their order, counts on the labels.
  const slab = page.locator('#sr12d luma-slab').first();
  const order = await slab.locator('.lc-slab-desk [data-act]').evaluateAll(els => els.map(e => (e as HTMLElement).dataset.act));
  if (!phone(info)) expect(order).toEqual(['select', 'new', 'copy', 'cut', 'paste', 'delete', 'oops', 'import', 'export', 'edit']);
  const paste = slab.locator('[data-act="paste"]').filter({ visible: true });
  if (await paste.count()) {
    await expect(paste).toHaveAttribute('data-why', 'Copy or Cut first');
    await press(page, info, paste, { force: true });
    await expect(page.locator('.lc-toast')).toHaveText('Copy or Cut first');
  }
  await expect(slab.locator('[data-act="delete"]').filter({ visible: true })).toHaveText('Delete 1');
});

test('grid: a value sent to another station is pending until it reports it; a refusal marks its cell', async ({ page }, info) => {
  // Decisions §30 "Remote station edits": row.pending / row.refused (the
  // Stations grid's rows carry them from the lumasessions feed).
  await open(page);
  await showroom(page);
  await press(page, info, page.locator('#sr12d button', { hasText: 'Pending + refused cells' }));
  const g = grid(page);
  const sent = cell(g, 'o1', 'rate');
  await expect(sent).toHaveClass(/pending/);
  await expect(sent.locator('.lc-gc-text')).toHaveText('44 Hz');
  await expect(sent).toHaveAttribute('title', /Sent 44 Hz — waiting for the station to report it \(now 30 Hz\)/);
  await expect(sent.locator('.lc-gc-wait')).toBeAttached();
  const refused = cell(g, 'o2', 'protocol');
  await expect(refused).toHaveClass(/refused/);
  await expect(refused).toHaveAttribute('title', /E-RIGHTS-001/);
});

test('fit: the setup frame, the slab and the edit card stay inside the screen (multiplatform rules)', async ({ page }, info) => {
  await open(page);
  await page.evaluate(() => (0, eval)('openSetupMenu("Network")'));
  const g = page.locator('luma-setup luma-grid');
  await expect(g.locator('[data-col="shell"]').first()).toBeVisible();
  expect(await fitIssues(page, 'network menu', { touch: touch(info) })).toEqual([]);
  await press(page, info, g.locator('[data-htap="shell"]'));
  await press(page, info, page.locator('luma-setup luma-slab [data-act="edit"]').filter({ visible: true }));
  await expect(page.locator('.lc-card')).toBeVisible();
  expect(await fitIssues(page, 'edit card', { touch: touch(info) })).toEqual([]);
});

// ── 12d part 4 (decisions §30 "12d part 4 built"): every setup menu through
// the real frame — Network, DMX Protocols, LumaSessions, Users & Profiles and
// Settings › Assets / About; the station table; the station asks.

const menu = (page: Page, line: string) => page.evaluate(l => (window as any).wsCommand(l), line);
const setupEl = (page: Page) => page.locator('luma-setup');
const slabCell = (page: Page, id: string) => setupEl(page).locator(`luma-slab [data-act="${id}"]`).filter({ visible: true });
const ownCell = (page: Page, id: string) => setupEl(page).locator(`luma-slab [data-own][data-id="${id}"]`).filter({ visible: true });
const powerBtn = (page: Page) => setupEl(page).locator('luma-power .lc-power').filter({ visible: true });
async function station() { return (await fetch(`${server.url}/api/station`)).json(); }

test('Network: hardware rows grey every row action; bound cells show the station right after login; Online Off asks', async ({ page }, info) => {
  // Right after the page loads — before the shell is up — the menu asks for
  // its bound cells; the answer still arrives (the part 3 finding).
  await server.open(page, '/', 'Admin');
  await page.waitForFunction(() => (0, eval)('typeof openSetupMenu === "function" && window.LumaKit'));
  await page.evaluate(() => (0, eval)('openSetupMenu("Network")'));
  const name = (await station()).name;
  const bound = setupEl(page).locator('luma-slab [data-bound]').filter({ visible: true });
  if (!phone(info)) await expect(bound.first()).toContainText(name);
  await connected(page);
  await page.evaluate(() => document.getElementById('fb-cards')?.replaceChildren());
  const g = setupEl(page).locator('luma-grid');
  await expect(g.locator('[data-col="shell"]').first()).toBeVisible();
  // A row selected: still no New / Copy / Cut / Paste / Delete / Import.
  const nic = await g.locator('[data-col="shell"]').first().getAttribute('data-k');
  await press(page, info, g.locator(`[data-gutter="${nic}"]`));
  for (const id of ['new', 'copy', 'cut', 'paste', 'delete', 'import']) {
    const c = slabCell(page, id);
    if (await c.count()) await expect(c).toHaveAttribute('data-why', 'Interfaces come from the hardware');
  }
  // The power toggle asks before Online Off (the station's dialog).
  await press(page, info, powerBtn(page));
  const d = dialog(page);
  await expect(d).toContainText('Go offline');
  await d.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  await expect(d).toHaveCount(0);
  await expect(powerBtn(page)).toHaveAttribute('data-state', 'on');
  // Networks: New asks the name; one Store line.
  await menu(page, 'Menu "Network" /Tab Networks');
  await expect(setupEl(page).locator('.lc-setup-tabs [data-tab="Networks"]')).toHaveClass(/\bon\b/);
  const from = server.mark();
  await press(page, info, phone(info) ? await sheetCell(setupEl(page), '[data-act="new"]') : slabCell(page, 'new'));
  const nd = dialog(page);
  const net = `Net${info.project.name.slice(0, 2)}`;
  await nd.locator('input').first().fill(net);
  await nd.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(new RegExp(`\\] Store Network "${net}" → ok`), from);
  await expect(setupEl(page).locator('luma-grid [data-col="name"]', { hasText: net })).toBeVisible();
  expect(await fitIssues(page, 'network menu · networks', { touch: touch(info) })).toEqual([]);
});

test('DMX Protocols: networks on the left, Lines · Routing · Discovery on top; HoldOutput and Delete ask', async ({ page }, info) => {
  await open(page);
  await menu(page, 'Menu "DMXProtocols"');
  const s = setupEl(page);
  // The left tabs are the show's networks, with the stripe of what is sent.
  await expect(s.locator('.lc-setup-tabs [data-left="Default"]')).toBeVisible();
  await expect(s.locator('.lc-setup-tabs [data-left="Default"] .lc-setup-stripe')).toBeAttached();
  await expect(s.locator('.lc-setup-strip [data-tab]')).toHaveText(['Lines', 'Routing', 'Discovery']);
  const g = s.locator('luma-grid');
  const line = await page.evaluate(() => (0, eval)('setupMenu.data.network.lines').find((l: any) => l.object === 'Sacn' && l.line === 1));
  await expect(g.locator(`[data-k="${line.guid}"][data-col="rate"]`)).toHaveText('30 Hz');
  // Rows mix protocols (decisions §34 "34f7c"): no bound cells; a row is selected for the slab.
  await press(page, info, g.locator(`[data-gutter="${line.guid}"]`));
  // Deleting a line is a station ask.
  await press(page, info, slabCell(page, 'delete'));
  let d = dialog(page);
  await expect(d).toContainText('Delete Sacn');
  await d.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  expect((await server.command('List Sacn')).message).toContain('7 Sacn line(s)');
  // The power toggle asks before holding the output.
  await press(page, info, powerBtn(page));
  d = dialog(page);
  await expect(d).toContainText('Hold DMX output');
  await d.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  // Routing: one row per route on this network.
  await press(page, info, s.locator('.lc-setup-strip [data-tab="Routing"]'));
  await expect(s.locator('luma-grid [data-col="line"]', { hasText: 'Sacn 1' }).first()).toBeVisible();
  // A phone keeps New in the More sheet (§37 1), greyed with its reason there too.
  await expect(phone(info) ? await sheetCell(s, '[data-act="new"]') : s.locator('luma-slab [data-act="new"]').filter({ visible: true })).toHaveAttribute('data-why', /Assign/);
  if (phone(info)) await closeSheet(page);
  expect(await fitIssues(page, 'dmx protocols · routing', { touch: touch(info) })).toEqual([]);
});

test('LumaSessions: this station and the station table; New row asks its name; several stations + Edit = ONE Set line; Cards', async ({ page }, info) => {
  const tag = info.project.name.slice(0, 2);
  for (const n of [`Tower${tag}`, `Delay${tag}`]) expect((await server.command(`Store Station "${n}"`)).ok).toBe(true);
  await open(page);
  await menu(page, 'Menu "LumaSessions"');
  const s = setupEl(page);
  const g = s.locator('luma-grid');
  await expect(g.locator('[data-col="name"]', { hasText: `Tower${tag}` })).toBeVisible();
  await expect(g.locator('[data-col="role"]').first()).toHaveText('Standalone');
  await expect(powerBtn(page)).toHaveAttribute('data-state', 'off');
  await expect(powerBtn(page).locator('.lc-power-word')).toHaveText('STANDALONE');
  // New: the row's station name is asked; one Store line.
  let from = server.mark();
  await press(page, info, phone(info) ? await sheetCell(setupEl(page), '[data-act="new"]') : slabCell(page, 'new'));
  const nd = dialog(page);
  await nd.locator('input').first().fill(`Rig${tag}`);
  await nd.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(new RegExp(`\\] Store Station "?Rig${tag}"? → ok`), from); // the dialog quotes the name (a value, §15)
  // Two stations by the gutter (+ latched), Edit → the card → their NIC on
  // the show's network Default: one line names both.
  const rows = await page.evaluate(() => (0, eval)('setupMenu.data.lumasessions.members'));
  const key = (n: string) => rows.find((r: any) => r.name === n).guid;
  await press(page, info, g.locator(`[data-gutter="${key(`Tower${tag}`)}"]`));
  await setMode(page, info, g, '+');
  await press(page, info, g.locator(`[data-gutter="${key(`Delay${tag}`)}"]`));
  await setMode(page, info, g, '+');
  await press(page, info, phone(info) ? await sheetCell(setupEl(page), '[data-act="edit"]') : slabCell(page, 'edit'));
  const card = page.locator('.lc-card');
  await expect(card).toBeVisible();
  await card.locator('[data-tab]', { hasText: 'Wiring' }).click();
  const nets = await page.evaluate(() => (0, eval)('setupMenu.data.lumasessions.networks'));
  const def = nets.find((n: any) => n.name === 'Default');
  from = server.mark();
  await card.locator(`[data-field="nic:${def.guid}"] .lc-set-tap`).click();
  const vd = dialog(page);
  await vd.locator('input').first().fill('en7');
  await vd.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(new RegExp(`\\] Set Station "Tower${tag}" \\+ "Delay${tag}" Network "Default" "Interface" "?en7"? → ok`), from);
  expect(server.sysmonSince(/\] Set Station .* Network "Default" "Interface"/, from)).toHaveLength(1);
  await card.locator('[data-x]').click();
  // The feed brings both rows back with it (a phone folds the column into the row).
  await expect.poll(() => page.evaluate(([t, g]) => (0, eval)('setupMenu.data.lumasessions.members').filter((r: any) => r.name === `Tower${t}` || r.name === `Delay${t}`).map((r: any) => r[`nic:${g}`]), [tag, def.guid]))
    .toEqual(['en7', 'en7']);
  if (!phone(info)) await expect(g.locator(`[data-k="${key(`Delay${tag}`)}"][data-col="nic:${def.guid}"]`)).toHaveText('en7');
  // Hand master needs one row of a session; Cards show the same rows.
  if (!phone(info)) await expect(ownCell(page, 'hand')).toHaveAttribute('data-why', 'Select one row');
  await press(page, info, s.locator('[data-view="cards"]'));
  await expect(s.locator('.lc-grid-cards')).toBeVisible();
  expect(await fitIssues(page, 'lumasessions · cards', { touch: touch(info) })).toEqual([]);
});

test('Users & Profiles on the grid and the slab: New is the station ask, several deletes ask once, Default is locked, a profile\'s settings', async ({ page }, info) => {
  const tag = info.project.name.slice(0, 2);
  for (const n of [`Ua${tag}`, `Ub${tag}`]) expect((await server.command(`Store User "${n}" /Rights Program /Profile New`)).ok).toBe(true);
  await open(page);
  await menu(page, 'Menu "Users"');
  const s = setupEl(page);
  const g = s.locator('luma-grid');
  await expect(g.locator('[data-col="name"]', { hasText: `Ua${tag}` })).toBeVisible();
  await expect(g.locator('[data-col="rights"]').first()).toHaveText('Admin');
  // New: the station's New user ask (12c2).
  await press(page, info, phone(info) ? await sheetCell(setupEl(page), '[data-act="new"]') : slabCell(page, 'new'));
  let d = dialog(page);
  await expect(d).toContainText('New user');
  await d.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  // Two users, Delete: one ask for both.
  const users = (await station()).users;
  const k = (n: string) => users.find((u: any) => u.name === n).guid;
  await press(page, info, g.locator(`[data-gutter="${k(`Ua${tag}`)}"]`));
  await setMode(page, info, g, '+');
  await press(page, info, g.locator(`[data-gutter="${k(`Ub${tag}`)}"]`));
  await setMode(page, info, g, '+');
  await expect(phone(info) ? await sheetCell(s, '[data-act="delete"]') : slabCell(page, 'delete')).toContainText('Delete 2');
  if (phone(info)) await closeSheet(page);
  await press(page, info, slabCell(page, 'delete'));
  d = dialog(page);
  await expect(d).toContainText('Delete 2 Users');
  await d.locator('.lc-dlg-btn', { hasText: 'Delete' }).click();
  await expect.poll(async () => (await station()).users.some((u: any) => u.name === `Ua${tag}` || u.name === `Ub${tag}`)).toBe(false);
  // Profiles: Default is the locked template.
  await press(page, info, s.locator('.lc-setup-tabs [data-tab="Profiles"]'));
  const profiles = (await station()).profiles;
  const dflt = profiles.find((p: any) => p.locked);
  await press(page, info, s.locator(`luma-grid [data-gutter="${dflt.guid}"]`));
  await expect(phone(info) ? await sheetCell(s, '[data-act="delete"]') : slabCell(page, 'delete')).toHaveAttribute('data-why', /locked template/);
  if (phone(info)) await closeSheet(page);
  // A profile's settings: its typed tabs (12b2) in the settings window.
  const admin = profiles.find((p: any) => p.name === 'Admin');
  await press(page, info, s.locator(`luma-grid [data-gutter="${admin.guid}"]`));
  await press(page, info, ownCell(page, 'settings'));
  const sw = page.locator('.lc-set-scrim');
  await expect(sw.locator('.lc-set-tabs button')).toHaveText(['Look', 'Input & keys', 'Oops', 'Command line', 'Attributes', 'MVR import', 'Lock screen', 'Meta', 'Tags']);
  await page.keyboard.press('Escape');
});

test('rights: a Program user sees New user greyed with the reason; their own password is theirs', async ({ page }, info) => {
  const name = `Prog4${info.project.name.slice(0, 1).toUpperCase()}`;
  await server.command(`Store User "${name}" /Rights Program`);
  await open(page, name);
  await menu(page, 'Menu "Users"');
  const g = setupEl(page).locator('luma-grid');
  await expect(phone(info) ? await sheetCell(setupEl(page), '[data-act="new"]') : slabCell(page, 'new')).toHaveAttribute('data-why', 'Needs Admin — you are Program');
  if (phone(info)) await closeSheet(page);
  const users = (await station()).users;
  await press(page, info, g.locator(`[data-gutter="${users.find((u: any) => u.name === 'Admin').guid}"]`));
  if (!phone(info)) await expect(ownCell(page, 'password')).toHaveAttribute('data-why', 'Needs Admin — you are Program');
  await press(page, info, g.locator(`[data-gutter="${users.find((u: any) => u.name === name).guid}"]`));
  if (!phone(info)) await expect(ownCell(page, 'password')).not.toHaveAttribute('data-why', /.+/);
});

test('Settings › Assets (New asks the drive name, the folder is this station\'s) and About (read only)', async ({ page }, info) => {
  await open(page);
  await menu(page, 'Menu "Settings" /Tab Assets');
  const s = setupEl(page);
  await expect(s.locator('.lc-setup-tabs [data-tab="Assets"]')).toHaveClass(/\bon\b/);
  const drive = `Video${info.project.name.slice(0, 2)}`;
  const from = server.mark();
  await press(page, info, phone(info) ? await sheetCell(setupEl(page), '[data-act="new"]') : slabCell(page, 'new'));
  const d = dialog(page);
  await d.locator('input').first().fill(drive);
  await d.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(new RegExp(`\\] Store AssetRoot "?${drive}"? → ok`), from);
  const row = s.locator('luma-grid [data-col="name"]', { hasText: drive });
  await expect(row).toBeVisible();
  const key = await row.getAttribute('data-k');
  await expect(s.locator(`luma-grid [data-k="${key}"][data-col="folder"]`)).toHaveText('None');
  await press(page, info, s.locator('.lc-setup-tabs [data-tab="About"]'));
  const fmt = s.locator('.lc-set-cell', { hasText: 'ShowFormat' });
  await expect(fmt.locator('.lc-set-value')).toHaveText('31'); // FORMAT_VERSION
  await expect(fmt).toHaveAttribute('data-why', /read-only|facts/);
});

// ── 12d part 5 (decisions §30 "12d part 5 built"): Backup on the setup frame
// — Library (the loaded show selected, the coloured show actions, the Load
// show dialog: partial = ONE Import Show line), Show · History (one row per
// save, List | Matrix, a cell's objects), Recovery (copies, Open copy and
// Clean up ask) and the startup recovery ask after a crash, once.

const shot = async (page: Page, info: TestInfo, name: string) => {
  if (info.project.name !== 'desktop' && info.project.name !== 'phone') return;
  await page.waitForTimeout(250);
  await page.screenshot({ path: path.join(REPO, 'target/screenshots/12d-part5', `${info.project.name}-${name}.png`) });
};

test('Backup › Library: first tab, the loaded show selected, coloured show actions; the Load show dialog takes parts as ONE Import Show line', async ({ page }, info) => {
  const tag = info.project.name.slice(0, 2);
  const fest = `Festival${tag}`;
  expect((await server.command(`SaveShow "${fest}"`)).ok).toBe(true);
  expect((await server.command('SaveShow "demo"')).ok).toBe(true);
  await open(page);
  await menu(page, 'Menu "Backup"');
  const s = setupEl(page);
  await expect(s.locator('.lc-setup-name')).toHaveText('Backup');
  await expect(s.locator('[data-tab]')).toHaveText([/Library/, /Show · History/, /Recovery/]);
  await expect(s.locator('[data-tab="Library"]')).toHaveClass(/\bon\b/);
  const g = s.locator('luma-grid');
  await expect(g.locator('[data-row="demo"]')).toHaveClass(/\bwhole\b/);
  await expect(g.locator(`[data-k="${fest}"][data-col="format_text"]`)).toContainText('loads');
  // The show actions: one tone each (Load the gold primary); an upright phone's bar keeps two
  // cells, Load · Save (§37 1), the rest sit in the More sheet.
  const tones: Record<string, RegExp> = { load: /t-load/, save: /t-save/, saveas: /t-save/, new: /t-neutral/, info: /t-info/, history: /t-history/ };
  for (const [id, re] of Object.entries(tones)) {
    if (phone(info) && !['load', 'save'].includes(id)) continue;
    await expect(ownCell(page, id)).toHaveClass(re);
  }
  await shot(page, info, 'library');
  expect(await fitIssues(page, 'backup › library', { touch: touch(info) })).toEqual([]);
  // Load show… on another show: its facts and parts.
  await press(page, info, g.locator(`[data-gutter="${fest}"]`));
  await press(page, info, ownCell(page, 'load'));
  const d = dialog(page);
  await expect(d).toContainText(`Load show · ${fest}`);
  await expect(d.locator('.lc-dlg-select')).toHaveCount(5);
  await shot(page, info, 'load-dialog');
  expect(await fitIssues(page, 'load show dialog', { touch: touch(info) })).toEqual([]);
  // Everything ticked is LoadShow: the station asks first.
  await d.locator('.lc-dlg-btn', { hasText: /^Load$/ }).click();
  const ask = dialog(page);
  await expect(ask).toContainText('Load show');
  await ask.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  // Patch unticked: Import Show with the other parts, one line.
  await press(page, info, ownCell(page, 'load'));
  const d2 = dialog(page);
  await d2.locator('.lc-dlg-select', { hasText: 'Patch' }).locator('[data-choice="None"]').click();
  const from = server.mark();
  await d2.locator('.lc-dlg-btn', { hasText: /^Load$/ }).click();
  await server.waitSysmon(new RegExp(`\\] Import Show "?${fest}"? Data Protocols Users Stations → ok`), from);
  expect(server.sysmonSince(/\] Import Show /, from)).toHaveLength(1);
});

test('Backup › Show · History: one row per save (by, station, reason, note, changes, kept); List | Matrix; a cell names its objects', async ({ page }, info) => {
  const tag = info.project.name.slice(0, 2);
  const show = `Hist${tag}`;
  expect((await server.command(`SaveShow "${show}"`)).ok).toBe(true);
  expect((await server.command(`Store User "H${tag}" /Rights Program /Profile New`)).ok).toBe(true);
  expect((await server.command('SaveShow /Note "rehearsal"')).ok).toBe(true);
  await open(page);
  await menu(page, 'Menu "Backup" /Tab History');
  const s = setupEl(page);
  await expect(s.locator('[data-tab="History"]')).toHaveClass(/\bon\b/);
  const g = s.locator('luma-grid');
  // Save As keeps the history: the older rows are the show it came from.
  await expect.poll(() => page.evaluate(() => (0, eval)('setupMenu.data.shows?.history?.length') || 0)).toBeGreaterThan(1);
  const rows = await page.evaluate(() => (0, eval)('setupMenu.data.shows.history'));
  await expect(g.locator('.lc-grid-row')).toHaveCount(rows.length);
  expect(rows.slice(0, 2).map((r: any) => r.kept)).toEqual(['current', 'version']);
  expect(rows[1].reason).toBe('Save As');
  expect(rows[0].reason).toBe('Quick');
  expect(rows[0].changes).toContain('+1 user');
  await expect(g.locator(`[data-k="${rows[0].id}"][data-col="saved"]`)).toContainText('today');
  if (!phone(info)) await expect(g.locator(`[data-k="${rows[0].id}"][data-col="note"]`)).toHaveText('rehearsal');
  // The caption (a phone folds the frame's chrome and has none, §37 1).
  if (!phone(info)) await expect(s.locator('.lc-grid-cap')).toContainText(new RegExp(`${show}.* — saved today .* by Admin`));
  await shot(page, info, 'history-list');
  // The matrix: a column per object kind; a tapped cell names the objects.
  await press(page, info, s.locator('[data-view="matrix"]'));
  const users = g.locator(`[data-k="${rows[0].id}"][data-col="m_users"]`);
  await expect(users).toHaveText('+1');
  await press(page, info, users);
  await expect(s.locator('[data-last]')).toContainText(`Users: + User "H${tag}"`);
  await shot(page, info, 'history-matrix');
  expect(await fitIssues(page, 'backup › history matrix', { touch: touch(info) })).toEqual([]);
  await press(page, info, s.locator('[data-view="grid"]'));
  // The current file can't be deleted; the version opens as unsaved changes (asks).
  await press(page, info, g.locator(`[data-gutter="${rows[0].id}"]`));
  await expect(phone(info) ? await sheetCell(setupEl(page), '[data-act="delete"]') : slabCell(page, 'delete')).toHaveAttribute('data-why', /current file/);
  if (phone(info)) await closeSheet(page);
  await press(page, info, g.locator(`[data-gutter="${rows[1].id}"]`));
  if (!phone(info)) {
    await press(page, info, ownCell(page, 'open'));
    const ask = dialog(page);
    await expect(ask).toContainText('Open the version');
    await ask.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  }
});

test('Backup › Recovery: a load over unsaved work keeps a copy; Open copy and Clean up ask; after a crash the station asks once', async ({ page }, info) => {
  const tag = info.project.name.slice(0, 2);
  // demo loaded and changed, then another show loaded without saving.
  expect((await server.command(`SaveShow "Rec${tag}"`)).ok).toBe(true);
  expect((await server.command('SaveShow "demo"')).ok).toBe(true);
  expect((await server.command(`Store User "R${tag}" /Rights Program /Profile New`)).ok).toBe(true);
  const out = await server.command(`LoadShow "Rec${tag}" /NoSave /NoConfirmation`);
  expect(out.message).toContain('recovery copy');
  await open(page);
  await menu(page, 'Menu "Backup" /Tab Recovery');
  const s = setupEl(page);
  const g = s.locator('luma-grid');
  await expect.poll(() => page.evaluate(() => (0, eval)('setupMenu.data.shows?.recovery?.length') || 0)).toBeGreaterThan(0);
  const copies = await page.evaluate(() => (0, eval)('setupMenu.data.shows.recovery'));
  const copy = copies.find((c: any) => c.show === 'demo' && c.reason === 'Before load');
  expect(copy).toBeTruthy();
  await expect(g.locator(`[data-k="${copy.id}"][data-col="reason"]`)).toHaveText('Before load');
  await shot(page, info, 'recovery');
  expect(await fitIssues(page, 'backup › recovery', { touch: touch(info) })).toEqual([]);
  await press(page, info, g.locator(`[data-gutter="${copy.id}"]`));
  await press(page, info, ownCell(page, 'open'));
  let ask = dialog(page);
  await expect(ask).toContainText('Open the recovery copy');
  await ask.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  await press(page, info, ownCell(page, 'clean'));
  ask = dialog(page);
  await expect(ask).toContainText('Clean Up');
  await ask.locator('.lc-dlg-btn', { hasText: 'Cancel' }).click();
  // A crash, a start: the recovery copy of demo is newer than its file — one ask.
  await page.close();
  await server.crash();
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  const p2 = await page.context().newPage();
  await server.open(p2, '/', 'Admin');
  await connected(p2);
  const rec = dialog(p2);
  await expect(rec).toContainText('stopped unexpectedly');
  await expect(rec.locator('.lc-dlg-btn', { hasText: 'Open copy' })).toBeVisible();
  await shot(p2, info, 'recovery-ask');
  await rec.locator('.lc-dlg-btn', { hasText: 'Open saved show' }).click();
  await expect(rec).toHaveCount(0);
  // A clean stop and start: no ask.
  await server.stop();
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  const p3 = await page.context().newPage();
  await server.open(p3, '/', 'Admin');
  await connected(p3);
  await p3.waitForTimeout(1500);
  await expect(dialog(p3)).toHaveCount(0);
});
