// The Patch setup dialog (decisions §34 "34g4 patch hardened" 9, 12; the locked mockup
// docs/dev/ui-kit/mockups/2026-10-18-34g4/, part 3b): `Menu "Patch"` draws Devices › DMX › Patch
// in <luma-setup> — the crumb, the kind rail, the tree sidebar (MVR tree / Class / Tag), the Patch
// grid on the `patch` feed. The entry (§34 "34g4 patch entry hardened", patch-lock.html): one Patch
// tile, the dialog opens Live, a hold on the claim button claims, a press or the ✕ leaves (asked). Part 3c: a tree node's ≡ menu
// (its lines from the feed) — Move… picks where and the station asks before nesting.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { slabCell, closeSheet } from '../support/slab';
import { REPO, Server, answer, cmd, connected, dialog, hold, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
const setup = (page: Page) => page.locator('luma-setup');
// The list is in tree order (§40 40d): by number for a test that names rows by their number.
const byNumber = (s: Locator) => s.locator('luma-grid').first().evaluate((g: any) => g.sortBy('no', 'asc'));
// The rig's grid is virtual (1 348 rows): scroll the patch grid so the row of a number is drawn.
// (A redraw puts its saved scroll back, so this scrolls until the row is in the DOM.)
const reveal = (s: Locator, no: number) => expect.poll(() => s.locator('luma-grid').first().evaluate((g: any, no: number) => {
  const rows = g.shownRows(), i = rows.findIndex((r: any) => String(r.no) === String(no));
  if (i < 0) return 'no such row';
  g.querySelector('[data-scroll]').scrollTop = Math.max(0, i - 2) * g.rowH;
  return g.querySelector(`.lc-grid-row[data-row="${CSS.escape(g.keyOf(rows[i]))}"]`) ? 'drawn' : 'scrolled';
}, no), { timeout: 10_000 }).toBe('drawn');
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/patch-${name}.png`) });

test('the Patch tile opens Live; hold the claim button to claim, press or ✕ to leave', async ({ page }) => {
  // A phone folds the crumb, the rail and the sidebar and claims from the bar's lock cell
  // (decisions §37 1): phone.spec.ts covers that flow.
  test.skip(test.info().project.name === 'phone', 'the phone layout: phone.spec.ts');
  await open(page);
  // One Patch tile on the Menu (decisions §34 "34g4 patch entry hardened").
  await page.evaluate(() => (window as any).openMenu());
  await expect(page.locator('#menu-tiles .lc-tile-label', { hasText: 'Live Patch' })).toHaveCount(0);
  await page.locator('#menu-tiles [data-tile="patch"]').click();
  const s = setup(page);
  await expect(s.locator('.lc-setup-crumb')).toHaveText(/Devices\s*›\s*DMX\s*›\s*Patch/);
  await expect(s.locator('.lc-setup-rail button.on')).toHaveText('DMX');
  await expect(s.locator('.lc-setup-gby:not(.sub) button.on')).toHaveText('MVR tree');
  // The rig's rows (the grid draws the rows in view).
  await expect.poll(() => s.locator('luma-grid [data-col="name"]').count()).toBeGreaterThan(5);
  await expect(s.locator('[data-chip="area"]')).toContainText('Live patch · nobody has claimed it');
  const power = s.locator('luma-power:visible .lc-power');
  await expect(power).toContainText('Live');
  await expect(power).toContainText('Hold to claim and edit');
  await shot(page, 'live');
  // Slab states in Live (§34 round 3, 18): what makes, copies or removes rows needs the claim and
  // says why on a tap (a word the object lacks keeps its own reason: no Copy Fixture yet); Paste
  // is greyed with an empty clipboard; Select, Oops work.
  // The visible half of the slab (a phone shows New · Delete · Edit · Copy · Oops · Import).
  const cell = (id: string) => s.locator(`luma-slab [data-act="${id}"]:visible`);
  const phone = test.info().project.name === 'phone';
  for (const id of phone ? ['new', 'delete'] : ['new', 'cut', 'delete']) {
    await expect(cell(id)).toHaveAttribute('data-why', 'Claim the draft to change it');
  }
  if (!phone) await expect(cell('paste')).toHaveAttribute('data-why', /clipboard is empty|Claim the draft/);
  await expect(cell('oops')).not.toHaveAttribute('data-why', /./);
  await cell('new').click({ force: true });
  await expect(page.locator('.lc-toast').last()).toContainText('Claim the draft to change it');
  // The claim button (round 3, 9): the open lock in Live, a gold fill while held, drained on an
  // early release; the hold ring is drawn above every dialog.
  await expect(power.locator('luma-icon')).toHaveAttribute('name', 'lock-open');
  const box = (await power.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await expect(power).toHaveClass(/filling/);
  await page.mouse.up();
  await expect(power).not.toHaveClass(/filling/);
  await expect(power).toContainText('Live');
  expect(await page.evaluate(() => {
    const z = (sel: string) => { const el = document.createElement('div'); el.className = sel; document.body.append(el); const v = +getComputedStyle(el).zIndex; el.remove(); return v; };
    return z('lc-ring') > Math.max(z('lc-setup-scrim'), z('lc-dlg-scrim'), z('lc-card-scrim'), z('lc-toast'));
  })).toBeTruthy();
  // A press only says what a hold does.
  await power.click();
  await expect(s.locator('[data-chip="area"]')).toContainText('Live patch');
  // Hold: claimed, the draft is yours. (Held down until the station answers — a loaded WebKit
  // worker fires the 600 ms hold late; a release after a fired hold is swallowed anyway.)
  {
    const box = (await power.boundingBox())!;
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await expect(power).toHaveClass(/draft/, { timeout: 15_000 });
    await page.mouse.up();
  }
  await expect(s.locator('[data-chip="area"]')).toContainText('Devices draft · Admin');
  await expect(power).toHaveClass(/draft/);
  await expect(power).toContainText('Claimed');
  await shot(page, 'draft');
  // In your draft the slab works; Paste counts what the clipboard holds.
  await expect(cell('new')).not.toHaveAttribute('data-why', /./);
  if (!phone) {
    await cmd(page, 'Cut Fixture 101 Thru 102');
    await expect(cell('paste')).toContainText('Paste 2');
    await expect(cell('paste')).not.toHaveAttribute('data-why', /./);
  }
  // The Tag grouping: Direct | Deep under the switch.
  await s.locator('[data-gby="tag"]').click();
  await expect(s.locator('[data-deep]')).toHaveCount(2);
  await s.locator('[data-gby="tree"]').click();
  // A change, then a press: Leave the draft — Discard, back to Live.
  await cmd(page, `Set Fixture 103 "Note" "e2e ${Date.now()}"`);
  await power.click();
  await expect(dialog(page)).toContainText('Leave the draft');
  await answer(page, 'Discard');
  await expect(power).toContainText('Hold to claim and edit');
  // Claim again, change, ✕: the same question, and Save closes the dialog.
  await hold(page, power);
  await expect(power).toHaveClass(/draft/);
  await cmd(page, `Set Fixture 103 "Note" "e2e ${Date.now()}"`);
  await s.locator('[data-x]').click();
  await expect(dialog(page)).toContainText('Leave the draft');
  await answer(page, 'Save');
  await expect(s).toBeHidden();
  // Nothing claimed any more: the tile opens Live again.
  await page.evaluate(() => (window as any).openMenu());
  await page.locator('#menu-tiles [data-tile="patch"]').click();
  await expect(s.locator('[data-chip="area"]')).toContainText('nobody has claimed it');
});

test("another user's draft: the claim button shows theirs, greyed, and a hold takes nothing", async ({ page, browser }) => {
  // Bob is logged in on his own page (present, so the 5 s lost-holder watch keeps his claim).
  expect((await server.command('Store User "Bob"')).ok).toBeTruthy();
  expect((await server.command('Set User "Bob" "Rights" Setup')).ok).toBeTruthy();
  const bob = await (await browser.newContext()).newPage();
  await server.open(bob, '/', 'Bob');
  await connected(bob);
  await kbdMode(bob);
  await cmd(bob, 'Claim Root.ShowData.Devices');
  await open(page);
  await page.evaluate(() => (window as any).openMenu());
  await page.locator('#menu-tiles [data-tile="patch"]').click();
  const s = setup(page);
  const power = s.locator('luma-power:visible .lc-power');
  await expect(power).toContainText('Claimed · Bob');
  await expect(power).toHaveClass(/theirs/);
  await expect(power).toHaveAttribute('aria-disabled', 'true');
  await expect(s.locator('[data-chip="area"]')).toContainText('Bob');
  await shot(page, 'theirs');
  // A hold takes nothing: the draft stays Bob's.
  await hold(page, power);
  await expect(power).toContainText('Claimed · Bob');
  await cmd(bob, 'Unclaim Root.ShowData.Devices /Discard');
  await expect(power).toContainText('Hold to claim and edit');
  await bob.context().close();
});

test("a tree node's ≡ menu: Move… nests after the question, Rename, New group here", async ({ page }) => {
  await open(page);
  await cmd(page, 'Claim Root.ShowData.Devices');
  // The rig brings one layer ("Fixture Layer", with PIXSTICK): a stored group goes into the
  // first layer, so these are MvrGroup 2 and 3 at 1.2 and 1.3; "Rig" is MvrLayer 2, empty.
  await cmd(page, 'Store MvrLayer "Rig"');
  await cmd(page, 'Store MvrGroup "Truss 1"');
  await cmd(page, 'Store MvrGroup "Truss 2"');
  await cmd(page, 'Assign Fixture 101 Thru 102 At MvrGroup 3');
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  const menu = page.locator('[data-kit="menu"]:not([hidden])');
  // A phone keeps the tree in a sheet (decisions §37 1): the tree chip opens it; a pick closes it.
  const tree = async () => { if (!(await s.locator('.lc-setup-tree').count())) await s.locator('[data-treechip]').click(); };
  // Fixture Layer › PIXSTICK (1.1), Truss 1 (1.2), Truss 2 (1.3): pick Truss 2, its ≡ menu.
  await tree();
  await s.locator('.lc-tree-row[data-path="1.3"]').click();
  await tree();
  await expect(s.locator('.lc-tree-row[data-path="1.3"] .lc-tree-count')).toHaveText('2 fixtures');
  await tree();
  await s.locator('.lc-tree-menu').click();
  await expect(menu.locator('.chdr')).toHaveText('MvrGroup 3 · Truss 2');
  await expect(menu.locator('.ci.off', { hasText: 'Duplicate…' })).toHaveCount(1);
  await shot(page, 'node-menu');
  await menu.locator('.ci', { hasText: 'Move…' }).click();
  // Neither itself nor its own groups: the two layers, PIXSTICK, Truss 1 and the rig's 23 folders
  // (a folder is a node, §40 40d).
  await expect(page.locator('luma-popup .lc-pick-row[data-no]')).toHaveCount(27);
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: 'MvrGroup 2' }).click();
  await expect(dialog(page)).toContainText('MvrGroup 3 "Truss 2" becomes a child of MvrGroup 2 "Truss 1" (1 group, 2 fixtures)?');
  await answer(page, 'Nest');
  // Truss 2 is under Truss 1 now; Truss 1 counts its fixtures.
  await tree();
  await expect(s.locator('.lc-tree-row[data-path="1.2.1"]')).toContainText('Truss 2');
  await tree();
  await expect(s.locator('.lc-tree-row[data-path="1.2"] .lc-tree-count')).toHaveText('2 fixtures');
  // Rename and New group here, on Truss 1.
  await tree();
  await s.locator('.lc-tree-row[data-path="1.2"]').click();
  await tree();
  await s.locator('.lc-tree-menu').click();
  await menu.locator('.ci', { hasText: 'Rename' }).click();
  await dialog(page).locator('input').fill('Front truss');
  await answer(page, 'Please');
  await tree();
  await expect(s.locator('.lc-tree-row[data-path="1.2"]')).toContainText('Front truss');
  await tree();
  await s.locator('.lc-tree-menu').click();
  await menu.locator('.ci', { hasText: 'New group here' }).click();
  await dialog(page).locator('input').fill('Booms');
  await answer(page, 'Please');
  await tree();
  await expect(s.locator('.lc-tree-row[data-path^="1.2."]', { hasText: 'Booms' })).toHaveCount(1);
  await shot(page, 'nested');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Keep');
});

test('the one context menu on the patch grid: Live greys what needs the claim, a pick is one line', async ({ page }) => {
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await byNumber(s);
  // A row by its FID (the row's text starts with it; its address would match "104" too).
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) }).locator('[data-gutter]');
  const menu = page.locator('[data-kit="context"]');
  const item = (label: string) => menu.locator('.lc-cm-item', { hasText: label }).first();
  // Right-click a row (the first: the grid draws the rows in view): the common actions, Set
  // with the grid's columns, the fixture's own lines.
  await row(1).click({ button: 'right' });
  await expect(menu.locator('.lc-cm-head')).toContainText('1 Fixture · 1');
  // A phone's sheet folds Set to one line (decisions §37 7): open it.
  if (await menu.locator('[data-k="fold"]').count()) await menu.locator('[data-k="fold"]').click();
  await expect(menu.locator('.lc-cm-head')).toContainText('Live patch');
  // Live: the FID, Copy and Delete need the claim; the name and the address are live.
  await expect(item('Renumber…')).toHaveAttribute('data-why', 'Claim the draft to change it');
  await expect(item('Delete')).toHaveAttribute('data-why', 'Claim the draft to change it');
  // The Set row by its exact label (a phone lists the fixture's own lines, Next free address, first).
  const address = menu.locator('.lc-cm-item', { has: page.locator('.lc-cm-lbl', { hasText: /^Address$/ }) });
  await expect(address).not.toHaveAttribute('data-why', /./);
  await expect(address.locator('.lc-cm-val')).toContainText('1.001');
  await expect(item('Next free address')).not.toHaveAttribute('data-why', /./);
  await shot(page, 'context-live');
  // Set › Name: the dialog, then one line naming the row.
  const from = server.mark();
  await item('Name').click();
  await expect(menu).toHaveCount(0);
  await dialog(page).locator('input').fill('Front 1');
  await answer(page, 'Please');
  await server.waitSysmon(/\] Set Fixture 1 "Name" "Front 1" → ok/, from);
  await expect(s.locator('luma-grid')).toContainText('Front 1');
  // Esc closes it.
  await row(2).click({ button: 'right' });
  await expect(menu).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
});

test('the value dialog: Renumber and Address preview old → new, Apply sends one line', async ({ page }) => {
  await open(page);
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await byNumber(s);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  // The grid draws the rows in view, by number with the folders last: a renumbered row sits near
  // the end — scroll there and up, a screen at a time, until it is drawn.
  // (A redraw after the feed puts the grid back at its top: each try scrolls from the end anew.)
  const find = async (fid: number) => {
    for (let i = 0; i < 20; i++) {
      await s.locator('luma-grid').first().evaluate((g: any, i: number) => { const sc = g.querySelector('[data-scroll]'); sc.scrollTop = Math.max(0, sc.scrollHeight - sc.clientHeight - i * sc.clientHeight * 0.8); }, i);
      await page.waitForTimeout(300);
      if (await row(fid).count()) return true;
    }
    return false;
  };
  const menu = page.locator('[data-kit="context"]');
  const vd = page.locator('[data-kit="value"]');
  // Select 1 Thru 3 (click, Shift adds), the menu, Renumber….
  await row(1).locator('[data-col="mode"]').click();
  await row(2).locator('[data-col="mode"]').click({ modifiers: ['Shift'] });
  await row(3).locator('[data-col="mode"]').click({ modifiers: ['Shift'] });
  await row(3).locator('[data-col="mode"]').click({ button: 'right' });
  await menu.locator('.lc-cm-item', { hasText: 'Renumber…' }).click();
  await expect(vd).toBeVisible();
  await expect(vd.locator('.lc-vd-tg')).toContainText('Skip taken');
  // Skip taken jumps over the rig's 9996: from 9995 the rows split around it; a free run says so.
  await vd.locator('input').fill('9995');
  await expect(vd.locator('.lc-vd-hint')).toContainText('→ 9995 + 9997 Thru 9998 · split around taken numbers');
  await vd.locator('input').fill('20001');
  await expect(vd.locator('.lc-vd-hint')).toContainText('→ 20001 Thru 20003 · all free');
  // The grid behind it shows old → new; nothing is sent yet.
  await expect(s.locator('luma-grid .lc-gc-new').first()).toHaveText('20001');
  await shot(page, 'renumber');
  const from = server.mark();
  await vd.locator('input').press('Enter');
  await expect(vd).toHaveCount(0);
  await server.waitSysmon(/\] Move Fixture 1 Thru 3 At 20001 \/Skip → ok/, from);
  // (The feed brings the renumbered rows a moment later.)
  await expect.poll(() => find(20003), { timeout: 20_000 }).toBe(true);
  // The redraws settle, then the row is found once more where it stays.
  await page.waitForTimeout(1500);
  await expect.poll(() => find(20003), { timeout: 20_000 }).toBe(true);
  // Address: Patch to… on 20003 (an Aeco 30 at 1.087) previews its new address; an overlap would
  // be red.
  await row(20003).locator('[data-col="mode"]').click();
  await row(20003).locator('[data-col="mode"]').click({ button: 'right' });
  await menu.locator('.lc-cm-item', { hasText: 'Patch to…' }).click();
  await expect(vd.locator('input')).toHaveValue('1.087');
  await vd.locator('input').fill('1.001');
  await expect(vd.locator('.lc-vd-hint')).toContainText('overlaps 20001');
  await vd.locator('input').fill('109.001');
  await expect(vd.locator('.lc-vd-hint')).toContainText(/→ 109\.001–109\.0\d\d · all free/);
  await expect(row(20003).locator('.lc-gc-new')).toHaveText('109.001');
  await shot(page, 'address');
  const at = server.mark();
  await vd.locator('[data-apply]').click();
  await server.waitSysmon(/\] Assign Fixture 20003 At DMX 109\.001 → ok/, at);
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});
test('the Live / Draft grid: locks with the reason, the state bar, gold dots, red addresses, the tree', async ({ page }) => {
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await byNumber(s);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  // A phone shows the state as the title's chip and keeps the tree in a sheet (decisions §37 1).
  const desk = test.info().project.name !== 'phone';
  const state = async (bar: string, text: string, chip: string) => {
    if (desk) await expect(s.locator(`.lc-setup-bar.${bar}`)).toContainText(text);
    else await expect(s.locator('[data-chip="area"]')).toHaveText(chip);
  };
  // Live: the bar says what changes at once; Mode (draft only) is locked, header and cells; the
  // title shows where the dialog is.
  await state('live', 'Live patch', 'Live');
  await expect(s.locator('.lc-gh', { hasText: 'Mode' }).locator('luma-icon[name="lock"]')).toHaveAttribute('title', 'Claim the draft to change it');
  await expect(row(1).locator('[data-col="mode"]')).toHaveClass(/locked/);
  await expect(row(1).locator('[data-col="address"]')).not.toHaveClass(/locked/);
  if (desk) {
    await expect(s.locator('.lc-setup-home')).toHaveText('Root.ShowData.Devices.Dmx.Patch');
    await expect(s.locator('.lc-setup-treebar [data-add]').first()).toHaveClass(/off/);
    await expect(s.locator('.lc-tree-count').first()).toContainText('fixtures');
  }
  // Your draft: gold bar, every column open, a gold dot on what you changed.
  await cmd(page, 'Claim Root.ShowData.Devices');
  await state('draft', 'Your draft', 'Draft');
  await expect(row(1).locator('[data-col="mode"]')).not.toHaveClass(/locked/);
  await cmd(page, 'Set Fixture 2 "Name" "Rigged"');
  await expect(row(2).locator('[data-col="name"] .lc-gc-mine')).toHaveCount(1);
  await expect(row(1).locator('.lc-gc-mine')).toHaveCount(0);
  // An overlap turns both addresses red.
  await cmd(page, 'Assign Fixture 2 At DMX 1.010');
  await expect(row(1).locator('[data-col="address"]')).toHaveClass(/t-alert/);
  await expect(row(2).locator('[data-col="address"]')).toHaveClass(/t-alert/);
  if (desk) await expect(s.locator('.lc-setup-treebar [data-add]').first()).not.toHaveClass(/off/);
  await shot(page, 'draft-grid');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
  await state('live', 'Live patch', 'Live');
});

test('the function block: Patch to… opens the address dialog, Next free address and Info run in Live', async ({ page }) => {
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  // On a phone a cell that doesn't work now waits in the bar's More sheet (decisions §37 1).
  const fn = (id: string) => slabCell(s, `[data-id="${id}"]`);
  // Nothing selected: the block says so.
  await expect(await fn('patch_to')).toHaveAttribute('data-why', 'Select rows first');
  await closeSheet(page);
  await byNumber(s);
  await row(3).locator('[data-col="mode"]').click();
  await expect(await fn('patch_to')).not.toHaveAttribute('data-why', /./);
  // Patch to…: the value dialog with the address kind; Esc closes it unsent.
  await (await fn('patch_to')).click();
  const vd = page.locator('[data-kit="value"]');
  await expect(vd.locator('input')).toHaveValue('1.087');
  await page.keyboard.press('Escape');
  await expect(vd).toHaveCount(0);
  // Next free address: one line, in Live (an address is a live column).
  const from = server.mark();
  await (await fn('next_free')).click();
  await server.waitSysmon(/\] Assign Fixture 3 At DMX → ok/, from);
});

test("the Insert fixtures data: the show types, a library upload, one mode's channel map", async ({ page }) => {
  await open(page);
  const gdtf = await import('node:fs').then(fs => fs.readFileSync(path.join(REPO, 'tests/fixtures/AuraXB.gdtf')));
  const got = await page.evaluate(async bytes => {
    const json = async (r: Response) => ({ status: r.status, body: await r.json() });
    const types = await json(await fetch('/api/fixture-types'));
    const bad = await json(await fetch('/api/gdtf?name=notes.txt', { method: 'POST', body: new Uint8Array([1, 2, 3]) }));
    const up = await json(await fetch('/api/gdtf?name=Uploaded%20Aura.gdtf', { method: 'POST', body: new Uint8Array(bytes) }));
    const after = await json(await fetch('/api/fixture-types'));
    const map = await json(await fetch('/api/fixture-type?file=Uploaded%20Aura.gdtf&mode=1'));
    const none = await fetch('/api/fixture-type?no=1&mode=99');
    return { types, bad, up, after, map, none: none.status };
  }, [...gdtf]);
  // The demo show's type 1 with its modes and footprints; six fixtures use it.
  // The rig's types: the first is a folder's (23 used, no channels); the Photon has its 245.
  const photon = got.types.body.show.find((t: any) => /PHOTON/i.test(t.name));
  expect(photon.used).toBeGreaterThan(100);
  expect(photon.modes[0].footprint).toBeGreaterThan(0);
  // A file that isn't a fixture file is refused; a real one lands in the library.
  expect(got.bad.body.ok).toBe(false);
  expect(got.up.body).toEqual({ ok: true, name: 'Uploaded Aura.gdtf' });
  const lib = got.after.body.library.find((f: any) => f.file === 'Uploaded Aura.gdtf');
  expect(lib.manufacturer).toBeTruthy();
  // Its first mode's channel map, with the file's own details.
  expect(got.map.body.channels.length).toBeGreaterThan(0);
  expect(got.map.body.details.long_name).toContain('Aura');
  expect(got.none).toBe(404);
});

test('Insert fixtures…: choose a type and mode, place them, Create runs the one Store line', async ({ page }) => {
  test.skip(test.info().project.name === 'phone', 'the phone: one step a screen, phone.spec.ts (decisions §37 4)');
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  const ins = page.locator('[data-kit="insert"]');
  // In Live it needs the claim; in your draft New opens the dialog.
  await expect(s.locator('luma-slab [data-id="insert"]:visible')).toHaveAttribute('data-why', 'Claim the draft to change it');
  await cmd(page, 'Claim Root.ShowData.Devices');
  await s.locator('luma-slab [data-act="new"]:visible').click();
  await expect(ins).toBeVisible();
  // 1 Choose: a type with channels and a thumbnail (the rig's first type is a folder's, 0 ch): the Icarus, its first mode, the channel map.
  const trow = ins.locator('.lc-ins-types .lc-grid-row[data-row]', { hasText: /ICARUS/i }).first();
  const tno = (await trow.innerText()).trim().split(/\s+/)[0]; // the frozen No cell leads the row
  await trow.locator('[data-col="name"]').click();
  await expect(ins.locator('[data-mode="1"]')).toHaveAttribute('aria-selected', 'true');
  await expect(ins.locator('.lc-ins-map .lc-grid-row[data-row]').first()).toBeVisible();
  await ins.locator('[data-details]').click();
  // Details shows the file's thumbnail (the show's type: its embedded file).
  await expect.poll(() => ins.locator('.lc-ins-thumb').evaluate((img: HTMLImageElement) => img.naturalWidth).catch(() => 0)).toBeGreaterThan(0);
  await shot(page, 'insert-choose');
  await ins.locator('[data-next]').click();
  // 2 Place: two fixtures from 20301 (past the rig), at the first free address; the line shows before Create.
  await ins.locator('[data-f="qty"]').fill('2');
  await ins.locator('[data-f="fid"]').fill('20301');
  await ins.locator('[data-free="addr"]').first().click();
  await ins.locator('[data-f="name"]').fill('Side');
  const lineRe = new RegExp(`Store Fixture 20301 Thru 20302 At DMX (\\d+\\.\\d+) /FixtureType ${tno} /Mode 1`);
  await expect(ins.locator('.lc-ins-line code').first()).toHaveText(lineRe);
  const addr = (await ins.locator('.lc-ins-line code').first().innerText()).match(lineRe)![1];
  await expect(ins.locator('.lc-ins-land')).toContainText('all free');
  await shot(page, 'insert-place');
  const from = server.mark();
  await ins.locator('[data-create]').click();
  await expect(ins).toHaveCount(0);
  await server.waitSysmon(new RegExp(`\\] Store Fixture 20301 Thru 20302 At DMX ${addr.replace('.', '\\.')} /FixtureType ${tno} /Mode 1 → ok`), from);
  await server.waitSysmon(/\] Label Fixture 20301 Thru 20302 "Side" → ok/, from);
  await expect.poll(async () => { await reveal(s, 20302); return s.locator('luma-grid .lc-grid-row[data-row]', { hasText: /^\s*20302\b/ }).count(); }).toBe(1);
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});

test('the function block: Sort orders the grid, Offset spaces Next free, Multipatch… adds the next copies', async ({ page }) => {
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  // On a phone a cell may wait in the bar's More sheet (decisions §37 1).
  const fn = (id: string) => slabCell(s, `[data-id="${id}"]`);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  const desk = test.info().project.name !== 'phone';
  if (desk) {
    // Sort: a pick of the columns; the grid follows and the cell says it.
    await (await fn('sort')).click();
    await page.locator('luma-popup .lc-pick-row', { hasText: 'Address' }).click();
    await expect(await fn('sort')).toContainText('Sort · Address');
    // Offset 4: Next free address packs the rig's 28-channel Photons 32 apart.
    await (await fn('offset')).click();
    const pad = page.locator('luma-keypad');
    await pad.locator('.lc-kp-key[data-k="4"]').click();
    await pad.locator('.lc-kp-key[data-k="Please"]').click();
    await expect(await fn('offset')).toContainText('Offset · 4');
  }
  // The virtual grid draws the rows in view: the phone by number, then scrolled to 105 either way.
  if (!desk) await byNumber(s);
  await reveal(s, 105);
  await row(105).locator('[data-gutter]').click();
  await row(106).locator('[data-gutter]').click({ modifiers: ['Shift'] });
  let from = server.mark();
  await (await fn('next_free')).click();
  await server.waitSysmon(desk ? /\] Assign Fixture 105 (Thru|\+) 106 At DMX \/Every 32 → ok/ : /\] Assign Fixture 105 (Thru|\+) 106 At DMX → ok/, from);
  // Multipatch… needs the draft: the address dialog, then each fixture's next copy.
  await cmd(page, 'Claim Root.ShowData.Devices');
  await (await fn('multipatch')).click();
  const vd = page.locator('[data-kit="value"]');
  await expect(vd).toContainText('Multipatch at');
  await vd.locator('input').fill('39.001'); // universe 39 is free on the rig (3 is full)
  from = server.mark();
  await vd.locator('[data-apply]').click();
  await server.waitSysmon(/\] Store Fixture 105 (Thru|\+) 106 Multipatch At DMX 39\.001.* → ok/, from);
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});

test("your draft shows a violet dot where someone's live edit came in", async ({ page, browser }) => {
  // Ann (Setup) on her own page; Admin holds the draft.
  expect((await server.command('Store User "Ann"')).ok).toBeTruthy();
  expect((await server.command('Set User "Ann" "Rights" Setup')).ok).toBeTruthy();
  const ann = await (await browser.newContext()).newPage();
  await server.open(ann, '/', 'Ann');
  await connected(ann);
  await kbdMode(ann);
  await open(page);
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  // Ann renames 103 live: it reaches Admin's draft with a violet dot naming her.
  await byNumber(s);
  await cmd(ann, 'Set Fixture 103 "Name" "Ann was here"');
  await reveal(s, 103);
  await expect(row(103).locator('[data-col="name"] .lc-gc-theirs')).toHaveAttribute('title', 'Ann changed it live');
  await expect(row(103).locator('[data-col="name"]')).toContainText('Ann was here');
  await shot(page, 'violet');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
  await expect(row(103).locator('.lc-gc-theirs')).toHaveCount(0);
  await ann.context().close();
});

test('the Universes Map: usage, blocks, a parked overlap with its fixes, a free run patches the selection', async ({ page }) => {
  test.skip(test.info().project.name === 'phone', 'the phone: a list of universes, phone.spec.ts (decisions §37 5)');
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await s.locator('[data-tab="Universes"]').click();
  await s.locator('[data-view="map"]').click();
  const map = s.locator('luma-dmxmap');
  const blk = (fid: number) => map.locator(`.lc-dmap-blk[data-no="${fid}"]`);
  await expect(blk(101)).toBeVisible();
  await expect(map.locator('.lc-dmap-conflicts.ok')).toContainText('No overlaps');
  await expect(map.locator('.lc-dmap-name small').first()).toContainText('/ 512');
  // 102 onto 101 (wherever the rig has it — its block's title says): both parked, striped, listed with one-tap fixes.
  const at101 = (await blk(101).getAttribute('title'))!.match(/· (\d+\.\d{3})/)![1];
  await cmd(page, `Assign Fixture 102 At DMX ${at101}`);
  await expect(blk(102)).toHaveClass(/parked/);
  await expect(blk(101)).toHaveClass(/parked/);
  await expect(map.locator('.lc-dmap-crow')).toContainText('Fixture 101');
  await shot(page, 'map-conflict');
  let from = server.mark();
  await map.locator('.lc-dmap-fix.primary').click();
  await server.waitSysmon(/\] Assign Fixture 102 At DMX → ok/, from);
  await expect(map.locator('.lc-dmap-conflicts.ok')).toBeVisible();
  await expect(blk(101)).not.toHaveClass(/parked/);
  // Tap a block, then a free run: the selection is patched there (one line). The rig's widest
  // free run on a drawn bar (an earlier test's Next free may have filled universe 1's).
  await blk(104).click();
  await expect(blk(104)).toHaveClass(/sel/);
  from = server.mark();
  const widest = await map.evaluate((m: Element) => {
    const runs = [...m.querySelectorAll('.lc-dmap-free')] as HTMLElement[];
    const best = runs.sort((a, b) => b.getBoundingClientRect().width - a.getBoundingClientRect().width)[0];
    best.scrollIntoView({ block: 'center' });
    best.dataset.e2e = 'widest';
    return best.closest('.lc-dmap-bar')!.getAttribute('data-universe');
  });
  await map.locator('.lc-dmap-free[data-e2e="widest"]').click({ position: { x: 3, y: 20 } });
  await server.waitSysmon(new RegExp(`\\] Assign Fixture 104 At DMX ${widest}\\.\\d{3} → ok`), from);
  await shot(page, 'map');
});

test('Edit Patch: four ways in, a tap patches the current fixture, Probe and Find flash, closing ends it', async ({ page }) => {
  await open(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  // The tab is kept on the profile (the Map test left Universes).
  await s.locator('[data-tab="Patch"]').click();
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  const fn = (id: string) => s.locator(`luma-slab [data-id="${id}"]:visible`);
  const ep = page.locator('[data-kit="editpatch"]');
  const slot = (n: number) => ep.locator(`.lc-dmap-slot[data-slot="${n}"]`);
  // Known ground on the rig: 104 and 105 a footprint apart in universe 108 (26 of its slots used, at 481–506).
  await cmd(page, 'Assign Fixture 104 At DMX 108.043');
  await cmd(page, 'Assign Fixture 105 At DMX 108.071');
  // The virtual grid draws the rows in view: scroll to the row first (a phone's sticky header
  // can cover one, so the row lands two rows down).
  const tap = async (fid: number, opts = {}) => {
    await reveal(s, fid);
    await row(fid).locator('[data-gutter]').click(opts);
  };
  await s.locator('luma-grid').first().evaluate((g: any) => g.none?.());
  await tap(104);
  // Ctrl / Meta adds one row (Shift is a range, and an earlier test sorted by Address).
  await tap(105, { modifiers: ['ControlOrMeta'] });
  // 1 · The function block's Edit Patch… cell (a phone: the context menu's).
  if (test.info().project.name === 'phone') {
    await row(105).locator('[data-gutter]').click({ button: 'right' });
    await page.locator('[data-kit="context"] .lc-cm-item', { hasText: 'Edit Patch…' }).first().click();
  } else await fn('edit_patch').click();
  await expect(ep.locator('.lc-ep-chip')).toContainText('2 fixtures · 104 Thru 105');
  await expect(ep.locator('.lc-dmap-slot.cur').first()).toBeVisible();
  // A tap on a slot patches the current fixture there, then the next one is current.
  let from = server.mark();
  await slot(201).click();
  await server.waitSysmon(/\] Assign Fixture 104 At DMX 108\.201 → ok/, from);
  // The patch feed redraws the dialog (its list shows the new address) — tap again only after
  // that, or a slot pressed while it is replaced never gets its tap (WebKit on the rig).
  const listed = (fid: number) => ep.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  await expect(listed(104)).toContainText('108.201');
  // Skip patched (on): 105 has an address, so 104 stays current; off, the next one is.
  await expect(ep.locator('.lc-dmap-slot.cur .fid')).toHaveText('104');
  await ep.locator('[data-fn="skip"]').click();
  from = server.mark();
  // (A slot inside 104's own 28-channel block, 201–228, would tap the fixture: 241 is free.)
  await slot(241).click();
  await server.waitSysmon(/\] Assign Fixture 104 At DMX 108\.241 → ok/, from);
  await expect(listed(104)).toContainText('108.241');
  await expect(ep.locator('.lc-dmap-slot.cur .fid')).toHaveText('105');
  // Probe: a tap flashes any slot; the strip says so.
  from = server.mark();
  await ep.locator('[data-fn="probe"]').click();
  await slot(300).click();
  await server.waitSysmon(/\] Identify DMX 108\.300 → ok/, from);
  // Still running a moment later: the station's check knows Admin is here.
  await page.waitForTimeout(1500);
  expect(server.lines.slice(from).some(l => l.includes('Identify left on'))).toBeFalsy();
  await expect(ep.locator('[data-warn]')).toContainText('Probe 108.300');
  await expect(slot(300)).toHaveClass(/flash/);
  await shot(page, 'edit-patch-probe');
  // Find: the current fixture's footprint in its shape, a footprint along, Patch here.
  from = server.mark();
  await ep.locator('[data-fn="find"]').click();
  await server.waitSysmon(/\] Identify DMX 108\.071 Thru 108\.098 \/As Fixture 105 → ok/, from);
  // A footprint back: 108.043, where 104 was before it moved.
  await ep.locator('[data-step="-1"]').click();
  from = server.mark();
  await ep.locator('[data-here]').click();
  await server.waitSysmon(/\] Assign Fixture 105 At DMX 108\.043 → ok/, from);
  // Closing ends what it made flash.
  from = server.mark();
  await page.keyboard.press('Escape');
  await expect(ep).toHaveCount(0);
  await server.waitSysmon(/\] Off Identify Fixture Thru → ok/, from);
  await server.waitSysmon(/\] Off Identify DMX Thru → ok/, from);
  // 2 · The context menu's Edit Patch….
  await tap(106, { button: 'right' });
  await page.locator('[data-kit="context"] .lc-cm-item', { hasText: 'Edit Patch…' }).first().click();
  await expect(ep.locator('.lc-ep-chip')).toContainText('1 fixture · 106');
  await ep.locator('[data-x]').click();
  // 3 · The value dialog's Slot grid…: the typed universe kept (106 is still selected).
  await fn('patch_to').click();
  const vd = page.locator('[data-kit="value"]');
  await vd.locator('input').fill('2.010');
  await vd.locator('[data-more]').click();
  await expect(ep.locator('[data-uname]')).toContainText('Universe 2');
  await ep.locator('[data-x]').click();
  // 4 · The Map view's zoom in.
  await s.locator('[data-tab="Universes"]').click();
  await s.locator('[data-view="map"]').click();
  // A phone's map is a card per universe: a tap on it zooms in (decisions §37 5).
  await s.locator('.lc-dmap-zoom[data-zoom="1"], .lc-dmap-card[data-zoom="1"]').first().click();
  await expect(ep.locator('[data-uname]')).toContainText('Universe 1');
  await shot(page, 'edit-patch');
  await ep.locator('[data-x]').click();
});

test('the MVR tree (§40 40d): rows lifted onto a sidebar node go into it, Move into… picks the node, Delete of a node asks with both ways out', async ({ page }) => {
  const phone = test.info().project.name === 'phone';
  await open(page);
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  // The tab is kept on the profile (the Edit Patch test left Universes): the fixtures grid.
  await s.locator('[data-tab="Patch"]').click();
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  // A phone keeps the tree in a sheet (decisions §37 1): the tree chip opens it; a pick closes it.
  const tree = async () => { if (!(await s.locator('.lc-setup-tree').count())) await s.locator('[data-treechip]').click(); };
  const smoke = () => s.locator('.lc-setup-tree .lc-tree-row', { hasText: 'Smoke' });
  // The rig's folder "Smoke" (Fixture Media 9) holds the six Fog units; the pane lists fixtures only.
  await tree();
  await expect(smoke().locator('.lc-tree-count')).toHaveText('6 fixtures');
  await expect(smoke().locator('.lc-tree-id')).toHaveText('Media 9');
  await expect(s.locator('.lc-setup-tree .lc-tree-row', { hasText: 'Fog' })).toHaveCount(0);
  await closeSheet(page);
  await byNumber(s);
  await s.locator('luma-grid').first().evaluate((g: any) => g.none?.());
  if (!phone) {
    // 1 · Select two Photons and lift them onto Smoke: the chip names the landing, one line runs.
    await reveal(s, 107);
    await row(107).locator('[data-gutter]').click();
    await row(108).locator('[data-gutter]').click({ modifiers: ['Shift'] });
    const from = (await row(107).locator('[data-col="name"]').boundingBox())!;
    await smoke().scrollIntoViewIfNeeded(); // a tablet's sidebar scrolls: the node in view first
    const to = (await smoke().boundingBox())!;
    let mark = server.mark();
    await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
    await page.mouse.down();
    await page.mouse.move(from.x + from.width / 2 + 14, from.y + from.height / 2 + 2, { steps: 3 });
    await page.mouse.move(to.x + to.width / 2, to.y + to.height / 2, { steps: 12 });
    await expect(page.locator('[data-kit="chip"]')).toContainText('2 fixtures → Smoke');
    await expect(smoke()).toHaveClass(/drop/);
    await shot(page, 'tree-lift');
    await page.mouse.up();
    await server.waitSysmon(/\] Assign Fixture 107 (Thru|\+) 108 At Fixture Media 9 → ok/, mark);
    await expect(page.locator('[data-kit="chip"]')).toHaveCount(0);
    await expect(smoke().locator('.lc-tree-count')).toHaveText('8 fixtures');
    await s.locator('luma-grid').first().evaluate((g: any) => g.none?.());
  }
  // 2 · Move into…: the context menu's pick of a node, the same line.
  await reveal(s, 109);
  await row(109).locator('[data-gutter]').click();
  await row(109).locator('[data-gutter]').click({ button: 'right' });
  await page.locator('[data-kit="context"] .lc-cm-item', { hasText: 'Move into…' }).first().click();
  const pick = page.locator('luma-popup .lc-pick-row[data-no]');
  await expect(pick.first()).toBeVisible();
  let mark = server.mark();
  await pick.filter({ hasText: 'Fixture Media 9 · Smoke' }).click();
  await server.waitSysmon(/\] Assign Fixture 109 At Fixture Media 9 → ok/, mark);
  await tree();
  await expect(smoke().locator('.lc-tree-count')).toHaveText(phone ? '7 fixtures' : '9 fixtures');
  await closeSheet(page);
  // 3 · Delete of the node asks with both ways out; Keep lifts what is under it to the layer.
  await cmd(page, 'Delete Fixture Media 9');
  await expect(dialog(page)).toContainText(/Delete Fixture Media 9 "Smoke" — (7|9) fixtures under it\?/);
  await expect(dialog(page).locator('.lc-dlg-btn')).toHaveText(['Delete all', 'Keep them: move up to MvrLayer 1 "Fixture Layer"', 'Cancel']);
  await shot(page, 'node-delete-ask');
  mark = server.mark();
  await answer(page, 'Keep them');
  await server.waitSysmon(/\] Delete Fixture Media 9 \/KeepChildren \/NoConfirmation → ok/, mark);
  await tree();
  await expect(smoke()).toHaveCount(0);
  await closeSheet(page);
  expect((await server.command('Info Fixture 3001')).ok).toBeTruthy();
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});
