// App mode: the server UI as the LumaCore app loads it — `window.LUMACORE_APP`
// injected (server, user, token, config, screen) and `window.ipc` for talking
// to the app. Here the app is replaced by a recorder of the IPC messages, so
// every interface path the app relies on is checked from the page side.

import { expect, test, type Page } from '@playwright/test';
import { Server, answer, cmd, giveConsent, connected, dialog, kbdMode, promptUser, screenText } from '../support/server';

const server = new Server();

test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

type Ipc = { type: string; [k: string]: unknown };

/** Open the UI the way the app does, as `user`. */
async function openApp(page: Page, user = 'Admin', opts: { local?: boolean; srv?: Server } = {}) {
  const srv = opts.srv ?? server;
  const token = await srv.login(user);
  await page.addInitScript(ctx => {
    (window as any).__ipc = [];
    (window as any).ipc = { postMessage: (m: string) => (window as any).__ipc.push(JSON.parse(m)) };
    (window as any).LUMACORE_APP = ctx;
  }, { local: opts.local ?? true, station: 'E2E', host: '127.0.0.1', user, token, config: 1, screen: 1, page: {} });
  await page.goto(`${srv.url}/?screen=1`);
  await connected(page);
  await kbdMode(page);
  return token;
}

const ipc = (page: Page) => page.evaluate(() => (window as any).__ipc as Ipc[]);
async function waitIpc(page: Page, type: string): Promise<Ipc> {
  try {
    await expect.poll(async () => (await ipc(page)).some(m => m.type === type), { message: `ipc ${type}` }).toBe(true);
  } catch (e) {
    const screen = (await screenText(page)).split('\n').filter(l => l.trim()).slice(-12).join('\n');
    const vs = await page.evaluate('liveScreen').catch(() => '?');
    throw new Error(`${(e as Error).message}\n--- ipc ---\n${JSON.stringify(await ipc(page))}\n--- liveScreen ${vs} ---\n--- screen ---\n${screen}`);
  }
  return (await ipc(page)).find(m => m.type === type)!;
}

test('the app window skips the login screen and connects with its token', async ({ page }) => {
  const from = server.mark();
  await openApp(page);
  await expect(page.locator('luma-surface')).toHaveCount(0);
  await promptUser(page, 'Admin');
  await server.waitSysmon(/shell login accepted for 'Admin' from 127\.0\.0\.1 \(web\)/, from);
});

test('Logout asks the app for the user list, Disconnect for the server list', async ({ page, context }) => {
  await openApp(page);
  await cmd(page, 'Logout');
  expect(await waitIpc(page, 'logged_out')).toMatchObject({ disconnect: false });

  const page2 = await context.newPage();
  await openApp(page2);
  await cmd(page2, 'Disconnect');
  expect(await waitIpc(page2, 'logged_out')).toMatchObject({ disconnect: true });
});

test('Logout as Debug reaches the app', async ({ page }) => {
  await openApp(page, 'Debug');
  await promptUser(page, 'Debug');
  // Typed, not `cmd`: after Logout the app owns the screen and the bar shows no prompt to wait for.
  await page.keyboard.type('Logout', { delay: 5 });
  await page.keyboard.press('Enter');
  expect(await waitIpc(page, 'logged_out')).toMatchObject({ disconnect: false });
});

test('an in-session Login tells the app the new user and token', async ({ page }) => {
  await openApp(page);
  await cmd(page, 'Login "Debug"');
  const m = await waitIpc(page, 'user_changed');
  expect(m.user).toBe('Debug');
  expect(String(m.token)).toMatch(/^v2\./);
});

test('Connect hands the target and login to the app; the password is masked', async ({ page }) => {
  await openApp(page);
  await cmd(page, 'Connect IP 10.9.9.9:47656 User "LD" "pw123"');
  expect(await waitIpc(page, 'connect')).toMatchObject({ target: '10.9.9.9:47656', user: 'LD', password: 'pw123' });
  expect(await screenText(page)).not.toContain('pw123');
});

test('Connect by station name', async ({ page }) => {
  await openApp(page);
  await cmd(page, 'Connect "FOH"');
  expect(await waitIpc(page, 'connect')).toMatchObject({ target: 'FOH' });
});

test('ScreenConfig switches through the app', async ({ page }) => {
  await openApp(page);
  await cmd(page, 'ScreenConfig 1');
  expect(await waitIpc(page, 'screen_config')).toMatchObject({ no: 1 });
});

test('hiding the bars (F12) is stored by the app', async ({ page }) => {
  await openApp(page);
  await page.keyboard.press('F12');
  const m = await waitIpc(page, 'pref');
  expect(m).toMatchObject({ key: 'sidebars' });
  // F12 = full screen of the windows (decisions §30 "12b2").
  expect((m.value as any).full).toBe(true);
});

test('Shutdown of a remote server warns; Cancel tells the app', async ({ page }) => {
  await openApp(page, 'Admin', { local: false });
  await cmd(page, 'Shutdown');
  await expect(dialog(page).locator('.lc-dlg-note')).toContainText('not this computer');
  await answer(page, 'Cancel');
  await waitIpc(page, 'power_cancel');
});

test('Shutdown of the local server has no remote warning', async ({ page }) => {
  await openApp(page, 'Admin', { local: true });
  await cmd(page, 'Shutdown');
  await expect(dialog(page)).toBeVisible();
  await expect(dialog(page).locator('.lc-dlg-note')).toBeHidden();
  await answer(page, 'Cancel');
});

test('losing the server shows the reconnect overlay; it reconnects, Cancel leaves', async ({ page }) => {
  const own = await new Server().start();
  try {
    await openApp(page, 'Admin', { srv: own });
    await own.crash();
    await expect(page.locator('#conn-overlay')).toHaveClass(/\bopen\b/);
    await expect(page.locator('#conn-text')).toContainText('reconnecting');
    // The app's notices land in the overlay.
    await page.evaluate(() => (window as any).lumacoreApp.notice('restarting it…'));
    await expect(page.locator('#conn-notice')).toHaveText('restarting it…');

    // Back up (same library, so the token is still valid): the overlay goes.
    await own.start();
    await expect(page.locator('#conn-overlay')).not.toHaveClass(/\bopen\b/, { timeout: 15_000 });
    await connected(page);

    await own.crash();
    await expect(page.locator('#conn-overlay')).toHaveClass(/\bopen\b/);
    await page.locator('#conn-cancel').click();
    await waitIpc(page, 'disconnect');
  } finally {
    await own.cleanup();
  }
});

test('a confirmed Shutdown goes back to the server list', async ({ page }) => {
  const own = await new Server().start();
  try {
    await openApp(page, 'Admin', { srv: own });
    await cmd(page, 'Shutdown');
    await dialog(page).locator('.lc-dlg-btn[data-primary="1"]').click();
    expect(await waitIpc(page, 'power')).toMatchObject({ command: 'Shutdown' });
    expect(await waitIpc(page, 'disconnect')).toMatchObject({ message: 'The server was shut down.' });
  } finally {
    await own.cleanup();
  }
});

// ── 12a3: the station's surface logs in; LumaHost (decisions §30 "12a3") ──

/** The app's pre-login window: no token yet, LumaHost recording. */
async function openAppNoLogin(page: Page, host: Record<string, unknown> = {}) {
  await page.addInitScript(([ctx, h]) => {
    (window as any).__ipc = [];
    const post = (m: unknown) => (window as any).__ipc.push(m);
    (window as any).ipc = { postMessage: (m: string) => post(JSON.parse(m)) };
    (window as any).LUMACORE_APP = ctx;
    (window as any).LumaHost = Object.assign(h, {
      fullscreen: (on: unknown) => post({ type: 'fullscreen', on: on ?? null }),
      loggedIn: (user: string, token: string) => post({ type: 'logged_in', user, token }),
      switchUser: (user: string, token: string) => post({ type: 'user_changed', user, token }),
      focusSurface: () => post({ type: 'focus_surface' }),
      showServers: () => post({ type: 'servers' }),
    });
  }, [{ local: true, station: 'E2E', host: '127.0.0.1', user: '', token: '', config: 0, screen: 1, page: {}, device: 'app-e2e', autologin: null },
      { device: 'app-e2e', kiosk: false, isPrimary: true, ...host }] as const);
  await page.goto(`${server.url}/?screen=1`);
}

test('before a login the station page shows its surface; the login goes to the app', async ({ page }) => {
  await openAppNoLogin(page);
  await expect(page.locator('luma-surface')).toBeVisible();
  await page.locator('luma-surface luma-avatar[data-user="Admin"] button').click();
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await giveConsent(page);
  const msg = await waitIpc(page, 'logged_in');
  expect(msg.user).toBe('Admin');
  expect(String(msg.token)).toMatch(/^v2\./);
});

test('Servers… on the surface goes back to the app\'s server list', async ({ page }) => {
  await openAppNoLogin(page);
  await page.locator('luma-surface .lc-sf-servers').click();
  await waitIpc(page, 'servers');
});

test('the app\'s device id rides on the shell; Meow+F12 asks the app for native full screen', async ({ page }) => {
  await openAppNoLogin(page);
  await page.locator('luma-surface luma-avatar[data-user="Admin"] button').click();
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await giveConsent(page);
  const { token } = await waitIpc(page, 'logged_in');
  // The app opens the screen with the token (here: the same page, reloaded).
  await page.evaluate(t => { (window as any).LUMACORE_APP.token = t; (window as any).LUMACORE_APP.user = 'Admin'; }, token);
  await page.evaluate(() => { (window as any).hideSurface(); (window as any).start('Admin'); });
  await connected(page);
  expect(await page.evaluate('DEVICE')).toBe('app-e2e');
  await page.locator('#cb-line').click();
  await page.keyboard.press('Shift+F12');
  await waitIpc(page, 'fullscreen');
});

test('a locked device: a window that isn\'t primary shows the blurred picture', async ({ page }) => {
  await openAppNoLogin(page, { isPrimary: false });
  await page.locator('luma-surface luma-avatar[data-user="Admin"] button').click();
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await giveConsent(page);
  await waitIpc(page, 'logged_in');
  await page.evaluate(() => { (window as any).hideSurface(); (window as any).start('Admin'); });
  await connected(page);
  await page.evaluate(() => (window as any).wsCommand('On DeskLock'));
  await expect.poll(() => page.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode)).toBe('blurred');
  await page.locator('luma-surface .lc-sf-center').click();
  await waitIpc(page, 'focus_surface');
  await page.evaluate(() => { (window as any).LumaHost.isPrimary = true; (window as any).lumacoreApp.primaryChanged(); });
  await expect.poll(() => page.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode)).toBe('locked');
});
