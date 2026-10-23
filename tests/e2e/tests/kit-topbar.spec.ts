// The top bar (decisions top-bar-status-strip): <luma-status-strip> — the mark with the station,
// the Show tab, the facts, the modes as UI keys (a tap toggles, the key lights), Oops with its
// step count, the clock; the six keys gone from the left sidebar; a tap on the mark opens the
// station's details; the Showroom's strips.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
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
  return page.locator('#tb-strip');
}
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/topbar-${name}.png`) });

test('the strip shows the mark, the show and the modes; the sidebar lost the six keys', async ({ page }) => {
  const strip = await open(page);
  await expect(strip.locator('.lc-ss-mark')).toHaveAttribute('data-state', 'ok');
  await expect(strip.locator('.lc-ss-name')).not.toHaveText('');
  for (const key of ['blind', 'highlight', 'freeze', 'solo', 'oops']) {
    await expect(strip.locator(`[data-key="${key}"]`)).toBeVisible();
    await expect(page.locator(`#sidebar-left [data-key="${key}"]`)).toHaveCount(0);
  }
  await expect(strip.locator('.lc-ss-meow')).toBeVisible();
  await expect(page.locator('#cb-net')).toHaveCount(0);
  await expect(strip.locator('.lc-ss-clock')).toHaveText(/^\d\d:\d\d:\d\d$/);
  await shot(page, 'strip');
});

test('a tap on BLIND toggles Blind and lights it; Oops counts the steps', async ({ page }) => {
  const strip = await open(page);
  const blind = strip.locator('[data-key="blind"]');
  await blind.click();
  await expect(blind).toHaveClass(/\bon\b/);
  await blind.click();
  await expect(blind).not.toHaveClass(/\bon\b/);
  await cmd(page, 'Fixture 101');
  await cmd(page, 'At 40');
  await expect(strip.locator('.lc-ss-oops .n')).toHaveText(/^\d+$/);
  await expect(strip.locator('.lc-ss-oops')).toHaveClass(/\bon\b/);
});

test('a tap on the mark opens the station\'s details', async ({ page }) => {
  const strip = await open(page);
  await strip.locator('.lc-ss-mark').click();
  const menu = page.locator('#ctx');
  await expect(menu).toContainText('LumaSession');
  await expect(menu).toContainText('LumaSessions…');
  await expect(menu).toBeVisible();
  await shot(page, 'mark');
  await page.keyboard.press('Escape');
});

test('the Showroom shows the strip in its four mark states', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const strips = page.locator('.lc-sr-strips luma-status-strip');
  await expect(strips).toHaveCount(4);
  await expect(strips.nth(1).locator('.lc-ss-mark')).toHaveAttribute('data-state', 'member');
  await expect(strips.nth(3).locator('.lc-ss-mark')).toHaveAttribute('data-state', 'lost');
});
