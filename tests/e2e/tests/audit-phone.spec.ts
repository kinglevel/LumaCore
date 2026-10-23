// The phone audit (2026-10-19, before the phone hardening round): every surface of the recent
// rounds at phone and tablet sizes, captured by Playwright, with what the page can measure about
// itself — sideways overflow, touch targets under 44 px, controls cut off at the screen's edge.
// Run with AUDIT_OUT=<dir>; the normal e2e run skips it. Writes <dir>/<size>/<nn>_<surface>.png
// and <dir>/audit.json.

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { test, type Browser, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';
import { slabCell } from '../support/slab';

const OUT = process.env.AUDIT_OUT;
test.skip(!OUT, 'the phone audit: run with AUDIT_OUT=<dir>');
test.describe.configure({ mode: 'serial' });
test.setTimeout(20 * 60_000);

const ALL_SIZES = [
  { name: 'phone-portrait', width: 390, height: 844, mobile: true },
  { name: 'phone-small', width: 360, height: 740, mobile: true },
  { name: 'phone-landscape', width: 844, height: 390, mobile: true },
  { name: 'tablet-portrait', width: 768, height: 1024, mobile: false },
  { name: 'tablet-landscape', width: 1024, height: 768, mobile: false },
];
// AUDIT_SIZES=phone-portrait,phone-landscape: only those.
const SIZES = ALL_SIZES.filter(s => !process.env.AUDIT_SIZES || process.env.AUDIT_SIZES.split(',').includes(s.name));

const server = new Server();
const report: Record<string, Record<string, unknown>> = {};
test.beforeAll(async () => { if (OUT) await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => {
  if (!OUT) return;
  writeFileSync(path.join(OUT, 'audit.json'), JSON.stringify(report, null, 2));
  await server.cleanup();
});

/** What the page measures about itself: overflow, small targets, controls past the edge. */
async function measure(page: Page) {
  return page.evaluate(() => {
    const vw = innerWidth, vh = innerHeight;
    const label = (el: Element) => (el.getAttribute('aria-label') || el.getAttribute('title') || (el.textContent || '').trim().replace(/\s+/g, ' ') || el.className.toString()).slice(0, 40);
    const small: string[] = [], cut: string[] = [];
    for (const el of document.querySelectorAll('button, [role="button"], input, select, a[href], [data-gutter], .lc-grid-row')) {
      const r = el.getBoundingClientRect();
      if (!r.width || !r.height || getComputedStyle(el).visibility === 'hidden') continue;
      if (r.bottom <= 0 || r.top >= vh || r.right <= 0 || r.left >= vw) continue;
      // Hidden behind a scroll box's edge is fine; past the screen's edge is not.
      if (r.right > vw + 1 || r.left < -1 || r.bottom > vh + 1) cut.push(label(el));
      if (el.matches('.lc-grid-row')) continue;
      if (r.width < 43.5 || r.height < 43.5) small.push(`${label(el)} ${Math.round(r.width)}×${Math.round(r.height)}`);
    }
    return {
      overflowX: document.documentElement.scrollWidth > vw + 1,
      small: small.length, smallExamples: [...new Set(small)].slice(0, 12),
      cut: cut.length, cutExamples: [...new Set(cut)].slice(0, 12),
    };
  });
}

/** Close everything back to the desk. */
async function reset(page: Page) {
  for (let i = 0; i < 4; i++) await page.keyboard.press('Escape').catch(() => {});
  await page.evaluate(() => { try { (0, eval)('closeSetupMenu()'); } catch (_) { /* none */ } }).catch(() => {});
}

async function run(browser: Browser, size: (typeof SIZES)[number]) {
  const dir = path.join(OUT!, size.name);
  mkdirSync(dir, { recursive: true });
  const ctx = await browser.newContext({ viewport: { width: size.width, height: size.height }, hasTouch: true, isMobile: size.mobile, deviceScaleFactor: 2 });
  const page = await ctx.newPage();
  // A surface that can't be reached is an error in the report, never a hang.
  page.setDefaultTimeout(8000);
  await server.open(page);
  await connected(page);
  await kbdMode(page);
  const out: Record<string, unknown> = {};
  report[size.name] = out;
  let n = 0;
  const shot = async (what: string, step: () => Promise<void>) => {
    try {
      await Promise.race([step(), new Promise((_, no) => setTimeout(() => no(new Error('step took over 20 s')), 20_000))]);
      await page.waitForTimeout(500);
      n += 1;
      await page.screenshot({ path: path.join(dir, `${String(n).padStart(2, '0')}_${what}.png`) });
      out[what] = await measure(page);
    } catch (e) {
      out[what] = { error: String(e).split('\n')[0].slice(0, 200) };
    }
  };
  const setup = page.locator('luma-setup');
  const row = (fid: number) => setup.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  const slab = (id: string) => ({ click: async () => (await slabCell(setup, `[data-id="${id}"]`)).click() });

  await shot('desk', async () => {});
  await shot('menu', async () => { await page.evaluate(() => (window as any).openMenu()); });
  await reset(page);
  // The patch dialog, Live.
  await shot('patch-live', async () => { await cmd(page, 'Menu "Patch" /Tab "Patch"'); });
  await shot('patch-row-selected', async () => { await row(104).locator('[data-gutter]').click(); });
  await shot('patch-context-menu', async () => { await row(104).locator('[data-gutter]').click({ button: 'right' }); });
  await reset(page);
  await shot('patch-value-dialog', async () => {
    await cmd(page, 'Menu "Patch" /Tab "Patch"');
    await row(105).locator('[data-gutter]').click();
    await slab('patch_to').click();
  });
  await shot('edit-patch', async () => { await page.locator('[data-kit="value"] [data-more]').click(); });
  await shot('edit-patch-probe', async () => {
    const ep = page.locator('[data-kit="editpatch"]');
    await ep.locator('[data-fn="probe"]').click();
    await ep.locator('.lc-dmap-slot[data-slot="300"]').click();
  });
  await reset(page);
  await shot('patch-edit-card', async () => {
    await cmd(page, 'Menu "Patch" /Tab "Patch"');
    await row(103).locator('[data-gutter]').click();
    // A standard cell is `data-act`, a menu's own cell `data-id`.
    await (await slabCell(setup, '[data-act="edit"]')).click();
  });
  await reset(page);
  await shot('universes-list', async () => { await cmd(page, 'Menu "Patch" /Tab "Universes"'); });
  await shot('universes-map', async () => { await setup.locator('[data-view="map"]').click(); });
  await shot('fixture-types', async () => { await setup.locator('[data-tab="FixtureTypes"]').click(); });
  // The patch dialog, claimed: Insert fixtures.
  await reset(page);
  await shot('patch-draft', async () => { await cmd(page, 'Claim Root.ShowData.Devices'); await cmd(page, 'Menu "Patch" /Tab "Patch"'); });
  await shot('insert-choose', async () => { await slab('insert').click(); });
  await shot('insert-place', async () => {
    const ins = page.locator('[data-kit="insert"]');
    // A phone (§37 4): a tap on a type goes on to Mode, a tap on a mode to Place.
    if (await ins.locator('[data-ptype]').count()) {
      await ins.locator('[data-ptype]').first().click();
      await ins.locator('[data-pmode]').first().click();
      return;
    }
    await ins.locator('luma-grid .lc-grid-row[data-row]').first().click();
    await ins.locator('[data-next]').click();
  });
  await reset(page);
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard').catch(() => {});
  // Every setup menu the station publishes.
  const menus: { name: string }[] = await page.evaluate(async () => {
    const r = await (await fetch('/api/menu-types')).json();
    return (Array.isArray(r) ? r : r.menus || []).map((m: any) => ({ name: m.name }));
  });
  for (const m of menus.filter(m => m.name !== 'Patch')) {
    await reset(page);
    await shot(`menu-${m.name}`, async () => { await cmd(page, `Menu "${m.name}"`); });
  }
  // Every window type, as the desk shows it.
  const types: { name: string; label: string; hidden: boolean }[] = await page.evaluate(async () => (await (await fetch('/api/window-types')).json()));
  for (const t of types.filter(t => !t.hidden)) {
    await reset(page);
    await shot(`window-${t.name}`, async () => { await cmd(page, `Store Window 1 "${t.label}" /Overwrite`); });
  }
  await reset(page);
  await shot('editsetting-show', async () => { await cmd(page, 'EditSetting Show'); });
  await reset(page);
  await ctx.close();
}

for (const size of SIZES) {
  test(`audit ${size.name}`, async ({ browser }) => { await run(browser, size); });
}
