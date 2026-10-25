import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright config for the admin console flows (t32-admin §3.4 e15/e16, t32-fast
 * R-F4): `e2e/admin/*.spec.ts`, one console flow per spec, end to end against a
 * running stack.
 *
 * Separate from apps/web/playwright.config.ts on purpose, so admin lanes never
 * edit the shared main config. Projects run the same specs in Desktop Chrome and
 * Desktop Firefox, and at a 390x844 phone viewport for tests whose title carries
 * `@phone`. Always `--workers=1`: the specs share one admin account and one
 * database.
 *
 * The stack is an engine-mode server (Greenmail) with `MW_UPSTREAM_DEFAULT` set and
 * the dev admin credentials (`MW_ADMIN_USER` / `MW_ADMIN_PASSWORD`, default
 * root / hunter2: DEV/CI ONLY). The console path follows `MW_E2E_ADMIN_PATH`
 * (default /admin), as in the other admin specs.
 *
 * Environment:
 *   PLAYWRIGHT_ADMIN_BASE_URL   the stack (default PLAYWRIGHT_ENGINE_BASE_URL, then http://localhost:8090)
 *   T32_PW_OUT                  test-results folder (default <repo>/.orchestration/reports/t32/pw-admin)
 */
const here = path.dirname(fileURLToPath(import.meta.url));

const baseURL =
  process.env['PLAYWRIGHT_ADMIN_BASE_URL'] ??
  process.env['PLAYWRIGHT_ENGINE_BASE_URL'] ??
  'http://localhost:8090';

export default defineConfig({
  testDir: here,
  testMatch: ['*.spec.ts'],
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 1 : 0,
  reporter: [['list']],
  timeout: 120_000,
  expect: { timeout: 10_000 },
  outputDir: process.env['T32_PW_OUT'] ?? path.resolve(here, '../../../../.orchestration/reports/t32/pw-admin'),
  use: {
    baseURL,
    serviceWorkers: 'block',
    actionTimeout: 15_000,
    navigationTimeout: 20_000,
    locale: 'en-US',
    timezoneId: 'UTC',
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    {
      name: 'phone',
      grep: /@phone/,
      use: { ...devices['Desktop Chrome'], viewport: { width: 390, height: 844 }, hasTouch: true },
    },
  ],
});
