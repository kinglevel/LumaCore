// The Tree window (decisions 0292, the approved mockup docs/dev/ui-kit/mockups/2026-10-23-tree-window):
// NAV's tree and inspector as a window of the page, on the rig. <luma-tree-window> on the `tree`
// and `inspector` feeds: the path bar's crumbs and its field (a path, a GUID prefix, a gone GUID's
// toast), a fold is `Set Self "Open"`, the inspector's copy buttons through the one clipboard door,
// a value changed through its settings cell (`Set Group 1 "Mode" …`) and ↺ (`Reset …`), a Used by
// link that jumps, Follow = Selected moved by a `Group 3` line, the row menu (the copies, the
// terminal's NAV, the registry's menu), the phone's sheet and the Showroom's pieces. A kit file:
// it runs on desktop, tablet, phone and WebKit.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });

const G1 = 'Root.ShowData.DataPools.1.Groups.1';
const isPhone = (name: string) => name === 'phone';

async function open(page: Page): Promise<Locator> {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  // One Tree window for the whole file (the tests share one server and its layout).
  const has = await page.evaluate(() => Object.values((window as any).eval('layout').windows).some((w: any) => w.kind === 'tree'));
  if (!has) await page.evaluate(() => (window as any).wsCommand('Store Window "Tree"'));
  const tab = page.locator('.ptab', { hasText: 'Tree' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const tw = page.locator('luma-tree-window').first();
  await expect(tw.locator('.lc-tree-row').first()).toBeVisible();
  return tw;
}
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/tree-window-${name}.png`) });
const row = (tw: Locator, p: string) => tw.locator(`.lc-tree-row[data-path="${p}"]`);
const insp = (tw: Locator) => tw.locator('.lc-tw-insp luma-inspector');
const toast = (page: Page) => page.locator('.lc-toast.on');

/** Type `text` into the path bar and go. */
async function goTo(page: Page, tw: Locator, text: string) {
  await tw.locator('luma-pathbar [data-edit]').click();
  const field = tw.locator('luma-pathbar input');
  await field.fill(text);
  await field.press('Enter');
}

test('the window opens on the rig; a fold opens a branch by a line', async ({ page }) => {
  const tw = await open(page);
  await expect(tw.locator('luma-pathbar .lc-pathbar-crumb')).toHaveText(['Root', 'ShowData']);
  await expect(row(tw, 'Root.ShowData.DataPools')).toBeVisible();
  const from = server.mark();
  await row(tw, 'Root.ShowData.DataPools').locator('.lc-tree-fold').click();
  await server.waitSysmon(/Set Window [\d.]+ "Open" "Root\.ShowData,Root\.ShowData\.DataPools"/, from);
  await expect(row(tw, 'Root.ShowData.DataPools.1')).toBeVisible();
  await shot(page, 'open');
});

test('the path bar goes to Group 1; the GUID copies to the clipboard', async ({ page, browserName }, info) => {
  test.skip(isPhone(info.project.name), 'a phone shows the inspector as a sheet (its own test)');
  if (browserName === 'chromium') await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: server.url });
  const tw = await open(page);
  const from = server.mark();
  await goTo(page, tw, G1);
  await server.waitSysmon(/Set Window [\d.]+ "Path" "Root\.ShowData\.DataPools\.1\.Groups\.1"/, from);
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
  await expect(row(tw, G1)).toHaveClass(/\bsel\b/);
  await expect(tw.locator('luma-pathbar .lc-pathbar-crumb').last()).toHaveText('1');
  const guid = (await insp(tw).locator('[data-row="GUID"] .lc-insp-v').textContent())!.trim();
  expect(guid).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-/);
  await insp(tw).locator('[data-row="GUID"] [data-copy]').click();
  await expect(toast(page)).toHaveText(/Copied GUID/);
  if (browserName === 'chromium') expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(guid);
  await shot(page, 'group1');
});

test('a value changes through its settings cell, ↺ resets it', async ({ page }, info) => {
  test.skip(isPhone(info.project.name), 'a phone shows the inspector as a sheet (its own test)');
  const tw = await open(page);
  await goTo(page, tw, G1);
  const mode = insp(tw).locator('.lc-set-cell[data-setting="Mode"]');
  await expect(mode).toBeVisible();
  let from = server.mark();
  await mode.locator('.lc-set-tap').click();
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: 'Additive' }).click();
  await server.waitSysmon(/Set Group 1 "Mode" Additive/, from);
  await expect(mode.locator('.lc-set-value')).toHaveText('Additive');
  from = server.mark();
  await insp(tw).locator('.lc-set-cell[data-setting="Mode"] [data-reset]').click();
  await server.waitSysmon(/Reset Group 1 "Mode"/, from);
  await expect(insp(tw).locator('.lc-set-cell[data-setting="Mode"] .lc-set-value')).not.toHaveText('Additive');
});

test('a link jumps; a GUID prefix finds its object; a gone GUID says so', async ({ page }, info) => {
  test.skip(isPhone(info.project.name), 'a phone shows the inspector as a sheet (its own test)');
  const tw = await open(page);
  await goTo(page, tw, G1);
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
  const guid = (await insp(tw).locator('[data-row="GUID"] .lc-insp-v').textContent())!.trim();
  // Links to → a fixture; its Used by → back to Group 1.
  await insp(tw).locator('.lc-insp-link[data-go^="Root.ShowData.Devices.Dmx.Patch.Fixtures."]').first().click();
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Fixture \d+/);
  await insp(tw).locator(`.lc-insp-link[data-go="${G1}"]`).click();
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
  // Elsewhere, then the first 8 digits of Group 1's GUID.
  await tw.locator('luma-pathbar .lc-pathbar-crumb', { hasText: 'ShowData' }).click();
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText('ShowData');
  await goTo(page, tw, guid.slice(0, 8));
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
  // A GUID nothing has: the kit's toast, the window stays.
  await goTo(page, tw, 'deadbeef');
  await expect(toast(page)).toHaveText('No object with GUID deadbeef in this show');
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
});

test('Follow = Selected: a Group 3 line moves the window', async ({ page }, info) => {
  test.skip(isPhone(info.project.name), 'a phone shows the inspector as a sheet (its own test)');
  const tw = await open(page);
  await goTo(page, tw, G1);
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 1 "/);
  await page.locator('#cb-line').click();
  await cmd(page, 'Group 3');
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Group 3 "/);
  await expect(row(tw, 'Root.ShowData.DataPools.1.Groups.3')).toHaveClass(/\bsel\b/);
  await cmd(page, 'Fixture 101');
  await expect(insp(tw).locator('.lc-insp-name')).toHaveText(/^Fixture 101/);
  await cmd(page, 'Clear');
});

test('the row menu: the copies, the terminal\'s NAV, the registry\'s menu', async ({ page }) => {
  const tw = await open(page);
  await goTo(page, tw, G1);
  await expect(row(tw, 'Root.ShowData.DataPools.1.Groups.2')).toBeVisible();
  await row(tw, 'Root.ShowData.DataPools.1.Groups.2').locator('.lc-tree-name').click({ button: 'right' });
  const menu = page.locator('.lc-cm');
  await expect(menu).toBeVisible();
  for (const label of ['Copy GUID', 'Copy path', 'Copy as target', 'Copy Set line', 'Open in the terminal\'s NAV', 'Bookmark…']) {
    await expect(menu.locator('.lc-cm-item', { hasText: label })).toBeVisible();
  }
  await expect(menu.locator('.lc-cm-item', { hasText: 'Copy as target' })).toContainText('Group 2');
  // The registry's own entries follow (Label…, Delete, the Set section).
  await expect(menu.locator('.lc-cm-item', { hasText: /^\s*Delete/ })).toBeVisible();
  await shot(page, 'menu');
  await menu.locator('.lc-cm-item', { hasText: 'Copy as target' }).click();
  await expect(toast(page)).toHaveText(/Copied target/);
  // Open in the terminal's NAV: cd to its folder, then Shortcuts Navigator — two lines.
  await row(tw, 'Root.ShowData.DataPools.1.Groups.2').locator('.lc-tree-name').click({ button: 'right' });
  const from = server.mark();
  await page.locator('.lc-cm .lc-cm-item', { hasText: 'Open in the terminal\'s NAV' }).click();
  await server.waitSysmon(/cd Root\.ShowData\.DataPools\.1\.Groups/, from);
  await server.waitSysmon(/Shortcuts Navigator/, from);
  await expect(page.locator('#cb-mode .cb-text')).toHaveText('NAV');
  await cmd(page, 'Shortcuts Keyboard');
});

test('a phone: the tree fills the pane, a tap opens the inspector as the sheet', async ({ page }, info) => {
  test.skip(!isPhone(info.project.name), 'the phone\'s sheet');
  const tw = await open(page);
  await expect(tw.locator('.lc-tw-insp')).toBeHidden();
  const g = row(tw, 'Root.ShowData.DataPools');
  await g.locator('.lc-tree-name').click();
  const sheet = page.locator('.lc-tw-sheet');
  await expect(sheet).toBeVisible();
  await expect(sheet.locator('luma-inspector .lc-insp-name')).toHaveText('DataPools');
  await shot(page, 'phone-sheet');
});

test('the Showroom shows the path bar and the Tree window', async ({ page }) => {
  await server.open(page);
  await connected(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const tab = page.locator('.ptab', { hasText: 'Showroom' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const sr = page.locator('[data-kit="showroom"] .lc-sr-tree-window');
  await sr.scrollIntoViewIfNeeded();
  await expect(sr.locator('.lc-tree-row[aria-selected="true"]')).toContainText('Photon');
  await expect(page.locator('[data-kit="showroom"] .lc-sr-pathbar .lc-pathbar-crumb')).toHaveCount(6);
  await shot(page, 'showroom');
});
