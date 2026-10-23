// One feedback system (handoffs/2026-10-10-feedback-system.md): progress
// pill, "Show saved" flash, cards, critical pop-ups and station statuses —
// driven in two browsers (two users on one server) to check that station
// events reach everyone and that important events can't be muted.

import { expect, test, type Browser, type Page } from '@playwright/test';
import { Server, cmd, connected, kbdMode, promptUser, dialog, wsMessages } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(); });
test.afterAll(async () => { await server.cleanup(); });

/** A page logged in as `user` (own browser context = own cookie). */
async function as(browser: Browser, user: string, srv = server): Promise<Page> {
  const context = await browser.newContext();
  const page = await context.newPage();
  await srv.open(page);
  await connected(page);
  if (user !== 'Admin') {
    await kbdMode(page);
    await cmd(page, `Login "${user}"`);
    await promptUser(page, user);
  }
  await kbdMode(page);
  // Feedback stream up (settings loaded).
  await page.waitForFunction('fb.settings !== null && fb.live');
  return page;
}

const cards = (page: Page) => page.locator('#fb-cards .fb-card');

test('setup: a second user', async ({ browser }) => {
  const admin = await as(browser, 'Admin');
  await cmd(admin, 'Store User "LD"');
  await admin.context().close();
});

test('SaveShow: progress pill and "Show saved" flash for everyone', async ({ browser }) => {
  const admin = await as(browser, 'Admin');
  const ld = await as(browser, 'LD');
  const from = server.mark();
  await cmd(admin, 'SaveShow "feedback"');
  for (const page of [admin, ld]) {
    await expect(page.locator('#fb-progress .fb-pill')).toContainText(/Saving showfile feedback : \d+\.\d{3} MB/);
    await expect(page.locator('#fb-flash')).toHaveClass(/\bon\b/);
    await expect(page.locator('#fb-flash')).toHaveText('Show saved');
  }
  // Gone again: the pill fades, the flash too.
  await expect(admin.locator('#fb-progress .fb-pill')).toHaveCount(0, { timeout: 5000 });
  await expect(admin.locator('#fb-flash')).not.toHaveClass(/\bon\b/, { timeout: 5000 });
  // In the System Monitor log.
  await server.waitSysmon(/Show : Saving showfile: feedback : /, from);
  await server.waitSysmon(/Show : Show saved/, from);
  await admin.context().close();
  await ld.context().close();
});

test('LoadShow: progress + "Show loaded"', async ({ browser }) => {
  const admin = await as(browser, 'Admin');
  await cmd(admin, 'LoadShow "feedback" /NoConfirmation /NoSave');
  await expect(admin.locator('#fb-flash')).toHaveText('Show loaded');
  await server.waitSysmon(/Show : Show loaded/);
  await admin.context().close();
});

test("a user's mode (Highlight) shows on that user's UI only and goes away", async ({ browser }) => {
  // Mode objects are per user (decisions §16): `On Highlight` verb-first.
  const admin = await as(browser, 'Admin');
  const ld = await as(browser, 'LD');
  const hi = (page: Page) => page.locator('#tb-strip [data-key="highlight"].on');
  await cmd(admin, 'On Highlight');
  await expect(hi(admin)).toHaveCount(1);
  await expect(hi(ld)).toHaveCount(0);
  await cmd(admin, 'Off Highlight');
  await expect(hi(admin)).toHaveCount(0);
  await admin.context().close();
  await ld.context().close();
});

test('a user connecting and leaving is a card for everyone; a reload is not', async ({ browser }) => {
  const admin = await as(browser, 'Admin');
  // The previous test's LD left < 3 s ago: wait out the leave grace time, or
  // LD coming back counts as a reconnect (no card, by design).
  await admin.waitForTimeout(3500);
  await admin.evaluate(() => document.querySelectorAll('#fb-cards .fb-card').forEach(c => c.remove()));
  const ld = await as(browser, 'LD');
  await expect(cards(admin).filter({ hasText: 'Connected' }).filter({ hasText: 'LD from 127.0.0.1 (web)' })).toHaveCount(1);

  // A reload within the grace time: no disconnected / connected cards.
  await admin.evaluate(() => document.querySelectorAll('#fb-cards .fb-card').forEach(c => c.remove()));
  await ld.reload();
  await connected(ld);
  await admin.waitForTimeout(4000);
  await expect(cards(admin).filter({ hasText: 'LD from' })).toHaveCount(0);

  // Leaving for real: "Disconnected" after the grace time.
  await ld.context().close();
  await expect(cards(admin).filter({ hasText: 'Disconnected' }).filter({ hasText: 'LD from' })).toHaveCount(1, { timeout: 8000 });
  await admin.context().close();
});

// Waits for the Notification object (decisions §33 3c3: `Set Notifications …` went with the
// target-less forms; where a user's notification choices live is in hardening).
test.fixme('important events can not be muted; permanent mode keeps cards', async ({ browser }) => {
  const admin = await as(browser, 'Admin');
  await admin.waitForTimeout(3500); // leave grace of the previous test's LD
  await cmd(admin, 'Set Notifications network info none');
  await cmd(admin, 'Set UserProfile "NotificationMode" Permanent');
  await admin.waitForFunction('fb.settings.mode === "permanent" && fb.settings.surfaces["network.info"] === "none"');
  await admin.evaluate(() => document.querySelectorAll('#fb-cards .fb-card').forEach(c => c.remove()));
  const ld = await as(browser, 'LD');
  // "Connected" is important: still a card despite network.info = none.
  const card = cards(admin).filter({ hasText: 'LD from' });
  await expect(card).toHaveCount(1);
  await admin.waitForTimeout(6000); // past the 5 s of timed mode
  await expect(card).toHaveCount(1);
  await card.locator('.fb-x').click();
  await expect(card).toHaveCount(0);
  await cmd(admin, 'Set Notifications network info default');
  await cmd(admin, 'Set UserProfile "NotificationMode" Timed');
  await ld.context().close();
  await admin.context().close();
});

test('Restart: a critical pop-up for everyone, then "Server restarting — reconnecting…"', async ({ browser }) => {
  const own = await new Server().start();
  try {
    const admin = await as(browser, 'Admin', own);
    const other = await as(browser, 'Admin', own);
    await cmd(admin, 'Restart /NoConfirmation');
    await expect(dialog(other).locator('.lc-dlg-title span').first()).toHaveText('Server restarting');
    await expect(dialog(other).locator('.lc-dlg-text')).toHaveText('by Admin');
    await dialog(other).locator('.lc-dlg-btn', { hasText: 'OK' }).click();
    await expect(other.locator('#conn-text')).toHaveText('Server restarting — reconnecting…');
    // Restart re-executes the server in place: it comes back by itself.
    await expect(other.locator('#conn-overlay')).not.toHaveClass(/\bopen\b/, { timeout: 15_000 });
  } finally {
    await own.cleanup();
  }
});

test('/ws/feeds (the feedback feed) needs a login', async ({ page }) => {
  // The socket opens (decisions §30 "12d"); the feedback feed waits for `auth`.
  const got = await wsMessages(page, `ws://127.0.0.1:${server.http}/ws/feeds`, { op: 'sub', id: 3, feed: 'feedback' },
    got => got.some(x => x.op === 'nosub' || x.op === 'subbed'));
  const reply = got.find(x => x.op === 'nosub' || x.op === 'subbed');
  expect(reply).toMatchObject({ op: 'nosub', id: 3, code: 'E-FEED-002' });
});
