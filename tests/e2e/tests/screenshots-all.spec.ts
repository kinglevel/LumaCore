// Dev captures (decisions §29 "11d"): `scripts/unix/screenshots` runs this
// with SCREENSHOTS_OUT set; the normal e2e run skips it. A test station with
// the demo show; every window type, every menu a UI key opens and every
// overlay a menu tile opens — all taken from what the station publishes, so
// new ones are picked up — captured through the real path (`Screenshot` →
// the page renders itself → POST /api/screenshots) and copied to
// $SCREENSHOTS_OUT as <nn>_<what>.png. The app's connect page and the lock
// surface aren't screens: Playwright captures them directly.

import { copyFileSync, mkdirSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const OUT = process.env.SCREENSHOTS_OUT;
test.skip(!OUT, 'dev captures: run scripts/unix/screenshots');
test.describe.configure({ mode: 'serial' });
test.setTimeout(10 * 60_000);

const server = new Server();
let n = 0;

test.beforeAll(async () => {
  if (!OUT) return;
  mkdirSync(OUT, { recursive: true });
  server.dir = mkdtempSync(path.join(tmpdir(), 'lumacore-shots-'));
  mkdirSync(path.join(server.dir, 'library/shows'), { recursive: true });
  copyFileSync(path.join(REPO, 'library/shows/demo.luma'), path.join(server.dir, 'library/shows/demo.luma'));
  await server.start();
  const loaded = await server.command('LoadShow "demo" /NoConfirmation');
  expect(loaded.message).toMatch(/loaded/);
});
test.afterAll(async () => { if (OUT) await server.cleanup(); });

/** One capture through the real path, copied to OUT. */
async function capture(what: string) {
  await new Promise(r => setTimeout(r, 700)); // feeds (DMX, health) draw once
  const from = server.mark();
  const reply = await server.command('Screenshot');
  expect(reply.message, what).toMatch(/asked from/);
  const line = await server.waitSysmon(/Screenshot saved: (\S+\.png)/, from, 15_000);
  const file = line.match(/Screenshot saved: (\S+\.png)/)![1];
  n += 1;
  const name = `${String(n).padStart(2, '0')}_${what.replace(/[^A-Za-z0-9]+/g, '-')}.png`;
  copyFileSync(path.join(server.dir, file), path.join(OUT!, name));
}

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction('typeof screenFeed !== "undefined" && screenFeed.live !== null && screenFeed.live === liveScreen');
}

/** Close whatever overlay is open. */
async function closeAll(page: Page) {
  for (let i = 0; i < 4 && (await page.locator('.mo-overlay.open').count()); i++) await page.keyboard.press('Escape');
  await page.evaluate(() => document.querySelectorAll('.mo-overlay.open').forEach(o => o.classList.remove('open')));
}

test('every window type', async ({ page }) => {
  await open(page);
  await capture('start');
  const types: { name: string; label: string; hidden: boolean }[] =
    await (await fetch(`${server.url}/api/window-types`)).json();
  for (const t of types.filter(t => !t.hidden)) {
    await cmd(page, `Store Window 1 "${t.label}" /Overwrite`);
    await capture(`window-${t.name}`);
  }
});

test('every menu a UI key opens, and every overlay a menu tile opens', async ({ page }) => {
  await open(page);
  const keys: { id: string; press: string }[] = (await page.evaluate(
    async () => (await (await fetch('/api/ui-keys')).json()).keys,
  )) ?? [];
  for (const key of keys.filter(k => /^Menu\b/i.test(k.press ?? ''))) {
    const button = page.locator(`.sl-btn[data-key="${key.id}"]`);
    if (!(await button.count())) continue;
    await button.first().click();
    await expect(page.locator('.mo-overlay.open').first()).toBeVisible();
    await capture(`menu-${key.id}`);
    // The menu's tiles that open menus (Network, DMX Protocols, LumaSessions,
    // Users & Profiles, Settings, Backup on the setup frame — 12d part 4 / 5;
    // the kit's tile grid, decisions §30 "12c2").
    const OPENS = ':is([data-tile="network"], [data-tile="protocols"], [data-tile="backup"], [data-tile="lumasessions"], [data-tile="users"], [data-tile="settings"])';
    const tiles = page.locator(`.mo-overlay.open .lc-tile${OPENS}`);
    const labels = await tiles.allInnerTexts();
    for (let i = 0; i < labels.length; i++) {
      if (!(await page.locator('.mo-overlay.open').count())) await button.first().click();
      await page.locator(`.mo-overlay.open .lc-tile${OPENS}`).nth(i).click();
      await expect(page.locator('luma-setup, .mo-overlay.open').first()).toBeVisible();
      await page.waitForTimeout(600);
      await capture(`menu-${key.id}-${labels[i]}`);
      await page.evaluate(() => (0, eval)('closeSetupMenu()'));
      await closeAll(page);
    }
    await closeAll(page);
  }
});

test('pop-ups: object settings', async ({ page }) => {
  await open(page);
  await cmd(page, 'EditSetting Show');
  await capture('popup-editsetting-show');
  await closeAll(page);
});

test('the app connect page (bundled; played like the app; captured by Playwright)', async ({ page }) => {
  const WEB = path.join(REPO, 'crates/ui/lumacore-webui/web');
  const MIME: Record<string, string> = { js: 'text/javascript', css: 'text/css', html: 'text/html', svg: 'image/svg+xml', jpg: 'image/jpeg', woff2: 'font/woff2' };
  await page.route('http://connect.test/**', route => {
    const p = new URL(route.request().url()).pathname;
    const file = path.join(WEB, p === '/' ? 'connect.html' : p);
    try { route.fulfill({ body: readFileSync(file), contentType: MIME[file.split('.').pop()!] }); }
    catch { route.fulfill({ status: 404, body: '' }); }
  });
  await page.addInitScript(() => { (window as any).ipc = { postMessage: () => {} }; });
  await page.goto('http://connect.test/');
  await page.waitForFunction('typeof home !== "undefined"');
  const servers = [
    { id: 'a1', name: 'FOH', host: '10.0.0.5', port: 47656, version: '0.1.0', show: 'Gala' },
    { id: 'b2', name: 'Stage', host: '10.0.0.6', port: 47656, version: '0.0.9', show: null, wrong_version: true },
  ];
  const shot = async (what: string) => {
    await page.waitForTimeout(400);
    await page.screenshot({ path: path.join(OUT!, `connect-${what}.png`) });
  };
  await page.evaluate(`void home.countdown({local: false, name: 'FOH', host: '10.0.0.5', user: 'LD', seconds: 30})`);
  await shot('1-countdown');
  await page.evaluate(`home.state(${JSON.stringify({ view: 'servers', servers, auto_start: true, last_key: 'a1', local: { running: true, station: { name: 'desk1', show: 'Gala' } } })})`);
  await shot('2-servers');
  await page.evaluate(`void home.confirmClean({running: true, show: 'Gala', dirty: true})`);
  await shot('3-clean-start');
  await page.keyboard.press('Escape');
  await page.evaluate(`void home.confirmOther({reason: 'another client protocol', build: 'abc123', pid: 4120, usable: false, show: 'Gala', dirty: false})`);
  await shot('4-other-version');
});

test('the lock / login surface (the station page)', async ({ page }) => {
  await server.command('Store User "LD" /Rights Program');
  const shot = async (what: string) => {
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(OUT!, `surface-${what}.png`) });
  };
  await page.goto(server.url);
  await expect(page.locator('luma-surface')).toBeVisible();
  await shot('1-who');
  await page.locator('luma-surface luma-avatar[data-user="Admin"] button').click();
  await shot('2-admin-password');
  await page.locator('luma-surface button[aria-label="Keyboard"]').click();
  await shot('3-keyboard');
  await page.locator('luma-surface .lc-sf-pw').fill('admin');
  await page.locator('luma-surface .lc-sf-pw').press('Enter');
  await connected(page);
  await page.evaluate(() => (window as any).wsCommand('DeskLock'));
  await expect.poll(() => page.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode)).toBe('locked');
  await shot('4-locked');
  await page.evaluate(() => (window as any).wsCommand('Off DeskLock'));
});
