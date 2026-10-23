// The theme editor (archive §30 "12g"; the approved ThemeBg board): `Theme` opens it on your
// profile's theme — the base, six adjusters (Background hue / sat. replacing Tint), the 14
// colours; a base tap and a slider's release are each one line and the page wears the result;
// Reset all; Pick on screen names the colour under the tap; the Showroom opens it on samples.

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
  await cmd(page, 'Theme');
  const ed = page.locator('[data-kit="theme-editor"]');
  await expect(ed).toBeVisible();
  return ed;
}
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/theme-${name}.png`) });
const token = (page: Page, name: string) => page.evaluate(n => getComputedStyle(document.documentElement).getPropertyValue(`--lc-${n}`).trim(), name);
/** Move a slider like a finger: the drag (input) previews, the release (change) is the line. */
async function slide(page: Page, ed: import('@playwright/test').Locator, name: string, value: number) {
  await ed.locator(`#te-${name}`).evaluate((el: HTMLInputElement, v) => {
    el.value = String(v);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

test('Theme opens the editor: the base, six adjusters, the 14 colours', async ({ page }) => {
  const ed = await open(page);
  await expect(ed.locator('[data-base="Dark"]')).toHaveClass(/\bon\b/);
  await expect(ed.locator('.lc-te-adj')).toHaveCount(6);
  await expect(ed.locator('#te-BackgroundHue')).toBeVisible();
  await expect(ed.locator('#te-BackgroundSaturation')).toBeVisible();
  await expect(ed.locator('.lc-te-colour')).toHaveCount(14);
  await expect(ed.locator('[data-readout]')).toContainText(':1');
  await shot(page, 'editor');
});

test('a base tap is one line and the page wears it', async ({ page }) => {
  const ed = await open(page);
  let from = server.mark();
  await ed.locator('[data-base="Daylight"]').click();
  await server.waitSysmon(/Theme "Base" Daylight/, from);
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'Daylight');
  await expect(ed.locator('[data-base="Daylight"]')).toHaveClass(/\bon\b/);
  from = server.mark();
  await ed.locator('[data-base="Dark"]').click();
  await server.waitSysmon(/Theme "Base" Dark/, from);
});

test('the background adjusters turn the neutral greys; the release is one line; Reset all', async ({ page }) => {
  const ed = await open(page);
  const before = await token(page, 'bg');
  let from = server.mark();
  await slide(page, ed, 'BackgroundHue', 210);
  await server.waitSysmon(/Theme "BackgroundHue" 210/, from);
  // The station's fresh fields redraw the editor: slide the next one on the new slider.
  await expect(ed.locator('[data-adj="BackgroundHue"] .lc-te-reset')).toBeEnabled();
  from = server.mark();
  await slide(page, ed, 'BackgroundSaturation', 80);
  await server.waitSysmon(/Theme "BackgroundSaturation" 80/, from);
  await expect.poll(() => token(page, 'bg')).not.toBe(before);
  // Text is not a neutral: it stays.
  await shot(page, 'blue');
  from = server.mark();
  await ed.locator('[data-reset-all]').click();
  await server.waitSysmon(/Reset (UserProfile \d+ )?Theme/, from);
  await expect.poll(() => token(page, 'bg')).toBe(before);
});

test('Pick on screen names the colour under the tap', async ({ page }) => {
  const ed = await open(page);
  await ed.locator('[data-pick]').click();
  // The editor steps aside with its hint; a tap in the middle of the screen is the pick.
  await expect(page.locator('.lc-te-pickhint')).toBeVisible();
  const vp = page.viewportSize()!;
  await page.mouse.click(vp.width / 2, vp.height / 2);
  await expect(ed).toBeVisible();
  await expect(page.locator('.lc-te-pickhint')).toHaveCount(0);
  await expect(ed.locator('.lc-te-note')).toContainText(/That is|derived/);
});

test('the Showroom opens the editor on samples', async ({ page }) => {
  await server.open(page);
  await connected(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  // The Showroom redraws while it loads: find the button when it is there, then press it.
  await expect(page.locator('luma-button[label="Theme editor"]')).toHaveCount(1);
  await page.evaluate(() => (document.querySelector('luma-button[label="Theme editor"]') as HTMLElement).click());
  await expect(page.locator('[data-kit="theme-editor"]')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.locator('[data-kit="theme-editor"]')).toHaveCount(0);
});
