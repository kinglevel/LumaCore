// The Presets window (decisions 0288): one preset pool that follows the encoder bar's feature group,
// narrowed by tag chips; cells name how a preset fits the selection, its mode letter, a preview and
// a square per feature group; a tap runs the window's action (At); Store + a tap on an empty cell
// stores the programmer there with the lit tags; Fixed slots keeps a hidden preset's place; hold →
// Edit values… opens the editor. The rig stores Preset 1 "Full" and 2 "Half" (Universal, tagged
// Wash) and 3 "Split" (Selective, 101 + 102). On desktop, tablet, phone and WebKit.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, hold, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });

const win = (page: Page) => page.locator('.pool-win').filter({ has: page.locator('.lc-pre-tags:not([hidden])') }).last();
const cell = (page: Page, n: number) => win(page).locator(`.lc-pool-cell[data-no="${n}"]`);
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/presets-${name}.png`) });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  if (!(await win(page).count())) await page.evaluate(() => (window as any).wsCommand('Store Window "Presets"'));
  await expect(win(page)).toBeVisible();
}
/** The window's own line (`Set Self …`), as its bar sends it. */
async function self(page: Page, line: string) {
  const no = await win(page).getAttribute('data-win');
  await page.evaluate(([l, n]) => (window as any).wsCommand(l, Number(n)), [line, no]);
}

test('the rig\'s presets show with their mode letters while the encoder bar is on Dimmer', async ({ page }) => {
  await open(page);
  await page.locator('#encbar [data-group]', { hasText: 'Dimmer' }).first().click();
  await expect(cell(page, 1)).toContainText('Full');
  await expect(cell(page, 1).locator('.lc-pre-mode')).toHaveText('U');
  await expect(cell(page, 3).locator('.lc-pre-mode')).toHaveText('S');
  await expect(cell(page, 1).locator('.lc-pre-fg i')).toHaveCount(1);
  await expect(win(page).locator('.pool-title')).toContainText('Following: Dimmer');
  await shot(page, 'dimmer');
});

test('it follows the encoder bar: on Position the dimmer presets go', async ({ page }) => {
  await open(page);
  const position = page.locator('#encbar [data-group]', { hasText: 'Position' }).first();
  test.skip(!(await position.count()), 'the profile\'s encoder bar has no Position button');
  await position.click();
  await expect(win(page).locator('.lc-pool-cell', { hasText: 'Full' })).toHaveCount(0);
  await page.locator('#encbar [data-group]', { hasText: 'Dimmer' }).first().click();
  await expect(cell(page, 1)).toContainText('Full');
});

test('the cells fit the selection; a tap runs At and marks the preset as yours', async ({ page }) => {
  await open(page);
  await cmd(page, 'ResetAll');
  await cmd(page, 'Fixture 500');
  await expect(cell(page, 3)).toHaveClass(/fit-none/, { timeout: 10_000 });
  await expect(cell(page, 1)).not.toHaveClass(/fit-none/);
  const from = server.mark();
  await cell(page, 1).click();
  await server.waitSysmon(/At Preset 1/, from);
  await expect(cell(page, 1)).toHaveClass(/mine/);
  await shot(page, 'fit');
});

test('a tag chip narrows the pool; Fixed slots keeps a hidden preset\'s place', async ({ page }) => {
  await open(page);
  const chip = win(page).locator('.lc-pre-chip', { hasText: 'Wash' });
  await chip.click();
  await expect(chip).toHaveAttribute('aria-pressed', 'true');
  await expect(cell(page, 1)).toContainText('Full');
  await expect(win(page).locator('.lc-pool-cell', { hasText: 'Split' })).toHaveCount(0);
  await self(page, 'Set Self "Layout" Fixed');
  await expect(cell(page, 3)).toHaveClass(/gap/);
  await self(page, 'Set Self "Layout" Packed');
  await shot(page, 'tagged');
});

test('Store + a tap on an empty cell stores the programmer there with the lit tag', async ({ page }) => {
  await open(page);
  await cmd(page, 'ResetAll');
  await cmd(page, 'Fixture 103 Thru 104 At 70');
  const empty = win(page).locator('.lc-pool-cell.empty').first();
  const no = Number(await empty.getAttribute('data-no'));
  const from = server.mark();
  await page.keyboard.type('Store ', { delay: 5 });
  await empty.click();
  await server.waitSysmon(new RegExp(`Store Preset ${no}`), from);
  await expect(cell(page, no).locator('.lc-pre-mode')).toHaveText('U');
  // The Wash chip is still lit and the new preset is in view: it carries the tag.
  await expect(win(page).locator('.lc-pre-chip', { hasText: 'Wash' })).toHaveAttribute('aria-pressed', 'true');
});

test('hold → Edit values… opens the editor with the preset\'s values', async ({ page }) => {
  await open(page);
  await self(page, 'Set Self "Tags" ""');
  await expect(win(page).locator('.lc-pre-chip', { hasText: 'Wash' })).toHaveAttribute('aria-pressed', 'false');
  await expect(cell(page, 3)).toContainText('Split');
  await page.waitForTimeout(300);
  await hold(page, cell(page, 3));
  await page.locator('[data-kit="context"] .lc-cm-item, #ctx .ci', { hasText: 'Edit values' }).first().click();
  const ed = page.locator('[data-kit="preset-editor"]');
  await expect(ed).toBeVisible();
  await expect(ed).toContainText('Preset 3 "Split"');
  await expect(ed).toContainText('Selective');
  await expect(ed.locator('luma-grid')).toContainText('Fixture 101');
  await shot(page, 'editor');
  await ed.locator('.lc-close').click();
  await expect(ed).toHaveCount(0);
});
