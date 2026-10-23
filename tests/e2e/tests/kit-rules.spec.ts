// Object Paint in the page and the Rules window (decisions 0294, the approved mockup
// docs/dev/ui-kit/mockups/2026-10-23-paint/04-paint.html), on the rig. Every colour comes resolved
// from the station (the colour this viewer sees): a pool cell wears it, a <luma-grid> row is tinted
// (the Tags window's members), the Tree window's row has its swatch and the inspector its Colour
// from, the Device Sheet's name its bar, a cue row its tint, a view bar button its colour. The Rules
// window (<luma-rules-window> on the `rules` feed): the rules in order, Which rule? frames the row
// that places a name, a drag to the top is `Move AttributeRule n At 1`, a pattern edited through the
// edit card is the Set line, Delete through the slab (the station asks). A kit file: desktop, tablet, phone and WebKit.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, answer, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  for (const line of ['Set Group 1 "Paint" "#3f73c8"', 'Set Fixture 101 "Paint" Red',
    'Store Tag 910 "Front"', 'Set Tag 910 "Paint" Green', 'Assign Tag 910 At Group 3']) {
    const r = await server.command(line);
    expect(r.ok, `${line}: ${r.message}`).toBe(true);
  }
});
test.afterAll(async () => { await server.cleanup(); });

const G1 = 'Root.ShowData.DataPools.1.Groups.1';
const HEX = /^#[0-9a-f]{6}$/i;
const isPhone = (name: string) => name === 'phone';
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/rules-${name}.png`) });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
/** One window of type `kind` for the whole file (the tests share one server and its layout), in front. */
async function windowOf(page: Page, kind: string, label: string) {
  const has = await page.evaluate(k => Object.values((window as any).eval('layout').windows).some((w: any) => w.kind === k), kind);
  if (!has) await page.evaluate(l => (window as any).wsCommand(`Store Window "${l}"`), label);
  const tab = page.locator('.ptab', { hasText: label }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
}
const prop = (l: Locator, name: string) => l.evaluate((el, n) => (el as HTMLElement).style.getPropertyValue(n).trim(), name);

test('a pool cell wears the paint; a list row is tinted by its tag', async ({ page }) => {
  await open(page);
  await windowOf(page, 'groups', 'Groups');
  const cell = page.locator('.panel .pool-win .lc-pool-cell[data-no="1"]').first();
  await expect(cell).toBeVisible();
  await expect.poll(() => prop(cell, '--lc-obj')).toBe('#3f73c8');
  // The Tags window: tag 910 "Front" (Green) on Group 3 → the member row tinted green.
  await windowOf(page, 'tags', 'Tags');
  const pool = page.locator('luma-tags-pool');
  await pool.locator('luma-tree .lc-tree-row[data-path="910"] .lc-tree-name').click();
  await expect(pool.locator('.lc-tp-title')).toContainText('Front');
  const row = pool.locator('luma-grid .lc-grid-row[data-row]').first();
  await expect(row).toHaveClass(/\blc-painted\b/);
  expect(await prop(row, '--lc-row-paint')).toMatch(HEX);
  // The tinted ground is the mix, not the bare surface.
  const bg = await row.evaluate(el => getComputedStyle(el).backgroundColor);
  expect(bg).not.toBe('rgba(0, 0, 0, 0)');
  await shot(page, 'tags-tint');
});

test('the Tree window: Group 1 has its swatch, the inspector says where its colour comes from', async ({ page }, info) => {
  await open(page);
  await windowOf(page, 'tree', 'Tree');
  const tw = page.locator('luma-tree-window').first();
  await tw.locator('luma-pathbar [data-edit]').click();
  await tw.locator('luma-pathbar input').fill(G1);
  await tw.locator('luma-pathbar input').press('Enter');
  const row = tw.locator(`.lc-tree-row[data-path="${G1}"]`);
  await expect(row.locator('.lc-tree-sw')).toBeVisible();
  await expect.poll(() => prop(row.locator('.lc-tree-sw'), '--lc-tree-sw')).toBe('#3f73c8');
  if (!isPhone(info.project.name)) {
    const from = tw.locator('.lc-tw-insp luma-inspector [data-row="Colour from"]');
    await expect(from).toContainText('#3f73c8');
    await expect(from.locator('.lc-tree-sw')).toBeVisible();
  }
  await shot(page, 'tree');
});

test('the Device Sheet: fixture 101\'s name carries its paint bar', async ({ page }, info) => {
  test.skip(isPhone(info.project.name), 'a phone draws Channel mode (decisions device-sheet-window)');
  await open(page);
  await cmd(page, 'Fixture 101');
  await cmd(page, 'At 50');
  await windowOf(page, 'devices', 'Device Sheet');
  const grid = page.locator('luma-device-grid').first();
  // Programmer only (the window bar's checkbox, one Set Self line): fixture 101 is the one row.
  const item = grid.locator('xpath=ancestor::*[contains(@class,"win-wrap")][1]').locator(':scope > luma-window-bar [data-id="devices/programmer_only"]');
  if ((await item.getAttribute('aria-pressed')) !== 'true') await item.click();
  await expect(item).toHaveAttribute('aria-pressed', 'true');
  const name = grid.locator('.lc-dg-name:not(.folder)');
  await expect(name).toHaveCount(1);
  await expect(name).toHaveClass(/\blc-painted\b/);
  expect(await prop(name, '--lc-name-paint')).toMatch(HEX);
  // The row itself stays untinted: the values keep their marks.
  await expect(grid.locator('.lc-grid-row.lc-painted')).toHaveCount(0);
  await shot(page, 'devices');
});

test('the cue sheet: a painted cue is tinted; the view bar button wears its group\'s paint', async ({ page }, info) => {
  await open(page);
  for (const line of ['Fixture 101 Thru 102', 'At 100', 'Store Sequence 902 Cue 1', 'At 40', 'Store Sequence 902 Cue 2',
    'Select Sequence 902', 'Set Sequence 902 Cue 2 "Paint" Theme "Dimmer"', 'Assign Group 1 At ViewButton 1']) {
    await page.evaluate(l => (window as any).wsCommand(l), line);
  }
  await windowOf(page, 'sequence', 'Sequence');
  const sheet = page.locator('.seq-win luma-cue-sheet').first();
  const cue2 = sheet.locator('.lc-cs-row[data-row="2"]');
  await expect(cue2).toHaveClass(/\blc-painted\b/);
  expect(await prop(cue2, '--lc-row-paint')).toMatch(HEX);
  await expect(cue2).toHaveAttribute('data-paint-from', /Dimmer/);
  await expect(sheet.locator('.lc-cs-row[data-row="1"]')).not.toHaveClass(/\blc-painted\b/);
  await shot(page, 'cues');
  if (await page.locator('#sidebar-right').isVisible()) {
    const btn = page.locator('#view-list .vbtn[data-slot="1"]');
    await expect(btn).toHaveClass(/filled/);
    await expect.poll(() => prop(btn, '--vb-kind')).toBe('#3f73c8');
  }
});

// ── The Rules window ──
async function rules(page: Page): Promise<Locator> {
  await open(page);
  await windowOf(page, 'rules', 'Rules');
  const rw = page.locator('luma-rules-window').first();
  await expect(rw.locator('.lc-grid-row[data-row]').first()).toBeVisible();
  return rw;
}
const ruleRows = (rw: Locator) => rw.locator('luma-grid .lc-grid-row[data-row]');
const patternOf = (row: Locator) => row.locator('[data-col="pattern"] .lc-gc-text');

test('the Rules window lists the rules in order, first match wins', async ({ page }) => {
  const rw = await rules(page);
  await expect(rw.locator('.lc-rules-cap')).toHaveText('Attribute rules · first match wins');
  await expect(rw.locator('.lc-rules-count')).toHaveText(/\d+ standard · \d+ learned/);
  expect(await ruleRows(rw).count()).toBeGreaterThan(10);
  await expect(ruleRows(rw).first().locator('.lc-gc.gutter')).toHaveText('1');
  // Rows in a feature group with a colour are tinted in it.
  expect(await rw.locator('.lc-grid-row.lc-painted').count()).toBeGreaterThan(0);
  await shot(page, 'window');
});

test('Which rule? frames the rule that places a name', async ({ page }) => {
  const r = await server.command('Store AttributeRule "Pixel*" FeatureGroup "FX"');
  expect(r.ok, r.message).toBe(true);
  const rw = await rules(page);
  await expect(ruleRows(rw).filter({ has: page.locator('[data-col="pattern"] .lc-gc-text', { hasText: /^Pixel\*$/ }) })).toHaveCount(1);
  const from = server.mark();
  await rw.locator('[data-which]').fill('PixelMSpeed');
  await rw.locator('[data-which]').press('Enter');
  await server.waitSysmon(/Set Window [\d.]+ "Which" "PixelMSpeed"/, from);
  await expect(rw.locator('.lc-rules-answer')).toBeVisible();
  await expect(rw.locator('.lc-rules-answer')).toContainText('PixelMSpeed');
  const hit = rw.locator('.lc-grid-row.lc-hit');
  await expect(hit).toHaveCount(1);
  await expect(patternOf(hit)).toHaveText('Pixel*');
  await expect(rw.locator('.lc-rules-answer')).toContainText(/Rule \d+/);
  await shot(page, 'which');
});

test('a row dragged to the top is one Move line', async ({ page }) => {
  await rules(page);
  // The window maximised (its own line), so the list has room for a drag on every screen.
  await page.evaluate(() => {
    const no = Object.entries((window as any).eval('layout').windows).find(([, w]: any) => w.kind === 'rules')![0];
    (window as any).wsCommand('Set Self "Maximized" On', Number(no));
  });
  await expect(page.locator('luma-maxbar')).toBeVisible();
  const rw = page.locator('luma-rules-window').first();
  const row = ruleRows(rw).filter({ has: page.locator('[data-col="pattern"] .lc-gc-text', { hasText: /^Pixel\*$/ }) });
  const no = (await row.locator('.lc-gc.gutter').textContent())!.trim();
  // The row just under the header (a phone's list shows few rows).
  await rw.locator('luma-grid .lc-grid-scroll').evaluate((s, k) => {
    const r = s.querySelector(`.lc-grid-row[data-row="${k}"]`)!.getBoundingClientRect();
    s.scrollTop += r.top - s.querySelector('.lc-grid-head')!.getBoundingClientRect().bottom;
  }, await row.getAttribute('data-row'));
  const box = (await row.locator('[data-col="family"], [data-col="pattern"]').first().boundingBox())!;
  const head = (await rw.locator('luma-grid .lc-grid-head').boundingBox())!;
  const from = server.mark();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 - 20, { steps: 4 });
  // Over the header row: the list scrolls to its top under the lifted row, then it lands on rule 1.
  await page.mouse.move(box.x + box.width / 2, head.y + 4, { steps: 8 });
  await expect.poll(() => rw.locator('luma-grid .lc-grid-scroll').evaluate(s => s.scrollTop)).toBe(0);
  await expect(ruleRows(rw).first()).toHaveClass(/\blc-drop\b/);
  await page.mouse.up();
  await server.waitSysmon(new RegExp(`Move AttributeRule ${no} At 1`), from);
  await expect(patternOf(ruleRows(rw).first())).toHaveText('Pixel*');
  await page.locator('luma-maxbar [data-go="close"]').click();
  await expect(page.locator('luma-maxbar')).toHaveCount(0);
});

test('a pattern edited in the edit card is the Set line; Delete on the slab', async ({ page }) => {
  const rw = await rules(page);
  const first = ruleRows(rw).first();
  await expect(patternOf(first)).toHaveText('Pixel*');
  let from = server.mark();
  await first.locator('[data-col="pattern"]').dblclick();
  const input = page.locator('[data-kit="dialog"] input[data-field="0"]');
  await expect(input).toBeVisible();
  await input.fill('Pixel*Speed');
  await input.press('Enter');
  await server.waitSysmon(/Set AttributeRule 1 "Pattern" "Pixel\*Speed"/, from);
  await page.keyboard.press('Escape');
  await expect(page.locator('.lc-card')).toHaveCount(0);
  await expect(patternOf(ruleRows(rw).first())).toHaveText('Pixel*Speed');
  // The row's number selects it; Delete names it.
  await ruleRows(rw).first().locator('.lc-gc.gutter').click();
  from = server.mark();
  await rw.locator('luma-slab [data-act="delete"]').filter({ visible: true }).first().click();
  await server.waitSysmon(/Delete AttributeRule 1 → asks/, from);
  // Deleting a rule is a station ask (its answer is the line).
  await answer(page, /^Delete/);
  await expect(patternOf(ruleRows(rw).first())).not.toHaveText('Pixel*Speed');
  await shot(page, 'deleted');
});
