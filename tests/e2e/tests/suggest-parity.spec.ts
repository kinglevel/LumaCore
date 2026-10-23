// The CLI's TUI and the web command line bar show the same suggestions
// (decisions §30 "12b2": one suggestion engine in the station). A terminal
// client like lumacore-cli types a line over the shell port and its TUI's
// live list is read off the screen; the page types the same line into the
// command line bar and its <luma-suggest> list is read. Row for row, the
// titles must agree.

import path from 'node:path';
import { expect, test } from '@playwright/test';
import { Cli, REPO, Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  // Its own user, so the terminal and the page don't share one command line.
  const r = await server.command('Store User "Cli" /Rights Admin');
  expect(r.ok, r.message).toBe(true);
});
test.afterAll(async () => { await server.cleanup(); });

for (const line of ['Gr', 'Store Gr', 'Group 1 At ', 'Fix', 'Set Win']) {
  test(`"${line}": the terminal and the bar suggest the same`, async ({ page, browser }) => {
    await server.open(page, '/', 'Admin');
    await connected(page);
    await kbdMode(page);
    const screen = await (await browser.newContext()).newPage();
    const cli = await Cli.connect(server, screen, 'Cli');
    try {
      await expect.poll(async () => (await cli.text()).some(l => /\]\s.*>/.test(l))).toBe(true);
      cli.type(line);
      // The whole line has reached the prompt (not only its first keys) before the list is read.
      await expect.poll(async () => (await cli.text()).some(l => /\]\s.*>/.test(l) && l.includes(`> ${line.trimEnd()}`))).toBe(true);
      await expect.poll(() => cli.suggestions().then(s => s.length)).toBeGreaterThan(0);
      const tui = await cli.suggestions();

      await page.keyboard.type(line, { delay: 20 });
      const rows = page.locator('#cb-suggest .lc-sg-row .lc-sg-title');
      await expect(rows.first()).toBeVisible();
      // The list for the whole line arrives after the queued keys ran (a big show takes a
      // moment): read it once it agrees, or report the last read.
      const read = async () => (await rows.allInnerTexts()).map(t => [...t.trim()].slice(0, 24).join('').trimEnd());
      let web = await read();
      await expect.poll(async () => { web = await read(); const n = Math.min(tui.length, web.length); return n > 0 && tui.slice(0, n).join('\n') === web.slice(0, n).join('\n'); }, { timeout: 8000 }).toBe(true).catch(() => {});
      const n = Math.min(tui.length, web.length);
      expect(n, `terminal ${JSON.stringify(tui)} · bar ${JSON.stringify(web)}`).toBeGreaterThan(0);
      expect(tui.slice(0, n), `"${line}": terminal vs bar`).toEqual(web.slice(0, n));
    } finally {
      // Both sessions keep their line: leave them empty for the next line.
      for (let i = 0; i < line.length + 2; i++) cli.key('Backspace');
      for (let i = 0; i < line.length + 2; i++) await page.keyboard.press('Backspace');
      await page.waitForTimeout(150);
      cli.close();
      await screen.context().close();
    }
  });
}
