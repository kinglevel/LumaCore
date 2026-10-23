// Browser mode: the server's own web UI. Logging in and out every way the
// interface allows — on the lock / login surface (decisions §30 "12a3":
// Admin's password is always `admin`) and on the command line — and what
// sysmon reports for each (server output and a live monitor client, like
// `lumacore-cli --monitor`).

import { expect, test, type Page } from '@playwright/test';
import { Monitor, Server, cmd, connected, consentDialog, giveConsent, kbdMode, lanAddress, promptUser, screenText, dialog, wsMessages } from '../support/server';

const server = new Server();
let monitor: Monitor;

test.describe.configure({ mode: 'serial' });

test.beforeAll(async () => {
  await server.start();
  monitor = await Monitor.connect(server);
});
test.afterAll(async () => {
  monitor?.close();
  await server.cleanup();
});

async function open(page: Page, url = server.url) {
  await page.goto(url);
  // A fresh tab shows the surface: Admin always asks for its password.
  if (!(await page.locator('#tb-strip .lc-ss-mark:is([data-state="ok"], [data-state="member"], [data-state="busy"], [data-state="idle"])').count())) {
    await surface(page);
    const pw = page.locator('luma-surface .lc-sf-pw');
    if (await pw.isVisible()) { await pw.fill('admin'); await pw.press('Enter'); }
    else await loginAs(page, 'Admin', 'admin');
  }
  await connected(page);
  await kbdMode(page);
}

async function surface(page: Page) {
  try {
    await expect(page.locator('luma-surface')).toBeVisible();
  } catch (e) {
    const screen = (await screenText(page)).split('\n').filter(l => l.trim()).slice(-15).join('\n');
    throw new Error(`${(e as Error).message}\n--- screen ---\n${screen}\n--- server ---\n${server.lines.slice(-15).join('\n')}`);
  }
}

async function loginAs(page: Page, user: string, password?: string) {
  await surface(page);
  await page.locator(`luma-surface luma-avatar[data-user="${user}"] button`).click();
  if (password !== undefined) {
    await page.locator('luma-surface .lc-sf-pw').fill(password);
    await page.locator('luma-surface .lc-sf-pw').press('Enter');
  } else {
    await page.locator('luma-surface .lc-sf-pill.go').click();
  }
}

test('logging in as Admin on the surface: sysmon shows both steps', async ({ page }) => {
  const from = server.mark();
  await open(page);
  await promptUser(page, 'Admin');
  await server.waitSysmon(/Session : login 'Admin' from 127\.0\.0\.1 \(HTTP, local\)/, from);
  await server.waitSysmon(/shell login accepted for 'Admin' from 127\.0\.0\.1 \(web\)/, from);
  // The same lines reach a monitor client live.
  await monitor.wait(/login 'Admin' from 127\.0\.0\.1 \(HTTP, local\)/);
  await monitor.wait(/shell login accepted for 'Admin' from 127\.0\.0\.1 \(web\)/);
});

test('typed commands create a user with a password', async ({ page }) => {
  await open(page);
  await cmd(page, 'Store User "LD" /Password "secret"');
  await cmd(page, 'List User');
  await expect.poll(() => screenText(page)).toContain('LD');
  // The password never shows: not in the echo, not in the history.
  expect(await screenText(page)).not.toContain('secret');
});

test('Logout shows the surface; a wrong password is refused and logged', async ({ page }) => {
  await open(page);
  await cmd(page, 'Logout');
  await surface(page);
  await expect(page.locator('luma-surface luma-avatar[data-user="LD"]')).toHaveCount(1);

  const from = server.mark();
  await loginAs(page, 'LD', 'nope');
  await expect(page.locator('luma-surface .lc-sf-error')).toContainText(/wrong password/i);
  await server.waitSysmon(/login 'LD' refused from 127\.0\.0\.1 \(HTTP\): Wrong password/, from);
  await monitor.wait(/login 'LD' refused/);

  await page.locator('luma-surface .lc-sf-pw').fill('secret');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await connected(page);
  await promptUser(page, 'LD');
  await server.waitSysmon(/login 'LD' from 127\.0\.0\.1 \(HTTP, local\)/, from);
  await server.waitSysmon(/shell login accepted for 'LD' from 127\.0\.0\.1 \(web\)/, from);
});

test('every login asks for the Meow License: Not me goes back, I consent finishes it (decisions 0290)', async ({ page }) => {
  await page.goto(server.url);
  await surface(page);
  await loginAs(page, 'Debug');
  // One station dialog over the surface; the login is not finished yet.
  const dlg = consentDialog(page);
  await expect(dlg).toBeVisible();
  await expect(page.locator('luma-surface')).toBeVisible();
  await expect(dlg.locator('.lc-dlg-text')).toContainText('Share all your good vibes and purrs with LumaCore');
  await expect(dlg.locator('.lc-dlg-details .tone-alert')).toContainText('Fully vibe-coded software.');
  await expect(dlg.locator('.lc-dlg-btn')).toHaveText(['I consent — let the meow in', 'Not me']);
  await expect(dlg.locator('.lc-dlg-note')).toHaveText('(LumaCore accepts no liability for spontaneous keyboard naps.)');
  // Its token opens nothing yet: the page's REST sees no login (the app's screens wait too).
  const me = await page.request.get(`${server.url}/api/me`);
  expect(me.status()).toBe(401);
  expect(await me.json()).toMatchObject({ consent: true, user: 'Debug', code: 'E-USER-010' });

  // Not me: logged out, back on the surface.
  await dlg.locator('.lc-dlg-btn', { hasText: 'Not me' }).click();
  await expect(dlg).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode ?? null)).toBe('login');
  expect((await page.request.get(`${server.url}/api/me`)).status()).toBe(401);

  // Again, and I consent: the desk, and the station log says so.
  const from = server.mark();
  await loginAs(page, 'Debug');
  await giveConsent(page);
  await connected(page);
  await expect(page.locator('luma-surface')).toHaveCount(0);
  await promptUser(page, 'Debug');
  await server.waitSysmon(/Debug consented to the Meow License/, from);
  expect((await page.request.get(`${server.url}/api/me`)).status()).toBe(200);
});

test('Login on the command line switches user (with and without a password prompt)', async ({ page }) => {
  await open(page);
  await promptUser(page, 'Admin');

  let from = server.mark();
  await cmd(page, 'Login "LD" "secret"');
  await promptUser(page, 'LD');
  await server.waitSysmon(/logged in as LD/, from);

  from = server.mark();
  await cmd(page, 'Login "Admin" "admin"');
  await promptUser(page, 'Admin');
  await server.waitSysmon(/logged in as Admin/, from);

  // No password given for LD: the station's dialog asks in a hidden field.
  await cmd(page, 'Login "LD"');
  const pw = dialog(page).locator('input[data-field]');
  await expect(pw).toHaveAttribute('type', 'password');
  await pw.fill('secret');
  await pw.press('Enter');
  await promptUser(page, 'LD');
  expect(await screenText(page)).not.toContain('secret');
});

test('Logout works for Debug (the surface, nobody picked, no silent reconnect)', async ({ page }) => {
  await open(page);
  await cmd(page, 'Logout');
  await loginAs(page, 'Debug');
  await connected(page);
  await promptUser(page, 'Debug');

  const from = server.mark();
  await cmd(page, 'Logout');
  await surface(page);
  await expect(page.locator('luma-surface .lc-sf-ask')).toHaveText(/Who's working here/);
  // Still logged out a moment later: the page must not reconnect as Debug.
  await page.waitForTimeout(3000);
  await surface(page);
  expect(server.sysmonSince(/shell login accepted for 'Debug'/, from)).toEqual([]);
});

test('Logout works right after an in-session Login as Debug', async ({ page }) => {
  await open(page);
  await cmd(page, 'Login "Debug"');
  await promptUser(page, 'Debug');
  await cmd(page, 'Logout');
  await surface(page);
  await page.waitForTimeout(3000);
  await surface(page);
});

test('the Shutdown menu Logout and Disconnect tiles log out', async ({ page }) => {
  for (const tile of ['Logout', 'Disconnect']) {
    await open(page);
    // Meow+F11 (decisions §30 "12a3": core keys are F keys, never letters).
    await page.keyboard.press('Shift+F11');
    await expect(page.locator('#power-overlay')).toHaveClass(/\bopen\b/);
    await page.locator('#power-overlay .lc-tile', { hasText: tile }).click();
    await surface(page);
  }
});

test('a shell with a bad token is refused and logged', async ({ page }) => {
  const from = server.mark();
  const [message] = await wsMessages(page, `ws://127.0.0.1:${server.http}/ws/shell?token=v1.bogus`, null, got => got.length > 0);
  expect(message.type).toBe('error');
  await server.waitSysmon(/shell login refused from 127\.0\.0\.1 \(web\)/, from);
});

test('a login from another address is remote and logged with that address', async ({ page }) => {
  const lan = lanAddress();
  test.skip(!lan, 'no LAN address on this machine');
  const from = server.mark();
  await open(page, `http://${lan}:${server.http}`);
  await server.waitSysmon(new RegExp(`login 'Admin' from ${lan!.replace(/\./g, '\\.')} \\(HTTP, remote\\)`), from);
  await server.waitSysmon(new RegExp(`shell login accepted for 'Admin' from ${lan!.replace(/\./g, '\\.')} \\(web\\)`), from);
});

test('command suggestions offer each command once', async ({ page }) => {
  await open(page);
  await page.keyboard.type('log');
  // The suggestion list above the command line bar (decisions §30 "12b2").
  const titles = page.locator('#cb-suggest .lc-sg-row .lc-sg-title');
  await expect(titles.first()).toBeVisible();
  const rows = (await titles.allInnerTexts()).map(t => t.trim()).filter(Boolean);
  const lower = rows.map(r => r.toLowerCase());
  expect(lower.filter(r => r === 'logout'), rows.join(', ')).toHaveLength(1);
  expect(lower.filter(r => r === 'login'), rows.join(', ')).toHaveLength(1);
  expect(lower, 'an alias next to its own command').not.toContain('logoff');
  expect(new Set(lower).size, `duplicates: ${rows.join(', ')}`).toBe(lower.length);
  await page.keyboard.press('Escape');
  // The second Escape clears the line: Admin's session (and its typed text) outlives this page.
  await page.keyboard.press('Escape');
  await expect.poll(async () => (await page.locator('#cb-line .xterm-rows').innerText()).trim()).toMatch(/>$/);
});

test('Enter runs the typed command even while suggestions are open', async ({ page }) => {
  await open(page);
  // `List User` leaves suggestions (UserProfile, the user names) open.
  await cmd(page, 'List User');
  await expect.poll(() => screenText(page)).toMatch(/Users:[\s\S]*Admin/);
  expect(await page.locator('#cb-line .xterm-rows').innerText()).not.toContain('List User');
});
