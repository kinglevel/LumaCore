// Presets linked into cues (decisions 0293), on the rig: its Sequence 1 "Presets" links Presets
// 1–3. The Update ask is one station dialog holding the kit's tree in pick mode — each preset your
// values came from with the cues that follow it, Content and two switches, the lines it will run
// redrawn as you tick; Preview in Blind is greyed (planned); Update runs the lines as one
// statement. The Device Sheet's Show draws a linked value as its value, its link or both, with
// the preset bar. On desktop, tablet, phone and WebKit.

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, answer, cmd, connected, dialog, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
});
test.afterAll(async () => { await server.cleanup(); });

const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/update-${name}.png`) });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}

/** Run `text` as this page's line and wait until the station logged it. */
async function line(page: Page, text: string) {
  const from = server.mark();
  await page.evaluate(t => (window as any).wsCommand(t), text);
  const esc = text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  await server.waitSysmon(new RegExp(`\\] ${esc} →`), from);
}

const lines = (page: Page) => dialog(page).locator('[data-kit="lines"] li');
const row = (page: Page, level: number, text: string | RegExp) => dialog(page).locator(`.lc-dlg-tree .lc-tree-row[aria-level="${level}"]`, { hasText: text });

test('Update asks with a tree of presets and the cues that follow; ticks redraw the lines', async ({ page }) => {
  await open(page);
  for (const l of ['ResetAll', 'Fixture 101 Thru 110', 'At Preset 2', 'At 75']) await line(page, l);
  await page.evaluate(() => (window as any).wsCommand('Update'));
  await expect(dialog(page).locator('.lc-dlg-title span').first()).toHaveText('Update');
  const preset = row(page, 1, 'Half');
  await expect(preset).toContainText('P2');
  await expect(preset.locator('.lc-tree-box')).toHaveAttribute('data-state', 'on');
  const cue = row(page, 2, /Cue 2/);
  await expect(cue).toContainText('follows');
  await expect(lines(page)).toHaveText(['Update Preset 2 /Merge']);
  await expect(dialog(page).locator('.lc-dlg-btn', { hasText: 'Preview in Blind' })).toBeDisabled();
  await shot(page, 'ask');
  // AddNew: the Recast switch is on, so its Recast line joins.
  await dialog(page).locator('.lc-dlg-seg button', { hasText: 'AddNew' }).click();
  await expect(lines(page)).toHaveText(['Update Preset 2 /Add', 'Recast Preset 2']);
  // Untick the cue: it keeps its old look (an Extract first).
  await cue.locator('.lc-tree-box').click();
  await expect(cue).toContainText('keeps its old look');
  await expect(lines(page)).toHaveText(['Extract Preset 2 Sequence 1 Cue 2', 'Update Preset 2 /Add', 'Recast Preset 2']);
  await shot(page, 'ticked');
  const from = server.mark();
  await answer(page, /^Update/);
  await server.waitSysmon(/Update `Extract Preset 2 Sequence 1 Cue 2; Update Preset 2 \/Add; Recast Preset 2` →/, from);
  await expect(dialog(page)).toHaveCount(0);
  // The whole update is one Oops step.
  const oops = server.mark();
  await line(page, 'Oops');
  await server.waitSysmon(/Oops/, oops);
});

test('the Device Sheet shows a linked value by its Show setting, with the preset bar', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'a phone draws Channel mode (decisions device-sheet-window)');
  await open(page);
  for (const l of ['ResetAll', 'Goto Sequence 1 Cue 3', 'Fixture 101']) await line(page, l);
  const has = await page.evaluate(() => Object.values((window as any).eval('layout').windows).some((w: any) => w.kind === 'devices'));
  if (!has) await page.evaluate(() => (window as any).wsCommand('Store Window "Device Sheet"'));
  await page.locator('.ptab', { hasText: 'Device Sheet' }).first().click();
  const grid = page.locator('luma-device-grid').first();
  await expect(grid).toBeVisible();
  const winNo = await grid.evaluate(el => el.closest('[data-win]')?.getAttribute('data-win'));
  const self = (l: string) => page.evaluate(([t, n]) => (window as any).wsCommand(t, Number(n)), [l, winNo]);
  await self('Set Self "SelectedOnly" On');
  await self('Set Self "Folders" Off');
  const dim: Locator = grid.locator('.lc-gc.lc-v[data-col="Dimmer"]').first();
  await expect(dim).toHaveText(/^P3 Split · \d+/);
  await expect(dim.locator('.lc-v-bars i[data-l="preset"]')).toBeVisible();
  await expect(dim).toHaveAttribute('title', /Preset 3 "Split"/);
  await shot(page, 'sheet-both');
  await self('Set Self "Show" Link');
  await expect(dim).toHaveText('P3 Split');
  await self('Set Self "Show" Value');
  await expect(dim).toHaveText(/^\d+$/);
  await expect(dim.locator('.lc-v-bars i[data-l="preset"]')).toBeVisible();
  await self('Set Self "Show" Both');
  await self('Set Self "SelectedOnly" Off');
  await self('Set Self "Folders" On');
});
