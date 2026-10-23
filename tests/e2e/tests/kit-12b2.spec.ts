// Step 12b2 (decisions §30 "12b2 hardening", the 12b2 boards): the Meow key
// held lights the sidebar button; F12 full screen hides every bar, typing
// floats the command line in, the corner handle brings the bars back; text
// is selectable only where you read or type; the Menu's tiles carry icons and
// open Users & Profiles, whose profile settings are typed tabs.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode, box as boxOf } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page, '/', 'Admin');
  await connected(page);
  await kbdMode(page);
}

test("holding the Meow key (Shift) lights the top bar's Meow", async ({ page, isMobile }) => {
  test.skip(isMobile, 'a hardware keyboard');
  await open(page);
  const meow = page.locator('#tb-strip .lc-ss-meow');
  // The profile's key settings are loaded (GET /api/ui-keys) before the key goes down.
  await expect.poll(() => page.evaluate(() => (0, eval)('uiKeys.defs.size'))).toBeGreaterThan(0);
  await page.keyboard.down('Shift');
  await expect(meow).toHaveClass(/held/);
  await page.keyboard.up('Shift');
  await expect(meow).not.toHaveClass(/held/);
});

test('F12 full screen: every bar hidden, typing floats the line in, the corner handle comes back', async ({ page, isMobile }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Off Sidebars'));
  await expect(page.locator('body')).toHaveClass(/fullwin/);
  for (const id of ['#topbar', '#cmdbar', '#sidebar-left', '#sidebar-right']) await expect(page.locator(id)).toBeHidden();
  const handle = page.locator('#fs-handle');
  await expect(handle).toBeVisible();
  if (!isMobile) {
    await page.locator('#panel-area').click({ position: { x: 5, y: 5 } });
    await page.keyboard.type('Cl');
    await expect(page.locator('#cmdbar')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.locator('#cmdbar')).toBeHidden();
  }
  await handle.click();
  await expect(page.locator('body')).not.toHaveClass(/fullwin/);
  await expect(page.locator('#cmdbar')).toBeVisible();
});

test('dragging over a pool selects no text; the System Monitor is selectable', async ({ page, isMobile }) => {
  test.skip(isMobile, 'a mouse drag');
  await open(page);
  if (!(await page.locator('luma-pool').count())) await page.evaluate(() => (window as any).wsCommand('Store Window "Groups"'));
  const pool = page.locator('luma-pool .lc-pool-scroll').first();
  await expect(pool).toBeVisible();
  const box = await boxOf(pool);
  await page.mouse.move(box.x + 10, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 10, box.y + box.height / 2, { steps: 8 });
  await page.mouse.up();
  expect(await page.evaluate(() => getSelection()?.toString() ?? '')).toBe('');
  const userSelect = (sel: string) => page.evaluate(s => {
    const el = document.querySelector(s);
    return el ? getComputedStyle(el).userSelect || (getComputedStyle(el) as any).webkitUserSelect : null;
  }, sel);
  expect(await userSelect('luma-pool .lc-pool-scroll')).toBe('none');
  // System Monitor rows are where you read (xterm selects text itself).
  if (!(await page.locator('.sysmon').count())) await page.evaluate(() => (window as any).wsCommand('Store Window "System Monitor"'));
  await expect(page.locator('.sm-row').first()).toBeVisible();
  expect(await userSelect('.sm-row')).toBe('text');
});

test('the Menu has icons; Users & Profiles shows users, profiles and a profile\'s typed tabs', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).openMenu());
  const tiles = page.locator('#menu-tiles .lc-tile');
  expect(await tiles.count()).toBeGreaterThan(10);
  expect(await tiles.locator('luma-icon').count()).toBe(await tiles.count());
  await tiles.filter({ hasText: 'Users' }).click();
  // Users & Profiles is the setup frame (12d part 4): a Users grid and a
  // Profiles grid; a profile's settings open with its typed tabs.
  const setup = page.locator('luma-setup');
  await expect(setup.locator('luma-grid [data-col="name"]', { hasText: 'Admin' }).first()).toBeVisible();
  await setup.locator('.lc-setup-tabs [data-tab="Profiles"]').click();
  const admin = (await (await fetch(`${server.url}/api/station`)).json()).profiles.find((p: any) => p.name === 'Admin');
  await setup.locator(`luma-grid [data-gutter="${admin.guid}"]`).click();
  await setup.locator('luma-slab [data-own][data-id="settings"]').filter({ visible: true }).click();
  const sw = page.locator('.lc-set-scrim');
  await expect(sw.locator('.lc-set-tabs button')).toHaveText(['Look', 'Input & keys', 'Oops', 'Command line', 'Update', 'Attributes', 'MVR import', 'Paint slots', 'Lock screen', 'Meta', 'Tags']);
  await sw.locator('.lc-set-tabs button', { hasText: 'Input & keys' }).click();
  const meow = sw.locator('.lc-set-cell', { hasText: 'MeowKey' });
  await meow.click();
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: 'Ctrl' }).click();
  await expect(meow.locator('.lc-set-value')).toHaveText('Ctrl');
  await expect.poll(async () => (await server.command('EditSetting UserProfile 2')).message).toMatch(/MeowKey\s+Ctrl/);
  await page.evaluate(() => (window as any).wsCommand('Set UserProfile 2 "MeowKey" Shift'));
});
