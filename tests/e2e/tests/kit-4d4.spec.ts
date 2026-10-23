// 4d4 — drag and drop + the lasso in the DMX map (decisions §34 round 3, 18; "4d4 hardened"
// 1–4; kit/pointer.js `lift`, kit/dmxmap.js): a block drags along its bar or onto another
// universe's, a run drags in Edit Patch's slot grid, a lifted run held over ‹ › springs the grid
// to the next universe, a lasso over blocks selects them, a tap on a run makes it current; on a
// phone the hold is the press and the map's cards take the Patch list's selection (Patch here).

import path from 'node:path';
import { expect, test, type Locator, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
const setup = (page: Page) => page.locator('luma-setup');
const shot = (page: Page, name: string) => page.screenshot({ path: path.join(REPO, `target/screenshots/patch-${name}.png`) });
const center = async (loc: ReturnType<Page['locator']>) => { const b = (await loc.boundingBox())!; return { x: b.x + b.width / 2, y: b.y + b.height / 2 }; };
/** A mouse drag from one point to another, in steps (the lift engine needs the slop passed). */
async function dragMouse(page: Page, from: { x: number; y: number }, to: { x: number; y: number }, between?: () => Promise<void>) {
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(from.x + 12, from.y + 2, { steps: 3 });
  await page.mouse.move(to.x, to.y, { steps: 10 });
  if (between) await between();
  await page.mouse.up();
}
/** A touch press as the kit sees it: pointer events with pointerType touch, dispatched on what is under the point. */
async function touchDrag(page: Page, from: { x: number; y: number }, to: { x: number; y: number }, holdMs: number, between?: () => Promise<void>) {
  const ev = (type: string, x: number, y: number) => page.evaluate(([type, x, y]) => {
    const el = document.elementFromPoint(x as number, y as number) || document.body;
    el.dispatchEvent(new PointerEvent(type as string, { pointerType: 'touch', pointerId: 9, isPrimary: true, clientX: x as number, clientY: y as number, bubbles: true, cancelable: true, button: 0 }));
  }, [type, x, y]);
  await ev('pointerdown', from.x, from.y);
  await page.waitForTimeout(holdMs);
  await ev('pointermove', from.x + 12, from.y + 12);
  await ev('pointermove', to.x, to.y);
  await page.waitForTimeout(80);
  if (between) await between();
  await ev('pointerup', to.x, to.y);
}

/// Universes 109 and 110 (the rig's long gap, 109–186), the six Photons 101–106 on 109 from
/// 109.001 (30 channels each: 1, 31, 61, 91, 121, 151).
async function twoUniverses(page: Page) {
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Store Universe 109');
  await cmd(page, 'Store Universe 110');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Keep');
  await cmd(page, 'Assign Fixture 101 Thru 106 At DMX 109.001');
}

// The list is in tree order (§40 40d): by number for a test that names rows by their number.
const byNumber = (s: Locator) => s.locator('luma-grid').first().evaluate((g: any) => g.sortBy('no', 'asc'));

test('the Map: a block drags along its universe and onto another, the chip names the landing, a lasso selects', async ({ page }) => {
  test.skip(test.info().project.name === 'phone', 'the phone: cards (decisions §37 5), below');
  await open(page);
  // The rig fills universes 1 and 2: the six Photons 101–106 (30 channels each) move to two
  // universes of their own, 109 and 110.
  await twoUniverses(page);
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await s.locator('[data-tab="Universes"]').click();
  await s.locator('[data-view="map"]').click();
  const map = s.locator('luma-dmxmap');
  const blk = (fid: number) => map.locator(`.lc-dmap-blk[data-no="${fid}"]`);
  const bar = (u: number) => map.locator(`.lc-dmap-bar[data-universe="${u}"]`);
  // (The map redraws on every feed tick for a while: scroll until a redraw doesn't interrupt.)
  const into = async (u: number) => {
    // (One evaluate on the map: the bar is found inside it, so a redraw between can't detach it.)
    await expect.poll(() => map.evaluate((m: any, u: number) => {
      const b = m.querySelector(`.lc-dmap-bar[data-universe="${u}"]`), sc = m.querySelector('.lc-dmap-scroll');
      if (!b || !sc) return false;
      sc.scrollTop = Math.max(0, b.offsetTop - sc.clientHeight / 2);
      return true;
    }, u), { timeout: 20_000 }).toBe(true);
    // Settled: the same place twice, a moment apart (the map puts a saved scroll back after a redraw).
    await expect.poll(async () => {
      const a = JSON.stringify(await bar(u).boundingBox());
      await page.waitForTimeout(300);
      return a === JSON.stringify(await bar(u).boundingBox());
    }).toBe(true);
  };
  await into(109);
  await expect(blk(106)).toBeVisible();
  // Along universe 109: 106 (109.151) to about slot 300.
  const b1 = (await bar(109).boundingBox())!;
  let from = server.mark();
  await dragMouse(page, await center(blk(106)), { x: b1.x + b1.width * 0.6, y: b1.y + b1.height / 2 }, async () => {
    await expect(page.locator('[data-kit="chip"]')).toContainText(/Fixture 106 → 109\.\d{3}/);
    await expect(map.locator('.lc-dmap-ghost')).toHaveCount(1);
    await shot(page, 'drag-map');
  });
  await server.waitSysmon(/\] Assign Fixture 106 At DMX 109\.[23]\d{2} → ok/, from);
  await expect(page.locator('[data-kit="chip"]')).toHaveCount(0);
  // Onto universe 110's bar, the next one: the drop lands there. The map redrew after the
  // Assign and is back at its top: the bars into view first, then the drag.
  // The patch feed redraws the map a few times after the Assign (the rig's plan recompiles):
  // let it settle, then scroll.
  await page.waitForTimeout(1500);
  // Bar 109 centred: 110 sits right under it, inside the scroller (a lift near its edge scrolls it).
  await into(109);
  await expect(blk(105)).toBeVisible();
  const b2 = (await bar(110).boundingBox())!;
  from = server.mark();
  await dragMouse(page, await center(blk(105)), { x: b2.x + b2.width * 0.2, y: b2.y + b2.height / 2 });
  await server.waitSysmon(/\] Assign Fixture 105 At DMX 110\.\d{3} → ok/, from);
  await expect(blk(105)).toBeVisible();
  // (The patch feed redraws the map a moment later on the rig; a loaded WebKit worker takes longer.)
  await expect.poll(() => blk(105).evaluate(el => el.closest('.lc-dmap-bar')!.getAttribute('data-universe')), { timeout: 15_000 }).toBe('110');
  // A lasso from the bar's free end back over the first blocks selects them (the bar where it
  // is now, after the scroll and the redraw).
  await page.waitForTimeout(1000);
  await into(109);
  const b1b = (await bar(109).boundingBox())!;
  await dragMouse(page, { x: b1b.x + b1b.width * 0.5, y: b1b.y + b1b.height / 2 }, { x: b1b.x + 4, y: b1b.y + 4 });
  await expect(map.locator('.lc-dmap-blk.sel')).toHaveCount(4);
  await expect(blk(101)).toHaveClass(/sel/);
});

test('Edit Patch: a run drags in the grid, a tap on a run makes it current, ‹ › spring under a lifted run', async ({ page }) => {
  test.skip(test.info().project.name === 'phone', 'the phone: the hold is the press, below');
  await open(page);
  // The grid draws the rows in view: fixtures 1 and 2 (the first rows) on a universe of their
  // own, 111, with 112 free after it for the spring (the rig's long gap; the map test above
  // holds 111 and 112 on this server).
  await cmd(page, 'Claim Root.ShowData.Devices');
  await cmd(page, 'Store Universe 111');
  await cmd(page, 'Store Universe 112');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Keep');
  await cmd(page, 'Assign Fixture 1 At DMX 111.001');
  await cmd(page, 'Assign Fixture 2 At DMX 111.057');
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await s.locator('[data-tab="Patch"]').click();
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  await byNumber(s);
  await s.locator('luma-grid').first().evaluate((g: any) => g.none?.());
  await row(1).locator('[data-gutter]').click();
  await row(2).locator('[data-gutter]').click({ modifiers: ['ControlOrMeta'] });
  await s.locator('luma-slab [data-id="edit_patch"]:visible').click();
  const ep = page.locator('[data-kit="editpatch"]');
  const slot = (n: number) => ep.locator(`.lc-dmap-slot[data-slot="${n}"]`);
  await expect(ep.locator('.lc-ep-chip')).toContainText('2 fixtures');
  await expect(ep.locator('.lc-dmap-slot.cur .fid')).toHaveText('1');
  // 1's run (111.001) to 111.101: the ribbon ghost on the way, one line on the drop.
  let from = server.mark();
  await dragMouse(page, await center(slot(1)), await center(slot(101)), async () => {
    await expect(ep.locator('.lc-dmap-slot.ghost.st')).toHaveCount(1);
    await expect(page.locator('[data-kit="chip"]')).toContainText('Fixture 1 → 111.101');
    await shot(page, 'drag-grid');
  });
  await server.waitSysmon(/\] Assign Fixture 1 At DMX 111\.101 → ok/, from);
  // A tap on 2's run makes it current (no patching over it).
  await slot(58).click();
  await expect(ep.locator('.lc-dmap-slot.cur .fid')).toHaveText('2');
  // Hold 2's run over › past the hold time: the grid springs to universe 112, still lifted.
  const next = ep.locator('[data-u="1"]');
  from = server.mark();
  await dragMouse(page, await center(slot(58)), await center(next), async () => {
    await expect(next).toHaveClass(/lc-sprung/);
    await expect(ep.locator('[data-uname]')).toContainText('Universe 112', { timeout: 3000 });
    await page.mouse.move(...Object.values(await center(slot(10))) as [number, number], { steps: 6 });
    await expect(page.locator('[data-kit="chip"]')).toContainText('Fixture 2 → 112.0');
    await shot(page, 'drag-sprung');
  });
  await server.waitSysmon(/\] Assign Fixture 2 At DMX 112\.0\d{2} → ok/, from);
  await page.keyboard.press('Escape');
  await cmd(page, 'Assign Fixture 1 At DMX 1.001');
  await cmd(page, 'Assign Fixture 2 At DMX 1.044');
});
test('a phone: the cards take the Patch list\'s selection (Patch here), the hold is the press in the grid', async ({ page }) => {
  test.skip(test.info().project.name !== 'phone', 'the phone only');
  await open(page);
  // Two universes of the Photons' own (the rig fills 1 and 2); 104 is the grid's row 104, so
  // the first row — Fixture 1, an Aeco 30 — is the one the phone picks.
  await twoUniverses(page);
  await cmd(page, 'Menu "Patch" /Tab "Patch"');
  const s = setup(page);
  const row = (fid: number) => s.locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`^\\s*${fid}\\b`) });
  await byNumber(s);
  await s.locator('luma-grid').first().evaluate((g: any) => g.none?.());
  await row(1).evaluate(el => el.scrollIntoView({ block: 'center' }));
  await row(1).locator('[data-gutter]').click();
  await s.locator('[data-tab="Universes"]').click();
  await s.locator('[data-view="map"]').click();
  const map = s.locator('luma-dmxmap');
  const go = map.locator('.lc-dmap-go[data-go="110"]');
  await go.scrollIntoViewIfNeeded();
  await expect(go).toContainText('Patch here');
  await expect(go).toContainText('110.001');
  let from = server.mark();
  await go.click();
  await server.waitSysmon(/\] Assign Fixture 1 At DMX 110\.001 → ok/, from);
  await expect(map.locator('.lc-dmap-card[data-zoom="110"]')).toContainText('1 fixture');
  await shot(page, 'phone-patch-here');
  // The card opens Edit Patch with the Patch list's selection; hold a run, drag, let go.
  await map.locator('.lc-dmap-card[data-zoom="110"]').click();
  const ep = page.locator('[data-kit="editpatch"]');
  await expect(ep.locator('.lc-ep-chip')).toContainText('1 fixture');
  const slot = (n: number) => ep.locator(`.lc-dmap-slot[data-slot="${n}"]`);
  await expect(slot(1)).toHaveClass(/cur/);
  from = server.mark();
  await touchDrag(page, await center(slot(2)), await center(slot(32)), 700, async () => {
    await expect(page.locator('[data-kit="chip"]')).toContainText('Fixture 1 → 110.031');
    await expect(ep.locator('.lc-dmap-slot.ghost.cont .fid.again').first()).toHaveText('1');
    await shot(page, 'drag-phone');
  });
  await server.waitSysmon(/\] Assign Fixture 1 At DMX 110\.031 → ok/, from);
  await page.keyboard.press('Escape');
});
