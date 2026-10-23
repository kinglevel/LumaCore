// The System Monitor window: opened from the window
// picker, fed by the `sysmon` feed on /ws/feeds (decisions §30 "12b2"): the
// whole history in one batch, live lines of every command (password masked),
// Echo, level / subsystem / search filters over a virtual list (only the
// visible rows exist), the tinted "scrolled up" pause, the daily log file and
// the login requirement. Its bar (Pause, Wrap, Level, Time, Clear, the search
// chip) is the kit's window bar (decisions §30 "12c"): every tap a `Set Self`
// line; the filters are the window's settings.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { Server, cmd, connected, kbdMode, wsMessages } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}

/** Add a System Monitor like a user: tab context menu › Add window… › type › Enter. */
async function openSysmon(page: Page) {
  if (await page.locator('.sysmon').count()) return;
  await page.locator('.ptab').first().click({ button: 'right' });
  await page.locator('#ctx .ci', { hasText: 'Add window' }).click();
  await expect(page.locator('#picker-overlay')).toBeVisible();
  await page.keyboard.type('monitor');
  await expect(page.locator('.pk-item.sel')).toContainText('System Monitor');
  await page.keyboard.press('Enter');
  await expect(page.locator('.sysmon')).toBeVisible();
}

/** Focus the monitor (a typed `Self` line is the focused window). */
async function focusSysmon(page: Page) {
  await page.locator('.sysmon .sm-list').click();
  await page.locator('#cb-line').click();
}

const rows = (page: Page) => page.locator('.sysmon .sm-row');
const msgs = async (page: Page) => (await page.locator('.sysmon .sm-row').allInnerTexts());
/** Every line the page holds (the virtual list shows only what fits). */
const stored = (page: Page) => page.evaluate('sysmon.lines.map(l => `${l.subsystem} ${l.message}`).join("\\n")') as Promise<string>;

test('the picker offers System Monitor; it shows the history and logins', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  // Startup lines from before the window existed (in the one history batch),
  // and this login.
  await expect.poll(() => stored(page)).toContain('HTTP engine listening');
  await expect(rows(page).filter({ hasText: "login 'Admin' from 127.0.0.1 (HTTP, local)" })).not.toHaveCount(0);
  // The virtual list: far fewer rows in the page than lines held.
  const held = await page.evaluate('sysmon.lines.length') as number;
  expect(held).toBeGreaterThan(20);
  await expect(rows(page).first()).toHaveClass(/lc-selectable/);
  await expect(page.locator('.sysmon .sm-list')).not.toHaveClass(/\bpaused\b/);
  // Time: Session (since the station started) by default, the wall clock on hover;
  // the bar's Time: Wall clock swaps them.
  const time = page.locator('.sysmon .sm-row .sm-time').first();
  await expect(time).toHaveText(/^\d+:\d\d:\d\d\.\d{3}$/);
  await expect(time).toHaveAttribute('title', /^\d\d:\d\d:\d\d\.\d{3}$/);
  await page.locator('[data-id="sysmon/time"]').click();
  await page.locator('luma-popup .lc-pick-row', { hasText: 'Wall clock' }).click();
  await expect(time).toHaveAttribute('title', /since start/);
  await expect(page.locator('[data-id="sysmon/time"]')).toContainText('Wall clock');
});

test('every command gets a line: user, screen, result; errors are red', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  await cmd(page, 'List User');
  const ok = rows(page).filter({ hasText: '[Admin@Screen 1] List User → ok' });
  await expect(ok).toHaveCount(1);
  await expect(ok.locator('.sm-sub')).toHaveText('Cmd');
  await expect(ok).not.toHaveClass(/\berror\b/);

  // An error keeps its first line.
  await cmd(page, 'Label Group 999 "x"');
  const err = rows(page).filter({ hasText: '[Admin@Screen 1] Label Group 999 "x" →' });
  await expect(err).toHaveClass(/\berror\b/);
  // No such object is E-CMD-007 (CLAUDE.md "Errors have codes").
  await expect(err).toContainText('No such Group. [E-CMD-007]');

  await cmd(page, 'Login "Nobody"');
  await expect(rows(page).filter({ hasText: 'No user "Nobody"' })).toHaveClass(/\berror\b/);
});

test('passwords never reach the monitor', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  await cmd(page, 'Store User "Mon" /Password "hunter2"');
  await expect(rows(page).filter({ hasText: /Store User "?Mon"? \/Password "\*\*\*"/ })).toHaveCount(1);
  expect((await msgs(page)).join('\n')).not.toContain('hunter2');
  expect(await stored(page)).not.toContain('hunter2');
});

test('Echo writes its own line (and no Cmd line)', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  await cmd(page, 'Echo "Act 2 starts"');
  const echo = rows(page).filter({ hasText: '[Admin] Act 2 starts' });
  await expect(echo).toHaveCount(1);
  await expect(echo.locator('.sm-sub')).toHaveText('Echo');
  await expect(rows(page).filter({ hasText: 'Echo "Act 2 starts" →' })).toHaveCount(0);
});

test('level filter: errors only', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  // The bar's Level: the pick list; the answer is `Set Self "Level" Errors`.
  const mark = server.mark();
  await page.locator('[data-id="sysmon/level"]').click();
  await page.locator('luma-popup .lc-pick-row span', { hasText: /^Errors$/ }).click();
  await server.waitSysmon(/Set Window 1\.\d+ "Level" Errors → ok/, mark);
  await expect(page.locator('[data-id="sysmon/level"]')).toContainText('Errors');
  await expect(rows(page).first()).toBeVisible();
  await expect.poll(async () => (await rows(page).evaluateAll(els => els.map(e => e.className))).every(c => /\berror\b/.test(c))).toBe(true);
  await page.locator('[data-id="sysmon/level"]').click();
  await page.locator('luma-popup .lc-pick-row span', { hasText: /^All$/ }).click();
  await expect(page.locator('[data-id="sysmon/level"]')).toContainText('All');
});

test('search filters and highlights', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  // Typed on the command line: `Self` is the focused window (the monitor).
  await focusSysmon(page);
  await cmd(page, 'Set Self "Search" "act 2"');
  // The Echo line, and the search's own Cmd line (it holds the text too).
  const echo = rows(page).filter({ hasText: '[Admin] Act 2 starts' });
  await expect(echo).toHaveCount(1);
  await expect(echo.locator('mark')).toHaveText('Act 2');
  await expect.poll(async () => (await msgs(page)).every(m => /act 2/i.test(m))).toBe(true);
  // The auto chip shows while the search is on; its × clears it.
  const chip = page.locator('[data-id="sysmon/search"]');
  await expect(chip).toHaveText('× Search “act 2”');
  await chip.click();
  await expect(chip).toHaveCount(0);
  await expect.poll(() => rows(page).count()).toBeGreaterThan(5);
});

/** The settings window (decisions §30 "12b2"): tap the Subsystems cell, type, Please. */
async function setSubsystems(page: Page, text: string) {
  const cell = page.locator('.lc-set-scrim .lc-set-cell', { hasText: 'Subsystems' });
  await cell.click();
  const input = page.locator('[data-kit="dialog"] input[data-field]');
  await input.fill(text);
  await input.press('Enter');
  await expect(cell.locator('.lc-set-value')).toHaveText(text || '—');
  await page.keyboard.press('Escape');
  await expect(page.locator('.lc-set-scrim')).toHaveCount(0);
}

test('subsystem filter from the settings, kept with the window', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  await page.locator('.sysmon .sm-list').click({ button: 'right' });
  await page.locator('#ctx .ci', { hasText: 'settings' }).click();
  await setSubsystems(page, 'Echo');
  await expect(rows(page)).toHaveCount(1);
  await expect(rows(page).first().locator('.sm-sub')).toHaveText('Echo');

  // Stored in the layout: a reload shows the same filter.
  await page.reload();
  await connected(page);
  await expect(page.locator('.sysmon')).toBeVisible();
  await expect(rows(page)).toHaveCount(1);

  await page.locator('.sysmon .sm-list').click({ button: 'right' });
  await page.locator('#ctx .ci', { hasText: 'settings' }).click();
  await setSubsystems(page, '');
  await expect.poll(() => rows(page).count()).toBeGreaterThan(5);
});

test('scrolled up: tinted and paused, new lines wait; Jump to newest goes live again', async ({ page }) => {
  await open(page);
  await openSysmon(page);
  for (let i = 0; i < 25; i++) await cmd(page, `Echo "filler ${i}"`);
  const list = page.locator('.sysmon .sm-list');
  const liveBg = await list.evaluate(el => getComputedStyle(el).backgroundColor);
  await list.evaluate(el => { el.scrollTop = 0; el.dispatchEvent(new Event('scroll')); });
  await expect(list).toHaveClass(/\bpaused\b/);
  // A tinted background while paused (a warn tint from the
  // theme, not an error red — decisions §30), after the short fade.
  await expect.poll(() => list.evaluate(el => getComputedStyle(el).backgroundColor)).not.toBe(liveBg);

  await cmd(page, 'Echo "while paused"');
  await expect.poll(() => stored(page)).toContain('while paused');
  expect(await list.evaluate(el => el.scrollTop)).toBe(0); // did not follow
  await expect(list).toHaveClass(/\bpaused\b/);

  // Hold / right-click: Jump to newest.
  await list.click({ button: 'right' });
  await page.locator('#ctx .ci', { hasText: 'Jump to newest' }).click();
  await expect(rows(page).filter({ hasText: 'while paused' })).toHaveCount(1);
  await expect(list).not.toHaveClass(/\bpaused\b/);
  await cmd(page, 'Echo "live again"');
  await expect.poll(() => list.evaluate(el => el.scrollHeight - el.scrollTop - el.clientHeight)).toBeLessThan(4);
});

test('a daily log file keeps the lines (password masked)', async () => {
  const d = new Date();
  const day = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  const file = path.join(server.dir, 'library/logs', `sysmon-${day}.log`);
  await expect.poll(() => { try { return readFileSync(file, 'utf8'); } catch { return ''; } }).toContain('[Admin] Act 2 starts');
  const text = readFileSync(file, 'utf8');
  expect(text).toMatch(/^\d{4}-\d\d-\d\d \d\d:\d\d:\d\d\.\d{3} /m);
  expect(text).toContain('/Password "***"');
  expect(text).not.toContain('hunter2');
});

test('/ws/feeds opens without a login but only open feeds work; the sysmon feed sends the history as one columnar batch', async ({ page }) => {
  // Decisions §30 "12d": the socket opens, says hello, refuses sysmon before a login.
  const refused = await wsMessages(page, `ws://127.0.0.1:${server.http}/ws/feeds`, { op: 'sub', id: 1, feed: 'sysmon' },
    got => got.some(x => x.op === 'nosub'));
  expect(refused[0]).toMatchObject({ op: 'hello', client_protocol: expect.any(Number), user: null });
  expect(refused.find(m => m.op === 'nosub')).toMatchObject({ id: 1, code: 'E-FEED-002' });

  const token = await server.login('Admin');
  const got = await wsMessages(page, `ws://127.0.0.1:${server.http}/ws/feeds?token=${token}`, { op: 'sub', id: 1, feed: 'sysmon' },
    got => got.some(x => !x.op && x.feed === 'sysmon'));
  const first = got.find(x => !x.op && x.feed === 'sysmon');
  expect(first).toMatchObject({ feed: 'sysmon', kind: 'samples', id: 1, epoch: expect.any(String), gap: null });
  const c = first.cols;
  expect(c.seq.length).toBeGreaterThan(20);
  expect(first.from_seq).toBe(c.seq[0]);
  expect(first.to_seq).toBe(c.seq[c.seq.length - 1]);
  expect(c.level.every((l: string) => /info|warn|error|alert/.test(l))).toBe(true);
  expect(c.time_ms[0]).toBeGreaterThan(1_700_000_000_000);
});
