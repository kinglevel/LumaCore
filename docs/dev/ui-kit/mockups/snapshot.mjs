// Renders every mockup board offline (support.js runtime) to png/<board>.png.
// Usage: node docs/dev/ui-kit/mockups/snapshot.mjs [2026-10-12]
// Uses the Playwright install of tests/e2e (cd tests/e2e && npm ci first).
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const dir = resolve(here, process.argv[2] || '2026-10-12');
const require = createRequire(join(here, '../../../../tests/e2e/package.json'));
const { chromium } = require('playwright');

const canvas = JSON.parse(readFileSync(join(dir, 'canvas.json'), 'utf8'));
const browser = await chromium.launch();
for (const name of canvas.order) {
  const b = canvas.boards[name];
  const page = await browser.newPage({ viewport: { width: b.w, height: b.h } });
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  await page.goto(pathToFileURL(join(dir, name)).href);
  await page.waitForTimeout(300);
  await page.screenshot({ path: join(dir, 'png', name.replace('.dc.html', '.png')) });
  console.log(name, errors.length ? 'ERRORS: ' + errors.join(' | ') : 'ok');
  await page.close();
}
await browser.close();
