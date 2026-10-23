// The key editor (decisions 0295, the approved mockups
// docs/dev/ui-kit/mockups/2026-10-23-profile-editor/02-keys.html and 03-cycles.html), on the rig:
// `Edit KeyBindings` opens <luma-keymap> on the `keys` feed — the keyboard per shortcut layer, PRG's
// keys as LumaCore's (E cycles Edit › EditSetting), the core keys locked, Meow relabelling the keys;
// editing a key's press cycle is one Bind + Set UIKey line and the key becomes yours; Unbind brings
// LumaCore's back. A kit file: desktop, tablet, phone and WebKit.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/keys-${name}.png`) });

async function editor(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  await page.evaluate(() => (window as any).wsCommand('Edit KeyBindings'));
  const ed = page.locator('[data-kit="key-editor"]');
  await expect(ed).toBeVisible({ timeout: 10_000 });
  await expect(ed.locator('.lc-km-board')).toBeVisible();
  return ed;
}

test('the keyboard per mode: PRG\'s E cycles, F10 is core, Meow relabels', async ({ page }) => {
  const ed = await editor(page);
  await ed.locator('[data-layer="PRG"]').click();
  const e = ed.locator('.lc-km-key[data-code="KeyE"]');
  await expect(e).toHaveClass(/\blumacore\b/);
  await expect(e.locator('.act')).toHaveText('Edit');
  await expect(e.locator('.lc-km-cyc')).toHaveText('2');
  await expect(ed.locator('.lc-km-key[data-code="F10"]')).toHaveClass(/\bcore\b/);
  // KBD: a letter types.
  await ed.locator('[data-layer="KBD"]').click();
  await expect(ed.locator('.lc-km-key[data-code="KeyE"]')).toHaveClass(/\btypes\b/);
  // PRG with Meow: F is FixtureType.
  await ed.locator('[data-layer="PRG"]').click();
  await ed.locator('.lc-km-bar [data-mod="Shift"]').click();
  await expect(ed.locator('.lc-km-key[data-code="KeyF"] .act')).toHaveText('FixtureType');
  await shot(page, `prg-meow-${test.info().project.name}`);
});

test('editing E\'s cycle is one Bind + Set UIKey line; Unbind brings LumaCore\'s back', async ({ page }) => {
  const ed = await editor(page);
  await ed.locator('[data-layer="PRG"]').click();
  await ed.locator('.lc-km-key[data-code="KeyE"]').click();
  const panel = ed.locator('.lc-km-panel');
  await expect(panel.locator('.lc-km-head b')).toHaveText('E');
  await expect(panel.locator('[data-act="save"]')).toBeDisabled();
  await panel.locator('[data-add="press"]').click();
  const third = panel.locator('.lc-km-row[data-which="press"][data-row="2"] [data-line]');
  await third.fill('Label');
  await third.press('Tab');
  const from = server.mark();
  await panel.locator('[data-act="save"]').click();
  // One line, two statements (the station logs each).
  await server.waitSysmon(/Bind "E" `Edit` `EditSetting` `Label` \/Shortcuts "PRG" → ok/, from);
  await server.waitSysmon(/Set UIKey "user\.prg\.e" "Types" "1 2 3" → ok/, from);
  const e = ed.locator('.lc-km-key[data-code="KeyE"]');
  await expect(e).toHaveClass(/\bprofile\b/);
  await expect(e.locator('.lc-km-cyc')).toHaveText('3');
  await shot(page, `mine-${test.info().project.name}`);
  await ed.locator('.lc-km-key[data-code="KeyE"]').click();
  await panel.locator('[data-act="unbind"]').click();
  await expect(e).toHaveClass(/\blumacore\b/);
  await expect(e.locator('.lc-km-cyc')).toHaveText('2');
});
