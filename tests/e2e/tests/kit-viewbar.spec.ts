// The view bar and the pool windows per pool (decisions 0287): the right sidebar rides the
// `viewbar` feed of this screen; a button holds any object (a view, a group, a sequence, a
// macro) drawn in its kind's colour, and a tap runs the object's default verb; an empty button
// only says so; the hold menu assigns (Assign… → a kind → an object); Assign + a pool tap + a
// button tap is one line; a Views pool cell dragged onto the bar assigns it; the Views pool
// draws each view's windows. On the desktop, a touch tablet, a phone and WebKit where the bar
// shows (a phone folds it into the strip's menu).

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, box, cmd, connected, hold, kbdMode } from '../support/server';

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
}
const bar = (page: Page) => page.locator('#sidebar-right');
const button = (page: Page, n: number) => page.locator(`#view-list .vbtn[data-slot="${n}"]`);
const ctx = (page: Page) => page.locator('#ctx');
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/viewbar-${name}.png`) });
/** The bar is in the sidebar; a phone hides the sidebar. */
const shown = async (page: Page) => bar(page).isVisible();

test('a stored view and an assigned sequence show on the bar; a tap selects the sequence', async ({ page }) => {
  await open(page);
  test.skip(!(await shown(page)), 'the bar folds away on a phone');
  await cmd(page, 'Store ViewButton 1 "Busk"');
  await expect(button(page, 1)).toContainText('Busk');
  await expect(button(page, 1)).toHaveClass(/filled/);
  await cmd(page, 'Store Sequence 901 "Chase"');
  await cmd(page, 'Assign Sequence 901 At ViewButton 2');
  await expect(button(page, 2)).toContainText('Chase');
  await expect(button(page, 2).locator('luma-icon')).toHaveAttribute('name', 'list-ordered');
  await shot(page, 'filled');
  // The tap is the sequence's own line (Select Sequence 901): the strip names it.
  await button(page, 2).click();
  await expect(page.locator('#tb-strip .lc-ss-facts')).toContainText(/Sequence: .*(Chase|901)/);
});

test('an empty button only says so: the screen keeps its windows', async ({ page }) => {
  await open(page);
  test.skip(!(await shown(page)), 'the bar folds away on a phone');
  const panes = await page.locator('.panel').count();
  const from = server.mark();
  await button(page, 3).click();
  await server.waitSysmon(/viewbutton 3/i, from);
  await page.waitForTimeout(300);
  await expect(page.locator('.panel')).toHaveCount(panes);
  await expect(button(page, 3)).not.toHaveClass(/filled/);
});

test('hold → Assign… → Group… puts a group on the button', async ({ page }) => {
  await open(page);
  test.skip(!(await shown(page)), 'the bar folds away on a phone');
  await hold(page, button(page, 4));
  await ctx(page).getByText('Assign…').click();
  await ctx(page).getByText('Group…').click();
  await expect(ctx(page)).toContainText('Group → ViewButton');
  await ctx(page).locator('.ci', { has: page.locator('.clbl', { hasText: /^1\b/ }) }).first().click();
  await expect(button(page, 4)).toHaveClass(/filled/);
  await expect(button(page, 4).locator('luma-icon')).toHaveAttribute('name', 'layout-grid');
});

test('Assign, a pool tap and a button tap are one line', async ({ page }) => {
  await open(page);
  test.skip(!(await shown(page)), 'the bar folds away on a phone');
  if (!(await page.locator('.pool-win').count())) await page.locator('.ptab', { hasText: 'Groups' }).first().click();
  const cell = page.locator('.panel .pool-win .lc-pool-cell[data-no="3"]').first();
  await expect(cell).toBeVisible();
  const from = server.mark();
  await page.keyboard.type('Assign ', { delay: 5 });
  await cell.click();
  await button(page, 5).click();
  await server.waitSysmon(/Assign Group 3 At ViewButton 5/, from);
  await expect(button(page, 5)).toContainText('Blinder');
});

test('a Views pool cell dragged onto the bar assigns the view; the cell draws its windows', async ({ page }) => {
  await open(page);
  test.skip(!(await shown(page)), 'the bar folds away on a phone');
  test.skip(await page.evaluate(() => matchMedia('(any-pointer: coarse)').matches), 'the pool drags with a mouse');
  await page.evaluate(() => (window as any).wsCommand('Store Window "Views"'));
  const views = page.locator('.panel .pool-win').filter({ has: page.locator('luma-pool') }).last();
  const cell = views.locator('.lc-pool-cell:not(.empty)', { hasText: 'Busk' }).first();
  await expect(cell).toBeVisible();
  await expect(cell.locator('.lc-pool-scr i').first()).toBeVisible();
  await expect(cell).toContainText('VB 1.1');
  await shot(page, 'views-pool');
  const a = await box(cell);
  const b = await box(button(page, 7));
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  await page.mouse.move(a.x + a.width / 2 + 20, a.y + a.height / 2 + 20, { steps: 4 });
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 8 });
  await expect(button(page, 7)).toHaveClass(/drop/);
  await page.mouse.up();
  await expect(button(page, 7)).toContainText('Busk');
  await expect(cell).toContainText('VB 1.7');
});

test('the Macros window is a pool of its own', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Macros"'));
  await expect(page.locator('.pool-win .pt-info', { hasText: 'Go' }).last()).toBeVisible();
});
