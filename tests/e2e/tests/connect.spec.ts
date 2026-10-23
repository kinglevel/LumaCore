// The app's connect page (crates/ui/lumacore-webui/web/connect.html, decisions §30
// "12a3", the Connect / Prompts boards): built from kit elements, bundled with
// the app. Served here the way the app serves it (lumacore://localhost/ → the
// bundled files) with `window.ipc` recording what it asks the app; the test
// plays the app by calling `home.*` like app.rs does. Logging in is not here:
// the station's own page shows its surface after a server is picked.

import path from 'node:path';
import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { REPO } from '../support/server';

const WEB = path.join(REPO, 'crates/ui/lumacore-webui/web');
const ORIGIN = 'http://connect.test';
type Ipc = { type: string; [k: string]: unknown };

const MIME: Record<string, string> = { js: 'text/javascript', css: 'text/css', html: 'text/html', svg: 'image/svg+xml', jpg: 'image/jpeg', woff2: 'font/woff2' };

async function openConnect(page: Page) {
  await page.route(`${ORIGIN}/**`, route => {
    const p = new URL(route.request().url()).pathname;
    const file = path.join(WEB, p === '/' ? 'connect.html' : p);
    try {
      route.fulfill({ body: readFileSync(file), contentType: MIME[file.split('.').pop()!] || 'application/octet-stream' });
    } catch { route.fulfill({ status: 404, body: '' }); }
  });
  await page.addInitScript(() => {
    (window as any).__ipc = [];
    (window as any).ipc = { postMessage: (m: string) => (window as any).__ipc.push(JSON.parse(m)) };
  });
  await page.goto(`${ORIGIN}/`);
  await waitIpc(page, 'ready');
}
const ipc = (page: Page) => page.evaluate(() => (window as any).__ipc as Ipc[]);
async function waitIpc(page: Page, type: string, n = 1): Promise<Ipc> {
  await expect.poll(async () => (await ipc(page)).filter(m => m.type === type).length, { message: `ipc ${type}` })
    .toBeGreaterThanOrEqual(n);
  return (await ipc(page)).filter(m => m.type === type)[n - 1];
}
const call = (page: Page, js: string) => page.evaluate(`void (${js})`);
const sf = (page: Page) => page.locator('luma-surface');

const SERVERS = [
  { id: 'a1', name: 'FOH', host: '10.0.0.5', port: 47656, version: '0.1.0', show: 'Gala' },
  { id: 'b2', name: 'Stage', host: '10.0.0.6', port: 47656, version: '0.0.9', show: null, wrong_version: true },
];

test('the countdown connects by itself, or now; Esc goes to the servers', async ({ page }) => {
  await openConnect(page);
  await call(page, `home.countdown({local: false, name: 'FOH', host: '10.0.0.5', user: 'LD', seconds: 1})`);
  await expect(sf(page)).toContainText('Connecting to FOH as LD');
  await waitIpc(page, 'autostart_go');
  await call(page, `home.countdown({local: true, user: 'Admin', seconds: 30})`);
  await page.locator('luma-surface .lc-sf-pill.go', { hasText: 'Connect now' }).click();
  await waitIpc(page, 'autostart_go', 2);
  await call(page, `home.countdown({local: true, user: 'Admin', seconds: 30})`);
  await page.keyboard.press('Escape');
  await waitIpc(page, 'autostart_cancel');
  await expect(page.locator('luma-surface .lc-sf-panel')).toBeVisible();
});

test('servers: discovered, wrong version, local server, address, toggles', async ({ page }) => {
  await openConnect(page);
  await call(page, `home.state({servers: ${JSON.stringify(SERVERS)}, local: {running: false}, last_key: 'a1', auto_start: true, kiosk: false, view: 'servers'})`);
  const rows = page.locator('luma-surface .lc-sf-server');
  await expect(rows).toHaveCount(2);
  await expect(rows.first()).toHaveClass(/current/);
  await expect(rows.nth(1)).toContainText('wrong version');
  await rows.nth(1).click();
  await expect(page.locator('luma-surface .lc-sf-error')).toContainText('E-WIRE-001');
  await rows.first().click();
  expect(await waitIpc(page, 'open')).toMatchObject({ host: '10.0.0.5', port: 47656 });
  await expect(page.locator('luma-surface .lc-sf-ask')).toContainText('Connecting to FOH');

  await call(page, `home.state({view: 'servers'})`);
  await expect(page.locator('luma-surface .lc-sf-local')).toContainText('No server running');
  await page.locator('luma-surface .lc-sf-local button', { hasText: 'Start local server' }).click();
  await waitIpc(page, 'start_local');

  await call(page, `home.state({view: 'servers'})`);
  await page.locator('luma-surface .lc-sf-addr input').fill('[::1]:47700');
  await page.locator('luma-surface .lc-sf-addr input').press('Enter');
  expect(await waitIpc(page, 'open', 2)).toMatchObject({ host: '::1', port: 47700 });

  await call(page, `home.state({view: 'servers'})`);
  await page.locator('luma-surface .lc-sf-check', { hasText: 'last server' }).click();
  expect(await waitIpc(page, 'auto_start')).toMatchObject({ on: false });
  await page.locator('luma-surface .lc-sf-check', { hasText: 'Kiosk' }).click();
  expect(await waitIpc(page, 'kiosk')).toMatchObject({ on: true });
});

test('servers: green when the beacon is fresh, amber "not answering" when late, grey wrong version', async ({ page }) => {
  // §30 "12b2 hardening": the app sends not_answering_s once a beacon is
  // late (> 1.5 s) and drops the server after 4 s or on its leaving beacon.
  await openConnect(page);
  const list = [
    { id: 'a1', name: 'FOH', host: '10.0.0.5', port: 47656, version: '0.1.0', show: 'Gala', not_answering_s: null },
    { id: 'c3', name: 'Spare', host: '10.0.0.7', port: 47656, version: '0.1.0', show: null, not_answering_s: 2 },
    { id: 'b2', name: 'Stage', host: '10.0.0.6', port: 47656, version: '0.0.9', show: null, wrong_version: true },
  ];
  await call(page, `home.state({servers: ${JSON.stringify(list)}, local: {running: false}, last_key: null, view: 'servers'})`);
  const rows = page.locator('luma-surface .lc-sf-server');
  await expect(rows).toHaveCount(3);
  await expect(rows.nth(0).locator('.lc-sf-dot')).toHaveClass(/\bok\b/);
  await expect(rows.nth(0).locator('em')).toHaveText('OK');
  await expect(rows.nth(1).locator('.lc-sf-dot')).toHaveClass(/\bwarn\b/);
  await expect(rows.nth(1).locator('em')).toHaveText('not answering 2 s');
  await expect(rows.nth(1).locator('em')).toHaveClass(/\bwarn\b/);
  await expect(rows.nth(2).locator('.lc-sf-dot')).toHaveClass(/\boff\b/);
  // Gone from the app's list → gone from the page.
  await call(page, `home.state({servers: ${JSON.stringify(list.slice(0, 1))}})`);
  await expect(rows).toHaveCount(1);
});

test('Clean start asks in the standard dialog', async ({ page }) => {
  await openConnect(page);
  await call(page, `home.state({local: {running: true, station: {name: 'desk1'}}, view: 'servers'})`);
  await page.locator('luma-surface .lc-sf-local button', { hasText: 'Clean start' }).click();
  await waitIpc(page, 'clean_start');
  await call(page, `home.confirmClean({running: true, show: 'Gala', dirty: true})`);
  const dlg = page.locator('[data-kit="dialog"]');
  await expect(dlg).toContainText('unsaved changes');
  await dlg.locator('button', { hasText: 'Save, then clean start' }).click();
  expect(await waitIpc(page, 'clean_start_go')).toMatchObject({ save: true });
});

test('Quit, another version and a hung stop ask in the standard dialog', async ({ page }) => {
  await openConnect(page);
  const dlg = page.locator('[data-kit="dialog"]');
  await call(page, `home.confirmStop({show: 'Gala', dirty: true})`);
  await dlg.locator('button', { hasText: "Don't save" }).click();
  expect(await waitIpc(page, 'stop_local')).toMatchObject({ save: false });

  await call(page, `home.confirmOther({reason: 'another client protocol', build: 'abc', pid: 41, usable: false, show: null, dirty: false})`);
  await expect(dlg.locator('button', { hasText: 'Use it' })).toBeDisabled();
  await page.keyboard.press('Escape');
  await waitIpc(page, 'other_cancel');

  await call(page, `home.confirmHung({pid: 41})`);
  await dlg.locator('button', { hasText: 'Force stop' }).click();
  await waitIpc(page, 'force_stop');
  await call(page, `home.confirmHung({pid: null})`);
  await expect(dlg.locator('button', { hasText: 'Force stop' })).toHaveCount(0);
  await dlg.locator('button', { hasText: 'Keep waiting' }).click();
  await waitIpc(page, 'keep_waiting');
});
