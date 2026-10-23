// The Device Sheet (decisions device-sheet-window, device-grid-and-value-marks): <luma-device-grid>
// on the `devices` feed, on the rig. The folders are tree rows and the attributes sit under their
// feature groups; a programmer value is the programmer's ground with its bar, its row carries the
// name bar; a tap on a name selects (a line), a second tap on a value opens the keypad and Enter
// runs `Fixture <n> Attribute "<a>" At <v>`; Channel mode draws tiles; the Showroom shows the marks.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  // One Device Sheet for the whole file (the tests share one server and its layout).
  const has = await page.evaluate(() => Object.values((window as any).eval('layout').windows).some((w: any) => w.kind === 'devices'));
  if (!has) await page.evaluate(() => (window as any).wsCommand('Store Window "Device Sheet"'));
  await page.locator('.ptab', { hasText: 'Device Sheet' }).first().click();
  const grid = page.locator('luma-device-grid').first();
  await expect(grid).toBeVisible();
  return grid;
}
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/devices-${name}.png`) });
/** The sheet's window bar item `id` (a checkbox): set it to `on` with one tap when it differs. */
async function check(page: Page, grid: Locator, id: string, on: boolean) {
  const item = grid.locator('xpath=ancestor::*[contains(@class,"win-wrap")][1]').locator(`:scope > luma-window-bar [data-id="devices/${id}"]`);
  if ((await item.getAttribute('aria-pressed')) !== String(on)) await item.click();
  await expect(item).toHaveAttribute('aria-pressed', String(on));
}
/** A line typed for the sheet (its window focused, then the command line). */
async function selfLine(page: Page, grid: Locator, line: string) {
  await grid.locator('.lc-dg-groups:visible, .lc-dg-tsep:visible').first().click();
  await page.locator('#cb-line').click();
  await cmd(page, line);
}

test('the sheet opens on the rig: folders, feature groups, values', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'a phone draws Channel mode (decisions device-sheet-window)');
  const grid = await open(page);
  await expect(grid.locator('.lc-dg-name.folder').first()).toBeVisible();
  await expect(grid.locator('.lc-dg-group', { hasText: /dimmer/i }).first()).toBeVisible();
  await expect(grid.locator('.lc-gc.lc-v[data-s="def"]').first()).toBeVisible();
  await shot(page, 'open');
});

test('a programmer value is painted as yours, its row carries the bar', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'a phone draws Channel mode (decisions device-sheet-window)');
  const grid = await open(page);
  await cmd(page, 'Fixture 101');
  await cmd(page, 'At 50');
  // The window bar's checkbox: one Set Self line.
  await check(page, grid, 'programmer_only', true);
  const name = grid.locator('.lc-dg-name:not(.folder)');
  await expect(name).toHaveCount(1);
  await expect(name.locator('.lc-v-bars i[data-l="abs"]')).toBeVisible();
  const dim = grid.locator('.lc-gc.lc-v[data-col="Dimmer"][data-s="prgA"]');
  await expect(dim).toHaveText('50');
  await expect(dim.locator('.lc-v-bars i[data-l="abs"]')).toBeVisible();
  await shot(page, 'programmer');
});

test('a second tap on a value opens the keypad and Enter sets it', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'a phone draws Channel mode (decisions device-sheet-window)');
  const grid = await open(page);
  await check(page, grid, 'programmer_only', true);
  const dim = grid.locator('.lc-gc.lc-v[data-col="Dimmer"]').last();
  await dim.click();
  await dim.click();
  const pad = page.locator('luma-keypad');
  await expect(pad).toBeVisible();
  const from = server.mark();
  await page.keyboard.type('80');
  await page.keyboard.press('Enter');
  await server.waitSysmon(/Fixture 101 Attribute "Dimmer" At 80/, from);
  await expect(grid.locator('.lc-gc.lc-v[data-col="Dimmer"][data-s="prgA"]').last()).toHaveText('80');
});

test('a tap on a name selects and deselects by lines', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'a phone draws Channel mode (decisions device-sheet-window)');
  const grid = await open(page);
  await check(page, grid, 'programmer_only', false);
  const row = grid.locator('.lc-dg-name:not(.folder)').nth(2);
  let from = server.mark();
  await row.click();
  const sent = await server.waitSysmon(/Fixture \d+/, from);
  expect(sent).toBeTruthy();
  await expect(grid.locator('.lc-grid-row.whole').first()).toBeVisible();
  from = server.mark();
  await grid.locator('.lc-grid-row.whole .lc-dg-name').first().click();
  await server.waitSysmon(/- Fixture \d+/, from);
});

test('Channel mode draws a tile per device', async ({ page }) => {
  const grid = await open(page);
  await selfLine(page, grid, 'Set Self "Mode" Channel');
  await expect(grid.locator('.lc-dg-tile').first()).toBeVisible();
  await expect(grid.locator('.lc-dg-tsep').first()).toBeVisible();
  await expect(grid.locator('.lc-dg-tile .lc-v-t').first()).toHaveText(/\d/);
  await expect(grid.locator('.lc-grid')).toBeHidden();
  await shot(page, 'channel');
  await selfLine(page, grid, 'Set Self "Mode" Fixture');
});

test('the Showroom shows the value marks and the device grid', async ({ page }) => {
  await server.open(page);
  await connected(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const sr = page.locator('.lc-sr-marks').first();
  await sr.scrollIntoViewIfNeeded();
  await expect(sr.locator('.lc-v[data-s="prgA"]').first()).toBeVisible();
  // A phone draws the Showroom's grid as tiles too.
  await expect(sr.locator('luma-device-grid .lc-gc.lc-v[data-s="track"], luma-device-grid .lc-dg-tile').first()).toBeVisible();
  await shot(page, 'showroom');
});
