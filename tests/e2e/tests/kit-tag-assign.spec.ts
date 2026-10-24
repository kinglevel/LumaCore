// Assigning tags (decisions 0297, the approved mockup 2026-10-24-tag-assign): each row of the Tags
// window has a pool square. Assign + a square arms `Assign Tag 1 At`; a pool cell tap tags it and
// the line stays armed, the pools marking what carries the tag; an empty line + a square runs the
// tag's action. `Tag Group 3` opens the tag picker: each tick is one Assign / Unassign line, and a
// search that names no tag makes it (`Store Tag "x"; Assign Tag "x" At …`).

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  for (const line of ['Store Tag 1 "FOH"', 'Set Tag 1 "Paint" Blue', 'Store Tag 2 "Stage"', 'Assign Tag 2 At Fixture 101']) {
    const r = await server.command(line);
    expect(r.ok && !/E-[A-Z]+-\d{3}/.test(r.message ?? ''), `${line}: ${r.message}`).toBe(true);
  }
});
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
const line = (page: Page) => page.locator('#cb-line .xterm-rows').innerText().catch(() => '');
const tab = (page: Page, name: string) => page.locator('.ptab', { hasText: name }).first();
/** The Tags window (stored once, then brought to the front). */
async function tags(page: Page) {
  if (!(await tab(page, 'Tags').count())) await page.evaluate(() => (window as any).wsCommand('Store Window "Tags"'));
  await front(page, 'Tags');
}
async function front(page: Page, name: string) {
  await expect(tab(page, name)).toBeVisible({ timeout: 10_000 });
  if (!(await tab(page, name).evaluate(t => t.classList.contains('on')))) await tab(page, name).click();
}
const cell = (page: Page, n: number) => page.locator(`.panel .pool-win .lc-pool-cell[data-no="${n}"]`).first();
const picker = (page: Page) => page.locator('luma-tag-picker');

test('Assign + a tag square arms the line; a cell tags it, the line stays, the pools mark the carriers', async ({ page }) => {
  await open(page);
  await tags(page);
  const square = page.locator('luma-tags-pool .lc-tree-sq[data-sq="1"]');
  await expect(square).toBeVisible();
  await page.keyboard.type('Assign ', { delay: 5 });
  await square.click();
  await expect.poll(() => line(page)).toContain('Tag 1 At');
  await front(page, 'Groups');
  let from = server.mark();
  await cell(page, 3).click();
  await server.waitSysmon(/Assign Tag 1 At Group 3 → ok/, from);
  await expect(cell(page, 3)).toHaveClass(/tagged/);
  await expect(cell(page, 3).locator('.lc-pool-tick')).toBeVisible();
  await expect.poll(() => line(page)).toContain('Tag 1 At');
  from = server.mark();
  await cell(page, 4).click();
  await server.waitSysmon(/Assign Tag 1 At Group 4 → ok/, from);
  await expect(cell(page, 4)).toHaveClass(/tagged/);
  // Esc never clears the line: it stays armed. Clearing it ends it, and the marks go.
  await page.keyboard.press('Escape');
  await page.keyboard.press('Escape');
  await expect.poll(() => line(page)).toContain('Tag 1 At');
  await page.evaluate(() => (window as any).wsSend({ type: 'line_edit', start: 0, end: 9999, text: '' }));
  await expect.poll(() => line(page)).not.toContain('Tag 1 At');
  await front(page, 'Groups');
  await expect(page.locator('.pool-win .lc-pool-cell.tagged')).toHaveCount(0);
});

test('an empty line + a square runs the tag action', async ({ page }) => {
  await open(page);
  await tags(page);
  const from = server.mark();
  await page.locator('luma-tags-pool .lc-tree-sq[data-sq="2"]').click();
  await server.waitSysmon(/Tag 2 (\/R )?→ ok/, from);
});

test('Tag Group 3 opens the tag picker: a tick assigns, a new name makes the tag', async ({ page }) => {
  await open(page);
  await cmd(page, 'Tag Group 3');
  await expect(picker(page)).toBeVisible();
  await expect(picker(page).locator('[data-title]')).toHaveText('Tags · Group 3 "Blinder"');
  const row = (p: string) => picker(page).locator(`.lc-tree-row[data-path="${p}"]`);
  await expect(row('1').locator('.lc-tree-box')).toHaveAttribute('data-state', 'on');
  let from = server.mark();
  await row('2').locator('.lc-tree-box').click();
  await server.waitSysmon(/Assign Tag 2 At Group 3 → ok/, from);
  await expect(row('2').locator('.lc-tree-box')).toHaveAttribute('data-state', 'on');
  from = server.mark();
  await row('1').locator('.lc-tree-name').click();
  await server.waitSysmon(/Unassign Tag 1 At Group 3 → ok/, from);
  await expect(row('1').locator('.lc-tree-box')).toHaveAttribute('data-state', 'off');
  await picker(page).locator('.lc-tree-search input').fill('Riser');
  const make = picker(page).locator('[data-new]');
  await expect(make).toBeVisible();
  from = server.mark();
  await make.click();
  await server.waitSysmon(/Assign Tag "Riser" At Group 3 → ok/, from);
  await expect(picker(page).locator('.lc-tree-row', { hasText: 'Riser' }).locator('.lc-tree-box')).toHaveAttribute('data-state', 'on');
  await picker(page).locator('[data-done]').click();
  await expect(picker(page)).toHaveCount(0);
});

test('the Showroom opens the tag picker with three-way boxes', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  await front(page, 'Showroom');
  const btn = page.locator('[data-kit="showroom"] luma-button', { hasText: 'the tag picker' });
  await btn.scrollIntoViewIfNeeded();
  await btn.click();
  await expect(picker(page).locator('.lc-tree-box[data-state="part"]').first()).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(picker(page)).toHaveCount(0);
});
