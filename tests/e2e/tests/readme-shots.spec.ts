// The README's screenshots: `README_SHOTS=<dir> npx playwright test tests/readme-shots.spec.ts`
// writes them into <dir> (images/screenshots/ for the README); the normal e2e
// run skips it. A test station with the demo show, captured with Playwright
// at a fixed size so the pictures stay the same shape from one run to the next.

import { copyFileSync, mkdirSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, consentDialog, giveConsent, kbdMode } from '../support/server';

const OUT = process.env.README_SHOTS;
test.skip(!OUT, 'README screenshots: set README_SHOTS');
test.describe.configure({ mode: 'serial' });
test.setTimeout(5 * 60_000);

const server = new Server();

test.beforeAll(async () => {
  if (!OUT) return;
  mkdirSync(OUT, { recursive: true });
  server.dir = mkdtempSync(path.join(tmpdir(), 'lumacore-readme-'));
  mkdirSync(path.join(server.dir, 'library/shows'), { recursive: true });
  copyFileSync(path.join(REPO, 'library/shows/demo.luma'), path.join(server.dir, 'library/shows/demo.luma'));
  await server.start();
  const loaded = await server.command('LoadShow "demo" /NoConfirmation');
  expect(loaded.message).toMatch(/loaded/);
});
test.afterAll(async () => { if (OUT) await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction('typeof screenFeed !== "undefined" && screenFeed.live !== null && screenFeed.live === liveScreen');
}

async function shot(page: Page, name: string) {
  await page.waitForTimeout(900); // feeds (DMX, health) draw once
  await page.screenshot({ path: path.join(OUT!, `${name}.png`) });
}

/** Wait until a grid has drawn rows from its feed. */
async function rows(page: Page, grid = 'luma-device-grid') {
  await expect(page.locator(`${grid} .lc-grid-row[data-row]`).first()).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(1500);
}

type Node = { type: 'leaf'; wins: number[]; active: number } | { type: 'split'; dir: 'h' | 'v'; ratio: number; children: [Node, Node] };
const leaf = (...wins: number[]): Node => ({ type: 'leaf', wins, active: 0 });
const split = (dir: 'h' | 'v', ratio: number, a: Node, b: Node): Node => ({ type: 'split', dir, ratio, children: [a, b] });

/** Lay the screen out: `wins` are window kinds by number, `root` the split tree. */
async function layout(page: Page, wins: Record<number, string>, root: Node) {
  await page.evaluate(([wins, root]) => {
    const w = (0, eval);
    const l = w('layout');
    l.windows = Object.fromEntries(Object.entries(wins).map(([no, kind]) => [no, { no: Number(no), kind, label: null, settings: {} }]));
    l.root = root;
    w('focusPath = []');
    w('commit()');
  }, [wins, root] as const);
  await expect(page.locator('.fb-card')).toHaveCount(0, { timeout: 15_000 });
}

test('desk', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await open(page);
  await cmd(page, 'Fixture 1 Thru 8 At 70');
  await layout(page, { 1: 'devices', 2: 'groups', 3: 'presets' }, split('h', 0.56, leaf(1), split('v', 0.62, leaf(2), leaf(3))));
  await rows(page);
  await page.locator('luma-device-grid').first().locator('xpath=ancestor::*[contains(@class,"win-wrap")]').getByText('Programmer only').click();
  await rows(page);
  await shot(page, 'desk');
});

test('patch', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await open(page);
  await cmd(page, 'Menu "Patch"');
  await expect(page.locator('luma-setup .lc-setup-crumb')).toHaveText(/Patch/, { timeout: 15_000 });
  await rows(page, 'luma-setup luma-grid');
  await shot(page, 'patch');
  await page.keyboard.press('Escape');
});

test('treeshell', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await open(page);
  await layout(page, { 1: 'terminal' }, leaf(1));
  await cmd(page, 'cls');
  await cmd(page, 'cd ShowData');
  await cmd(page, 'ls');
  await cmd(page, 'Fixture 1 Thru 8 At Full');
  await shot(page, 'treeshell');
});

test('sysmon', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await open(page);
  await layout(page, { 1: 'sysmon', 2: 'dmx' }, split('h', 0.5, leaf(1), leaf(2)));
  await shot(page, 'sysmon');
});

test('phone', async ({ browser }) => {
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, deviceScaleFactor: 3 });
  const page = await ctx.newPage();
  await open(page);
  await cmd(page, 'Fixture 1 Thru 8 At 70');
  await layout(page, { 1: 'devices' }, leaf(1));
  await rows(page).catch(() => page.waitForTimeout(4000));
  await shot(page, 'phone');
  await ctx.close();
});

test('consent', async ({ page }) => {
  // A fresh tab: the login surface; the login asks for the Meow License (decisions 0290).
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(server.url);
  await expect(page.locator('luma-surface')).toBeVisible();
  const pw = page.locator('luma-surface .lc-sf-pw');
  if (!(await pw.isVisible())) await page.locator('luma-surface luma-avatar[data-user="Admin"] button').click();
  await pw.fill('admin');
  await pw.press('Enter');
  await expect(consentDialog(page)).toBeVisible({ timeout: 15_000 });
  await shot(page, 'consent');
  await giveConsent(page);
});
