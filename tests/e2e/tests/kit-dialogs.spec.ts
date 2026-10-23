// The station's dialogs (decisions §30 "12b", the 12b boards): every question
// the station asks is held by the station and drawn by the kit's standard
// dialog — a line's confirm only on its device, a claim ask on all of the
// owner's devices, the first answer wins and closes it everywhere; inputs
// (text with recent-name chips, numbers on the keypad), the pick list for a
// link, "n waiting" while locked, the VK button with its Keyboard mode. On
// the desktop, a touch tablet, a touch phone and WebKit.

import path from 'node:path';
import { expect, test, type Browser, type Page } from '@playwright/test';
import { REPO, Server, answer, connected, dialog, kbdMode, screenText } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  for (const line of ['Store User "LD" /Rights Program', 'Store User "Busk" /Rights Program']) {
    const r = await server.command(line);
    expect(r.ok, r.message).toBe(true);
  }
});
test.afterAll(async () => { await server.cleanup(); });

/** Run `text` as a line of this page's device (the page's own command
 *  path) and wait until the station logged it. Typing is the VK test's job:
 *  a phone's one-row command line can't show a long prompt (12e). */
async function line(page: Page, text: string) {
  const from = server.mark();
  await page.evaluate(t => (window as any).wsCommand(t), text);
  const esc = text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  await server.waitSysmon(new RegExp(`\\] ${esc} →`), from);
}

async function open(page: Page, user = 'Admin') {
  await server.open(page, '/', user);
  await connected(page);
  await kbdMode(page);
}

/** A second browser (its own device) logged in as `user`. */
async function other(browser: Browser, user: string) {
  const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage();
  await open(page, user);
  return page;
}

test('a line\'s question shows only on its device; the answer runs the line once', async ({ page, context }) => {
  await open(page);
  const second = await context.newPage(); // the same user, another tab = another device
  await open(second);
  await line(page, 'Debug');
  await expect(dialog(page).locator('.lc-dlg-title span').first()).toHaveText('Debug');
  await expect(second.locator('[data-kit="dialog"]')).toHaveCount(0);
  await answer(page, 'Enable');
  await expect(page.locator('[data-kit="dialog"]')).toHaveCount(0);
  await expect.poll(() => screenText(page)).toContain('Debug mode enabled');
  await line(page, 'Debug /NoConfirmation');
  await second.close();
});

test('Esc answers Cancel; a touch stops a countdown', async ({ page }) => {
  await open(page);
  await line(page, 'Shutdown /NoSave'); // (unsaved changes would ask Save first, no countdown)
  const meta = dialog(page).locator('.lc-dlg-meta');
  await expect(meta).toContainText(/Shutdown in \d+ s/);
  await dialog(page).locator('.lc-dlg-text').click();
  await expect(meta).toHaveText('paused');
  await page.keyboard.press('Escape');
  await expect(page.locator('[data-kit="dialog"]')).toHaveCount(0);
  await expect.poll(() => fetch(`${server.url}/api/station`).then(r => r.ok)).toBe(true);
});

test('Label asks for the name (chips remember it); empty clears; Move asks a number on the keypad', async ({ page }) => {
  await open(page);
  await line(page, 'Label Group 1');
  const name = dialog(page).locator('input[data-field]');
  await expect(name).toHaveValue('Photon');
  await name.fill('Front warm');
  await name.press('Enter');
  await expect.poll(() => screenText(page)).toContain('labeled "Front warm"');
  // Next time the last name is a chip.
  await line(page, 'Label Group 1');
  await expect(dialog(page).locator('.lc-dlg-chips button', { hasText: 'Front warm' })).toBeVisible();
  await dialog(page).locator('input[data-field]').fill('');
  await answer(page, 'Please');
  await expect.poll(() => screenText(page)).toContain('label cleared');
  // Move without At: the keypad (old value empty, type 125 — past the rig's 77 —, Please).
  await line(page, 'Move Group 2');
  await dialog(page).locator('.lc-dlg-kb').click();
  const pad = page.locator('luma-keypad');
  await expect(pad.locator('.lc-kp-key')).toHaveCount(20);
  await pad.locator('.lc-kp-key[data-k="1"]').click();
  await pad.locator('.lc-kp-key[data-k="2"]').click();
  await pad.locator('.lc-kp-key[data-k="5"]').click();
  await pad.locator('.lc-kp-key[data-k="Please"]').click();
  await expect(dialog(page).locator('input[data-field]')).toHaveValue('125');
  await answer(page, 'Please');
  await expect.poll(() => screenText(page)).toMatch(/Group 2\b.*125|moved/i);
});

test('a claim ask reaches every client of the owner; the first answer closes it everywhere', async ({ page, browser }) => {
  const ld = await other(browser, 'LD');
  const ld2 = await ld.context().newPage();
  await open(ld2, 'LD');
  // A group of LD's own, past the rig's 77.
  await line(ld, 'Fixture 101');
  await line(ld, 'Store Group 1015');
  await line(ld, 'Claim Group 1015');
  await open(page);
  await line(page, 'Claim Group 1015');
  for (const p of [ld, ld2]) await expect(dialog(p).locator('.lc-dlg-title span').first()).toHaveText(/Admin asks for Group 1015/);
  await expect(page.locator('[data-kit="dialog"]')).toHaveCount(0);
  // The remember toggle, then Keep on one tab: the other closes and says who.
  await dialog(ld).locator('.lc-dlg-toggle button').click();
  await answer(ld, 'Keep');
  await expect(ld2.locator('[data-kit="dialog"]')).toHaveCount(0);
  await expect.poll(() => screenText(ld)).toContain('You keep Group 1015');
  // Remembered: Admin's next ask is refused at once, nobody is asked.
  await line(page, 'Claim Group 1015');
  await expect.poll(() => screenText(page)).toContain('keeps their objects from you');
  await expect(ld.locator('[data-kit="dialog"]')).toHaveCount(0);
  await ld.context().close();
});

test('while locked a question waits ("1 waiting") and shows after unlocking', async ({ page, browser }) => {
  const ld = await other(browser, 'Busk'); // LD remembered Keep for Admin
  await line(ld, 'Fixture 101');
  await line(ld, 'Store Group 1016');
  await line(ld, 'Claim Group 1016');
  await ld.evaluate(() => (window as any).wsCommand('On DeskLock'));
  await expect.poll(() => ld.evaluate(() => (document.querySelector('luma-surface') as any)?.data?.mode)).toBe('locked');
  await open(page);
  await line(page, 'Claim Group 1016');
  await expect(ld.locator('luma-surface')).toContainText('1 waiting');
  await expect(ld.locator('[data-kit="dialog"]')).toHaveCount(0);
  // The four corner taps unlock LD (no password): the question shows.
  const box = (await ld.locator('luma-surface').boundingBox())!;
  for (const [x, y] of [[5, 5], [box.width - 5, box.height - 5], [box.width - 5, 5], [5, box.height - 5]]) {
    await ld.mouse.click(box.x + x, box.y + y);
  }
  await expect(dialog(ld).locator('.lc-dlg-title span').first()).toHaveText(/Admin asks for Group 1016/);
  await answer(ld, 'Keep');
  await ld.context().close();
});

test('a link setting opens the pick list: filter, the items, Add New', async ({ page }) => {
  await open(page);
  await line(page, 'Store Sacn 9 "Out" /Network "Rig"');
  await line(page, 'EditSetting Sacn 9');
  const field = page.locator('luma-settings .lc-set-cell', { hasText: 'Network' });
  await expect(field.locator('.lc-set-value')).toHaveText('Rig');
  await field.click();
  const pick = page.locator('luma-popup');
  const items = await pick.locator('.lc-pick-row[data-no]').count();
  expect(items).toBeGreaterThan(0);
  await expect(pick.locator('.lc-pick-row.on')).toContainText('Rig');
  await expect(pick.locator('.lc-pick-row.add')).toBeVisible();
  await expect(pick.locator('.lc-pick-row.none')).toHaveCount(0); // a line always has a network
  await pick.locator('input').fill('zz');
  await expect(pick.locator('.lc-pick-empty')).toHaveText('No match');
  await pick.locator('input').fill('');
  await pick.locator('.lc-pick-row.add').click();
  // Changes apply at once (decisions §30 "12b2"): the new network is stored and set.
  await expect(field.locator('.lc-set-value')).toHaveText(/^Network \d+$/);
  const added = (await field.locator('.lc-set-value').innerText()).split(' ')[1];
  await expect.poll(async () => (await server.command('List Sacn')).message).toContain(`Network ${added}`);
});

test('window settings: tabs from sections, DataPool follows or links by GUID, Columns Fit', async ({ page }) => {
  await open(page);
  const no = await page.evaluate(() => Object.values((0, eval)('layout').windows).find((w: any) => w.kind === 'groups')?.no);
  test.skip(!no, 'the demo screen has no Groups window');
  await page.evaluate(n => (window as any).openSettings({}, n), no);
  const set = page.locator('.lc-set-scrim luma-settings');
  await expect(set.locator('.lc-set-tabs button')).toHaveText(['Pool', 'Layout', 'Colours', 'Window bar']);
  const dp = set.locator('.lc-set-cell', { hasText: 'DataPool' });
  await expect(dp.locator('.lc-set-value')).toHaveText('Selected DataPool');
  await dp.click();
  await page.locator('luma-popup .lc-pick-row[data-no="1"]').click();
  await expect(dp.locator('.lc-set-value')).toHaveText(/^1 /);
  await expect.poll(() => page.evaluate(n => (0, eval)('layout').windows[n].settings.datapool, no)).toMatch(/^[0-9a-f-]{36}$/);
  await set.locator('.lc-set-tabs button', { hasText: 'Layout' }).click();
  await set.locator('.lc-set-cell', { hasText: 'Columns' }).click();
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: 'Fit' }).click();
  await expect(set.locator('.lc-set-cell', { hasText: 'Columns' }).locator('.lc-set-value')).toHaveText('Fit');
  await page.keyboard.press('Escape');
  await expect(page.locator('.lc-set-scrim')).toHaveCount(0);
});

test('the VK button: tap shows the built-in keyboard (it types into the command line), hold picks the mode', async ({ page }) => {
  await open(page);
  const vk = page.locator('#cb-vk');
  // Hold: the Keyboard menu; pick Built-in.
  await vk.click({ button: 'right' });
  await page.locator('#ctx .ci').filter({ has: page.locator('.clbl', { hasText: /^Built-in$/ }) }).click();
  await vk.click();
  const kb = page.locator('#cmd-keyboard');
  await expect(kb).toBeVisible();
  for (const ch of 'clock') await kb.locator(`button[data-act="ch:${ch}"]`).click();
  await kb.locator('button[data-act="please"]').click();
  await expect.poll(() => screenText(page)).toMatch(/\d\d:\d\d:\d\d/);
  await vk.click();
  await expect(kb).toHaveCount(0);
  // The mode stays with this screen of this device.
  await page.reload();
  await connected(page);
  // The page reads the screen's mode once it knows its screen (the first status): poll.
  await expect.poll(() => page.evaluate(() => (window as any).LumaKit.keyboard.mode())).toBe('Built-in');
  await page.evaluate(() => localStorage.clear());
});
