// Multiplatform (decisions §30 "12c2": the guard after the audit's 17
// findings). One station, five screens — phone 390×844 and tablet 1024×768
// (touch), laptop 1280×720, desktop 1920×1080 and WebKit 1280×720 — and on
// each every window type, the Menu and each of its windows, Users & Profiles
// (and a profile's tabs), a window's settings, the window menu, the Add
// Window picker, a station dialog, the keypad, the pick list, F12 full
// screen, the command line with its suggestions and the lock surface. Fails
// on: touch targets under 44 px (touch screens), anything outside the
// viewport, an overlay larger than the screen, horizontal page scroll, text
// under 12 px (support/fit.ts). Runs in the desktop project only; it opens
// its own browsers for the five screens.

import path from 'node:path';
import { chromium, expect, test, webkit, type Browser, type BrowserContextOptions, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode } from '../support/server';
import { fitIssues } from '../support/fit';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const SCREENS: { name: string; engine: 'chromium' | 'webkit'; touch: boolean; use: BrowserContextOptions }[] = [
  { name: 'phone', engine: 'chromium', touch: true, use: { viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true } },
  { name: 'tablet', engine: 'chromium', touch: true, use: { viewport: { width: 1024, height: 768 }, hasTouch: true } },
  { name: 'laptop', engine: 'chromium', touch: false, use: { viewport: { width: 1280, height: 720 } } },
  { name: 'desktop', engine: 'chromium', touch: false, use: { viewport: { width: 1920, height: 1080 } } },
  { name: 'webkit', engine: 'webkit', touch: false, use: { viewport: { width: 1280, height: 720 } } },
];

const browsers: Partial<Record<'chromium' | 'webkit', Browser>> = {};
test.afterAll(async () => { for (const b of Object.values(browsers)) await b?.close(); });

async function openScreen(s: (typeof SCREENS)[number]): Promise<Page> {
  const b = browsers[s.engine] ??= await (s.engine === 'webkit' ? webkit : chromium).launch();
  const ctx = await b.newContext(s.use);
  const page = await ctx.newPage();
  await server.open(page, '/', 'Admin');
  await connected(page);
  await kbdMode(page);
  await page.waitForFunction(() => (0, eval)('liveScreen && winFeed.sub && windowTypes.length') as boolean);
  // Every window type on screen 1 (the first screen adds what is missing).
  for (const [kind, type] of [['sysmon', 'System Monitor'], ['dmx', 'DMX'], ['realtime', 'Realtime'], ['showroom', 'Showroom']]) {
    if (await page.evaluate(k => Object.values((0, eval)('layout').windows).some((w: any) => w.kind === k), kind)) continue;
    await page.evaluate(t => (0, eval)('wsCommand')(`Store Window "${t}"`), type);
    await page.waitForFunction(k => Object.values((0, eval)('layout').windows).some((w: any) => w.kind === k), kind);
  }
  // One pane with every window as a tab (a phone's miniature desk is the
  // kit-12c2 spec's): each window at its real size.
  await page.evaluate(() => {
    const w = (0, eval);
    const nos = Object.keys(w('layout').windows).map(Number).sort((a, b) => a - b);
    w('layout').root = { type: 'leaf', wins: nos, active: 0 };
    w('focusPath = []');
    w('commit()');
  });
  // Cards (Connected …) go by themselves; they are not what is measured.
  await page.evaluate(() => document.getElementById('fb-cards')?.remove());
  return page;
}

async function closeAll(page: Page) {
  for (let i = 0; i < 3; i++) await page.keyboard.press('Escape');
  await page.evaluate(() => {
    const w = (0, eval);
    document.querySelectorAll('.mo-overlay.open').forEach(o => w('closeMenuOverlay')(o.id));
    w('closeSetupMenu()');
    document.querySelectorAll('.lc-set-scrim, .lc-pick-scrim, .lc-kp-scrim, .lc-card-scrim').forEach(s => s.remove());
    w('LumaKit.menu.closeMenu()');
    if (w('maximizedWin()') != null) w('setHeld(null)');
  });
  await page.waitForTimeout(250);
}

for (const s of SCREENS) {
  test(`${s.name}: every window, menu, settings window and dialog fits`, async () => {
    test.setTimeout(240_000);
    const page = await openScreen(s);
    const issues: string[] = [];
    const check = async (label: string) => { issues.push(...await fitIssues(page, label, { touch: s.touch })); };
    const ev = (js: string) => page.evaluate(js);
    const step = async (label: string, fn: () => Promise<unknown>) => {
      try { await fn(); await check(label); } catch (e) { issues.push(`[${label}] could not open: ${String(e).split('\n')[0]}`); }
      await closeAll(page);
    };

    // Every window type (a tab each).
    const wins = await page.evaluate(() => Object.values((0, eval)('layout').windows).map((w: any) => [w.no, w.kind]));
    expect(wins.map(w => w[1])).toEqual(expect.arrayContaining(['terminal', 'groups', 'sysmon', 'dmx', 'realtime', 'showroom']));
    for (const [no, kind] of wins) {
      await step(`window ${kind}`, async () => {
        await page.locator(`.ptab[data-win="${no}"]`).click();
        await page.waitForTimeout(700);
      });
    }
    await page.locator('.ptab').first().click();

    // The Menu and each window it opens.
    await step('menu', () => ev('openMenu()'));
    const tiles = await page.evaluate(() => (0, eval)('MENU_TILES').filter((t: any) => !t.disabled && t.fn && !['quicksave', 'screens'].includes(t.id)).map((t: any) => t.id));
    for (const id of tiles) {
      await step(`menu › ${id}`, async () => {
        await ev('openMenu()');
        await page.locator(`#menu-tiles [data-tile="${id}"]`).click();
        await page.waitForTimeout(900);
      });
    }
    await step('shutdown menu', () => ev('openPowerMenu()'));
    // Backup on the setup frame (12d part 5): each tab.
    for (const tab of ['Library', 'History', 'Recovery']) {
      await step(`backup › ${tab.toLowerCase()}`, async () => { await ev(`wsCommand('Menu "Backup" /Tab ${tab}')`); await page.waitForTimeout(700); });
    }
    await ev('closeSetupMenu()');

    // Users & Profiles (the setup frame, 12d part 4): a profile's settings
    // and each of their tabs.
    await ev('wsCommand(\'Menu "Users" /Tab Profiles\')');
    const setup = page.locator('luma-setup');
    await expect(setup.locator('luma-grid [data-col="name"]').first()).toBeAttached();
    await check('users & profiles › profiles');
    const admin = await page.evaluate(() => (0, eval)('station').profiles.find((p: any) => p.name === 'Admin').guid);
    await setup.locator(`luma-grid [data-gutter="${admin}"]`).click();
    await setup.locator('luma-slab [data-own][data-id="settings"]').filter({ visible: true }).click();
    const sw = page.locator('.lc-set-scrim');
    await expect(sw.locator('.lc-set-cell').first()).toBeVisible();
    const tabs = sw.locator('.lc-set-tabs button');
    for (let i = 0; i < await tabs.count(); i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(250);
      await check(`users & profiles › tab ${i + 1}`);
    }
    await page.keyboard.press('Escape');
    await page.evaluate(() => (0, eval)('closeSetupMenu()'));
    await closeAll(page);

    // A window's settings (Groups), its pick list, the window menu, the picker.
    const groups = wins.find(w => w[1] === 'groups')![0];
    await page.locator(`.ptab[data-win="${groups}"]`).click();
    await step('window settings', async () => { await ev(`openSettings({}, ${groups})`); await expect(page.locator('.lc-set-scrim .lc-set-cell').first()).toBeVisible(); });
    await step('pick list', async () => {
      await ev(`openSettings({}, ${groups})`);
      await page.locator('.lc-set-scrim .lc-set-cell', { hasText: 'DataPool' }).first().click();
      await expect(page.locator('luma-popup')).toBeVisible();
    });
    await step('window menu', async () => { await page.locator(`.ptab[data-win="${groups}"] .tab-dot`).click(); await expect(page.locator('#ctx')).toBeVisible(); });
    await step('picker', () => ev(`openPicker({ mode: 'add', win: ${groups} })`));

    // A station dialog and the keypad.
    await step('dialog', async () => { await ev(`wsCommand('Label Group 1')`); await expect(page.locator('[data-kit="dialog"]')).toBeVisible(); });
    await step('keypad', async () => {
      await ev(`wsCommand('Move Group 2')`);
      await page.locator('[data-kit="dialog"] .lc-dlg-kb').first().click();
      await expect(page.locator('luma-keypad')).toBeVisible();
    });
    await ev(`LumaKit.dialog.dialogOpen() && document.querySelectorAll('[data-kit="dialog"] .lc-close').forEach(b => b.click())`);

    // F12 full screen, then the command line with its suggestions.
    await step('F12 full screen', async () => { await ev(`pageAction('sidebars', false)`); await page.waitForTimeout(400); });
    await ev(`pageAction('sidebars', true)`);
    await step('command line + suggestions', async () => {
      await page.locator('#cb-line').click({ force: true });
      await page.keyboard.type('Sto', { delay: 30 });
      await expect(page.locator('#cb-suggest')).toBeVisible();
    });
    for (let i = 0; i < 4; i++) await page.keyboard.press('Backspace');
    // The phone's ⋯ (statuses, mode) on the command line bar.
    if (await page.locator('#cb-more').isVisible()) await step('command line ⋯', () => page.locator('#cb-more').click());

    // The lock surface (last: it stays locked).
    await ev(`wsCommand('On DeskLock')`);
    await expect(page.locator('luma-surface')).toBeVisible();
    await page.waitForTimeout(600);
    await check('lock surface');

    await page.context().close();
    expect(issues, issues.join('\n')).toEqual([]);
  });
}
