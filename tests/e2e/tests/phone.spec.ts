// The phone round (decisions §37): a phone is a screen whose short side is under 500 px. The setup
// frame folds its chrome and its slab (1), on its side one row of chrome and the bar on the right
// (2), slab cells take short labels (3), Insert fixtures is one step a screen (4), the Universes
// map is a list (5), the slot grid rows of 10 (6), the context menu a bottom sheet (7). Each test
// sets its own screen; the desktop project runs them.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, hold, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const UP = { width: 390, height: 844 };
const SIDE = { width: 844, height: 390 };

async function open(page: Page, size: { width: number; height: number }) {
  await page.setViewportSize(size);
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
const setup = (page: Page) => page.locator('luma-setup');
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/phone-${name}.png`) });
const row = (page: Page, no: number) => setup(page).locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${no}\\b`) });

// The list is in tree order (§40 40d): by number for a test that names rows by their number.
const byNumber = (page: Page) => page.locator('luma-setup luma-grid').first().evaluate((g: any) => g.sortBy('no', 'asc'));

test('upright the setup frame folds its chrome into two rows and its slab into one bar (§37 1)', async ({ page }) => {
  await open(page, UP);
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  await byNumber(page);
  const s = setup(page);
  await expect(s).toHaveAttribute('data-phone', 'up');
  // No crumb, no rail, no sidebar, no state bar: the state is a chip, the tree a chip.
  await expect(s.locator('.lc-setup-crumb, .lc-setup-rail, .lc-setup-side, .lc-setup-bar')).toHaveCount(0);
  await expect(s.locator('[data-chip="area"]')).toHaveText('Live');
  // The numbered fixtures; the 23 folders are nodes (§40 40d).
  await expect(s.locator('[data-treechip]')).toHaveText('All · 1348');
  // The grid's selection bar is its No cell.
  await expect(s.locator('luma-grid .lc-grid-bar')).toBeHidden();
  // The first row (the grid draws the rows in view; 104 is far down the rig).
  await row(page, 1).locator('[data-gutter]').click();
  await expect(s.locator('[data-selmenu]')).toHaveText('1');
  // One bar: what works now (Live, one row: Patch to…, Edit Patch…), Oops, More, the lock.
  const bar = s.locator('.lc-slab-bar1.up');
  await expect(bar.locator('> .lc-slab-cell')).toHaveCount(4);
  await expect(bar.locator('[data-id="patch_to"]')).toBeVisible();
  await expect(bar.locator('[data-id="edit_patch"]')).toBeVisible();
  await expect(bar.locator('[data-act="oops"]')).toBeVisible();
  await expect(bar.locator('luma-power.lc-slab-lock')).toContainText('Live');
  await shot(page, 'up');
  // The list keeps the room: its top and the bar's leave more than half the screen.
  const g = (await s.locator('luma-grid').boundingBox())!;
  expect(g.height).toBeGreaterThan(UP.height * 0.6);
  // ⋯ More: every cell, the tab's and the rows'; a greyed one says why.
  await bar.locator('[data-more]').click();
  const more = page.locator('.lc-slab-sheet.more');
  await expect(more.locator('[data-id="insert"]')).toHaveAttribute('data-why', 'Claim the draft to change it');
  await expect(more.locator('[data-act="delete"]')).toHaveAttribute('data-why', 'Claim the draft to change it');
  await shot(page, 'up-more');
  await page.keyboard.press('Escape');
  await expect(more).toHaveCount(0);
  await expect(s).toBeVisible();
  // The state chip: what Live means, with the full claim button.
  await s.locator('[data-chip="area"]').click();
  const lock = page.locator('.lc-slab-sheet.lock');
  await expect(lock).toContainText('Hold Claim to change the rest');
  await shot(page, 'up-lock');
  await lock.locator('[data-close]').click();
  // The tree chip: the sidebar in a sheet; a picked node closes it and filters the list.
  await s.locator('[data-treechip]').click();
  const tree = page.locator('.lc-setup-treesheet');
  await expect(tree.locator('.lc-setup-gby:not(.sub) button.on')).toHaveText('MVR tree');
  await shot(page, 'up-tree');
  await tree.locator('[data-close]').click();
  await expect(tree).toHaveCount(0);
});

test('on its side the chrome is one row and the bar a column on the right (§37 2)', async ({ page }) => {
  await open(page, SIDE);
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  const s = setup(page);
  await expect(s).toHaveAttribute('data-phone', 'side');
  await expect(s.locator('.lc-setup-title [data-treechip]')).toBeVisible();
  const bar = s.locator('.lc-slab-bar1.side');
  await expect(bar).toBeVisible();
  const box = (await bar.boundingBox())!;
  expect(box.x + box.width).toBeGreaterThan(SIDE.width - 2);
  expect(box.x + box.width).toBeLessThan(SIDE.width + 1);
  expect(box.width).toBeGreaterThan(90);
  await shot(page, 'side');
});

test('slab cells take short labels when narrow and never run into each other (§37 3)', async ({ page }) => {
  await open(page, { width: 1024, height: 768 });
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  const s = setup(page);
  await expect(s).not.toHaveAttribute('data-phone', /./);
  const insert = s.locator('luma-slab .lc-slab-desk [data-id="insert"]');
  await expect(insert.locator('.l-short')).toBeVisible();
  await expect(insert.locator('.l-short')).toHaveText('Insert…');
  await expect(insert.locator('.l-full')).toBeHidden();
  // No label wider than its cell.
  const over = await s.locator('luma-slab .lc-slab-desk .lc-slab-cell').evaluateAll(els => els.filter(b => [...b.querySelectorAll('span')].some(x => x.offsetWidth && x.scrollWidth > b.clientWidth + 1)).map(b => b.textContent?.trim()));
  expect(over).toEqual([]);
  await shot(page, 'tablet-slab');
});

test('Insert fixtures on a phone: Type, then Mode, then Place, one screen each (§37 4)', async ({ page }) => {
  await open(page, UP);
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  const s = setup(page);
  await s.locator('.lc-slab-bar1 [data-id="insert"]').click();
  const ins = page.locator('[data-kit="insert"]');
  await expect(ins.locator('.lc-ins-ptitle')).toContainText('Type');
  await expect(ins.locator('[data-tomode]')).toBeDisabled();
  await shot(page, 'insert-type');
  // ⚙ More: the other filters in a sheet.
  await ins.locator('[data-pmore]').click();
  await expect(page.locator('.lc-ins-moresheet [data-view="cols"]')).toBeVisible();
  await page.locator('.lc-ins-moresheet [data-close]').click();
  // A type with channels (the rig's first type is a folder's, 0 ch): the Photon.
  await ins.locator('[data-ptype]', { hasText: 'PHOTON' }).first().click();
  await expect(ins.locator('.lc-ins-ptitle')).toContainText('Mode');
  await ins.locator('[data-chan]').click();
  await expect(ins.locator('.lc-ins-map')).toBeVisible();
  await shot(page, 'insert-mode');
  await ins.locator('[data-pmode]').first().click();
  await expect(ins.locator('.lc-ins-ptitle')).toContainText('Place');
  await expect(ins.locator('.lc-ins-line code').first()).toContainText(/^Store Fixture \d+ At DMX \/FixtureType \d+ \/Mode \d+/);
  await shot(page, 'insert-place');
  // ‹ Back goes to Mode.
  await ins.locator('[data-back]').click();
  await expect(ins.locator('.lc-ins-ptitle')).toContainText('Mode');
  await ins.locator('[data-x]').click();
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});

test('the Universes map on a phone is a list; a tap zooms into rows of 10 (§37 5, 6)', async ({ page }) => {
  await open(page, UP);
  await cmd(page, 'Menu "Patch" /Tab "Universes"');
  const s = setup(page);
  await s.locator('[data-view="map"]').click();
  const cards = s.locator('.lc-dmap-card');
  // Universe 1's card: the rig's slots in use of 512.
  await expect(cards.first()).toContainText(/\d{2,3} \/ 512/);
  await expect(cards.first()).toContainText(/\d+ fixtures/);
  await expect(s.locator('.lc-dmap-bar')).toHaveCount(0);
  await shot(page, 'map');
  await cards.first().click();
  const ep = page.locator('[data-kit="editpatch"]');
  await expect(ep.locator('.lc-dmap-slots.phone')).toBeVisible();
  // Ten a row, the full width, at least 44 px high.
  const row = ep.locator('.lc-dmap-srow').nth(1);
  await expect(row.locator('.lc-dmap-lab')).toHaveText('1:011');
  await expect(row.locator('.lc-dmap-slot')).toHaveCount(10);
  const b = (await row.locator('.lc-dmap-slot').first().boundingBox())!;
  expect(b.height).toBeGreaterThanOrEqual(44);
  const scroll = await ep.locator('.lc-dmap-scroll').evaluate(e => e.scrollWidth - e.clientWidth);
  expect(scroll).toBeLessThanOrEqual(1);
  await expect(ep.locator('.lc-ep-one')).toBeVisible();
  await shot(page, 'slots');
  await ep.locator('[data-x]').click();
});

test('the context menu on a phone is a bottom sheet with Set folded (§37 7)', async ({ page }) => {
  await open(page, UP);
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  await byNumber(page);
  await row(page, 1).locator('[data-gutter]').click();
  await row(page, 1).locator('[data-gutter]').click({ button: 'right' });
  const cm = page.locator('[data-kit="context"]');
  await expect(cm).toHaveClass(/sheet/);
  await expect(cm.locator('.lc-cm-head')).toContainText('1 Fixture · 1');
  // Set is one line until tapped; the object's own lines come before it.
  await expect(cm.locator('.lc-cm-set')).toBeHidden();
  await cm.locator('[data-k="fold"]').click();
  await expect(cm.locator('.lc-cm-set')).toBeVisible();
  // At most about two-thirds of the screen; the row it acts on stays in sight above it.
  const top = (await cm.boundingBox())!.y;
  expect(top).toBeGreaterThan(UP.height * 0.3);
  const r = (await row(page, 1).boundingBox())!;
  expect(r.y + r.height).toBeLessThanOrEqual(top);
  await shot(page, 'context');
  await cm.locator('[data-x]').click();
  await expect(cm).toHaveCount(0);
});

test('the lock cell: a tap says what Live means, a hold claims, a press on Claimed leaves (§37 1)', async ({ page }) => {
  await open(page, UP);
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  const s = setup(page);
  const lock = s.locator('.lc-slab-bar1 luma-power.lc-slab-lock .lc-power');
  await lock.click();
  await expect(page.locator('.lc-slab-sheet.lock')).toContainText('Live patch');
  await page.locator('.lc-slab-sheet.lock [data-close]').click();
  await hold(page, lock);
  await expect(s.locator('[data-chip="area"]')).toHaveText('Draft');
  await expect(lock).toHaveClass(/draft/);
  // In the draft Insert… works now: it joins the bar.
  await expect(s.locator('.lc-slab-bar1 [data-id="insert"]')).toBeVisible();
  // A tap on the lock cell says what the draft means; the full button there leaves.
  // The hold's release click is swallowed for 700 ms (pointer.js): the claim replaced the element
  // the finger was on, so its release never reached the page.
  await page.waitForTimeout(800);
  await lock.click();
  const sheet = page.locator('.lc-slab-sheet.lock');
  await expect(sheet).toContainText('Your draft');
  // Nothing changed: the press leaves at once (a changed draft asks Save / Discard, kit-34g4).
  await sheet.locator('luma-power .lc-power').click();
  await expect(s.locator('[data-chip="area"]')).toHaveText('Live');
});
