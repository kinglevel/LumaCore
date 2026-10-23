// Screenshots (decisions §29 "11d"; Print Screen / F11): `Screenshot`
// asks every client showing the user's screens on the `screen` feed; the page
// renders itself as seen (vendored modern-screenshot), uploads the PNG to
// POST /api/screenshots, the station saves it under
// library/screenshots/<station>/ and the page flashes green.

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

const shots = (): string[] => {
  const root = path.join(server.dir, 'library/screenshots');
  try {
    return readdirSync(root).flatMap(station => readdirSync(path.join(root, station)).map(f => path.join(root, station, f)));
  } catch { return []; }
};

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  // Counted as a viewer of its screen (the event stream says which).
  await page.waitForFunction('typeof screenFeed !== "undefined" && screenFeed.live !== null && screenFeed.live === liveScreen');
}

test('nothing loads from the internet: xterm and the capture library are vendored', async ({ page }) => {
  const outside: string[] = [];
  page.on('request', r => { if (!r.url().startsWith(server.url)) outside.push(r.url()); });
  await open(page);
  expect(await page.evaluate('typeof modernScreenshot.domToBlob')).toBe('function');
  expect(outside).toEqual([]);
});

test('Screenshot saves a PNG of the screen as seen, with a sysmon line and a green flash', async ({ page }) => {
  await open(page);
  const before = shots().length;
  const from = server.mark();
  const reply = await server.command('Screenshot');
  expect(reply.message).toMatch(/asked from 1 client/);
  const saved = await server.waitSysmon(/Screenshot saved: (\S+)/, from);
  await expect(page.locator('.shot-flash')).toHaveCount(1);
  expect(shots().length).toBe(before + 1);
  const file = shots().find(f => saved.includes(path.basename(f)))!;
  expect(path.basename(file)).toMatch(/^\d{8}_\d{6}_Admin_screen1(_\d+)?\.png$/);
  const png = readFileSync(file);
  expect(png.subarray(1, 4).toString()).toBe('PNG');
  // As seen: the viewport size (device pixel ratio 1 here).
  expect(png.readUInt32BE(16)).toBe(1400);
  expect(png.readUInt32BE(20)).toBe(900);
});

test('F11 is bound to Screenshot', async ({ page }) => {
  await open(page);
  const from = server.mark();
  await page.keyboard.press('F11');
  await server.waitSysmon(/Screenshot saved:/, from);
});

test('no client shows the screen: a coded error, nothing asked', async () => {
  const reply = await server.command('Screenshot Screen 7');
  expect(reply.message).toMatch(/E-CMD-010/);
});

test('an upload that was not asked for is refused', async () => {
  const token = await server.login('Admin');
  const png = Buffer.from('89504e470d0a1a0a0000000d49484452', 'hex');
  const r = await fetch(`${server.url}/api/screenshots?id=999&screen=1`, {
    method: 'POST', headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'image/png' }, body: png,
  });
  expect(r.status).toBe(409);
});

test('a layout change from the command line reaches the page (screen_changed)', async ({ page }) => {
  // Found by the 11d captures: the page listened for an event name the
  // station no longer sent, so `Store Window` never showed until a reload.
  await open(page);
  await page.keyboard.type('Store Window 1 "DMX" /Overwrite', { delay: 5 });
  await page.keyboard.press('Enter');
  await expect(page.locator('.panel-tabs .ptab.on').first()).toContainText('DMX');
});
