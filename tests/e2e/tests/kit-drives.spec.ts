// Drives (decisions §38, part 38a): the station's drives — the local library, the volumes it
// finds, the folders added by hand. The server runs with `LUMACORE_DRIVES_PROBE=<dir>`: every
// folder in it is a "stick" (lumacore-drives probe), so one comes and goes as a folder here. The
// watcher reads it every 2 s. Checked: the Drives menu's rows on the station feed, `Set Drive
// "usb1" "Library" Yes` makes LumaCore/library on it, `SaveShow … /Drive "usb1"` writes there
// and `LoadShow … /Drive "usb1"` reads it, the tree shell's `~usb1`, a stick put in while the
// menu is open appears by itself, a folder added by hand is a Folder drive, Forget frees the id.

import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, cmd, connected, dialog, kbdMode } from '../support/server';
import { slabCell } from '../support/slab';

const server = new Server();
const probe = mkdtempSync(path.join(tmpdir(), 'lumacore-drives-'));
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  mkdirSync(path.join(probe, 'KINGSTON'));
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')], { LUMACORE_DRIVES_PROBE: probe });
});
test.afterAll(async () => { await server.cleanup(); rmSync(probe, { recursive: true, force: true }); });

async function open(page: Page) {
  await server.open(page);
  await connected(page);
  await kbdMode(page);
}
const setup = (page: Page) => page.locator('luma-setup');
const row = (page: Page, id: string) => setup(page).locator('luma-grid .lc-grid-row[data-row]', { hasText: new RegExp(`\\b${id}\\b`) });

test('the Drives menu: the local library and a stick the station found; Library made by a Set; the station feed follows', async ({ page }) => {
  await open(page);
  await server.waitSysmon(/Drives\s*: usb1 "KINGSTON" in/);
  await cmd(page, 'Menu "Drives"');
  const s = setup(page);
  await expect(s.locator('[data-tab="Drives"]')).toHaveClass(/\bon\b/);
  await expect(row(page, 'local-library')).toContainText('Internal');
  await expect(row(page, 'usb1')).toContainText('KINGSTON');
  await expect(row(page, 'usb1')).toContainText('USB');
  await expect(row(page, 'usb1').locator('[data-col="library"]')).toContainText(/\bNo\b/);
  // Reading never makes the tree; the first write (here: the setting) does.
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library'))).toBe(false);
  let from = server.mark();
  await cmd(page, 'Set Drive "usb1" "Library" Yes');
  await server.waitSysmon(/Drives\s*: usb1 "KINGSTON" library created/, from);
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows'))).toBe(true);
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'README'))).toBe(true);
  await expect(row(page, 'usb1').locator('[data-col="library"]')).toContainText(/\bYes\b/);
  // A second stick put in while the menu is open: the watcher sees it, usb2 appears by itself.
  from = server.mark();
  mkdirSync(path.join(probe, 'SanDisk'));
  await server.waitSysmon(/Drives\s*: usb2 "SanDisk" in/, from, 6000);
  await expect(row(page, 'usb2')).toContainText('SanDisk');
  // Out: the row stays, greyed Out; back in: the same id.
  from = server.mark();
  rmSync(path.join(probe, 'SanDisk'), { recursive: true });
  await server.waitSysmon(/Drives\s*: usb2 "SanDisk" out/, from, 6000);
  await expect(row(page, 'usb2').locator('[data-col="state"]')).toContainText('Out');
  from = server.mark();
  mkdirSync(path.join(probe, 'SanDisk'));
  await server.waitSysmon(/Drives\s*: usb2 "SanDisk" in/, from, 6000);
  // Forget: the id is free again.
  from = server.mark();
  await cmd(page, 'Delete Drive "usb2" /NoConfirmation');
  await server.waitSysmon(/Drives\s*: usb2 forgotten/, from);
  await expect(row(page, 'usb2')).toHaveCount(0);
  rmSync(path.join(probe, 'SanDisk'), { recursive: true });
});

test('a show saves to and loads from a drive with /Drive; the tree shell\'s ~usb1 is its library', async ({ page }) => {
  await open(page);
  let from = server.mark();
  await cmd(page, 'SaveShow "On the stick" /Drive "usb1"');
  await server.waitSysmon(/\] SaveShow "On the stick" \/Drive "usb1" → ok/, from);
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows', 'On the stick.luma'))).toBe(true);
  // Not in the local library.
  expect(existsSync(path.join(server.dir, 'library', 'shows', 'On the stick.luma'))).toBe(false);
  from = server.mark();
  await cmd(page, 'LoadShow "On the stick" /Drive "usb1" /NoSave /NoConfirmation');
  await server.waitSysmon(/\] LoadShow "On the stick" \/Drive "usb1" \/NoSave \/NoConfirmation → ok/, from);
  const out = await server.command('LoadShow "On the stick" /Drive "usb9" /NoSave /NoConfirmation');
  expect(out.message).toMatch(/No drive "usb9"/);
  // The fence: ~usb1 is the drive's library, ~ the local one, Root.Drives lists the drives.
  const ls = await server.command('ls ~usb1');
  expect(ls.message).toMatch(/shows/);
  expect(ls.message).toMatch(/mvr/);
  const drives = await server.command('ls Root.Drives');
  expect(drives.message).toMatch(/local-library/);
  expect(drives.message).toMatch(/usb1/);
  const home = await server.command('ls ~');
  expect(home.message).toMatch(/local-library/);
  expect(home.message).toMatch(/station_settings\.json/);
});

test('a folder added by hand is a Folder drive; Info Drive; the local library is Drive 1 for good', async ({ page }) => {
  await open(page);
  const nas = path.join(probe, '..', path.basename(probe) + '-nas');
  mkdirSync(nas, { recursive: true });
  let from = server.mark();
  await cmd(page, `Store Drive "Shows NAS" /Path "${nas}"`);
  await server.waitSysmon(/Drives\s*: folder1 "Shows NAS" added/, from);
  const info = await server.command('Info Drive "folder1"');
  expect(info.message).toMatch(/Kind\s+Folder/);
  expect(info.message).toMatch(/State\s+In/);
  const list = await server.command('List Drive');
  expect(list.message).toMatch(/local-library\s+Internal/);
  expect(list.message).toMatch(/folder1\s+Folder/);
  const no = await server.command('Delete Drive "local-library" /NoConfirmation');
  expect(no.message).toMatch(/Drive 1 for good/);
  await cmd(page, 'Delete Drive "folder1" /NoConfirmation');
  rmSync(nas, { recursive: true, force: true });
});

test('the files feed lists a drive\'s folder by kind; This device uploads onto a drive through one route (38b)', async ({ page }) => {
  await open(page);
  // The show saved on the stick above, with its manifest's facts.
  const sub = (params: Record<string, string>) => page.evaluate(p => new Promise<any>(res => (0, eval)('feeds')().sub('files', p, { batch: (m: any) => res(m.data) })), params);
  const shows = await sub({ drive: 'usb1', folder: 'shows', kind: 'shows' });
  const row = shows.files.find((f: any) => f.name === 'On the stick');
  expect(row).toBeTruthy();
  expect(row.format).toBe(31);
  expect(row.saved_by).toBe('Admin');
  // The local library's shows: the demo show the server started with is not saved here; the list is the folder as it is.
  const local = await sub({ drive: 'local-library', folder: 'shows', kind: 'shows' });
  expect(Array.isArray(local.files)).toBe(true);
  // An unknown drive says so, with no rows.
  const none = await sub({ drive: 'usb9', folder: 'shows', kind: 'shows' });
  expect(none.error).toMatch(/usb9/);
  // Upload from the browser onto the stick's mvr folder; the feed shows it.
  const up = await page.evaluate(async () => {
    const r = await fetch('/api/upload?drive=usb1&folder=mvr&name=rig.mvr', { method: 'POST', body: new Blob(['not really an mvr']) });
    return r.json();
  });
  expect(up.ok).toBe(true);
  expect(up.name).toBe('mvr/rig.mvr');
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'mvr', 'rig.mvr'))).toBe(true);
  const mvr = await sub({ drive: 'usb1', folder: 'mvr', kind: 'mvr' });
  expect(mvr.files.map((f: any) => f.name)).toContain('rig.mvr');
  // A path that leaves the library is refused.
  const bad = await page.evaluate(async () => (await fetch('/api/upload?drive=usb1&folder=../..&name=x', { method: 'POST', body: 'x' })).json());
  expect(bad.ok).toBe(false);
  // An MVR on the stick (38c-2): the real file replaces the fake uploaded above; the feed's row
  // carries the scene's facts, read from the zip's scene entry alone; a fake keeps its error.
  copyFileSync(path.join(REPO, 'tests/fixtures/rig.mvr'), path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'mvr', 'rig.mvr'));
  await page.evaluate(async () => (await fetch('/api/upload?drive=usb1&folder=mvr&name=broken.mvr', { method: 'POST', body: 'x' })).json());
  const rows = async () => (await sub({ drive: 'usb1', folder: 'mvr', kind: 'mvr' })).files as any[];
  await expect.poll(async () => (await rows()).find(f => f.file === 'rig.mvr')?.fixtures, { timeout: 10_000 }).toBe(6);
  const mvrs = await rows();
  expect(mvrs.find(f => f.file === 'rig.mvr')).toMatchObject({ scene: 'Vectorworks 2026', layers: 2, groups: 2, fixtures: 6, gdtf_embedded: 1, gdtf_missing: 1, gdtf_text: '1 embedded · 1 missing' });
  expect(mvrs.find(f => f.file === 'broken.mvr')?.error).toMatch(/not an MVR/);
});

test('the files element in the Showroom: the drives rail, a stick\'s shows, a pick, This device (38b)', async ({ page }) => {
  await open(page);
  await page.evaluate(() => (window as any).wsCommand('Store Window "Showroom"'));
  const tab = page.locator('.ptab', { hasText: 'Showroom' }).first();
  await expect(tab).toBeVisible({ timeout: 10_000 });
  if (!(await tab.evaluate(t => t.classList.contains('on')))) await tab.click();
  const lf = page.locator('[data-kit="showroom"] luma-files');
  await lf.scrollIntoViewIfNeeded();
  await expect(lf.locator('[data-drive="local-library"]')).toBeVisible();
  await expect(lf.locator('[data-drive="usb1"]')).toContainText('KINGSTON');
  // The phone rule (38c): in portrait the rail is a chip strip over the grid — the strip is one
  // row, the grid takes the whole width.
  if (test.info().project.name === 'phone') {
    await expect(lf).toHaveAttribute('data-phone', 'up');
    const rail = await lf.locator('.lc-files-rail').boundingBox();
    const grid = await lf.locator('luma-grid').boundingBox();
    expect(rail!.height).toBeLessThan(80);
    expect(grid!.width).toBeGreaterThan(rail!.width * 0.9);
  } else {
    await expect(lf).not.toHaveAttribute('data-phone', 'up');
  }
  // The stick's shows folder: the show saved there above.
  await lf.locator('[data-drive="usb1"]').click();
  await expect(lf.locator('[data-crumbs]')).toContainText('usb1');
  const row = lf.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'On the stick' });
  await expect(row).toBeVisible();
  if (test.info().project.name !== 'phone') await expect(row).toContainText('31'); // the format column folds on a phone
  // A double-tap picks it.
  await row.locator('[data-col="name"]').dblclick();
  await expect(page.locator('[data-kit="showroom"] .lc-sr-note', { hasText: 'pick usb1' })).toContainText('shows/On the stick');
  if (test.info().project.name === 'desktop' || test.info().project.name === 'phone') await lf.screenshot({ path: path.join(REPO, `target/screenshots/files-${test.info().project.name}.png`) });
  // ★ Library after a folder; This device uploads into the current folder.
  await lf.locator('[data-go="mvr"]').count().then(() => {});
  await lf.locator('[data-upload]').setInputFiles({ name: 'from browser.luma', mimeType: 'application/octet-stream', buffer: Buffer.from('x') });
  await expect(page.locator('[data-kit="showroom"] .lc-sr-note', { hasText: 'uploaded from browser.luma' })).toBeVisible();
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows', 'from browser.luma'))).toBe(true);
  // The feed follows the write: the new row appears.
  await expect(lf.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'from browser' })).toBeVisible();
});

test('Backup › Library on the drives rail: the stick\'s shows, Save as… / Delete / Load… on it (38b-3)', async ({ page }, info) => {
  await open(page);
  await cmd(page, 'Menu "Backup"');
  const s = setup(page);
  const lf = s.locator('luma-files');
  await expect(lf.locator('[data-drive="local-library"]')).toHaveClass(/\bon\b/);
  // The stick: its shows, the caption says where we are.
  await lf.locator('[data-drive="usb1"]').click();
  await expect(lf.locator('[data-drive="usb1"]')).toHaveClass(/\bon\b/);
  const stick = s.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'On the stick' });
  await expect(stick).toBeVisible();
  if (info.project.name !== 'phone') await expect(s.locator('.lc-grid-cap')).toContainText('on usb1'); // the phone's grid has no caption bar
  if (info.project.name === 'desktop' || info.project.name === 'phone') await s.screenshot({ path: path.join(REPO, `target/screenshots/backup-drives-${info.project.name}.png`) });
  // Save as… writes on the stick.
  let from = server.mark();
  await (await slabCell(s, '[data-own][data-id="saveas"]')).click();
  let d = dialog(page);
  await d.locator('input').first().fill('Second on stick');
  await d.locator('.lc-dlg-btn', { hasText: 'Please' }).click();
  await server.waitSysmon(/\] SaveShow "Second on stick" \/Drive "usb1" → ok/, from);
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows', 'Second on stick.luma'))).toBe(true);
  const second = s.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'Second on stick' });
  await expect(second).toBeVisible();
  // Save as… made "Second on stick" the loaded show. Delete from the slab names the drive: the
  // station's question, its answer line carries /Drive; the file goes to the stick's trash.
  await stick.locator('[data-col="name"]').click();
  from = server.mark();
  await (await slabCell(s, '[data-act="delete"]')).click();
  d = dialog(page);
  await expect(d).toContainText('On the stick');
  await d.locator('.lc-dlg-btn', { hasText: /^Delete$/ }).first().click();
  await server.waitSysmon(/\] DeleteShow "On the stick" \/Drive "usb1" \/NoConfirmation → ok/, from);
  expect(existsSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows', 'On the stick.luma'))).toBe(false);
  expect(readdirSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'shows', '.trash')).some(f => f.endsWith('_On the stick.luma'))).toBe(true);
  await expect(stick).toHaveCount(0);
  if (info.project.name !== 'phone') {
    // Load…: a double-tap opens the dialog on the stick's file; its line carries the drive.
    await second.locator('[data-col="name"]').dblclick();
    d = dialog(page);
    await expect(d).toContainText('Load show · Second on stick');
    from = server.mark();
    await d.locator('.lc-dlg-btn', { hasText: /^Load$/ }).click();
    // The station asks (the one dialog path); its answer carries the drive.
    await server.waitSysmon(/\] LoadShow "Second on stick" \/Drive "usb1" → asks: Load Show/, from);
    const ask = dialog(page);
    await expect(ask).toContainText('Load show "Second on stick"?');
    await ask.locator('.lc-dlg-btn', { hasText: /^Load$/ }).click();
    await server.waitSysmon(/\] LoadShow "Second on stick" \/Drive "usb1" \/NoConfirmation → ok/, from, 15_000);
  }
});

test('the station\'s "which file?" ask is the files element; Import / Export UserProfile and Import FixtureType take /Drive; Insert fixtures\' Library is the element (38b-4)', async ({ page }, info) => {
  await open(page);
  // Export your profile onto the stick: the first write there made the tree already.
  let from = server.mark();
  await cmd(page, 'Export UserProfile /Drive "usb1"');
  await server.waitSysmon(/\] Export UserProfile \/Drive "usb1" → ok/, from);
  const profiles = readdirSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'userprofiles')).filter(f => f.endsWith('.lumaprofile'));
  expect(profiles.length).toBeGreaterThan(0);
  // A bare Import UserProfile: the station asks with the files element on userprofiles/, the
  // drives on its rail; the stick's file picked, a name typed, the answer line carries /Drive.
  await cmd(page, 'Import UserProfile');
  let d = dialog(page);
  await expect(d).toContainText('Import UserProfile');
  const lf = d.locator('luma-files');
  await expect(lf.locator('[data-drive="usb1"]')).toBeVisible();
  await lf.locator('[data-drive="usb1"]').click();
  const row = lf.locator('luma-grid .lc-grid-row[data-row]', { hasText: profiles[0].replace(/\.lumaprofile$/, '') });
  await expect(row).toBeVisible();
  await row.locator('[data-col="name"]').click();
  await d.locator('input[data-field="1"]').fill(`Stick ${info.project.name}`);
  if (info.project.name === 'desktop' || info.project.name === 'phone') await d.screenshot({ path: path.join(REPO, `target/screenshots/files-ask-${info.project.name}.png`) });
  from = server.mark();
  await d.locator('.lc-dlg-btn', { hasText: 'Import' }).click();
  await server.waitSysmon(new RegExp(`\\] Import UserProfile "${profiles[0]}" /Drive "usb1" /As "Stick ${info.project.name}" → ok`), from);
  // A GDTF on the stick: the files feed shows its facts; Import FixtureType takes /Drive.
  mkdirSync(path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'gdtf'), { recursive: true });
  copyFileSync(path.join(REPO, 'tests/fixtures/AuraXB.gdtf'), path.join(probe, 'KINGSTON', 'LumaCore', 'library', 'gdtf', 'AuraXB.gdtf'));
  const sub = (params: Record<string, string>) => page.evaluate(p => new Promise<any>(res => (0, eval)('feeds')().sub('files', p, { batch: (m: any) => res(m.data) })), params);
  await expect.poll(async () => (await sub({ drive: 'usb1', folder: 'gdtf', kind: 'gdtf' })).files.find((f: any) => f.file === 'AuraXB.gdtf')?.manufacturer, { timeout: 10_000 }).toBeTruthy();
  if (info.project.name === 'phone') return; // the insert dialog is one step a screen there (phone.spec.ts)
  // Insert fixtures… › Library: the one files element, the stick's file picked, the line carries /Drive.
  await cmd(page, 'Menu "Patch"');
  const s = setup(page);
  await cmd(page, 'Claim Root.ShowData.Devices');
  const bad = await server.command('Import FixtureType "AuraXB.gdtf" /Drive "usb9"');
  expect(bad.message).toMatch(/No drive "usb9"/);
  from = server.mark();
  await cmd(page, 'Import FixtureType "AuraXB.gdtf" /Drive "usb1"');
  await server.waitSysmon(/\] Import FixtureType "AuraXB.gdtf" \/Drive "usb1" → ok/, from);
  await s.locator('luma-slab [data-act="new"]:visible').click();
  const ins = page.locator('[data-kit="insert"]');
  await expect(ins).toBeVisible();
  await ins.locator('[data-where="library"]').click();
  const ilf = ins.locator('luma-files.lc-ins-files');
  await expect(ilf.locator('[data-drive="usb1"]')).toBeVisible();
  await ilf.locator('[data-drive="usb1"]').click();
  const gdtf = ilf.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'AuraXB' });
  await expect(gdtf).toBeVisible();
  await expect(gdtf).toContainText('Martin');
  await gdtf.locator('[data-col="name"]').click();
  await expect(ins.locator('[data-mode="1"]')).toHaveAttribute('aria-selected', 'true');
  if (info.project.name === 'desktop') await ins.screenshot({ path: path.join(REPO, 'target/screenshots/insert-library-desktop.png') });
  await ins.locator('[data-next]').click();
  await expect(ins.locator('.lc-ins-line code').first()).toContainText('/FixtureType "AuraXB.gdtf" /Drive "usb1" /Mode 1');
  await page.keyboard.press('Escape');
  // Import MVR (part 4c): the slab's Import cell opens the two-step dialog — the files element,
  // the stick's rig.mvr picked, the Options step with the profile's MVR import preset — and
  // Review runs the line: the Patch grid turns into the review (the feed's review mode).
  await cmd(page, 'Set UserProfile "MvrNames" Take');
  // The file's universes land past the rig's (its 1.001… would clash with the rig's).
  await cmd(page, 'Set UserProfile "MvrUniverseOffset" 110');
  await s.locator('luma-slab [data-act="import"]:visible').click();
  const dlg = page.locator('[data-kit="mvrimport"]');
  await expect(dlg).toBeVisible();
  const mlf = dlg.locator('luma-files');
  await mlf.locator('[data-drive="usb1"]').click();
  const rigRow = mlf.locator('luma-grid .lc-grid-row[data-row]', { hasText: 'rig.mvr' });
  await expect(rigRow).toBeVisible();
  await expect(rigRow).toContainText('6'); // the MVR's fixtures on the feed
  await rigRow.locator('[data-col="name"]').click();
  await expect(dlg.locator('.lc-ins-line code')).toContainText('Import Mvr "rig.mvr" /Drive "usb1"');
  await dlg.locator('[data-next]').click();
  await expect(dlg.locator('luma-settings .lc-set-cell', { hasText: 'MvrMatch' })).toBeVisible({ timeout: 10_000 });
  if (info.project.name === 'desktop') await dlg.screenshot({ path: path.join(REPO, 'target/screenshots/mvr-import-options-desktop.png') });
  from = server.mark();
  await dlg.locator('[data-review]').click();
  await server.waitSysmon(/\] Import Mvr "rig.mvr" \/Drive "usb1" → ok/, from);
  await expect(dlg).toHaveCount(0);
  const patchFeed = () => page.evaluate(() => new Promise<any>(res => (0, eval)('feeds')().sub('patch', {}, { batch: (m: any) => res(m.data) })));
  await expect.poll(async () => (await patchFeed()).mode, { timeout: 10_000 }).toBe('review');
  const review = (await patchFeed()).review;
  expect(review.file).toBe('rig.mvr');
  // The file's six rows, every one a rig fixture by FID (the Spots meet 201–202); the rig's
  // fixtures the file lacks are listed Lost.
  expect(review.rows.filter((r: any) => r.status !== 'Lost').length).toBe(6);
  expect(review.rows.filter((r: any) => r.status === 'Changed').length).toBe(6);
  expect(review.rows.find((r: any) => r.no === 101)).toMatchObject({ status: 'Changed', match_key: 'FID', taken: true });
  // The grid is the review: a Status column, the ticked rows selected, the review's slab.
  const grid = s.locator('luma-grid');
  // (The grid draws the rows in view; the file's rows sit among the rig's 1 348 Lost ones.)
  await expect(grid.locator('[data-col="status"]').first()).toBeAttached();
  expect(review.rows.find((r: any) => r.name === 'Spot 1')).toMatchObject({ status: 'Changed', no: 201 });
  expect(review.rows.find((r: any) => r.no === 101)?.cells?.find((c: any) => c.field === 'name')).toMatchObject({ taken: true });
  await expect(s.locator('luma-slab [data-own][data-id="insert"]:visible')).toHaveCount(0); // the draft's cells hide in the review
  if (info.project.name === 'desktop') await s.screenshot({ path: path.join(REPO, 'target/screenshots/mvr-review-desktop.png') });
  await expect(s.locator('luma-slab [data-own][data-id="apply_review"]:visible')).toBeVisible();
  // (List Mvr's reply on the rig — 1 348 Lost rows — is past the command port's size; the review's
  // rows above are the check.)
  // Apply ticked from the slab (part 4b): every row met a rig fixture, so no group is gone and
  // nothing is asked (the ask: kit-34g4 / the Rust tests); the ticked rows go into the draft
  // and the review closes.
  from = server.mark();
  await s.locator('luma-slab [data-own][data-id="apply_review"]:visible').click();
  await server.waitSysmon(/\] Import Mvr [0-9 +Thru]*\/Apply → ok/, from);
  await expect.poll(async () => (await patchFeed()).mode, { timeout: 10_000 }).toBe('draft');
  await expect.poll(async () => (await patchFeed()).fixtures.find((f: any) => f.no === 101)?.name, { timeout: 10_000 }).toBe('Aura L1');
  await cmd(page, 'Set UserProfile "MvrNames" Offer');
  await cmd(page, 'Unclaim Root.ShowData.Devices /Discard');
});
