// Step 12e (decisions §30 "12e hardening" … "12e design locked", the boards in
// docs/dev/ui-kit/mockups/2026-10-13-12e/): one hold with one owner — a held
// tab inside a miniature pane opens the pane full screen, never the window
// menu as well; the command line bar on a phone shrinks its prompt and keeps
// the end of a long line in view.

import path from 'node:path';
import { expect, test, type Page, type TestInfo } from '@playwright/test';
import { REPO, Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const phone = (info: TestInfo) => info.project.name === 'phone';

async function open(page: Page) {
  await server.open(page, '/', 'Admin');
  await connected(page);
  await kbdMode(page);
}

type Node = { type: 'leaf'; wins: number[]; active: number } | { type: 'split'; dir: 'h' | 'v'; ratio: number; children: [Node, Node] };
const leaf = (...wins: number[]): Node => ({ type: 'leaf', wins, active: 0 });
const split = (dir: 'h' | 'v', ratio: number, a: Node, b: Node): Node => ({ type: 'split', dir, ratio, children: [a, b] });

test('a held tab inside a miniature opens the pane full screen, not the window menu too', async ({ page }, info) => {
  test.skip(!phone(info), 'miniatures are drawn on the phone');
  await open(page);
  await page.evaluate(([root]) => {
    const w = (0, eval);
    w('layout').windows = { 1: { no: 1, kind: 'terminal', label: null, settings: {} }, 2: { no: 2, kind: 'groups', label: null, settings: {} }, 3: { no: 3, kind: 'sysmon', label: null, settings: {} } };
    w('layout').root = root;
    w('focusPath = []');
    w('commit()');
  }, [split('h', 0.33, leaf(2), split('h', 0.5, leaf(3), leaf(1)))] as const);
  await expect(page.locator('.panel.lc-mini')).toHaveCount(3);
  // The login's feedback cards sit over the top of the screen for a moment.
  await expect(page.locator('.fb-card')).toHaveCount(0, { timeout: 15_000 });
  const tab = page.locator('.panel', { has: page.locator('.win-wrap[data-win="2"]') }).locator('.ptab').first();
  const box = (await tab.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(800);
  await page.mouse.up();
  await expect(page.locator('luma-maxbar .lc-mx-title')).toContainText('Groups');
  await expect(page.locator('#ctx')).toBeHidden();
  await page.locator('luma-maxbar [data-go="close"]').click();
});

test('the phone command line keeps its > and the end of a long line in view', async ({ page }, info) => {
  test.skip(!phone(info), 'the narrow bar');
  await open(page);
  await page.locator('#cb-line').click();
  await page.keyboard.type('Group 3 Thru 7 At 50 Fade 2 Delay 1');
  const bar = page.locator('#cb-line .xterm-rows');
  await expect(bar).toContainText('Delay 1');
  await expect(bar).toContainText('>');
  await expect(bar).toContainText('‹');
});

test('a pool window keeps its scroll: the first visible row, reported quietly', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'one screen size is enough');
  await open(page);
  await page.evaluate(() => {
    const w = (0, eval);
    w('layout').windows = { 2: { no: 2, kind: 'groups', label: null, settings: {} } };
    w('layout').root = { type: 'leaf', wins: [2], active: 0 };
    w('focusPath = []');
    w('commit()');
  });
  const pool = page.locator('.pool-win[data-win="2"] luma-pool');
  await expect(pool.locator('.lc-pool-cell').first()).toBeVisible();
  await pool.evaluate((p: any) => p.scrollToRow(1));
  const saved = () => page.evaluate('fetch(`/api/screens/${liveScreen}`).then(r => r.json()).then(s => s.windows?.["2"]?.scroll?.v ?? 0)');
  await expect.poll(saved).toBe(1);
  await page.reload();
  await connected(page);
  await expect(pool.locator('.lc-pool-cell').first()).toBeVisible();
  await expect.poll(() => pool.evaluate((p: any) => p.firstRow())).toBe(1);
});

test('phone: the left sidebar folds into ⋯; a full-screen pane swipes and stops at the ends', async ({ page }, info) => {
  test.skip(!phone(info), 'the phone');
  await open(page);
  await expect(page.locator('#sidebar-left')).toBeHidden();
  await page.locator('#cb-more').click();
  const menu = page.locator('#ctx');
  await expect(menu).toContainText('Sidebar');
  await expect(menu).toContainText('LOCK');
  // The modes live in the top bar's strip (decisions top-bar-status-strip).
  await expect(page.locator('#tb-strip [data-key="blind"]')).toBeVisible();
  await page.keyboard.press('Escape');
  await page.evaluate(([root]) => {
    const w = (0, eval);
    w('layout').windows = { 1: { no: 1, kind: 'terminal', label: null, settings: {} }, 2: { no: 2, kind: 'groups', label: null, settings: {} }, 3: { no: 3, kind: 'sysmon', label: null, settings: {} } };
    w('layout').root = root;
    w('focusPath = []');
    w('commit()');
  }, [split('h', 0.33, leaf(2), split('h', 0.5, leaf(3), leaf(1)))] as const);
  await expect(page.locator('.fb-card')).toHaveCount(0, { timeout: 15_000 });
  const pane = page.locator('.panel', { has: page.locator('.win-wrap[data-win="2"]') });
  const box = (await pane.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height * 0.75);
  await page.mouse.down();
  await page.waitForTimeout(800);
  await page.mouse.up();
  const title = page.locator('luma-maxbar .lc-mx-title');
  await expect(title).toContainText('Groups');
  await expect(page.locator('luma-maxbar [data-go="prev"]')).toBeDisabled();
  // A finger swipes left: the next pane.
  await page.locator('.panel.max-win .panel-body').evaluate(async el => {
    const r = el.getBoundingClientRect();
    const at = (type: string, x: number) => el.dispatchEvent(new PointerEvent(type, { pointerId: 7, pointerType: 'touch', isPrimary: true, clientX: x, clientY: r.top + 100, bubbles: true }));
    at('pointerdown', r.right - 40);
    for (let x = r.right - 60; x > r.left + 20; x -= 40) at('pointermove', x);
    at('pointerup', r.left + 20);
  });
  await expect(title).not.toContainText('Groups');
  await page.locator('luma-maxbar [data-go="close"]').click();
});

test('the Showroom draws the miniature lists: dense text and the hybrid', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'one screen size is enough');
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const tab = page.locator('.ptab', { hasText: 'Showroom' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const mini = page.locator('[data-kit="showroom"] .lc-grid-mini');
  await expect(mini).toHaveCount(2);
  await expect(mini.first().locator('tr.run')).toHaveCount(1);
  await expect(mini.first().locator('tr.run .prog')).toHaveCount(1);
  await expect(mini.nth(1).locator('td.lvl.tone')).not.toHaveCount(0);
  await expect(mini.nth(1).locator('.sw')).not.toHaveCount(0);
});
