// Renders the README hero and the logo lockups from brand.html to PNG (2×).
// Run from tests/e2e (it has Playwright): cp ../../images/brand/render.mjs . && BRAND_DIR=../../images/brand node render.mjs
import { chromium } from 'playwright';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = process.env.BRAND_DIR ?? path.dirname(fileURLToPath(import.meta.url));
const page = await (await chromium.launch()).newPage({ deviceScaleFactor: 2 });
for (const mark of ['prompt', 'moon']) {
  for (const kind of ['hero', 'lockup']) {
    await page.goto(`file://${here}/brand.html?mark=${mark}&kind=${kind}`);
    await page.evaluate(() => document.fonts.ready);
    await page.locator('#art').screenshot({ path: `${here}/${kind}-${mark}.png`, omitBackground: true });
  }
}
// The app icon as PNG (packaging) and at 64 px (the desktop app's window icon, which
// apps/lumacore-app takes as raw RGBA: make-icon-rgba.py turns the 64 px PNG into it).
for (const size of [512, 64]) {
  const icon = await (await chromium.launch()).newPage({ viewport: { width: size, height: size } });
  await icon.goto(`file://${here}/prompt-icon.svg`);
  await icon.evaluate(s => { const svg = document.documentElement; svg.setAttribute('width', s); svg.setAttribute('height', s); }, size);
  await icon.screenshot({ path: `${here}/prompt-icon-${size}.png`, omitBackground: true });
}
process.exit(0);
