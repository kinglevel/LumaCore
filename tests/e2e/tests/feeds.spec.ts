// Client feeds (decisions §30 "12b2 hardening": one feed rule, one /ws/feeds
// per page): the Realtime window gets every render frame and draws its
// graphs; the DMX window gets binary frames and follows a change at once;
// a hidden page drops fast feeds to 250 ms; the Showroom shows the new kit
// elements (luma-vlist, luma-graph).

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}

/** Add a window like a user: tab context menu › Add window… › type › Enter. */
async function addWindow(page: Page, search: string, label: string) {
  await page.locator('.ptab').first().click({ button: 'right' });
  await page.locator('#ctx .ci', { hasText: 'Add window' }).click();
  await expect(page.locator('#picker-overlay')).toBeVisible();
  await page.keyboard.type(search);
  await expect(page.locator('.pk-item.sel')).toContainText(label);
  await page.keyboard.press('Enter');
}

/** Count what the page's feed socket sends and receives. */
async function tapSocket(page: Page) {
  await page.evaluate(() => {
    const w = window as any;
    w.feedLog = { sent: [] as any[], binary: 0, text: 0 };
    const c = w.feeds();
    const recv = c.receive.bind(c);
    c.receive = (d: any) => { if (d instanceof ArrayBuffer) w.feedLog.binary++; else w.feedLog.text++; recv(d); };
    const proto = WebSocket.prototype as any;
    const send = proto.send;
    proto.send = function (m: any) { if (this === c.ws) w.feedLog.sent.push(JSON.parse(m)); return send.call(this, m); };
  });
}

test('the Realtime window: every frame into 10 s graphs, the tables at 4 Hz', async ({ page }) => {
  await open(page);
  await tapSocket(page);
  await addWindow(page, 'realtime', 'Realtime');
  await expect(page.locator('.rt luma-graph')).toHaveCount(8); // Frame, six stages, Headroom
  await expect(page.locator('.rt .rt-g-name').first()).toHaveText('Frame');
  await expect(page.locator('.rt .rt-g-name').last()).toHaveText('Headroom');
  // Subscribed at the native 60 fps; the ring fills with one point per frame.
  const sent = await page.evaluate('feedLog.sent') as any[];
  expect(sent).toContainEqual(expect.objectContaining({ op: 'sub', feed: 'health.render', period_ms: 16 }));
  expect(sent).toContainEqual(expect.objectContaining({ op: 'sub', feed: 'health', period_ms: 250 }));
  const filled = () => page.evaluate('[...rtViews.values()][0].ring.length') as Promise<number>;
  const before = await filled();
  await page.waitForTimeout(1000);
  // ~60 frames a second arrive (none skipped); the ring holds 600.
  expect((await filled()) - before).toBeGreaterThan(Math.min(40, 600 - before));
  await expect(page.locator('.rt .rt-g-now').first()).toHaveText(/µs|ms/);
  await expect(page.locator('.rt-head')).toContainText('Rate 60 Hz');
  await expect(page.locator('.rt-clock table')).toBeVisible();
  // Something was drawn on the canvas.
  const inked = await page.locator('.rt luma-graph canvas').first().evaluate((c: HTMLCanvasElement) => {
    const d = c.getContext('2d')!.getImageData(0, 0, c.width, c.height).data;
    for (let i = 3; i < d.length; i += 4) if (d[i]) return true;
    return false;
  });
  expect(inked).toBe(true);
});

test('a hidden page drops fast feeds to 250 ms and resumes after its last frame', async ({ page }) => {
  await open(page);
  await expect(page.locator('.rt luma-graph')).toHaveCount(8);
  await tapSocket(page);
  const setHidden = (hidden: boolean) => page.evaluate(h => {
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => (h ? 'hidden' : 'visible') });
    document.dispatchEvent(new Event('visibilitychange'));
  }, hidden);
  await setHidden(true);
  await expect.poll(() => page.evaluate('feedLog.sent.filter(m => m.feed === "health.render").map(m => m.period_ms)'))
    .toContainEqual(250);
  const resub = (await page.evaluate('feedLog.sent.find(m => m.feed === "health.render")')) as any;
  expect(resub.since).toBeGreaterThan(0); // no frame is sent twice or lost
  expect(typeof resub.epoch).toBe('string');
  await setHidden(false);
  await expect.poll(() => page.evaluate('feedLog.sent.filter(m => m.feed === "health.render").map(m => m.period_ms)'))
    .toContainEqual(16);
});

test('the DMX window: binary frames, a change shows at once', async ({ page }) => {
  await open(page);
  await tapSocket(page);
  await addWindow(page, 'dmx', 'DMX');
  await expect(page.locator('.dmx .dx-cell')).toHaveCount(512);
  const sent = await page.evaluate('feedLog.sent') as any[];
  expect(sent).toContainEqual(expect.objectContaining({ op: 'sub', feed: 'dmx', universes: [1], period_ms: 16 }));
  await expect.poll(() => page.evaluate('feedLog.binary')).toBeGreaterThan(0);
  // Idle universes send nothing.
  const idle = await page.evaluate('feedLog.binary') as number;
  await page.waitForTimeout(500);
  expect((await page.evaluate('feedLog.binary')) as number).toBeLessThanOrEqual(idle + 1);
  // Fixture 1 (an Aeco 30) is the rig's 1.001; its Dimmer is 1.032.
  const dimmer = page.locator('.dmx .dx-cell').nth(31);
  await expect(dimmer.locator('.dx-a')).toHaveText('Dimmer');
  const r = await server.command('Fixture 1 At 100');
  expect(r.ok, r.message).toBe(true);
  await expect(dimmer.locator('.dx-v')).toHaveText('100%');
  await server.command('Fixture 1 At 0');
  await expect(dimmer.locator('.dx-v')).toHaveText('0%');
});

test('the Showroom shows the virtual list and the graph', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  await page.locator('.ptab', { hasText: 'Showroom' }).first().click();
  const vl = page.locator('[data-kit="showroom"] luma-vlist');
  await expect(vl).toBeVisible();
  // 10 000 items, a few dozen rows in the page.
  const rows = await vl.locator('.lc-vl-row').count();
  expect(rows).toBeGreaterThan(5);
  expect(rows).toBeLessThan(60);
  await page.locator('[data-kit="showroom"] .lc-sr-vl input').fill('#9999 ');
  await expect(vl.locator('.lc-vl-row')).toHaveCount(1);
  await expect(page.locator('[data-kit="showroom"] luma-graph')).toBeVisible();
});

// Decisions §30 "12d hardening": everything live on /ws/feeds — the menus,
// Backup, Users & Profiles and the pool editor follow the station without a
// poll, and nothing opens an event stream.
test('menus and Backup live on feeds: no poll, no event stream', async ({ page }) => {
  const api: string[] = [];
  page.on('request', r => { const u = new URL(r.url()); if (u.pathname.startsWith('/api/')) api.push(u.pathname); });
  await open(page);
  await tapSocket(page);

  // Network (the setup frame, 12d part 4): drawn from the `network` feed; a
  // command's change arrives on it.
  await page.evaluate(() => (window as any).wsCommand('Menu "Network" /Tab Networks'));
  await expect(page.locator('luma-setup luma-grid')).toBeVisible();
  await server.command('Store Network "FeedNet"');
  await expect(page.locator('luma-setup luma-grid [data-col="name"]', { hasText: 'FeedNet' })).toBeVisible();

  // LumaSessions: a station setting changed elsewhere shows up.
  await page.evaluate(() => (window as any).wsCommand('Menu "LumaSessions"'));
  await expect(page.locator('luma-setup .lc-setup-name')).toHaveText('LumaSessions');
  await server.command('Set Station "Location" "Balcony" ');
  await expect(page.locator('luma-setup luma-grid [data-col="location"]').first()).toHaveText('Balcony');
  await page.evaluate('closeSetupMenu()');

  // Backup (the setup frame, 12d part 5): a show saved elsewhere joins the
  // Library on the `shows` feed.
  await page.evaluate(() => (window as any).wsCommand('Menu "Backup" /Tab Library'));
  await expect(page.locator('luma-setup .lc-setup-name')).toHaveText('Backup');
  await server.command('SaveShow "feedcopy"');
  await expect(page.locator('luma-setup luma-grid [data-col="name"]', { hasText: 'feedcopy' })).toBeVisible();
  await page.evaluate('closeSetupMenu()');

  const sent = await page.evaluate('feedLog.sent') as any[];
  for (const feed of ['network', 'lumasessions', 'shows'])
    expect(sent).toContainEqual(expect.objectContaining({ op: 'sub', feed }));
  // Closing a menu ends its subscription.
  const subOf = (feed: string) => sent.filter(m => m.op === 'sub' && m.feed === feed).map(m => m.id);
  expect(sent.filter(m => m.op === 'unsub').map(m => m.id)).toEqual(expect.arrayContaining([...subOf('lumasessions'), ...subOf('shows')]));

  // Open, idle: not one request while the station sends what changes.
  await page.evaluate(() => (window as any).wsCommand('Menu "DMXProtocols"'));
  await expect(page.locator('luma-setup .lc-setup-strip')).toBeVisible();
  await page.waitForTimeout(500);
  api.length = 0;
  await page.waitForTimeout(3000);
  expect(api).toEqual([]);
  await page.evaluate('closeSetupMenu()');

  // Never an event stream, on any page load.
  const all = await page.evaluate(() => performance.getEntriesByType('resource').map(e => new URL(e.name).pathname));
  expect(all.filter(p => p.includes('/events'))).toEqual([]);
});

test('the lock screen reads the open station feed: a new user shows without a reload', async ({ page }) => {
  await open(page);
  await tapSocket(page);
  await page.evaluate('logout()');
  await expect(page.locator('luma-surface')).toBeVisible();
  // Logged out, the socket stays: the station feed goes on.
  await server.command('Store User "Feedy"');
  await expect(page.locator('luma-surface luma-avatar[data-user="Feedy"]')).toHaveCount(1);
  const sent = await page.evaluate('feedLog.sent') as any[];
  expect(sent).toContainEqual({ op: 'auth', token: null });
  await server.command('Delete User "Feedy" /NoConfirmation');
});
