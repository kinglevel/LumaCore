// UI keys: buttons report press / double / hold, the
// server runs the key's command. Menu: press = open the menu, double press =
// Quick Save (Menu pressed twice quickly). Plus `List / Info UIKey`,
// the per-profile double-click time, /api/ui-keys, and CleanStart (command,
// rights, `--clean-start`).

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { Server, cmd, connected, kbdMode, screenText, dialog, answer } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  // Key definitions loaded (timing + which keys have a double press).
  await page.waitForFunction('uiKeys.defs.has("menu")');
}
const gear = (page: Page) => page.locator('.sl-btn[data-key="menu"]');
const menuOpen = (page: Page) => page.locator('#menu-overlay.open');
// Only the show files: every save keeps the file it replaces in `.versions/`
// (decisions §30 "12d part 5": Backup).
const shows = () => {
  try { return readdirSync(path.join(server.dir, 'library/shows')).filter(f => f.endsWith('.luma')); } catch { return []; }
};

test('one click on ⚙ opens the menu (Press runs Toggle Menu on the server)', async ({ page }) => {
  await open(page);
  const from = server.mark();
  await gear(page).click();
  await expect(menuOpen(page)).toHaveCount(1);
  await server.waitSysmon(/Cmd : \[Admin@Screen 1\] Toggle Menu → ok/, from);
  expect(server.sysmonSince(/SaveShow/, from)).toEqual([]);
  await page.keyboard.press('Escape');
});

test('double-click on ⚙ is a Quick Save: menu closes, notice, file on disk', async ({ page }) => {
  await open(page);
  const before = shows();
  const from = server.mark();
  await gear(page).dblclick();
  await server.waitSysmon(/Cmd : \[Admin@Screen 1\] SaveShow → ok/, from);
  // The feedback system: "Show saved" flash (no separate notice any more).
  await expect(page.locator('#fb-flash')).toHaveText('Show saved');
  await expect(menuOpen(page)).toHaveCount(0);
  // An untitled show gets a generated name (NewShow_<date>).
  await expect.poll(shows).not.toEqual(before);
  expect(shows().some(f => /^NewShow_.*\.luma$/.test(f))).toBe(true);
});

test('a second Quick Save keeps the name', async ({ page }) => {
  await open(page);
  const before = shows();
  await gear(page).dblclick();
  await expect(page.locator('#fb-flash')).toHaveText('Show saved');
  expect(shows()).toEqual(before);
});

test('the Power key opens the Shutdown menu', async ({ page }) => {
  await open(page);
  await page.locator('.sl-btn[data-key="power"]').click();
  await expect(page.locator('#power-overlay')).toHaveClass(/\bopen\b/);
  await page.keyboard.press('Escape');
});

test('List UIKey / Info UIKey publish the keys', async ({ page }) => {
  await open(page);
  // The list is longer than a screen now: its end shows PRG's keys (decisions 0295); Menu is Info's.
  await cmd(page, 'List UIKey');
  await expect.poll(() => screenText(page)).toMatch(/prg\.\S+\s+core\s+\S+\s+Press: /);
  await cmd(page, 'Info UIKey "Menu"');
  await expect.poll(() => screenText(page)).toMatch(/DoublePress:\s+SaveShow\s+\(replaces the press\)/);
  const r = await page.request.get(`${server.url}/api/ui-keys`);
  const d = await r.json();
  expect(d.double_click_ms).toBe(300);
  expect(d.keys.find((k: any) => k.id === 'menu')).toMatchObject({ locked: true, double_press: 'SaveShow', owner: 'core' });
});

test('Set UserProfile "DoublePress": two slow clicks are two presses, not a save', async ({ page }) => {
  await open(page);
  await cmd(page, 'Set UserProfile "DoublePress" 150');
  await page.reload();
  await connected(page);
  await page.waitForFunction('uiKeys.double === 150');
  const from = server.mark();
  await gear(page).click();
  await page.waitForTimeout(260);
  // Second click at the same spot (the menu overlay is on top now).
  const box = (await gear(page).boundingBox())!;
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  await page.waitForTimeout(500);
  expect(server.sysmonSince(/SaveShow/, from)).toEqual([]);
  await page.keyboard.press('Escape');
  await kbdMode(page);
  await cmd(page, 'Set UserProfile "DoublePress" 300');
});

test('CleanStart needs Admin', async ({ page }) => {
  await open(page);
  await cmd(page, 'Store User "Prog" /Rights Program');
  await cmd(page, 'Login "Prog"');
  await expect(page.locator('#cb-line .xterm-rows')).toContainText('[Prog@');
  await cmd(page, 'CleanStart');
  await expect.poll(() => screenText(page)).toContain("'CleanStart' needs Admin — you are Program");
  await cmd(page, 'Login "Admin" "admin"');
});

test('CleanStart: asks, then an empty show and default station settings', async ({ page }) => {
  await open(page);
  await cmd(page, 'Store User "Temp"');
  const from = server.mark();
  await cmd(page, 'CleanStart');
  await expect(dialog(page)).toContainText('unsaved changes');
  await answer(page, "Don't save");
  await server.waitSysmon(/\[WARN\] CleanStart by 'Admin'/, from);

  const station = await (await fetch(`${server.url}/api/station`)).json();
  expect(station.show).toBeNull();
  expect(station.users.map((u: any) => u.name)).not.toContain('Temp');
  const settings = JSON.parse(readFileSync(path.join(server.dir, 'library/station_settings.json'), 'utf8'));
  expect(settings.startup_show).toEqual({ mode: 'none' });
  expect(settings.station_id).toBe(station.id); // identity kept
  expect(shows().length).toBeGreaterThan(0);    // show files stay
});

test('--clean-start at server start; the next normal start stays empty (startup show None)', async () => {
  const own = await new Server().start();
  try {
    const token = await own.login('Admin');
    // A saved show that would autoload (startup show: Last).
    const sock = await import('node:dgram');
    const udp = sock.createSocket('udp4');
    await new Promise<void>(r => udp.send(JSON.stringify({ kind: 'command', user: 'Admin', cmd: 'SaveShow "keep"' }), own.cmd, '127.0.0.1', () => r()));
    await new Promise(r => setTimeout(r, 500));
    udp.close();
    expect(existsSync(path.join(own.dir, 'library/shows/keep.luma'))).toBe(true);
    expect((await (await fetch(`${own.url}/api/station`)).json()).show).toBe('keep');

    await own.stop();
    await own.start(['--clean-start']);
    const st = await (await fetch(`${own.url}/api/station`)).json();
    expect(st.show).toBeNull();
    expect(existsSync(path.join(own.dir, 'library/shows/keep.luma'))).toBe(true);
    void token;

    await own.stop();
    await own.start();
    expect((await (await fetch(`${own.url}/api/station`)).json()).show).toBeNull(); // startup show None
  } finally {
    await own.cleanup();
  }
});
