// Suggestions above the command line bar (decisions §30 "12b2", the Suggest
// board): the kit's <luma-suggest> opens above the line after a short pause
// while typing (profile Suggestions Auto), only on Tab (TabOnly) or never
// (Off); the station ranks the items and draws the grey ghost text in the
// line; Tab or a tap takes the selected item (the station writes the line);
// ↑ ↓ move only while it is open; Enter runs the line as typed; Esc closes.
// On the desktop and WebKit (a one-row phone bar is 12e's).

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode, screenText } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });
test.beforeEach(({}, info) => {
  test.skip(info.project.name === 'tablet' || info.project.name === 'phone', 'desktop and WebKit');
});

const list = (page: Page) => page.locator('#cb-suggest');
const rows = (page: Page) => list(page).locator('.lc-sg-row');
const bar = (page: Page) => page.locator('#cb-line .xterm-rows');

async function open(page: Page) {
  await server.open(page, '/');
  await connected(page);
  await kbdMode(page);
}

/** Clear the command line (the station's Ctrl+C on a typed line). */
async function clearLine(page: Page) {
  await page.keyboard.press('Escape');
  await page.keyboard.press('Control+c');
  await expect.poll(async () => (await bar(page).innerText()).trim()).toMatch(/>$/);
}

test('typing opens the list after a pause; Tab takes the selected item; ghost text in the line', async ({ page }) => {
  await open(page);
  await page.keyboard.type('Store Gr', { delay: 20 });
  await expect(list(page)).toBeVisible();
  await expect(rows(page).first()).toContainText('Group');
  await expect(rows(page).first()).toHaveClass(/\bsel\b/);
  // The grey ghost text is the station's, in the line: `Gr` + `oup`.
  await expect(bar(page)).toContainText('Store Group');
  // The list sits above the command line bar.
  const lb = await list(page).boundingBox();
  const cb = await page.locator('#cb-line').boundingBox();
  expect(lb!.y + lb!.height).toBeLessThanOrEqual(cb!.y + 1);
  await page.keyboard.press('Tab');
  // Auto goes on: the list opens again on the groups and the line may already show the
  // first one's grey ghost (`1`).
  await expect.poll(async () => (await bar(page).innerText()).replace(/\s+$/, '')).toMatch(/Store Group(\s*1)?$/);
  // Auto goes on: now the groups to store into (the rig has 77: Group 1, not 10–19).
  await expect(rows(page).filter({ hasText: /Group 1(?!\d)/ })).toBeVisible();
  await clearLine(page);
  await expect(list(page)).toBeHidden();
});

test('a number after Store completes to Store Cue (S 1 → Store Cue 1)', async ({ page }) => {
  await open(page);
  await page.keyboard.type('S 1', { delay: 20 });
  await expect(list(page)).toBeVisible();
  await expect(rows(page).first()).toContainText('Store Cue 1');
  await page.keyboard.press('Tab');
  await expect.poll(async () => (await bar(page).innerText()).replace(/\s+$/, '')).toMatch(/Store Cue 1$/);
  await clearLine(page);
});

test('↑ ↓ move only while it is open; Esc closes; a tap takes a row', async ({ page }) => {
  await open(page);
  await page.keyboard.type('Group ', { delay: 20 });
  await expect(rows(page).filter({ hasText: /Group 2(?!\d)/ })).toBeVisible();
  const second = rows(page).nth(1);
  await page.keyboard.press('ArrowDown');
  await expect(second).toHaveClass(/\bsel\b/);
  await page.keyboard.press('ArrowUp');
  await expect(rows(page).first()).toHaveClass(/\bsel\b/);
  await page.keyboard.press('Escape');
  await expect(list(page)).toBeHidden();
  // Closed: the line keeps what was typed.
  await expect(bar(page)).toContainText('Group');
  await page.keyboard.type('2', { delay: 20 });
  await expect(list(page)).toBeVisible();
  await rows(page).filter({ hasText: /Group 2(?!\d)/ }).first().click();
  // Taken with its space (the rig has Group 20–29 too); the station's grey hint of the next
  // word (All, the first of the list) may follow it.
  await expect.poll(async () => (await bar(page).innerText()).replace(/\s+$/, '')).toMatch(/> Group 2( All)?$/);
  await clearLine(page);
});

test('Enter runs the line as typed, not the highlighted item', async ({ page }) => {
  await open(page);
  await page.keyboard.type('Fixture 10', { delay: 20 });
  await expect(list(page)).toBeVisible();
  const from = server.mark();
  await page.keyboard.press('Enter');
  await server.waitSysmon(/\] Fixture 10 →/, from);
  await expect(list(page)).toBeHidden();
  // The typed line ran, not the selected item: the bar is an empty prompt.
  await expect.poll(async () => (await bar(page).innerText()).trim()).toMatch(/>$/);
  // ↑ on a closed list is the station's (history, a focused panel).
  await page.keyboard.press('ArrowUp');
  await expect(list(page)).toBeHidden();
  await clearLine(page);
});

test('TabOnly opens it only on Tab; Off never', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Set UserProfile "Suggestions" TabOnly'));
  await expect.poll(() => page.evaluate('barStatus?.suggestions')).toBe('TabOnly');
  await page.keyboard.type('Group ', { delay: 20 });
  await page.waitForTimeout(600);
  await expect(list(page)).toBeHidden();
  // Tab opens it (several fit) …
  await page.keyboard.press('Tab');
  await expect(list(page)).toBeVisible();
  // … and takes the selected one; then it stays closed until the next Tab.
  await page.keyboard.press('Tab');
  await expect.poll(async () => (await bar(page).innerText()).replace(/\s+$/, '')).toMatch(/> Group 1$/);
  await expect(list(page)).toBeHidden();
  await clearLine(page);
  // One candidate: Tab takes it at once.
  await page.keyboard.type('Store Gro', { delay: 20 });
  await page.keyboard.press('Tab');
  await expect.poll(async () => (await bar(page).innerText()).replace(/\s+$/, '')).toMatch(/Store Group$/);
  await clearLine(page);
  await page.evaluate(() => (window as any).wsCommand('Set UserProfile "Suggestions" Off'));
  await expect.poll(() => page.evaluate('barStatus?.suggestions')).toBe('Off');
  await page.keyboard.type('Store Gr', { delay: 20 });
  await page.waitForTimeout(600);
  await expect(list(page)).toBeHidden();
  await clearLine(page);
  await page.evaluate(() => (window as any).wsCommand('Set UserProfile "Suggestions" Auto'));
  await expect.poll(() => screenText(page)).toMatch(/Suggestions = "Auto"/);
});
