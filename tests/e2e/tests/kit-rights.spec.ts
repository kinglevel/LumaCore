// Step 12d part 2 (decisions §30 "12d hardening", "Rights: one level per
// user"): one rights level per user; Users & Profiles shows it
// and greys what needs more, with the reason; the lock surface shows each
// user's level; a refused line names the level it needs and yours. On the
// desktop, a touch tablet, a phone and WebKit.

import path from 'node:path';
import { slabCell, closeSheet } from '../support/slab';
import { expect, test, type Page } from '@playwright/test';
import { REPO, Server, connected, kbdMode } from '../support/server';

const server = new Server();
test.describe.configure({ mode: 'serial' });
test.beforeAll(async () => {
  await server.start(['--show', path.join(REPO, 'library/shows/demo.luma')]);
  for (const line of ['Store User "Lead" /Rights Program', 'Store User "Op" /Rights Playback']) {
    expect((await server.command(line)).ok).toBe(true);
  }
});
test.afterAll(async () => { await server.cleanup(); });

async function open(page: Page, user = 'Admin') {
  await server.open(page, '/', user);
  await connected(page);
  await kbdMode(page);
  await page.evaluate(() => document.getElementById('fb-cards')?.replaceChildren());
}

/** Menu › Users & Profiles: the setup frame since 12d part 4. */
async function people(page: Page) {
  await page.evaluate(() => (window as any).openMenu());
  await page.locator('#menu-tiles .lc-tile', { hasText: 'Users & Profiles' }).click();
  const el = page.locator('luma-setup');
  await expect(el.locator('luma-grid [data-col="name"]').first()).toBeAttached();
  return el;
}
async function guidOf(name: string) {
  return (await (await fetch(`${server.url}/api/station`)).json()).users.find((u: any) => u.name === name).guid;
}
/** On a phone the cell may sit in the bar's More sheet (decisions §37 1): it is opened. */
const slab = (page: Page, sel: string) => slabCell(page.locator('luma-setup'), sel);

test('Users & Profiles shows each level and greys what needs Admin, with the reason', async ({ page }) => {
  await open(page, 'Lead');
  const el = await people(page);
  const op = await guidOf('Op');
  await expect(el.locator(`luma-grid [data-k="${op}"][data-col="rights"]`)).toHaveText('Playback');
  await el.locator(`luma-grid [data-gutter="${op}"]`).click();
  await expect(await slab(page, '[data-act="new"]')).toHaveAttribute('data-why', 'Needs Admin — you are Program');
  // Another user's password is Admin's; your own is yours.
  const pw = await slab(page, '[data-own][data-id="password"]');
  if (await pw.count()) await expect(pw).toHaveAttribute('data-why', 'Needs Admin — you are Program');
  await closeSheet(page);
  await el.locator(`luma-grid [data-gutter="${await guidOf('Lead')}"]`).click();
  const pw2 = await slab(page, '[data-own][data-id="password"]');
  if (await pw2.count()) await expect(pw2).not.toHaveAttribute('data-why', /.+/);
  await expect(await slab(page, '[data-act="delete"]')).toHaveAttribute('data-why', 'Needs Admin — you are Program');
});

test('an Admin edits the level: the grid follows one Set line', async ({ page }) => {
  await open(page);
  const el = await people(page);
  const op = await guidOf('Op');
  await expect(await slab(page, '[data-act="new"]')).not.toHaveAttribute('data-why', /.+/);
  const r = await server.command('Set User "Op" "Rights" Presets');
  expect(r.ok, r.message).toBe(true);
  await expect(el.locator(`luma-grid [data-k="${op}"][data-col="rights"]`)).toHaveText('Presets');
  expect((await server.command('Set User "Op" "Rights" Playback')).ok).toBe(true);
});

test('the lock surface shows every user\'s level', async ({ page }) => {
  await page.goto(server.url + '/');
  const lead = page.locator('luma-surface luma-avatar[data-user="Lead"]');
  await expect(lead).toHaveCount(1);
  await expect(lead.locator('.lc-av-level')).toHaveText('Program');
  await expect(page.locator('luma-surface luma-avatar[data-user="Admin"] .lc-av-level')).toHaveText('Admin');
});

test('a refused line names the level it needs and yours', async () => {
  const r = await server.command('Store User "Z"', 'Lead');
  expect(r.message).toContain("needs Admin — you are Program");
  expect(r.message).toContain('E-RIGHTS-001');
  const go = await server.command('Store Group 9', 'Op');
  expect(go.message).toContain('needs Program — you are Playback');
});
