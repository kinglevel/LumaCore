// A slab cell, wherever it is: on a phone the setup frame's slab is one bar (decisions §37 1–2)
// — the cells that work now, Oops, ⋯ More, the lock — and every other cell sits in the More
// sheet. `slabCell(scope, '[data-act="edit"]')` opens More when the cell isn't in the bar and
// returns the visible one.

import type { Locator } from '@playwright/test';

export async function slabCell(scope: Locator, sel: string): Promise<Locator> {
  const inBar = scope.locator(`luma-slab ${sel}`).filter({ visible: true });
  const more = scope.locator('luma-slab [data-more]').filter({ visible: true });
  // The slab draws with the rows: give the cell or the More cell a moment to land.
  await scope.locator('luma-slab').first().waitFor({ state: 'visible' });
  for (let i = 0; i < 60 && !(await inBar.count()) && !(await more.count()); i++) await scope.page().waitForTimeout(50);
  if (await inBar.count()) return inBar.first();
  if (await more.count()) {
    const page = scope.page();
    const sheet = page.locator('.lc-slab-sheet.more');
    if (!(await sheet.count())) await more.first().click();
    return sheet.locator(sel).filter({ visible: true }).first();
  }
  return inBar.first();
}

/** Close an open phone sheet (More, the lock) so the list under it takes taps again. */
export async function closeSheet(page: import('@playwright/test').Page) {
  const x = page.locator('.lc-psheet [data-close]');
  if (await x.count()) await x.first().click();
}
