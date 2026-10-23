// Step 12f controls (decisions §30 "12f hardening" … "12f against §32", the boards in
// docs/dev/ui-kit/mockups/2026-10-14-12f/): the gesture channel, masters, the fader, the encoder
// bar, the Oops window, the Sequence pool and the sequence windows.

import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, hold } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => { await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]); });
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page) {
  await server.open(page, '/', 'Admin');
  await connected(page);
}

test('a gesture moves the grand master quietly and files one Faders entry at its end', async ({ page }) => {
  await open(page);
  const from = server.mark();
  await page.evaluate(() => {
    const send = (0, eval)('wsGesture');
    send({ id: 'g1', line: 'Master 2.1 At 90', phase: 'move', label: 'Master 2.1', from: '100', value: '90' });
    send({ id: 'g1', line: 'Master 2.1 At 75', phase: 'move', value: '75' });
    send({ id: 'g1', line: 'Master 2.1 At 63.5', phase: 'end', value: '63.5' });
  });
  await server.waitSysmon(/Faders.*Fader Master 2\.1: 100 → 63\.5 · Admin/, from);
  expect(server.sysmonSince(/Faders/, from)).toHaveLength(1);
  // No per-move command lines.
  expect(server.sysmonSince(/Cmd.*Master 2\.1 At/, from)).toEqual([]);
  expect((await server.command('Master 2.1')).message).toContain('63.5%');
});

test('a refused gesture says so once', async ({ page }) => {
  await open(page);
  await page.evaluate(() => {
    const send = (0, eval)('wsGesture');
    for (const v of ['1', '2', '3']) send({ id: 'g2', line: `Master 1 At ${v}`, phase: 'move', label: 'Master 1', value: v });
    send({ id: 'g2', line: 'Master 1 At 4', phase: 'end', value: '4' });
  });
  await expect(page.locator('.fb-card, .lc-toast, #fb-notices *', { hasText: 'Master 2.1' }).first()).toBeVisible();
});

test('a fader drag is one gesture: the grand master moves, one Faders entry; a tap never jumps', async ({ page }) => {
  await open(page);
  await server.command('Master 2.1 At 100');
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  await page.locator('.ptab', { hasText: 'Showroom' }).first().click();
  const fader = page.locator('[data-kit="showroom"] luma-fader').first();
  // Not scrollIntoViewIfNeeded: it waits for a "stable" box, and the Showroom keeps moving on
  // a slow WebKit (the hang seen under load).
  await fader.evaluate(el => el.scrollIntoView({ block: 'center' }));
  const track = fader.locator('.lc-fd-track');
  const box = (await track.boundingBox())!;
  const x = box.x + box.width / 2;
  // A tap: nothing moves, nothing is sent.
  const tapAt = server.mark();
  await page.mouse.click(x, box.y + box.height * 0.8);
  await page.waitForTimeout(300);
  expect(server.sysmonSince(/Faders/, tapAt)).toEqual([]);
  await expect(fader.locator('.lc-fd-val')).toHaveText('100');
  // A drag down half the travel: relative to where it was.
  const from = server.mark();
  const y0 = box.y + box.height * 0.3;
  await page.mouse.move(x, y0);
  await page.mouse.down();
  for (let i = 1; i <= 10; i++) await page.mouse.move(x, y0 + i * (box.height - 20) * 0.05);
  await expect(fader.locator('.lc-fd-bubble')).toContainText('Master 2.1 At');
  await page.mouse.up();
  const shown = Number(await fader.locator('.lc-fd-val').textContent());
  expect(shown).toBeGreaterThan(40);
  expect(shown).toBeLessThan(60);
  await server.waitSysmon(new RegExp(`Faders.*Fader Master 2\\.1: 100 → ${String(shown).replace('.', '\\.')} · Admin`), from);
  expect(server.sysmonSince(/Faders/, from)).toHaveLength(1);
  expect((await server.command('Master 2.1')).message).toContain(`${shown}%`);
});

test('the encoder bar: strip, a Value drag is one Oops step, the slider panel, Masters, off', async ({ page }) => {
  await open(page);
  const bar = page.locator('#encbar');
  await expect(bar).toBeVisible({ timeout: 15_000 });
  await server.command('Set Screen "EncoderMode" Attributes').catch(() => {});
  await page.evaluate(() => { const c = (window as any).wsCommand; c('Set Screen "EncoderMode" Attributes'); c('Fixture 101 Thru 104'); c('At 20'); });
  // A strip tap switches the feature.
  await bar.locator('.lc-eb-sb[data-group="2"]').click();
  await expect(bar.locator('.lc-eb-sb[data-group="2"]')).toHaveClass(/\bon\b/);
  await bar.locator('.lc-eb-sb[data-group="1"]').click();
  await expect(bar.locator('.lc-eb-sb[data-group="1"]')).toHaveClass(/\bon\b/);
  const value = bar.locator('.lc-eb-fd[data-key="Value"]').first();
  await expect(value.locator('.lc-eb-text')).toHaveText('20%');
  // A drag up: the programmer moves, one Faders entry.
  const before = (await server.command('List Undo')).message.split('\n').length;
  const from = server.mark();
  const box = (await value.boundingBox())!;
  const x = box.x + box.width / 2, y0 = box.y + box.height * 0.7;
  await page.mouse.move(x, y0);
  await page.mouse.down();
  for (let i = 1; i <= 8; i++) await page.mouse.move(x, y0 - i * 8);
  await page.mouse.up();
  await server.waitSysmon(/Faders.*Fader Dim: 20 → /, from);
  await expect(value.locator('.lc-eb-text')).not.toHaveText('20%');
  expect((await server.command('List Undo')).message.split('\n').length).toBe(before + 1);
  // A tap opens the slider panel; − steps; a tap outside closes it.
  await value.click();
  const panel = page.locator('[data-kit="slider-panel"]');
  await expect(panel).toBeVisible();
  await panel.locator('.lc-sp-step[data-d="-1"]').click();
  await page.mouse.click(5, 5);
  await expect(panel).toHaveCount(0);
  // MODE → Masters: the fader bank.
  await bar.locator('[data-act="mode"]').click();
  await page.locator('#ctx .ci', { hasText: 'Masters' }).click();
  await expect(bar.locator('luma-fader').first()).toBeVisible();
  await expect(bar.locator('luma-fader .lc-fd-lab').first()).toContainText('Grand');
  // Your own master follows (the station view's modes counter).
  await page.evaluate(() => (window as any).wsCommand('Master "Highlight" At 30'));
  await expect(bar.locator('luma-fader', { hasText: 'My HL' }).locator('.lc-fd-val')).toHaveText('30');
  // Off for this screen: gone.
  await page.evaluate(() => (window as any).wsCommand('Set Screen "EncoderBar" Off'));
  await expect(bar).toBeHidden();
  await page.evaluate(() => (window as any).wsCommand('Set Screen "EncoderBar" On'));
  await expect(bar).toBeVisible();
});

test('the encoder bar Cue timing modes: the current cue, a fade drag, a timeline handle', async ({ page }) => {
  await open(page);
  const bar = page.locator('#encbar');
  await expect(bar).toBeVisible({ timeout: 15_000 });
  for (const line of ['Fixture 101 Thru 102', 'At 100', 'Store Sequence 7 Cue 1', 'At 40', 'Store Sequence 7 Cue 2',
    'Select Sequence 7', 'Set Sequence 7 Cue 2 "Fade" 2', 'Goto Sequence 7 Cue 2', 'Set Screen "EncoderMode" CueTimingVertical']) {
    await page.evaluate(l => (window as any).wsCommand(l), line);
  }
  // Opened on the current cue (2).
  await expect(bar.locator('.lc-eb-sb.on')).toContainText('Cue 2');
  await expect(bar.locator('.lc-eb-sb.current')).toContainText('Cue 2');
  const fade = bar.locator('.lc-eb-fd[data-key="in_fade"]').first();
  await expect(fade.locator('.lc-eb-text')).toHaveText('2s');
  const from = server.mark();
  const box = (await fade.boundingBox())!;
  const x = box.x + box.width / 2, y0 = box.y + box.height * 0.8;
  await page.mouse.move(x, y0);
  await page.mouse.down();
  for (let i = 1; i <= 6; i++) await page.mouse.move(x, y0 - i * 10);
  await page.mouse.up();
  await server.waitSysmon(/Faders.*Fader Cue 2 Fade: 2 → /, from);
  await expect(fade.locator('.lc-eb-text')).not.toHaveText('2s');
  // Horizontal: drag the fade-end handle to the right.
  await page.evaluate(() => (window as any).wsCommand('Set Screen "EncoderMode" CueTimingHorizontal'));
  const handle = bar.locator('.lc-eb-tl[data-dir="in"] .lc-eb-tl-h.fade').first();
  await expect(handle).toBeVisible();
  const before = await bar.locator('.lc-eb-tl[data-dir="in"] .lc-eb-tl-text').first().textContent();
  const hb = (await handle.boundingBox())!;
  const from2 = server.mark();
  await page.mouse.move(hb.x + hb.width / 2, hb.y + hb.height / 2);
  await page.mouse.down();
  for (let i = 1; i <= 6; i++) await page.mouse.move(hb.x + hb.width / 2 + i * 6, hb.y + hb.height / 2);
  await page.mouse.up();
  await server.waitSysmon(/Faders.*Fader Cue 2 Fade: /, from2);
  await expect(bar.locator('.lc-eb-tl[data-dir="in"] .lc-eb-tl-text').first()).not.toHaveText(before!);
});

test('the Oops window: hold the Oops key, select a step, Oops 3 steps, Redo, a filter chip, tap outside', async ({ page }, info) => {
  await open(page);
  await page.evaluate(() => { const c = (window as any).wsCommand; c('Fixture 101'); for (const v of [10, 20, 30, 40]) c(`At ${v}`); });
  // A phone folds the left sidebar into ⋯ (12e): there the line opens it.
  if (info.project.name === 'phone') await page.evaluate(() => (window as any).wsCommand('Menu "Oops"'));
  else await hold(page, page.locator('#tb-strip [data-key="oops"]'));
  const win = page.locator('[data-kit="oops"]');
  await expect(win).toBeVisible();
  const rows = win.locator('.lc-oops-row:not(.undone)');
  await expect(rows.last()).toContainText('At 40');
  // The third-newest step selects it and every newer one.
  const count = await rows.count();
  await rows.nth(count - 3).click();
  await expect(win.locator('[data-act="oops"]')).toHaveText('Oops 3 steps');
  await win.locator('[data-act="oops"]').click();
  await expect(win.locator('.lc-oops-row.undone')).toHaveCount(3);
  await expect(rows.last()).toContainText('At 10');
  // Redo 2: select the second undone row.
  await win.locator('.lc-oops-row.undone').nth(1).click();
  await expect(win.locator('[data-act="redo"]')).toHaveText('Redo 2 steps');
  await win.locator('[data-act="redo"]').click();
  await expect(win.locator('.lc-oops-row.undone')).toHaveCount(1);
  // A chip is the profile's filter.
  const chip = win.locator('[data-filter="OopsSelection"]');
  const was = await chip.getAttribute('aria-pressed');
  await chip.click();
  await expect(chip).not.toHaveAttribute('aria-pressed', was!);
  await chip.click();
  // A tap outside closes it.
  await page.mouse.click(4, 4);
  await expect(win).toHaveCount(0);
});

test('the Sequence pool: executor cells, Select frames, Go runs, Top holds an echo', async ({ page }) => {
  await open(page);
  for (const line of ['Fixture 101 Thru 102', 'At 100', 'Store Sequence 5 Cue 1', 'At 50', 'Store Sequence 5 Cue 2', 'Store Window "Sequences"']) {
    await page.evaluate(l => (window as any).wsCommand(l), line);
  }
  const tab = page.locator('.ptab', { hasText: 'Sequences' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  await tab.click();
  const win = await tab.getAttribute('data-win');
  const cell = page.locator('luma-pool .lc-pool-cell[data-no="5"]');
  // Off: the first cue framed (a phone shows only that line).
  await expect(cell.locator('.lc-pool-exe span.off')).toContainText('1');
  // Its label is its name.
  await page.evaluate(() => (window as any).wsCommand('Label Sequence 5 "Busk"'));
  await expect(cell).toContainText('Busk');
  // Select (the default action): the gold frame.
  await cell.click();
  await expect(cell).toHaveClass(/\bsel\b/);
  // Go: the blue mark and the active cue in gold.
  await page.evaluate(w => (window as any).wsCommand(`Set Window ${w} "Action" Go`), win);
  await expect.poll(async () => (await server.command(`Set Window ${win} "Action" Go`)).ok).toBeTruthy();
  await page.waitForTimeout(300);
  await cell.click();
  await expect(cell).toHaveClass(/\brun\b/);
  await expect(cell.locator('.lc-pool-exe span.on')).toHaveCount(1);
  // Top: press = Top On, release = Top Off.
  await page.evaluate(w => (window as any).wsCommand(`Set Window ${w} "Action" Top`), win);
  await page.waitForTimeout(500);
  const from = server.mark();
  const box = (await cell.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await server.waitSysmon(/Top On Sequence 5 → ok/, from);
  await page.waitForTimeout(800);
  await page.mouse.up();
  await server.waitSysmon(/Top Off Sequence 5 → ok/, from);
});

test('the Sequence window: CueZero to OffCue, a cue opens its feature groups, a drag is one gesture, Path picks, Edit opens it', async ({ page }) => {
  await open(page);
  for (const line of ['Fixture 101 Thru 102', 'At 100', 'Store Sequence 6 Cue 1', 'At 40', 'Store Sequence 6 Cue 2', 'Label Sequence 6 "Sheet"', 'Select Sequence 6', 'Store Window "Sequence"']) {
    await page.evaluate(l => (window as any).wsCommand(l), line);
  }
  const tab = page.locator('.ptab', { hasText: 'Sheet' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  await tab.click();
  const sheet = page.locator('.seq-win luma-cue-sheet');
  const rows = sheet.locator('.lc-cs-row:not(.lc-cs-head)');
  await expect(rows).toHaveCount(4);
  await expect(rows.first()).toContainText('CueZero');
  await expect(rows.last()).toContainText('OffCue');
  // Go: the running cue is marked.
  await server.command('Go Sequence 6');
  await expect(sheet.locator('.lc-cs-row[data-row="1"]')).toHaveClass(/\brun\b/);
  // A cue opens: its feature groups, the curve.
  await sheet.locator('[data-exp="2"]').click();
  await expect(sheet.locator('.lc-cs-detail[data-detail="2"] .lc-cs-chips button').first()).toBeVisible();
  await expect(sheet.locator('.lc-cs-detail canvas')).toBeVisible();
  // Press and drag the cue's Fade up: live, one Faders entry at its end.
  const fade = sheet.locator('.lc-cs-row[data-row="2"] .lc-cs-t[data-k="fade"]');
  const b = (await fade.boundingBox())!;
  const from = server.mark();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
  await page.mouse.down();
  for (let i = 1; i <= 8; i++) await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2 - i * 11);
  await expect(page.locator('.lc-cs-gauge')).toBeVisible();
  await page.mouse.up();
  await server.waitSysmon(/Faders.*Cue 2 Fade: 0 → 4/, from);
  expect(server.sysmonSince(/Faders/, from)).toHaveLength(1);
  await expect(fade).toHaveText('4');
  // The feature groups that inherit it follow, in brackets.
  await expect(sheet.locator('.lc-cs-t.inh[data-cue="2"][data-k="fade"]').first()).toHaveText('(4)');
  // A Path cell picks the shape.
  await sheet.locator('.lc-cs-p[data-cue="2"][data-g="0"]').click();
  await page.locator('.ci', { hasText: 'SCurve' }).click();
  await expect(sheet.locator('.lc-cs-p[data-cue="2"][data-g="0"]')).toHaveText('SCurve');
  // Edit Sequence opens the sheet as a pop-up; a tap outside closes it.
  await page.evaluate(() => (window as any).wsCommand('Edit Sequence 6'));
  const pop = page.locator('.lc-cs-scrim');
  await expect(pop.locator('.lc-cs-pop-head')).toContainText('Sequence 6 · Sheet');
  await expect(pop.locator('.lc-cs-row[data-row="2"]')).toBeVisible();
  await page.mouse.click(4, 4);
  await expect(pop).toHaveCount(0);
});

test('the Sequence window as a timeline: a column per cue, a drag redraws the span live, a head tap = options, + add timing', async ({ page }) => {
  await open(page);
  for (const line of ['Fixture 101 Thru 102', 'At 100', 'Store Sequence 8 Cue 1', 'At 40', 'Store Sequence 8 Cue 2', 'Label Sequence 8 "Lanes"', 'Select Sequence 8', 'Store Window "Sequence"']) {
    await page.evaluate(l => (window as any).wsCommand(l), line);
  }
  const tab = page.locator('.ptab', { hasText: 'Lanes' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  await tab.click();
  // Layout Timeline is a window setting (a Set Self "line" from the window).
  const win = Number(await tab.getAttribute('data-win'));
  await page.evaluate(w => (window as any).wsCommand('Set Self "Layout" Timeline', w), win);
  const sheet = page.locator('.seq-win luma-cue-sheet');
  const cols = sheet.locator('.lc-tl-col');
  await expect(cols).toHaveCount(4);
  await expect(cols.first()).toContainText('CueZero');
  await expect(cols.last()).toContainText('OffCue');
  // Drag cue 2's Fade up: its span widens live, one Faders entry at the end.
  const fade = sheet.locator('.lc-tl-col[data-tl="2"] .lc-cs-t[data-k="fade"]').first();
  const lane = sheet.locator('.lc-tl-col[data-tl="2"] [data-lane="|fade"]');
  await fade.scrollIntoViewIfNeeded();
  const b = (await fade.boundingBox())!;
  const from = server.mark();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
  await page.mouse.down();
  for (let i = 1; i <= 8; i++) await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2 - i * 11);
  const w = await lane.locator('.lc-tl-shape').evaluate(e => e.getBoundingClientRect().width);
  expect(w).toBeGreaterThan(100);
  await page.mouse.up();
  await server.waitSysmon(/Faders.*Cue 2 Fade: 0 → 4/, from);
  expect(server.sysmonSince(/Faders/, from)).toHaveLength(1);
  await expect(fade).toHaveText('4');
  // + add timing: a feature group, then the type → the slider panel; its first move adds the row.
  await sheet.locator('[data-add="2"]').click();
  await page.locator('.ci', { hasText: 'Dimmer' }).click();
  await page.locator('.ci', { hasText: 'Delay' }).first().click();
  const panel = page.locator('[data-kit="slider-panel"]');
  await expect(panel.locator('.lc-sp-title')).toHaveText('Cue 2 Dimmer Delay');
  await panel.locator('.lc-sp-step[data-d="1"]').click();
  await expect(sheet.locator('.lc-tl-col[data-tl="2"] .lc-cs-t[data-g][data-k="delay"]')).toHaveText('0.1');
  // Escape closes the panel only.
  await page.keyboard.press('Escape');
  await expect(panel).toHaveCount(0);
  await expect(sheet.locator('.lc-tl-col[data-tl="2"]')).toBeVisible();
  // A tap on the head: the cue's options; Go to this cue runs Goto.
  await sheet.locator('[data-head="2"]').click();
  await page.locator('.ci', { hasText: 'Go to this cue' }).click();
  await expect(sheet.locator('.lc-tl-col[data-tl="2"]')).toHaveClass(/\brun\b/);
  // Put the group back to the cue's timing: the row goes.
  await sheet.locator('[data-head="2"]').click();
  await page.locator('.ci', { hasText: "Dimmer: the cue's timing" }).click();
  await expect(sheet.locator('.lc-tl-col[data-tl="2"] .lc-cs-t[data-g]')).toHaveCount(0);
});
