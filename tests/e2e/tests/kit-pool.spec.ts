// The unified pool window (decisions §22) on the kit's <luma-pool> (§30
// "12e"): the Groups window shows the server's cells live (the `pools` feed)
// and every gesture sends a real command — tap selects, a pending command
// takes the tapped object, a mouse drag moves, Ctrl-drag copies, a hold (or
// right-click) opens the object's sheet (Edit · Label · Store · Delete ·
// Move · Copy · Settings), its colour is the object's NodeMeta colour. On the
// desktop, a touch tablet, a phone and WebKit.

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
  // The Groups window may be a background tab: bring it to the front.
  if (!(await page.locator('.pool-win').count())) await page.locator('.ptab', { hasText: 'Groups' }).first().click();
  await expect(page.locator('.pool-win .lc-pool-cell:not(.empty)').first()).toBeVisible();
  // The window remembers its scroll: back to the top for every test.
  await reveal(page, 1);
}
const cell = (page: Page, n: number) => page.locator(`.panel .pool-win .lc-pool-cell[data-no="${n}"]`).first();
// The pool draws the rows in view: scroll a number into view first (the rig has groups 1–77;
// the tests' empty cells sit past them).
const reveal = async (page: Page, n: number) => {
  await page.locator('.panel .pool-win luma-pool').first().evaluate((p: any, n) => p.reveal?.(n), n);
  // The window saves its scroll row a moment later; a feed before that puts the old one back.
  await page.waitForTimeout(400);
};
// The one context menu (decisions §34 round 3, 18; it replaced the object sheet).
const menu = (page: Page) => page.locator('[data-kit="context"]');
const item = (page: Page, label: string) => menu(page).locator('.lc-cm-item', { hasText: label }).first();
async function holdCell(page: Page, n: number) {
  await reveal(page, n);
  const a = (await cell(page, n).boundingBox())!;
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(800);
  await page.mouse.up();
}
const desk = (page: Page) => page.evaluate(() => !(matchMedia('(any-pointer: coarse)').matches && innerWidth <= 600));
const ran = (re: RegExp, from: number) => server.waitSysmon(re, from);

test('the Groups window shows the cells: names, master badge and bar', async ({ page }) => {
  await open(page);
  await expect(cell(page, 1)).toContainText('Photon');
  await expect(cell(page, 2)).toContainText('Icarus');
  await expect(cell(page, 3)).toContainText('Blinder');
  // The desk design shows the badges and the bar (the phone matrix: number and name).
  if (await desk(page)) {
    await expect(cell(page, 1).locator('.lc-pool-badges')).toHaveText('Scaling');
    await expect(cell(page, 1).locator('.lc-pool-bar b')).toHaveText('100%');
  }
  await expect(page.locator('.pool-title .pt-info').first()).toContainText('DataPool 1');
});

test('a tap selects the group; the cell shows For All', async ({ page }) => {
  await open(page);
  const from = server.mark();
  await cell(page, 2).click();
  await ran(/\] Group 2 → ok/, from);
  if (!(await desk(page))) return;
  await expect(cell(page, 2).locator('.lc-pool-fit')).toHaveCount(1);
  // The level bar follows the master live.
  await cmd(page, 'Master Group 1 At 40');
  await expect(cell(page, 1).locator('.lc-pool-bar b')).toHaveText('40%');
});

test('a pending command takes the tapped group (Copy, then Delete)', async ({ page }) => {
  await open(page);
  let from = server.mark();
  await page.keyboard.type('Copy ', { delay: 5 });
  await cell(page, 3).click();
  await reveal(page, 85);
  await cell(page, 85).click();
  await ran(/\] Copy Group 3 At 85 → ok/, from);
  await reveal(page, 85);
  await expect(cell(page, 85)).toContainText('Blinder');
  from = server.mark();
  // The station asks before a Delete (R1-61); /nc answers it up front.
  await page.keyboard.type('Delete /nc ', { delay: 5 });
  await cell(page, 85).click();
  await ran(/\] Delete \/nc Group 85 → ok/, from);
  await reveal(page, 85);
  await expect(cell(page, 85)).toHaveClass(/empty/);
});

test('drag moves, Ctrl-drag copies', async ({ page }) => {
  await open(page);
  const drag = async (from: number, to: number, ctrl = false) => {
    const a = await cell(page, from).boundingBox();
    const b = await cell(page, to).boundingBox();
    if (!a || !b) throw new Error('no box');
    if (ctrl) await page.keyboard.down('ControlOrMeta');
    await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
    await page.mouse.down();
    await page.mouse.move(a.x + a.width / 2 + 20, a.y + a.height / 2, { steps: 3 });
    await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 5 });
    await page.mouse.up();
    if (ctrl) await page.keyboard.up('ControlOrMeta');
  };
  // From the rig's last row (77 "Snurr under", 76 "Fabric") to the empty row below it: both in view.
  await reveal(page, 86);
  let from = server.mark();
  await drag(77, 86);
  await ran(/\] Move Group 77 At 86 → ok/, from);
  await reveal(page, 86);
  await expect(cell(page, 86)).toContainText('Snurr under');
  await expect(cell(page, 77)).toHaveClass(/empty/);
  from = server.mark();
  await drag(76, 87, true);
  await ran(/\] Copy Group 76 At 87 → ok/, from);
  await reveal(page, 87);
  await expect(cell(page, 87)).toContainText('Fabric');
});

test('hold opens the context menu; Move from it picks the target with a tap', async ({ page }) => {
  await open(page);
  await holdCell(page, 2);
  await expect(menu(page)).toBeVisible();
  // The common actions, the Set part (Name with its value), the group's own lines, Delete last.
  await expect(menu(page).locator('.lc-cm-head')).toContainText('1 Group · 2');
  for (const l of ['Edit…', 'Renumber…', 'Label…', 'Move…', 'Select', 'Edit members', 'Info', 'Delete']) await expect(item(page, l)).toBeVisible();
  await expect(item(page, 'Name').locator('.lc-cm-val')).toContainText('Icarus');
  await item(page, 'Move…').click();
  await expect(menu(page)).toHaveCount(0);
  await expect(page.locator('.lc-pool-banner')).toContainText('Move Group 2: tap the target');
  const from = server.mark();
  await reveal(page, 88);
  await cell(page, 88).click();
  await ran(/\] Move Group 2 At 88 → ok/, from);
  await expect(page.locator('.lc-pool-banner')).toHaveCount(0);
  await reveal(page, 88);
  await expect(cell(page, 88)).toContainText('Icarus');
  // An empty slot's hold opens nothing.
  await reveal(page, 90);
  await holdCell(page, 90);
  await expect(menu(page)).toHaveCount(0);
});

test('a group takes its NodeMeta colour; Edit… in the menu is EditSetting', async ({ page }) => {
  await open(page);
  await cmd(page, 'Set Group 1 "Paint" "#3c8f3c"');
  await expect.poll(() => cell(page, 1).evaluate(el => el.style.getPropertyValue('--lc-obj'))).toContain('3c8f3c');
  await holdCell(page, 1);
  const from = server.mark();
  await item(page, 'Edit…').click();
  await ran(/\] EditSetting Group 1 → ok/, from);
  await page.keyboard.press('Escape');
});

test('right-click opens the menu; Edit members opens the group editor, which claims the group', async ({ page }) => {
  await open(page);
  // A six-member group of the test's own (the rig's groups have hundreds): 90 "Six" (a plain
  // select adds to what is selected: clear first).
  await cmd(page, 'ResetAll');
  await cmd(page, 'Fixture 101 Thru 106');
  await cmd(page, 'Store Group 90 "Six"');
  await cmd(page, 'ResetAll');
  await reveal(page, 90);
  await expect(cell(page, 90)).not.toHaveClass(/empty/);
  const from = server.mark();
  await cell(page, 90).click({ button: 'right' });
  await item(page, 'Edit members').click();
  await ran(/\] Edit Group 90 → ok/, from);
  const ed = page.locator('[data-kit="group-editor"]');
  await expect(ed.locator('luma-grid .lc-grid-row[data-row]')).toHaveCount(6);
  await expect(ed.locator('.lc-ged-cell')).toHaveCount(6);
  // ↓ on the first member: one Order line (the list follows the station).
  let mark = server.mark();
  await ed.locator('luma-grid .lc-grid-row[data-row]').first().locator('[data-down]').click();
  await ran(/\] Set Group 90 Fixture \d+ "Order" 2 → ok/, mark);
  // A tool: desk = the docked toolbar (Tools in the slab), phone = the low sheet.
  if (await desk(page)) {
    await ed.locator('luma-slab [data-id="tools"]:visible').first().click();
    await expect(ed.locator('[data-kit="toolbar"]')).toBeVisible();
    await expect(ed.locator('[data-kit="toolbar"] .lc-tool[data-tool="Linearize"] .lc-list-mark')).toHaveCount(1);
    mark = server.mark();
    await ed.locator('[data-kit="toolbar"] .lc-tool[data-tool="Linearize"]').click();
    await page.locator('[data-kit="tool-ask"] [data-run]').click();
    await ran(/\] Grid Group 90 "Linearize" "Number" → ok/, mark);
    await expect(ed.locator('[data-kit="toolbar"]')).toBeVisible(); // stays open
    // A canvas drag moves a member: Set Group … Fixture … X Y.
    const c = ed.locator('.lc-ged-cell').nth(5);
    const box = (await c.boundingBox())!;
    mark = server.mark();
    await page.mouse.move(box.x + 20, box.y + 20);
    await page.mouse.down();
    await page.mouse.move(box.x + 20, box.y + 20 + 44, { steps: 4 });
    await page.mouse.up();
    await ran(/\] Set Group 90 Fixture \d+ "Y" 1 → ok/, mark);
  } else {
    await ed.locator('[data-tools]').click();
    await expect(page.locator('[data-kit="tool-sheet"]')).toBeVisible();
    mark = server.mark();
    await page.locator('[data-kit="tool-sheet"] .lc-tool[data-tool="RemoveGaps"]').click();
    await ran(/\] Grid Group 90 "RemoveGaps" → ok/, mark);
    await expect(page.locator('[data-kit="tool-sheet"]')).toHaveCount(0); // closes after a tool
  }
  await ed.locator('.lc-close').first().click();
  await expect(ed).toHaveCount(0);
});
