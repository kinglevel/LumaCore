// Tags in the page (decisions §12 "Tags hardened"): EditSetting of any show object has a Meta tab
// (its NodeMeta and read-only facts) and a Tags tab — the tag tree as the kit's <luma-tree>
// picker. A tick is `Assign Tag <path> At <object>`, an untick `Unassign …`, New tag here
// `Store Tag …`; the tree keeps its folds across the station's fresh fields.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, answer, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  // Numbered: the rig has its own tag 901 "Wash", so a bare Store Tag would go to 902 and up.
  for (const line of ['Store Tag 1 "All"', 'Store Tag 1 Tag "Lx1"', 'Store Tag 1.1 Tag "Right"', 'Store Tag 2 "Busking"', 'Assign Tag 1.1 At Fixture 101']) {
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
const sw = (page: Page) => page.locator('.lc-set-scrim');
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/tags-${name}.png`) });

test('EditSetting of a fixture has Meta and Tags tabs', async ({ page }) => {
  await open(page);
  await cmd(page, 'EditSetting Fixture 101');
  await expect(sw(page).locator('.lc-set-tabs button', { hasText: 'Meta' })).toBeVisible();
  await sw(page).locator('.lc-set-tabs button', { hasText: 'Meta' }).click();
  await expect(sw(page).locator('.lc-set-cell', { hasText: 'GUID' })).toBeVisible();
  await expect(sw(page).locator('.lc-set-cell', { hasText: 'Path' }).locator('.lc-set-value')).toHaveText(/Root\.ShowData\.Devices\.Dmx\.Patch\.Fixtures\.101/);
  await shot(page, 'meta');
  await sw(page).locator('.lc-set-tabs button', { hasText: 'Tags' }).click();
  const tree = sw(page).locator('luma-tree');
  await expect(tree.locator('.lc-tree-row')).toHaveCount(5);
  await expect(tree.locator('.lc-tree-row[data-path="1.1"] .lc-tree-box')).toHaveAttribute('data-state', 'on');
  await shot(page, 'picker');
});

test('a tick assigns, an untick unassigns, New tag here stores', async ({ page }) => {
  await open(page);
  await cmd(page, 'EditSetting Fixture 101');
  await sw(page).locator('.lc-set-tabs button', { hasText: 'Tags' }).click();
  const tree = sw(page).locator('luma-tree');
  let from = server.mark();
  await tree.locator('.lc-tree-row[data-path="2"] .lc-tree-box').click();
  await server.waitSysmon(/Assign Tag 2 At Fixture 101/, from);
  await expect(tree.locator('.lc-tree-row[data-path="2"] .lc-tree-box')).toHaveAttribute('data-state', 'on');
  from = server.mark();
  await tree.locator('.lc-tree-row[data-path="1.1"] .lc-tree-box').click();
  await server.waitSysmon(/Unassign Tag 1\.1 At Fixture 101/, from);
  await expect(tree.locator('.lc-tree-row[data-path="1.1"] .lc-tree-box')).toHaveAttribute('data-state', 'off');
  // Folding survives the station's fresh fields.
  await tree.locator('.lc-tree-row[data-path="1"] .lc-tree-fold').click();
  await expect(tree.locator('.lc-tree-row[data-path="1.1"]')).toHaveCount(0);
  from = server.mark();
  await tree.locator('.lc-tree-row[data-path="2"] .lc-tree-box').click();
  await server.waitSysmon(/Unassign Tag 2 At Fixture 101/, from);
  await expect(tree.locator('.lc-tree-row[data-path="1.1"]')).toHaveCount(0);
  // New tag here, under the highlighted tag.
  await tree.locator('.lc-tree-row[data-path="2"] .lc-tree-name').click();
  await tree.locator('[data-new]').click();
  const dlg = page.locator('[data-kit="dialog"]');
  await dlg.locator('input').fill('Encore');
  from = server.mark();
  // The dialog's own Please (on touch the built-in keyboard sits inside the dialog with its own).
  await answer(page, 'Please');
  await server.waitSysmon(/Store Tag 2 Tag "Encore"/, from);
  await expect(tree.locator('.lc-tree-row[data-path="2.1"] .lc-tree-name')).toHaveText('Encore');
  await shot(page, 'picker-after');
});

test('the Showroom shows luma-tree and luma-tags-pool', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const tab = page.locator('.ptab', { hasText: 'Showroom' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const trees = page.locator('[data-kit="showroom"] .lc-sr-tree luma-tree');
  await expect(trees).toHaveCount(2);
  await trees.first().scrollIntoViewIfNeeded();
  await expect(trees.nth(1).locator('.lc-tree-box[data-state="part"]')).toHaveCount(1);
  await shot(page, 'showroom');
  const pool = page.locator('[data-kit="showroom"] luma-tags-pool');
  await pool.scrollIntoViewIfNeeded();
  await expect(pool.locator('luma-grid .lc-grid-row[data-row]')).toHaveCount(7);
  await pool.locator('[data-members="Direct"]').click();
  await expect(pool.locator('luma-grid .lc-grid-row[data-row]')).toHaveCount(3);
  await shot(page, 'showroom-pool');
});

test('the Tags pool window: a tap picks, Direct | Deep, Run action selects', async ({ page }) => {
  expect((await server.command('Assign Tag 1.1.1 At Fixture 101')).ok).toBe(true);
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Tags"'));
  const tab = page.locator('.ptab', { hasText: 'Tags' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const pool = page.locator('luma-tags-pool');
  const tree = pool.locator('luma-tree');
  await expect(tree.locator('.lc-tree-row')).toHaveCount(6);
  await expect(pool.locator('.lc-tp-empty')).toHaveText(/Tap a tag/);
  // A tap picks the tag: one Set Self line (logged as its window), the members pane follows.
  let from = server.mark();
  await tree.locator('.lc-tree-row[data-path="1"] .lc-tree-name').click();
  await server.waitSysmon(/Set Window [\d.]+ "Tag" 1 → ok/, from);
  await expect(pool.locator('.lc-tp-title')).toHaveText('All · 1');
  // Deep (the default): Fixture 101 through Right.
  await expect(pool.locator('luma-grid .lc-grid-row[data-row]')).toHaveCount(1);
  // A narrow grid (phone, tablet) folds the columns past Name behind the row's chevron.
  const row = pool.locator('luma-grid .lc-grid-row[data-row]').first();
  if (await row.locator('.lc-gc-fold').count()) await row.locator('.lc-gc-fold').click();
  await expect(row).toContainText('from Right');
  await shot(page, 'pool-deep');
  from = server.mark();
  await pool.locator('[data-members="Direct"]').click();
  await server.waitSysmon(/Set Window [\d.]+ "Members" Direct → ok/, from);
  await expect(pool.locator('.lc-tp-empty')).toHaveText(/Deep adds the tags under it/);
  from = server.mark();
  await pool.locator('[data-members="Deep"]').click();
  await server.waitSysmon(/Set Window [\d.]+ "Members" Deep → ok/, from);
  await expect(pool.locator('luma-grid .lc-grid-row[data-row]')).toHaveCount(1);
  // Run action: the window's default, Select fixtures, deep.
  from = server.mark();
  await pool.locator('[data-run]').click();
  await server.waitSysmon(/\] Tag 1 \/R → ok/, from);
  // The picked tag's ≡ menu runs its lines.
  await tree.locator('.lc-tree-row[data-path="1"] [data-menu]').click();
  from = server.mark();
  await page.locator('#ctx .ci', { hasText: 'Settings' }).click();
  await server.waitSysmon(/EditSetting Tag 1 → ok/, from);
  await page.keyboard.press('Escape');
  await shot(page, 'pool');
});
