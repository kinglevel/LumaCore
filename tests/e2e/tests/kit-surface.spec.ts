// The lock / login surface (decisions §30 "12a3", boards Lock / Surface /
// Screens / Overview), on the desktop, a touch tablet, a touch phone and
// WebKit: the countdown logs the last user in, DeskLock follows the device
// (a reload stays locked), unlocking needs the password (Admin: admin) or the
// corner taps, the built-in keyboard types into the field, Meow+F9 shows the
// normal theme, F12 hides the sidebars, and the overview lists the screens.

import { expect, test, type Page } from '@playwright/test';
import { Server, connected } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start();
  for (const line of ['Store User "LD" /Rights Program', 'Set UserProfile "LD" Theme "Base" Night']) {
    const r = await server.command(line);
    expect(r.ok, r.message).toBe(true);
  }
});
test.afterAll(async () => { await server.cleanup(); });

const surface = (page: Page) => page.locator('luma-surface');
const mode = (page: Page) => page.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode ?? null);
const ws = (page: Page, line: string) => page.evaluate(l => (window as any).wsCommand(l), line);

/** A tab whose last user here was `user` (the browser remembers it per station). */
async function lastUser(page: Page, user: string) {
  await page.goto(server.url + '/kit/tokens.css');
  await page.evaluate(u => { localStorage.clear(); localStorage.setItem('lumacore.lastUser', u); }, user);
}

test('the last user without a password logs in after the countdown; Esc stops it', async ({ page }) => {
  await lastUser(page, 'LD');
  await page.goto(server.url);
  await expect(surface(page)).toBeVisible();
  await expect(page.locator('luma-surface .lc-sf-note')).toContainText(/Logging in in \d s/);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(1500);
  expect(await mode(page)).toBe('login');
  // Tap the user: at once.
  await page.locator('luma-surface luma-avatar.big button').click();
  await page.locator('luma-surface .lc-sf-pill.go').click();
  await connected(page);
  await expect(surface(page)).toHaveCount(0);
});

test('?autologin=now skips the wait; Admin always asks for its password', async ({ page }) => {
  await lastUser(page, 'LD');
  await page.goto(server.url + '/?autologin=now');
  await connected(page);
  await lastUser(page, 'Admin');
  await page.context().clearCookies();
  await page.goto(server.url + '/?autologin=now');
  await expect(page.locator('luma-surface .lc-sf-pw')).toBeVisible();
  await page.locator('luma-surface .lc-sf-pw').fill('nope');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await expect(page.locator('luma-surface .lc-sf-error')).toContainText(/Wrong password for Admin/);
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await connected(page);
});

test('DeskLock locks this device, survives a reload, unlocks only with the password', async ({ page }) => {
  await server.open(page);
  await connected(page);
  await ws(page, 'DeskLock');
  await expect.poll(() => mode(page)).toBe('locked');
  await page.reload();
  await expect.poll(() => mode(page), { timeout: 10_000 }).toBe('locked');
  // F9 / a tap don't unlock Admin: the password does.
  await page.locator('luma-surface .lc-sf-pw').press('F9');
  await expect.poll(() => mode(page)).toBe('locked');
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await expect(surface(page)).toHaveCount(0);
});

test('the four corner taps unlock a user without a password', async ({ page }) => {
  await server.open(page, '/', 'LD');
  await connected(page);
  await ws(page, 'On DeskLock');
  await expect.poll(() => mode(page)).toBe('locked');
  const box = (await surface(page).boundingBox())!;
  for (const [x, y] of [[5, 5], [box.width - 5, box.height - 5], [box.width - 5, 5], [5, box.height - 5]]) {
    await page.mouse.click(box.x + x, box.y + y);
  }
  await expect(surface(page)).toHaveCount(0);
});

test('the built-in keyboard types into the password field', async ({ page }) => {
  await server.open(page);
  await connected(page);
  await ws(page, 'DeskLock');
  await expect.poll(() => mode(page)).toBe('locked');
  await page.locator('luma-surface button[aria-label="Keyboard"]').click();
  const kb = page.locator('luma-surface luma-keyboard');
  await expect(kb).toBeVisible();
  await expect(kb.locator('.lc-kb-words')).toHaveCount(0); // password mode: no command words
  for (const ch of 'admin') await kb.locator(`button[data-act="ch:${ch}"]`).click();
  await expect(page.locator('luma-surface .lc-sf-pw')).toHaveValue('admin');
  await kb.locator('button[data-act="please"]').click();
  await expect(surface(page)).toHaveCount(0);
});

test('Meow+F9 shows the normal theme (factory Dark) instead of the profile\'s Night', async ({ page }) => {
  await server.open(page, '/', 'LD');
  await connected(page);
  const preset = () => page.evaluate(() => (window as any).LumaKit.theme.preset());
  await expect.poll(preset).toBe('Night');
  await page.locator('#cb-line').click();
  await page.keyboard.press('Shift+F9');
  await expect.poll(preset).toBe('Dark');
  await page.keyboard.press('Shift+F9');
  await expect.poll(preset).toBe('Night');
});

test('F12 full screen of the windows and back; the Lock tile locks', async ({ page }) => {
  await server.open(page);
  await connected(page);
  const full = () => page.evaluate('!!sidebars.full') as Promise<boolean>;
  await page.locator('#cb-line').click();
  await page.keyboard.press('F12');
  await expect.poll(full).toBe(true);
  await expect(page.locator('#cmdbar')).toBeHidden();
  await page.keyboard.press('F12');
  await expect.poll(full).toBe(false);
  const tile = page.locator('#sidebar-left [data-key="desklock"]');
  if (await tile.isVisible()) {
    await tile.click();
    await expect.poll(() => mode(page)).toBe('locked');
    await page.locator('luma-surface .lc-sf-pw').fill('admin');
    await page.locator('luma-surface .lc-sf-pw').press('Enter');
    await expect(surface(page)).toHaveCount(0);
  }
});

test('the screens overview lists the screens and switches to tiled mode', async ({ page }) => {
  await server.open(page, '/?overview');
  const ov = page.locator('luma-overview');
  await expect(ov).toBeVisible();
  await expect(ov.locator('.lc-ov-row')).not.toHaveCount(0);
  await ov.locator('.lc-ov-tab[data-mode="tiled"]').click();
  await expect(ov.locator('.lc-ov-tile iframe').first()).toBeVisible();
});
