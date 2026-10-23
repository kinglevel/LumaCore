// Step 12c2 (decisions §30 "12c2 hardening" / "12c2 design locked", the boards
// in docs/dev/ui-kit/mockups/2026-10-13-12c2/): Users & Profiles asks
// nothing itself — New user, New profile, Delete user / profile are station
// asks (UserAsks board); the Menu, Shutdown and Backup grids are the kit's
// tile grid and fit every screen (MenuTiles); a phone shows the desk's panes
// in miniature, a hold opens one full screen with ‹ › ×, and with the
// profile's HoldFollows on every client of the screen — the desk too —
// maximises it (HoldFullscreen); a device picks its screen config
// (`Set Device "ScreenConfig"`). On the desktop, a touch tablet, a phone and
// WebKit.

import path from 'node:path';
import { slabCell } from '../support/slab';
import { expect, test, type Browser, type Page, type TestInfo } from '@playwright/test';
import { REPO, Server, connected, dialog, kbdMode } from '../support/server';
import { fitIssues } from '../support/fit';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const touch = (info: TestInfo) => info.project.name === 'phone' || info.project.name === 'tablet';
const phone = (info: TestInfo) => info.project.name === 'phone';
/** A name of its own per project (one server per project run, but kept tidy). */
const nm = (base: string, info: TestInfo) => `${base}${info.project.name.slice(0, 1).toUpperCase()}`;

async function open(page: Page, user = 'Admin') {
  await server.open(page, '/', user);
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction(() => (0, eval)('liveScreen && winFeed.sub && windowTypes.length') as boolean);
  // Cards (Connected …) would sit over the windows in the screenshots and taps.
  await page.evaluate(() => document.getElementById('fb-cards')?.replaceChildren());
}

async function station() { return (await fetch(`${server.url}/api/station`)).json(); }

/** Menu › Users & Profiles: the setup frame since 12d part 4 (a Users grid,
 *  a Profiles grid, the slab). */
async function people(page: Page) {
  await page.evaluate(() => (window as any).openMenu());
  await page.locator('#menu-tiles .lc-tile', { hasText: 'Users & Profiles' }).click();
  const el = page.locator('luma-setup');
  await expect(el.locator('luma-grid [data-col="name"]').first()).toBeAttached();
  return el;
}
/** The Users or Profiles tab. */
async function showList(page: Page, kind: 'users' | 'profiles') {
  const tab = page.locator(`luma-setup .lc-setup-tabs [data-tab="${kind === 'users' ? 'Users' : 'Profiles'}"]`);
  if (!(await tab.getAttribute('class'))?.includes('on')) await tab.click();
}
/** Select one row (its gutter) by name. */
async function pickRow(page: Page, kind: 'users' | 'profiles', text: string) {
  await showList(page, kind);
  const st = await station();
  const row = (kind === 'users' ? st.users : st.profiles).find((r: any) => r.name === text || r.name?.startsWith(text));
  await page.locator(`luma-setup luma-grid [data-gutter="${row.guid}"]`).click();
}
/** A slab cell: the standard New / Delete, or the tab's own (Rename…). */
/** On a phone the cell may sit in the bar's More sheet (decisions §37 1): it is opened. */
const act = (page: Page, label: string) => {
  const id = /^New/.test(label) ? 'new' : label === 'Delete' ? 'delete' : null;
  return slabCell(page.locator('luma-setup'), id ? `[data-act="${id}"]` : `[data-own]:has-text("${label}")`);
};

test('New user: the station asks; Create makes the user with their own copy of Default', async ({ page }, info) => {
  await open(page);
  const el = await people(page);
  const name = nm('Anna', info);
  // The slab's New runs the bare line (the station asks).
  await (await act(page, 'New user')).click();
  const d = dialog(page);
  await expect(d).toContainText('New user');
  await expect(d.locator('.lc-dlg-link')).toContainText('Own — a copy of Default');
  const nameField = d.locator('input[data-field="0"]');
  await nameField.fill(name);
  await d.locator('.lc-dlg-btn', { hasText: 'Create' }).click();
  await expect(d).toHaveCount(0);
  await expect.poll(async () => (await station()).users.find((u: any) => u.name === name)?.profile).toBe(name);
  const prof = (await station()).profiles.find((p: any) => p.name === name);
  expect(prof.used_by).toEqual([name]);
  expect(prof.locked).toBe(false);
  // The grid follows (the station feed).
  await showList(page, 'users');
  await expect(el.locator('luma-grid [data-col="name"]', { hasText: name })).toBeVisible();
});

test('New user sharing a profile: the Profile pick list', async ({ page }, info) => {
  await open(page);
  await people(page);
  const anna = nm('Anna', info), bea = nm('Bea', info);
  await page.evaluate(() => (window as any).wsCommand('Store User'));
  const d = dialog(page);
  await d.locator('input[data-field="0"]').fill(bea);
  await d.locator('.lc-dlg-link').click();
  // The pick list: Own first, then every profile but Default.
  const rows = page.locator('luma-popup .lc-pick-row[data-no]');
  await expect(rows.first()).toContainText('Own');
  await expect(page.locator('luma-popup .lc-pick-row[data-no]', { hasText: /^\s*\d*\s*Default\b/ })).toHaveCount(0);
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: anna }).click();
  await expect(d.locator('.lc-dlg-link')).toContainText(anna);
  await d.locator('.lc-dlg-btn', { hasText: 'Create' }).click();
  await expect.poll(async () => (await station()).users.find((u: any) => u.name === bea)?.profile).toBe(anna);
  expect((await station()).profiles.find((p: any) => p.name === anna).used_by.sort()).toEqual([anna, bea].sort());
});

test('New profile: from Default, then a copy of one (asked again with a pick list)', async ({ page }, info) => {
  await open(page);
  const el = await people(page);
  const stage = nm('Stage', info), copy = nm('Copy', info);
  await showList(page, 'profiles');
  await (await act(page, 'New profile')).click();
  let d = dialog(page);
  await expect(d).toContainText('New profile');
  await d.locator('input[data-field="0"]').fill(stage);
  await d.locator('[data-choice="From Default"]').click();
  await d.locator('.lc-dlg-btn', { hasText: 'Create' }).click();
  await expect.poll(async () => (await station()).profiles.some((p: any) => p.name === stage)).toBe(true);

  await page.evaluate(() => (window as any).wsCommand('Store UserProfile'));
  d = dialog(page);
  await d.locator('input[data-field="0"]').fill(copy);
  await d.locator('[data-choice="Copy of…"]').click();
  await d.locator('.lc-dlg-btn').first().click();
  // The second ask: which profile to copy (a link with its pick list).
  d = dialog(page);
  await expect(d.locator('.lc-dlg-link')).toBeVisible();
  await d.locator('.lc-dlg-link').click();
  await page.locator('luma-popup .lc-pick-row[data-no]', { hasText: stage }).click();
  await d.locator('.lc-dlg-btn').first().click();
  await expect.poll(async () => (await station()).profiles.some((p: any) => p.name === copy)).toBe(true);
});

test('Delete user: the ask says what goes; /WithProfile; the other session is logged out', async ({ page, browser }, info) => {
  const cid = nm('Cid', info);
  expect((await server.command(`Store User "${cid}" /Rights Program /Profile New`)).ok).toBe(true);
  // Cid logged in on another device (a second browser context).
  const other = await browser.newContext({ viewport: { width: 1000, height: 700 } });
  const theirs = await other.newPage();
  await open(theirs, cid);
  await open(page);
  await people(page);
  await pickRow(page, 'users', cid);
  await (await act(page, 'Delete')).click();
  const d = dialog(page);
  await expect(d).toContainText(`Delete User "${cid}"`);
  await expect(d).toContainText(`${cid} is logged in`);
  await expect(d.locator('.lc-dlg-rows')).toContainText('Logged in');
  await expect(d.locator('.lc-dlg-rows')).toContainText('Programmer');
  const toggle = d.locator('[data-toggle="0"]');
  await expect(toggle).toHaveAttribute('aria-checked', 'true');
  await d.locator('.lc-dlg-btn', { hasText: `Delete ${cid}` }).click();
  await expect.poll(async () => (await station()).users.some((u: any) => u.name === cid)).toBe(false);
  expect((await station()).profiles.some((p: any) => p.name === cid)).toBe(false);
  // Their page is back at the login surface.
  await expect(theirs.locator('luma-surface')).toBeVisible({ timeout: 10_000 });
  await other.close();
});

test('Delete profile in use is refused (names who uses it); Default is locked', async ({ page }, info) => {
  await open(page);
  await people(page);
  const anna = nm('Anna', info);
  const from = server.mark();
  await pickRow(page, 'profiles', anna);
  await (await act(page, 'Delete')).click();
  await server.waitSysmon(/E-USER-006/, from);
  await expect(dialog(page)).toHaveCount(0);
  // Default: the template, no rename / delete (greyed with the reason); the station refuses too.
  await pickRow(page, 'profiles', 'Default');
  // (A phone's slab keeps Rename… off its 2-column block.)
  if (!phone(info)) await expect(await act(page, 'Rename')).toHaveAttribute('data-why', /locked template/);
  await expect(await act(page, 'Delete')).toHaveAttribute('data-why', /locked template/);
  expect((await server.command('Label UserProfile 1 "Mine"')).message).toContain('E-USER-002');
  expect((await server.command('Delete UserProfile 1 /NoConfirmation')).message).toContain('E-USER-002');
});

test('the Menu and Shutdown tile grids fit the screen', async ({ page }, info) => {
  await open(page);
  const issues: string[] = [];
  // (Backup is a setup menu since 12d part 5: kit-12d.spec.ts.)
  for (const [label, open] of [['Menu', 'openMenu()'], ['Shutdown', 'openPowerMenu()']] as const) {
    await page.evaluate(open);
    const box = page.locator('.mo-overlay.open .mo-box');
    await expect(box).toBeVisible();
    await page.waitForTimeout(300);
    issues.push(...await fitIssues(page, label, { touch: touch(info) }));
    // Every tile inside the grid's box (it scrolls inside, never spills).
    const spill = await page.evaluate(() => {
      const grid = document.querySelector('.mo-overlay.open luma-tiles')!;
      const g = grid.getBoundingClientRect();
      return [...grid.querySelectorAll('.lc-tile')].filter(t => {
        const r = t.getBoundingClientRect();
        return r.left < g.left - 1 || r.right > g.right + 1;
      }).length;
    });
    expect(spill, `${label}: tiles outside the grid`).toBe(0);
    await page.evaluate(() => document.querySelectorAll('.mo-overlay.open').forEach(o => (window as any).closeMenuOverlay(o.id)));
  }
  expect(issues).toEqual([]);
  // The board's labels.
  await page.evaluate(() => (window as any).openMenu());
  await expect(page.locator('#menu-tiles .lc-tile-label')).toContainText(['Patch', 'Connectors', 'Desk lights', 'Network']);
});

// ── The desk on a phone (HoldFullscreen board) ──────────────────────────────

type Node = { type: 'leaf'; wins: number[]; active: number } | { type: 'split'; dir: 'h' | 'v'; ratio: number; children: [Node, Node] };
const leaf = (...wins: number[]): Node => ({ type: 'leaf', wins, active: 0 });
const split = (dir: 'h' | 'v', ratio: number, a: Node, b: Node): Node => ({ type: 'split', dir, ratio, children: [a, b] });
const DESK = split('v', 0.62, split('h', 0.3, leaf(2), split('h', 0.6, leaf(3), leaf(4))), split('h', 0.5, leaf(1), leaf(5)));
const DESK_WINDOWS: [number, string][] = [[1, 'terminal'], [2, 'groups'], [3, 'dmx'], [4, 'sysmon'], [5, 'realtime']];

async function layDesk(page: Page) {
  await page.evaluate(([root, windows]) => {
    const w = (0, eval);
    w('layout').windows = Object.fromEntries((windows as [number, string][]).map(([no, kind]) => [no, { no, kind, label: null, settings: {} }]));
    w('layout').root = root;
    w('focusPath = []');
    w('commit()');
  }, [DESK, DESK_WINDOWS] as const);
  await page.waitForFunction(() => (0, eval)('leaves().length') === 5);
}

async function deskPage(browser: Browser) {
  const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
  const desk = await ctx.newPage();
  await open(desk);
  return { ctx, desk };
}
const maxTitle = (page: Page) => page.locator('luma-maxbar .lc-mx-title');
/** Hold a pane still (the kit's hold: ≈ 0.4 s). */
async function holdPane(page: Page, no: number, ms = 700) {
  const pane = page.locator('.panel', { has: page.locator(`.win-wrap[data-win="${no}"]`) });
  const box = (await pane.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height * 0.75);
  await page.mouse.down();
  await page.waitForTimeout(ms);
  await page.mouse.up();
}

test('phone: the desk in miniature; a quick tap says "Too small", a hold opens the pane full screen; the desk follows', async ({ page, browser }, info) => {
  test.skip(!phone(info), 'the phone (tablets: the same rule, a smaller desk)');
  expect((await server.command('Set UserProfile 2 "HoldFollows" On')).ok).toBe(true);
  const { ctx, desk } = await deskPage(browser);
  await layDesk(desk);
  await open(page);
  await page.waitForFunction(() => (0, eval)('leaves().length') === 5);
  // Every pane in its place, drawn smaller.
  await expect(page.locator('.panel.lc-mini')).toHaveCount(5);
  // A quick tap on a tiny cell: the toast.
  const groups = page.locator('.panel', { has: page.locator('.win-wrap[data-win="2"]') });
  const gb = (await groups.boundingBox())!;
  await page.mouse.click(gb.x + gb.width / 2, gb.y + gb.height * 0.75);
  await expect(page.locator('.lc-toast.on')).toContainText('Too small — hold to open Groups');
  // Hold: full screen at real size, here and on the desk.
  await holdPane(page, 2);
  await expect(maxTitle(page)).toContainText('Groups');
  await expect(page.locator('.panel.lc-mini')).toHaveCount(0);
  await expect(maxTitle(desk)).toContainText('Groups', { timeout: 8000 });
  expect(await fitIssues(page, 'held', { touch: true })).toEqual([]);
  // › the neighbour pane (tree order), on both.
  await page.locator('luma-maxbar [data-go="next"]').click();
  await expect(maxTitle(page)).toContainText('DMX');
  await expect(maxTitle(desk)).toContainText('DMX', { timeout: 8000 });
  await page.locator('luma-maxbar [data-go="prev"]').click();
  await expect(maxTitle(desk)).toContainText('Groups', { timeout: 8000 });
  // × back to the whole screen, everywhere.
  await page.locator('luma-maxbar [data-go="close"]').click();
  await expect(page.locator('luma-maxbar')).toHaveCount(0);
  await expect(desk.locator('luma-maxbar')).toHaveCount(0, { timeout: 8000 });
  expect(await page.evaluate(async () => (await (await fetch('/api/screens/1')).json()).maximized ?? null)).toBeNull();
  await ctx.close();
});

test('phone: HoldFollows Off keeps the held pane on this device', async ({ page, browser }, info) => {
  test.skip(!phone(info), 'the phone');
  expect((await server.command('Set UserProfile 2 "HoldFollows" Off')).ok).toBe(true);
  const { ctx, desk } = await deskPage(browser);
  await layDesk(desk);
  await open(page);
  await page.waitForFunction(() => (0, eval)('barStatus.hold_follows') === false);
  await holdPane(page, 3);
  await expect(maxTitle(page)).toContainText('DMX');
  await desk.waitForTimeout(1500);
  await expect(desk.locator('luma-maxbar')).toHaveCount(0);
  expect(await page.evaluate(async () => (await (await fetch('/api/screens/1')).json()).maximized ?? null)).toBeNull();
  await page.locator('luma-maxbar [data-go="close"]').click();
  await expect(page.locator('luma-maxbar')).toHaveCount(0);
  expect((await server.command('Set UserProfile 2 "HoldFollows" On')).ok).toBe(true);
  await ctx.close();
});

test('the window menu maximises a window on every client (desk: Maximize, ‹ › ×)', async ({ page, browser }, info) => {
  test.skip(touch(info), 'the desk side (the phone holds)');
  await open(page);
  await layDesk(page);
  const other = await browser.newContext({ viewport: { width: 1200, height: 800 } });
  const second = await other.newPage();
  await open(second);
  await page.locator('.ptab[data-win="4"] .tab-dot').click();
  await page.locator('#ctx .ci', { hasText: 'Maximize' }).click();
  await expect(maxTitle(page)).toContainText('System Monitor');
  await expect(maxTitle(second)).toContainText('System Monitor', { timeout: 8000 });
  await second.locator('luma-maxbar [data-go="close"]').click();
  await expect(page.locator('luma-maxbar')).toHaveCount(0, { timeout: 8000 });
  // F11 / F12 by touch or mouse: in the window menu too.
  await page.locator('.ptab[data-win="4"] .tab-dot').click();
  await expect(page.locator('#ctx .ci', { hasText: 'Screenshot' })).toBeVisible();
  await expect(page.locator('#ctx .ci', { hasText: 'Full screen' })).toBeVisible();
  await page.keyboard.press('Escape');
  await other.close();
});

test('the device picks its screen config (Shutdown menu › Screen config)', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).openPowerMenu());
  const tile = page.locator('#power-tiles .lc-tile', { hasText: 'Screen config' });
  await expect(tile).toContainText('Follow user');
  await tile.click();
  await page.locator('luma-popup .lc-pick-row[data-no]').first().click();
  await expect.poll(() => page.evaluate(() => (0, eval)('deviceStatus.screen_config') ?? null)).toBe(1);
  await page.evaluate(() => (window as any).openPowerMenu());
  await expect(page.locator('#power-tiles .lc-tile', { hasText: 'Screen config' })).toContainText('Config 1');
  await page.locator('#power-tiles .lc-tile', { hasText: 'Screen config' }).click();
  await page.locator('luma-popup .lc-pick-row.follow').click();
  await expect.poll(() => page.evaluate(() => (0, eval)('deviceStatus.screen_config') ?? null)).toBeNull();
  // The phone's command line bar: PURR · line · VK · ⋯ (statuses in ⋯).
  const narrow = await page.evaluate(() => innerWidth <= 600);
  await expect(page.locator('#cb-more')).toBeVisible({ visible: narrow });
  await expect(page.locator('#tb-strip .lc-ss-mark')).toBeVisible();
});
