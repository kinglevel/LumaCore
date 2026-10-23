// The UI kit's foundations (decisions §30, step 12a1), on the desktop, a touch
// tablet, a touch phone and WebKit: the kit loads (icons, fonts, tokens), the
// gold tab dot opens the window menu, a hold opens it too, split handles drag
// with the pointer core, the Showroom shows the kit, and touch screens get
// console-sized targets.

import { expect, test, type Page } from '@playwright/test';
import { Server, connected, hold, tap } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  const errors: string[] = [];
  page.on('pageerror', e => errors.push(e.message));
  await server.open(page);
  await connected(page);
  await expect(page.locator('.ptab').first()).toBeVisible();
  return errors;
}
const coarse = (page: Page) => page.evaluate(() => matchMedia('(any-pointer: coarse)').matches);
const menu = (page: Page) => page.locator('#ctx');

test('the kit loads: icons, fonts, theme tokens, no page errors', async ({ page }) => {
  const errors = await open(page);
  expect(await page.evaluate(() => !!(window as any).LumaKit)).toBe(true);
  await expect(page.locator('#lc-sprite symbol#i-settings')).toHaveCount(1);
  await expect(page.locator('#sidebar-left luma-icon[name="power"] svg')).toHaveCount(1);
  expect(await page.evaluate(() => document.fonts.check('14px "Source Sans 3"'))).toBe(true);
  // The Dark preset is the concept's palette.
  expect(await page.evaluate(() => (window as any).LumaKit.theme.token('bg'))).toBe('#121212');
  expect(errors).toEqual([]);
});

test('every tab carries the gold dot; a tap on it opens the window menu', async ({ page }) => {
  await open(page);
  const tabs = page.locator('.panel-tabs .ptab');
  expect(await tabs.count()).toBeGreaterThan(0);
  await expect(page.locator('.panel-tabs .ptab .tab-dot')).toHaveCount(await tabs.count());
  await tap(page, page.locator('.ptab.on .tab-dot').first());
  await expect(menu(page)).toBeVisible();
  await expect(menu(page).locator('.ci', { hasText: 'Split right' })).toBeVisible();
  await expect(menu(page).locator('.ci', { hasText: 'Add window' })).toBeVisible();
  // A tap elsewhere closes it.
  await tap(page, page.locator('#cb-line'));
  await expect(menu(page)).toBeHidden();
});

test('holding a tab opens the window menu (the ring shows while held)', async ({ page }) => {
  await open(page);
  const tab = page.locator('.ptab.on').first();
  const box = (await tab.boundingBox())!;
  await page.mouse.move(box.x + box.width - 12, box.y + box.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(300);
  await expect(page.locator('.lc-ring')).toHaveCount(1);
  await page.waitForTimeout(400);
  await page.mouse.up();
  await expect(menu(page)).toBeVisible();
  await expect(page.locator('.lc-ring')).toHaveCount(0);
  await page.keyboard.press('Escape');
});

test('the Showroom window shows the kit; presets preview on the page', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const tab = page.locator('.ptab', { hasText: 'Showroom' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tap(page, tab);
  await expect(page.locator('[data-kit="showroom"]')).toBeVisible();
  await expect(page.locator('[data-kit="showroom"] luma-button')).not.toHaveCount(0);
  await tap(page, page.locator('[data-kit="showroom"] luma-button[label="Daylight"] button'));
  expect(await page.evaluate(() => (window as any).LumaKit.theme.token('bg'))).toBe('#e4e4e4');
  await tap(page, page.locator('[data-kit="showroom"] luma-button[label="Dark"] button'));
  expect(await page.evaluate(() => (window as any).LumaKit.theme.token('bg'))).toBe('#121212');
  // The hold tile: hold → menu.
  await hold(page, page.locator('.lc-sr-hold'));
  await expect(menu(page)).toBeVisible();
  await page.keyboard.press('Escape');
});

test('a split handle drags with the pointer core and saves one layout step', async ({ page }) => {
  await open(page);
  // A feedback card (someone connected) can sit over the tab row on a phone: close any first.
  for (const x of await page.locator('.fb-card .fb-x').all()) await x.click().catch(() => {});
  await expect(page.locator('.fb-card')).toHaveCount(0);
  // Split the focused panel through the window menu.
  await tap(page, page.locator('.ptab.on .tab-dot').first());
  await tap(page, menu(page).locator('.ci', { hasText: 'Split right' }));
  const handle = page.locator('.snode > .lc-split-h').first();
  await expect(handle).toBeVisible();
  // Touch: only the grip pill is a target (§30 "12e hardening"); a mouse grabs the line.
  const touch = await coarse(page);
  const grab = touch ? handle.locator('.lc-split-pill') : handle;
  if (touch) await expect(grab).toBeVisible();
  else await expect(handle.locator('.lc-split-pill')).toBeHidden();
  const before = await page.locator('.schild').first().boundingBox();
  const hb = (await grab.boundingBox())!;
  await page.mouse.move(hb.x + hb.width / 2, hb.y + hb.height / 2);
  await page.mouse.down();
  await page.mouse.move(hb.x - 80, hb.y + hb.height / 2, { steps: 6 });
  await page.mouse.up();
  const after = await page.locator('.schild').first().boundingBox();
  expect(after!.width).toBeLessThan(before!.width - 40);
  // Saved: the server's screen has the new ratio.
  await expect.poll(async () => {
    const s = await page.evaluate('fetch(`/api/screens/${liveScreen}`).then(r => r.json())') as any;
    return s.root?.type === 'split' ? Number(s.root.ratio) : 1;
  }).toBeLessThan(0.5);
  // A double-tap puts it back to 50 / 50 (one more step). The saved layout comes back from
  // the server and redraws the split, so a double-tap can meet a fresh handle (its second tap
  // is then a first): like a user, tap again until it takes.
  await expect.poll(async () => {
    const g2 = (await grab.boundingBox())!;
    await page.mouse.dblclick(g2.x + g2.width / 2, g2.y + g2.height / 2);
    await page.waitForTimeout(250);
    const s = await page.evaluate('fetch(`/api/screens/${liveScreen}`).then(r => r.json())') as any;
    return s.root?.type === 'split' ? Number(s.root.ratio) : 0;
  }, { timeout: 15_000 }).toBeCloseTo(0.5, 2);
});

test('touch screens get console-sized targets; nothing hides behind hover', async ({ page }) => {
  await open(page);
  test.skip(!(await coarse(page)), 'touch screens only');
  // One pane again: the split test's narrow panes are miniatures on a phone
  // (decisions §30 "12c2": drawn smaller, held to open — kit-12c2.spec).
  await page.evaluate(() => {
    const w = (0, eval);
    w('layout').root = { type: 'leaf', wins: Object.keys(w('layout').windows).map(Number), active: 0 };
    w('focusPath = []');
    w('commit()');
  });
  await expect(page.locator('.panel.lc-mini')).toHaveCount(0);
  // A phone's bar keeps only PURR · line · VK · ⋯ (the statuses in ⋯), and
  // its left sidebar folds into ⋯ too (§30 "12e").
  const phone = await page.evaluate(() => innerWidth <= 600);
  for (const sel of ['.ptab', phone ? '#cb-more' : '#sidebar-left .sl-btn', '#cmdbar .cb-icon:visible']) {
    const h = (await page.locator(sel).first().boundingBox())!.height;
    expect(h, sel).toBeGreaterThanOrEqual(44);
  }
  await tap(page, page.locator('.ptab.on .tab-dot').first());
  const row = (await menu(page).locator('.ci').first().boundingBox())!.height;
  expect(row).toBeGreaterThanOrEqual(44);
  // Fields use 16 px text (iOS never zooms on focus).
  expect(await page.evaluate(() => parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--lc-fs')))).toBeGreaterThanOrEqual(16);
});
