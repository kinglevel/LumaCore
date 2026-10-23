import { cpus } from 'node:os';
import { defineConfig, devices } from '@playwright/test';

// Each spec file starts its own isolated server (temp library, a free port
// block — support/server.ts), so files run in parallel, one worker each, and
// the tests inside a file share that server in order. `E2E_WORKERS=n` sets the
// count (default: half the cores, at most 8 — measured here: 6 → 5.3 min, 8 → 4.5, 12 → 4.9,
// CPU-bound past 8); `E2E_TRACE=1` keeps a Playwright trace of every failed test (off by
// default: recording costs every test); `E2E_RETRIES=1` reruns a failed file once (a serial
// file retries whole; a test that then passes is reported flaky, not failed).
//
// Projects (decisions §30 "12a hardening"): every spec on the desktop; the
// UI kit's specs (`kit*.spec.ts`) also on a touch tablet, a touch phone and
// WebKit (closest to WKWebView / Safari on iPad). Run one with
// `npx playwright test --project=phone`.
const KIT = /kit.*\.spec\.ts$/;

export default defineConfig({
  testDir: './tests',
  globalSetup: './support/global-setup.ts',
  fullyParallel: false,
  workers: process.env.E2E_WORKERS ? Number(process.env.E2E_WORKERS) : Math.max(1, Math.min(8, Math.floor(cpus().length / 2))),
  retries: process.env.E2E_RETRIES ? Number(process.env.E2E_RETRIES) : 0,
  timeout: 60_000,
  expect: { timeout: 8_000 },
  reporter: [['list']],
  use: {
    headless: true,
    trace: process.env.E2E_TRACE ? 'retain-on-failure' : 'off',
  },
  // Project order is start order. WebKit's jobs are the longest (a test takes ~2.5× Chromium's
  // time) but putting them first made the start CPU-bound (eight WebKit jobs at once, 5.4 min
  // against 4.5); spread among the Chromium projects they overlap better.
  projects: [
    { name: 'desktop', use: { browserName: 'chromium', viewport: { width: 1400, height: 900 } } },
    { name: 'tablet', testMatch: KIT, use: { browserName: 'chromium', viewport: { width: 1024, height: 768 }, hasTouch: true } },
    { name: 'phone', testMatch: KIT, use: { browserName: 'chromium', viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, deviceScaleFactor: 3 } },
    { name: 'webkit', testMatch: KIT, use: { ...devices['Desktop Safari'], viewport: { width: 1400, height: 900 } } },
  ],
});
