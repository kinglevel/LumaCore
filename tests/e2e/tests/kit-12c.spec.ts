// Step 12c (decisions §30 "12c hardening" / "12c design locked", the boards in
// docs/dev/ui-kit/mockups/2026-10-13-12c/): every window has the kit's
// <luma-window-bar> — pinned settings and its type's actions, drawn from the
// window type and the `windows` feed; every tap is a `Self` line sent with the
// window as owner. A live title on the tab; `…` when narrow; below the type's
// minimum size the window menu lists the items. The settings window pins
// cells to the bar, ↺ resets one, the Window bar tab orders the items, and
// the header saves / loads a WindowPreset or resets the window. On the
// desktop, a touch tablet and WebKit (the phone runs the Showroom check: its
// windows are below the bars' minimum widths).

import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
import { REPO, Server, cmd, connected, dialog, kbdMode, box } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const phone = (info: TestInfo) => info.project.name === 'phone';

async function open(page: Page) {
  await server.open(page, '/', 'Admin');
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction(() => (0, eval)('liveScreen && winFeed.sub && windowTypes.length') as boolean);
}

type Node = { type: 'leaf'; wins: number[]; active: number } | { type: 'split'; dir: 'h' | 'v'; ratio: number; children: [Node, Node] };
const leaf = (...wins: number[]): Node => ({ type: 'leaf', wins, active: 0 });
const split = (dir: 'h' | 'v', ratio: number, a: Node, b: Node): Node => ({ type: 'split', dir, ratio, children: [a, b] });

/** Lay the screen out like a user splitting panes (the layout path); every
 *  window starts with its type's defaults. Waits until the feed has them.
 *  The layout PUT never writes window settings (decisions §30 "12c"): a
 *  number an earlier test used keeps its settings and bar, so each window
 *  is put back to its type's defaults with `Reset Window`. */
async function lay(page: Page, root: Node, windows: [number, string][]) {
  await page.evaluate(([root, windows]) => {
    const w = (0, eval);
    const wins = Object.fromEntries((windows as [number, string][]).map(([no, kind]) => [no, { no, kind, label: null, settings: {} }]));
    w('layout').windows = wins;
    w('layout').root = root;
    w('focusPath = []');
    w('commit()');
  }, [root, windows] as const);
  await page.waitForFunction(nos => (nos as number[]).every(n => {
    const v = (0, eval)('winFeed.views')[n];
    return v && v.type === (0, eval)('layout').windows[n]?.kind;
  }), windows.map(w => w[0]));
  for (const [no] of windows) expect((await server.command(`Reset Window 1.${no}`)).ok).toBe(true);
}

/** A window's state on the feed (what the station resolved). */
const view = (page: Page, no: number) => page.evaluate(n => (0, eval)('winFeed.views')[n], no);
/** A window's stored settings on the station (GET /api/screens/1). */
const stored = (page: Page, no: number) => page.evaluate(async n => (await (await fetch('/api/screens/1')).json()).windows[n].settings, no);
const bar = (page: Page, no: number) => page.locator(`.win-wrap[data-win="${no}"] > luma-window-bar`);

/** Open window `no`'s settings from its window menu (tab dot › Settings…). */
async function settingsOf(page: Page, no: number) {
  await page.locator(`.ptab[data-win="${no}"] .tab-dot`).click();
  await page.locator('#ctx .ci', { hasText: 'Settings…' }).click();
  const set = page.locator('.lc-set-scrim luma-settings');
  await expect(set).toBeVisible();
  return set;
}

test('System Monitor bar: Pause = AutoScroll on the station, the box follows the feed', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(3), [[3, 'sysmon']]);
  const pause = bar(page, 3).locator('[data-id="sysmon/pause"]');
  await expect(pause).toHaveAttribute('aria-pressed', 'false');
  const mark = server.mark();
  await pause.click();
  // Journal and System Monitor show the resolved window; the station's setting moved.
  await server.waitSysmon(/\[Admin@Screen 1\] Set Window 1\.3 "AutoScroll" Toggle → ok/, mark);
  await expect.poll(async () => (await stored(page, 3)).auto_scroll).toBe(false);
  await expect(pause).toHaveAttribute('aria-pressed', 'true');
  await expect(pause.locator('.lc-wb-box.on')).toHaveCount(1);
  // Changed elsewhere (another client): the box follows the feed.
  await server.command('Set Window 1.3 "AutoScroll" On');
  await expect(pause).toHaveAttribute('aria-pressed', 'false');
  // Every drawn item names its window type and setting / action.
  const ids = await bar(page, 3).locator('[data-id]').evaluateAll(els => els.map(e => e.getAttribute('data-id')));
  expect(ids).toEqual(expect.arrayContaining(['sysmon/pause', 'sysmon/wrap', 'sysmon/level', 'sysmon/time', 'sysmon/clear']));
  for (const id of ids) expect(id).toMatch(/^sysmon\/[a-z_]+$/);
});

test('Clear marks "cleared here" in that window only; the search chip shows and clears', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  // A touch tablet (1024 × 768) draws two stacked sysmon panes below their
  // natural size as miniatures (decisions §30 "12c2": nothing under 44 px is
  // tappable there, hold opens the pane full screen), so there the two
  // windows sit side by side.
  await lay(page, split(info.project.name === 'tablet' ? 'h' : 'v', 0.5, leaf(3), leaf(5)), [[3, 'sysmon'], [5, 'sysmon']]);
  await cmd(page, 'Echo "before the clear"');
  await expect(page.locator('.win-wrap[data-win="3"] .sm-row', { hasText: 'before the clear' })).toHaveCount(1);
  await bar(page, 3).locator('[data-id="sysmon/clear"]').click();
  await expect(page.locator('.win-wrap[data-win="3"] .sm-cleared')).toBeVisible();
  await expect(page.locator('.win-wrap[data-win="3"] .sm-row', { hasText: 'before the clear' })).toHaveCount(0);
  expect((await stored(page, 3)).since).toBeGreaterThan(1_700_000_000_000);
  // The other monitor keeps everything (the station keeps its history).
  await expect(page.locator('.win-wrap[data-win="5"] .sm-cleared')).toBeHidden();
  await expect(page.locator('.win-wrap[data-win="5"] .sm-row', { hasText: 'before the clear' })).toHaveCount(1);

  // A search (window 5 focused, typed): the auto chip appears; its × clears.
  await page.locator('.win-wrap[data-win="5"] .sm-list').click();
  await page.locator('#cb-line').click();
  await cmd(page, 'Set Self "Search" "clear"');
  const chip = bar(page, 5).locator('[data-id="sysmon/search"]');
  await expect(chip).toHaveText('× Search “clear”');
  await expect(bar(page, 3).locator('[data-id="sysmon/search"]')).toHaveCount(0);
  // A narrow bar (the tablet's half-width pane) keeps the chip in its … menu.
  if (await chip.isHidden()) {
    await bar(page, 5).locator('.lc-wb-more').click();
    await page.locator('#ctx .ci[data-id="sysmon/search"]').click();
  } else await chip.click();
  await expect(chip).toHaveCount(0);
  expect((await stored(page, 5)).search).toBe('');
});

test('DMX bar: ‹ › step the Universe, the live title follows', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(4), [[4, 'dmx']]);
  const title = page.locator('.ptab[data-win="4"] .tab-title');
  await expect(title).toHaveText(/^U 1\b/);
  // A stepper: Universe: ‹ 1 ›
  await expect(bar(page, 4).locator('.lc-wb-stepper')).toContainText('Universe:');
  await bar(page, 4).locator('[data-id="dmx/next"]').click();
  await expect(title).toHaveText(/^U 2\b/);
  await expect(bar(page, 4).locator('[data-id="dmx/universe"]')).toHaveText(/^2\b/);
  expect((await stored(page, 4)).universe).toBe(2);
  await bar(page, 4).locator('[data-id="dmx/prev"]').click();
  await bar(page, 4).locator('[data-id="dmx/prev"]').click();
  // Never below 1.
  await expect(title).toHaveText(/^U 1\b/);
  await expect.poll(async () => (await stored(page, 4)).universe).toBe(1);
  // Values: the pick list; the answer is a Set Self "line".
  await bar(page, 4).locator('[data-id="dmx/readout"]').click();
  await page.locator('luma-popup .lc-pick-row span', { hasText: /^Hex$/ }).click();
  await expect(bar(page, 4).locator('[data-id="dmx/readout"]')).toContainText('Hex');
  expect((await stored(page, 4)).readout).toBe('hex');
});

test('settings: a pin puts a setting in the bar, unpinning takes it out; ↺ resets one setting', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(4), [[4, 'dmx']]);
  const set = await settingsOf(page, 4);
  await expect(set.locator('.lc-set-name')).toHaveText(/^Settings · Window 1\.4 · DMX/);
  await expect(set.locator('.lc-set-tabs button')).toHaveText(['Display', 'Mask', 'Window bar']);
  // Pinned cells: the type's bar (universe, readout).
  await expect(set.locator('[data-setting="readout"]')).toHaveClass(/pinned/);
  await expect(set.locator('[data-setting="view"]')).not.toHaveClass(/pinned/);
  await set.locator('[data-setting="view"] .lc-set-pin').click();
  await expect(set.locator('[data-setting="view"]')).toHaveClass(/pinned/);
  await expect(bar(page, 4).locator('[data-id="dmx/view"]')).toBeVisible();
  await set.locator('[data-setting="readout"] .lc-set-pin').click();
  await expect(set.locator('[data-setting="readout"]')).not.toHaveClass(/pinned/);
  await expect(bar(page, 4).locator('[data-id="dmx/readout"]')).toHaveCount(0);
  expect((await view(page, 4)).items).toEqual(['universe', 'prev', 'next', 'view']);

  // ↺ only on a changed cell; it puts that one setting back.
  await expect(set.locator('[data-setting="readout"] .lc-set-reset')).toHaveCount(0);
  await server.command('Set Window 1.4 "Readout" Decimal');
  await expect(set.locator('[data-setting="readout"] .lc-set-value')).toHaveText('Decimal');
  await set.locator('[data-setting="readout"] .lc-set-reset').click();
  await expect(set.locator('[data-setting="readout"] .lc-set-value')).toHaveText('Percent');
  await expect(set.locator('[data-setting="readout"] .lc-set-reset')).toHaveCount(0);
  await page.keyboard.press('Escape');
});

test('Window bar tab: order, remove, add an action, Load defaults, bar off', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(4), [[4, 'dmx']]);
  const set = await settingsOf(page, 4);
  await set.locator('.lc-set-tabs button', { hasText: 'Window bar' }).click();
  const rows = set.locator('.lc-set-bar-item');
  await expect(rows.locator('.lc-set-bar-name')).toHaveText(['Universe', 'Universe −', 'Universe +', 'Readout']);
  // ↓ on the first: one Set Self "WindowBarItems" line with the new order.
  await rows.nth(0).locator('[data-down]').click();
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['prev', 'universe', 'next', 'readout']);
  await expect(rows.locator('.lc-set-bar-name')).toHaveText(['Universe −', 'Universe', 'Universe +', 'Readout']);
  // × removes; + adds a declared action at the end.
  await rows.nth(3).locator('[data-drop]').click();
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['prev', 'universe', 'next']);
  // The rows follow the station before the next change (the add builds on what they show).
  await expect(rows.locator('.lc-set-bar-name')).toHaveText(['Universe −', 'Universe', 'Universe +']);
  await set.locator('[data-add="first"]').click();
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['prev', 'universe', 'next', 'first']);
  await expect(bar(page, 4).locator('[data-id="dmx/first"]')).toHaveText('First universe');
  // Drag the handle of the last row to the top.
  await expect(rows.locator('.lc-set-bar-name')).toHaveText(['Universe −', 'Universe', 'Universe +', 'First universe']);
  const grip = rows.nth(3).locator('.lc-set-grip');
  const g = await box(grip);
  const top = await box(rows.nth(0));
  await page.mouse.move(g.x + g.width / 2, g.y + g.height / 2);
  await page.mouse.down();
  await page.mouse.move(g.x + g.width / 2, top.y + 10, { steps: 8 });
  await page.mouse.up();
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['first', 'prev', 'universe', 'next']);
  // Load defaults: the type's bar again.
  await set.locator('[data-defaults]').click();
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['universe', 'prev', 'next', 'readout']);
  // The bar off and on.
  await set.locator('[data-setting="window_bar"]').click();
  await expect(bar(page, 4)).toBeHidden();
  await set.locator('[data-setting="window_bar"]').click();
  await expect(bar(page, 4)).toBeVisible();
  await page.keyboard.press('Escape');
});

test('Reset window: from the settings title and from the window menu', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(4), [[4, 'dmx']]);
  for (const l of ['Set Window 1.4 "Readout" Hex', 'Set Window 1.4 "Universe" 3', 'Set Window 1.4 "WindowBarItems" "universe"']) expect((await server.command(l)).ok).toBe(true);
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['universe']);
  const set = await settingsOf(page, 4);
  const mark = server.mark();
  await set.locator('.lc-set-act', { hasText: 'Reset window' }).click();
  await server.waitSysmon(/Reset Window 1\.4 → ok/, mark);
  await expect.poll(async () => (await view(page, 4)).items).toEqual(['universe', 'prev', 'next', 'readout']);
  await expect(set.locator('[data-setting="readout"] .lc-set-value')).toHaveText('Percent');
  await expect.poll(async () => (await stored(page, 4)).universe).toBe(1);
  await page.keyboard.press('Escape');

  expect((await server.command('Set Window 1.4 "Readout" Hex')).ok).toBe(true);
  await expect.poll(async () => (await stored(page, 4)).readout).toBe('hex');
  await page.locator('.ptab[data-win="4"] .tab-dot').click();
  await page.locator('#ctx .ci', { hasText: 'Reset window…' }).click();
  await expect.poll(async () => (await stored(page, 4)).readout).toBe('percent');
});

test('a narrow window puts what does not fit in …', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  const width = await page.locator('#panel-area').evaluate(el => el.clientWidth);
  // The right pane ~330 px: above the Groups bar's minimum (260), too narrow for five items.
  await lay(page, split('h', 1 - 330 / width, leaf(1), leaf(2)), [[1, 'terminal'], [2, 'groups']]);
  const more = bar(page, 2).locator('[data-id="groups/more"]');
  await expect(more).toBeVisible();
  const hidden = await bar(page, 2).locator('.lc-wb-row > [hidden]').count();
  expect(hidden).toBeGreaterThan(0);
  await more.click();
  await expect(page.locator('#ctx .chdr')).toHaveText('Didn’t fit in the bar');
  const row = page.locator('#ctx .ci[data-id="groups/show_empty"]');
  await expect(row).toBeVisible();
  await expect(row.locator('.cbox')).toHaveClass(/\bon\b/);
  // A menu row is the same gesture: Set Self "ShowEmpty" Toggle.
  await row.click();
  await expect.poll(async () => (await stored(page, 2)).show_empty).toBe(false);
  // Wide again: everything fits, no ….
  await lay(page, leaf(2), [[2, 'groups']]);
  await expect(more).toBeHidden();
});

test('a short window hides its bar; the window menu lists the items', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  // A touch screen draws a pane below its type's minimum in miniature instead
  // (decisions §30 "12c2": hold it to open it full screen — kit-12c2.spec).
  test.skip(info.project.name === 'tablet', 'touch: a pane below its minimum is a miniature');
  await open(page);
  const height = await page.locator('#panel-area').evaluate(el => el.clientHeight);
  // The top pane's content below the System Monitor's minimum height (120 px).
  await lay(page, split('v', 130 / height, leaf(3), leaf(1)), [[3, 'sysmon'], [1, 'terminal']]);
  await expect(bar(page, 3)).toBeHidden();
  await page.locator('.ptab[data-win="3"] .tab-dot').click();
  await expect(page.locator('#ctx .chdr', { hasText: 'Bar hidden — its items:' })).toBeVisible();
  for (const id of ['pause', 'wrap', 'level', 'time', 'clear']) await expect(page.locator(`#ctx .ci[data-id="sysmon/${id}"]`)).toBeVisible();
  await expect(page.locator('#ctx .ci', { hasText: 'Settings…' })).toContainText('EditSetting Self');
  await page.locator('#ctx .ci[data-id="sysmon/wrap"]').click();
  await expect.poll(async () => (await stored(page, 3)).wrap).toBe(true);
  // Tall again: the bar is back.
  await lay(page, leaf(3), [[3, 'sysmon']]);
  await expect(bar(page, 3)).toBeVisible();
});

test('WindowPreset: Save preset and Load preset round trip through the station asks', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, leaf(4), [[4, 'dmx']]);
  expect((await server.command('Set Window 1.4 "Readout" Hex')).ok).toBe(true);
  const set = await settingsOf(page, 4);
  await set.locator('.lc-set-act', { hasText: 'Save preset' }).click();
  await expect(dialog(page)).toContainText('Save preset · DMX');
  const name = dialog(page).locator('input[data-field]').first();
  await name.fill('Big DMX');
  await name.press('Enter');
  await expect(set.locator('.lc-set-sub')).toContainText('Big DMX');
  const preset = (await view(page, 4)).preset;
  expect(preset).toMatch(/^[0-9a-f-]{36}$/);

  await set.locator('.lc-set-act', { hasText: 'Reset window' }).click();
  await expect(set.locator('[data-setting="readout"] .lc-set-value')).toHaveText('Percent');
  await set.locator('.lc-set-act', { hasText: 'Load preset' }).click();
  await expect(dialog(page)).toContainText('Load preset · DMX');
  await dialog(page).locator('.lc-dlg-btn', { hasText: 'Load' }).click();
  await expect(set.locator('[data-setting="readout"] .lc-set-value')).toHaveText('Hex');
  await expect.poll(async () => (await view(page, 4)).preset).toBe(preset);
  await expect(set.locator('.lc-set-sub')).toContainText('from preset');
  await page.keyboard.press('Escape');
});

test('EditSetting Self typed on the command line opens the focused window', async ({ page }, info) => {
  test.skip(phone(info), 'desktop-size windows');
  await open(page);
  await lay(page, split('h', 0.5, leaf(3), leaf(4)), [[3, 'sysmon'], [4, 'dmx']]);
  await page.locator('.win-wrap[data-win="4"] .dx-grid').click();
  await page.locator('#cb-line').click();
  await cmd(page, 'EditSetting Self');
  await expect(page.locator('.lc-set-scrim .lc-set-name')).toHaveText(/^Settings · Window 1\.4 · DMX/);
  await page.keyboard.press('Escape');
  await page.locator('.win-wrap[data-win="3"] .sm-list').click();
  await page.locator('#cb-line').click();
  await cmd(page, 'EditSetting Self');
  await expect(page.locator('.lc-set-scrim .lc-set-name')).toHaveText(/^Settings · Window 1\.3 · System Monitor/);
  await page.keyboard.press('Escape');
});

test('the Showroom shows luma-window-bar: items, a chip, … when narrow; a tap is a Self line', async ({ page }) => {
  await open(page);
  await lay(page, leaf(7), [[7, 'showroom']]);
  const room = page.locator('.lc-showroom');
  const bars = room.locator('luma-window-bar');
  await expect(bars).toHaveCount(3);
  await bars.first().scrollIntoViewIfNeeded();
  await expect(bars.nth(0).locator('[data-id="sysmon/search"]')).toHaveText('× Search “act 2”');
  await expect(bars.nth(1).locator('.lc-wb-stepper')).toContainText('Universe:');
  await expect(bars.nth(2).locator('[data-id="sysmon/more"]')).toBeVisible();
  await bars.nth(1).locator('[data-id="dmx/next"]').click();
  await expect(room.locator('.lc-sr-note', { hasText: '›' })).toHaveText('› Set Self "Universe" + 1');
  await expect(bars.nth(1).locator('[data-id="dmx/universe"]')).toHaveText('2');
});
