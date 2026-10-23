// Copy to the user's clipboard (the NAV inspector's copy keys; docs/dev/ui-kit/mockups/2026-10-23-nav-mode,
// "Copy on every client"): the station's TUI sends `copy_text` on the page's shell, and the page puts the
// text on this computer's clipboard and says "Copied <label>" — by the browser's clipboard, by the older
// copy command while the key press still counts as the user's, by the app's IPC, or else a toast that
// holds the text selected for Ctrl+C. Never the station's object clipboard (Copy / Paste).
//
// The first case presses NAV's copy key in the page's terminal (decisions 0291); the fallbacks put the
// station's message on the page's real /ws/shell (routed through Playwright). The wire's round trip and
// the TUI's door are Rust tests (lumacore-wire clipboard/tests.rs, lumacore-terminal repl/clipboard/tests.rs,
// repl/tui_nav/tests.rs).

import { expect, test, type Page, type WebSocketRoute } from '@playwright/test';
import { Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

const LINE = 'Set Fixture 101 "Name" "Spot SR"';

/** Open the page with its shell socket routed through the test; returns the route to speak as the station. */
async function open(page: Page, init?: () => void): Promise<() => WebSocketRoute> {
  await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: server.url });
  if (init) await page.addInitScript(init);
  let shell: WebSocketRoute | null = null;
  await page.routeWebSocket(/\/ws\/shell/, ws => { ws.connectToServer(); shell = ws; });
  await server.open(page);
  await connected(page);
  await expect.poll(() => shell !== null).toBe(true);
  return () => shell!;
}

/** The station's TUI copies `text` (what `repl::clipboard::copy` sends). */
const copy = (shell: WebSocketRoute, label: string, text: string) =>
  shell.send(JSON.stringify({ type: 'ui', event: { action: 'copy_text', label, text } }));

const toast = (page: Page) => page.locator('.lc-toast.on');
/** The clipboard as another page of this browser reads it. */
async function clipboard(page: Page): Promise<string> {
  const other = await page.context().newPage();
  await other.goto(server.url + '/api/station');
  const text = await other.evaluate(() => navigator.clipboard.readText());
  await other.close();
  return text;
}

test('NAV\'s yp puts the row\'s path on the browser\'s clipboard and a toast says so', async ({ page }) => {
  await open(page);
  await kbdMode(page);
  await cmd(page, 'cd /');
  await cmd(page, 'Shortcuts Navigator');
  // The first row of the tree at the root; `yp` copies its path (decisions 0291).
  await page.keyboard.type('yp', { delay: 20 });
  await expect(toast(page)).toHaveText('Copied path');
  expect(await page.evaluate(() => navigator.clipboard.readText())).toMatch(/^Root\.[A-Za-z]+$/);
  await page.keyboard.press('Escape'); // leaves NAV
  await expect(page.locator('#cb-mode .cb-text')).toHaveText('KBD');
});

test('without the browser\'s clipboard the older copy works after the key press', async ({ page }) => {
  // A plain-http address has no navigator.clipboard.
  const shell = await open(page, () => Object.defineProperty(navigator, 'clipboard', { value: undefined }));
  await page.keyboard.press('ArrowDown'); // the user's key goes to the station; its answer comes back
  copy(shell(), 'Set line', LINE);
  await expect(toast(page)).toHaveText('Copied Set line');
  expect(await clipboard(page)).toBe(LINE);
});

test('when nothing can copy, the toast holds the text selected for Ctrl+C', async ({ page }) => {
  const shell = await open(page, () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined });
    document.execCommand = () => false;
  });
  copy(shell(), 'Path', 'Root.ShowData.Groups');
  const field = toast(page).locator('input');
  await expect(toast(page)).toContainText('Copy Path: Ctrl+C');
  await expect(field).toHaveValue('Root.ShowData.Groups');
  await expect(field).toBeFocused();
  expect(await field.evaluate((f: HTMLInputElement) => f.value.slice(f.selectionStart!, f.selectionEnd!))).toBe('Root.ShowData.Groups');
  await page.keyboard.press('ControlOrMeta+c');
  await expect(toast(page)).toHaveText('Copied Path');
  expect(await clipboard(page)).toBe('Root.ShowData.Groups');
});

test('in the app the page hands the copy to the app, which answers', async ({ page }) => {
  const shell = await open(page, () => {
    (window as any).__ipc = [];
    (window as any).LumaHost = { copy: (text: string) => (window as any).__ipc.push({ type: 'copy', text }) };
  });
  copy(shell(), 'Target', 'Fixture 101');
  await expect.poll(() => page.evaluate(() => (window as any).__ipc)).toEqual([{ type: 'copy', text: 'Fixture 101' }]);
  await page.evaluate(() => (window as any).lumacoreApp.copied(true));
  await expect(toast(page)).toHaveText('Copied Target');
  // The app couldn't (no clipboard on that machine): the page uses the browser's.
  copy(shell(), 'Name', 'Spot SR');
  await expect.poll(() => page.evaluate(() => (window as any).__ipc.length)).toBe(2);
  await page.evaluate(() => (window as any).lumacoreApp.copied(false));
  await expect(toast(page)).toHaveText('Copied Name');
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe('Spot SR');
});
