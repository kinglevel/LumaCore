// The paint picker (decisions 0294, the round 4 board) on the rig: `EditSetting Group 1`'s Paint
// cell opens `<luma-paint>` on the `paint` feed — Palette (the station's hues and tones; a named
// swatch is its word: `Set Group 1 "Paint" Blue`), Recent (the station remembers it), the mode
// kept on the profile (`PaintMode`, reopens on Free), Theme (`Set Group 1 "Paint" Theme
// "Position"`), the Examples switch, the Hex field; the theme editor's Paint tab (15 slots as a
// grid, change Position, Add a slot, Reset to LumaCore); the Showroom's buttons.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });

const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/paint-${name}.png`) });
const picker = (page: Page) => page.locator('luma-paint');

async function start(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}

/** `EditSetting Group 1`, the tab holding Paint; returns the Paint cell. */
async function paintCell(page: Page): Promise<Locator> {
  await cmd(page, 'EditSetting Group 1');
  const set = page.locator('.lc-set-scrim luma-settings');
  await expect(set).toBeVisible();
  const cell = set.locator('.lc-set-cell[data-kind="paint"]');
  const tabs = set.locator('[data-tab]');
  for (let i = 0; i < (await tabs.count()) && !(await cell.count()); i++) await tabs.nth(i).click();
  await expect(cell).toHaveCount(1);
  return cell;
}

async function openPicker(page: Page) {
  const cell = await paintCell(page);
  await cell.locator('.lc-set-tap').click();
  await expect(picker(page)).toBeVisible();
  // The station's palette is there (the feed's first batch).
  await expect(picker(page).locator('.lc-pk-pal button, .lc-pk-theme button, [data-sv]').first()).toBeVisible();
  return { cell, pk: picker(page) };
}

test('Palette: a named swatch is its word; Please is one line; Recent remembers it', async ({ page }) => {
  await start(page);
  const { cell, pk } = await openPicker(page);
  await expect(pk.locator('[data-mode="Palette"]')).toHaveClass(/\bon\b/);
  await expect(pk.locator('.lc-pk-nm')).toHaveCount(12);
  await expect(pk.locator('.lc-pk-greys .none')).toContainText('none');
  await expect(pk.locator('[data-meta]')).toHaveText('tags, then the pool');
  const blue = pk.locator('.lc-pk-pal button[data-name="Blue"]');
  const hex = (await blue.getAttribute('data-hex'))!;
  await blue.click();
  await expect(pk.locator('[data-line]')).toHaveText('Set Group 1 "Paint" Blue');
  await expect(pk.locator('[data-meta]')).toHaveText('Blue');
  await expect(pk.locator('[data-hexin]')).toHaveValue(hex);
  await shot(page, 'palette');
  const from = server.mark();
  await pk.locator('[data-please]').click();
  await server.waitSysmon(/Set Group 1 "Paint" Blue/, from);
  await expect(picker(page)).toHaveCount(0);
  // The cell shows the stored colour (the station keeps a palette word as its hex).
  await expect(cell.locator('.lc-set-value')).toContainText(hex);
  await expect(cell.locator('.lc-set-sw')).not.toHaveClass(/\bnone\b/);
  // Reopened: Recent holds it.
  await cell.locator('.lc-set-tap').click();
  await expect(picker(page).locator(`[data-recent] button[data-hex="${hex}"]`)).toHaveCount(1);
  await expect(picker(page).locator('.lc-pk-pal button[data-name="Blue"]')).toHaveClass(/\bon\b/);
  await page.keyboard.press('Escape');
  await expect(picker(page)).toHaveCount(0);
  // Esc closed only the picker: the settings stay.
  await expect(page.locator('.lc-set-scrim luma-settings')).toBeVisible();
});

test('the mode is kept: Free is remembered on reopen; the square picks a colour', async ({ page }) => {
  await start(page);
  const { cell, pk } = await openPicker(page);
  const from = server.mark();
  await pk.locator('[data-mode="Free"]').click();
  await server.waitSysmon(/"PaintMode" Free/, from);
  const sv = pk.locator('[data-sv]');
  await expect(sv).toBeVisible();
  await expect(pk.locator('[data-hue]')).toBeVisible();
  const box = (await sv.boundingBox())!;
  await sv.click({ position: { x: box.width * 0.8, y: box.height * 0.2 } });
  await expect(pk.locator('[data-line]')).toHaveText(/^Set Group 1 "Paint" ("#[0-9a-f]{6}"|[A-Za-z]+)$/);
  await shot(page, 'free');
  await pk.locator('[data-cancel]').click();
  await expect(picker(page)).toHaveCount(0);
  await cell.locator('.lc-set-tap').click();
  await expect(picker(page).locator('[data-mode="Free"]')).toHaveClass(/\bon\b/);
  await expect(picker(page).locator('[data-sv]')).toBeVisible();
  await picker(page).locator('[data-cancel]').click();
});

test('Theme: a slot is a link — Set Group 1 "Paint" Theme "Position"', async ({ page }) => {
  await start(page);
  const { cell, pk } = await openPicker(page);
  let from = server.mark();
  await pk.locator('[data-mode="Theme"]').click();
  await server.waitSysmon(/"PaintMode" Theme/, from);
  await expect(pk.locator('.lc-pk-theme button')).toHaveCount(15);
  await pk.locator('.lc-pk-theme button[data-slot="Position"]').click();
  await expect(pk.locator('[data-line]')).toHaveText('Set Group 1 "Paint" Theme "Position"');
  await expect(pk.locator('[data-meta]')).toHaveText('Theme · Position');
  await expect(pk.locator('[data-hexin]')).toBeDisabled();
  await shot(page, 'theme');
  from = server.mark();
  await pk.locator('[data-please]').click();
  await server.waitSysmon(/Set Group 1 "Paint" Theme "Position"/, from);
  await expect(cell.locator('.lc-set-value')).toContainText('Theme "Position"');
  // Back to the palette for the next tests (and none, through the picker).
  await cell.locator('.lc-set-tap').click();
  await expect(picker(page).locator('.lc-pk-theme button[data-slot="Position"]')).toHaveClass(/\bon\b/);
  from = server.mark();
  await picker(page).locator('[data-mode="Palette"]').click();
  await server.waitSysmon(/"PaintMode" Palette/, from);
  await picker(page).locator('.lc-pk-greys .none').click();
  await expect(picker(page).locator('[data-line]')).toHaveText('Set Group 1 "Paint" none');
  await picker(page).locator('[data-please]').click();
  await server.waitSysmon(/Set Group 1 "Paint" none/, from);
  await expect(cell.locator('.lc-set-value')).toHaveText('none');
});

test('Examples open on their switch; typing a hex picks it', async ({ page }) => {
  await start(page);
  const { pk } = await openPicker(page);
  await expect(pk.locator('[data-examples]')).toBeHidden();
  await pk.locator('[data-ex]').click();
  await expect(pk.locator('[data-ex]')).toHaveAttribute('aria-checked', 'true');
  await expect(pk.locator('[data-examples]')).toBeVisible();
  await expect(pk.locator('[data-examples] > div')).toHaveCount(6);
  const hex = pk.locator('[data-hexin]');
  await hex.fill('#3F73C8');
  await expect(pk.locator('[data-line]')).toHaveText('Set Group 1 "Paint" "#3f73c8"');
  await expect(pk.locator('[data-meta]')).toHaveText('#3f73c8');
  await shot(page, 'examples');
  await hex.fill('3f73c');
  await expect(pk.locator('[data-line]')).toHaveText('Set Group 1 "Paint" "#3f73c8"');
  await pk.locator('[data-cancel]').click();
  await expect(picker(page)).toHaveCount(0);
});

test('the theme editor\'s Paint tab: 15 slots, change Position, Add a slot, Reset to LumaCore', async ({ page }) => {
  await start(page);
  await cmd(page, 'Theme');
  const ed = page.locator('[data-kit="theme-editor"]');
  await expect(ed).toBeVisible();
  await ed.locator('[data-te-tab="Paint"]').click();
  const tab = ed.locator('[data-kit="theme-paint"]');
  const rows = tab.locator('.lc-grid-row');
  await expect(rows).toHaveCount(15);
  await expect(tab.locator('[data-paint-count]')).toHaveText('15 slots');
  await expect(tab.locator('.lc-grid-row.lc-painted')).toHaveCount(15);
  await shot(page, 'slots');
  // Your colour of Position: the picker in Palette / Free only, the slot's own line.
  let from = server.mark();
  await tab.locator('.lc-gc[data-k="Position"][data-col="colour"]').click();
  const pk = picker(page);
  await expect(pk).toBeVisible();
  await expect(pk.locator('[data-mode]')).toHaveCount(2);
  await expect(pk.locator('[data-mode="Theme"]')).toHaveCount(0);
  const red = pk.locator('.lc-pk-pal button[data-name="Red"]');
  const hex = (await red.getAttribute('data-hex'))!;
  await red.click();
  await expect(pk.locator('[data-line]')).toHaveText(`Set Theme Paint "Position" "${hex}"`);
  await pk.locator('[data-please]').click();
  await server.waitSysmon(/Set Theme Paint "Position" "#[0-9a-f]{6}"/, from);
  await expect(tab.locator('[data-paint-count]')).toHaveText('15 slots · 1 changed');
  await expect(tab.locator('.lc-gc[data-k="Position"][data-col="colour"]')).toContainText(hex);
  // Add a slot: the kit dialog asks the name.
  from = server.mark();
  await tab.locator('[data-paint-add]').click();
  const dlg = page.locator('[data-kit="dialog"]');
  await expect(dlg).toBeVisible();
  await dlg.locator('input[data-field="0"]').fill('Haze');
  await dlg.locator('.lc-dlg-btn[data-primary="1"]').click();
  await server.waitSysmon(/Store Theme Paint "Haze"/, from);
  await expect(rows).toHaveCount(16);
  await expect(tab.locator('.lc-gc[data-k="Haze"][data-col="kind"]')).toHaveText('yours');
  await expect(ed).toBeVisible();
  // Reset to LumaCore: Position back, Haze gone.
  from = server.mark();
  await tab.locator('[data-paint-reset]').click();
  await server.waitSysmon(/Reset Theme Paint "(Position|Haze)"/, from);
  await expect(rows).toHaveCount(15);
  await expect(tab.locator('[data-paint-count]')).toHaveText('15 slots');
  await expect(tab.locator('[data-paint-reset]')).toBeDisabled();
  // Esc closes the editor; the Look tab is still the editor's first.
  await page.keyboard.press('Escape');
  await expect(ed).toHaveCount(0);
});

test('the Showroom opens the picker in each mode on the live feed', async ({ page }) => {
  await start(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  for (const mode of ['Palette', 'Free', 'Theme']) {
    const btn = `luma-button[label="Paint · ${mode}"]`;
    await expect(page.locator(btn)).toHaveCount(1);
    await page.evaluate(s => (document.querySelector(s) as HTMLElement).click(), btn);
    await expect(picker(page)).toBeVisible();
    await expect(picker(page).locator(`[data-mode="${mode}"]`)).toHaveClass(/\bon\b/);
    await page.keyboard.press('Escape');
    await expect(picker(page)).toHaveCount(0);
  }
});
