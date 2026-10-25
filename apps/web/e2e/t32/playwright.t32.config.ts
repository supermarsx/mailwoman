import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright config for the t32 feature specs (t32-admin §11 M7, t32-fast R-F4).
 *
 * Separate from apps/web/playwright.config.ts on purpose: every t32 feature
 * spec is named `e2e/t32/t32-<feature>.spec.ts` and runs here, so a feature
 * lane never edits the shared main config. The final stage check (S1) asserts a
 * non-zero executed count per spec file in every project that matches it.
 *
 * Projects, all `--workers=1` (the specs share one Greenmail account per stack):
 *   - `engine`         Desktop Chrome against the engine-mode stack (Greenmail).
 *   - `engine-firefox` the same specs in Desktop Firefox.
 *   - `phone`          Pixel 7 against the engine-mode stack; only tests whose
 *                      title carries `@phone`.
 *   - `mock`           Desktop Chrome against the proxy-mode stack (JMAP mock);
 *                      only tests whose title carries `@mock`.
 * A test tagged `@mock` runs only in `mock`; every other test runs in `engine`
 * and `engine-firefox`, and also in `phone` when tagged `@phone`.
 *
 * Stacks need `MW_UPSTREAM_DEFAULT` (sign-in names no server since t32-h9) and
 * the dev admin credentials (`MW_ADMIN_USER` / `MW_ADMIN_PASSWORD`, default
 * root / hunter2: DEV/CI ONLY).
 *
 * Environment:
 *   PLAYWRIGHT_ENGINE_BASE_URL  engine-mode stack (default http://localhost:8090)
 *   PLAYWRIGHT_MOCK_BASE_URL    proxy-mode stack (default http://localhost:8080)
 *   T32_PW_OUT                  test-results folder (default <repo>/.orchestration/reports/t32/pw)
 */
const here = path.dirname(fileURLToPath(import.meta.url));

const engineBaseURL = process.env['PLAYWRIGHT_ENGINE_BASE_URL'] ?? 'http://localhost:8090';
const mockBaseURL = process.env['PLAYWRIGHT_MOCK_BASE_URL'] ?? 'http://localhost:8080';

export default defineConfig({
  testDir: here,
  testMatch: ['t32-*.spec.ts'],
  fullyParallel: false,
  workers: 1,
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 1 : 0,
  reporter: [['list']],
  timeout: 90_000,
  expect: { timeout: 10_000 },
  outputDir: process.env['T32_PW_OUT'] ?? path.resolve(here, '../../../../.orchestration/reports/t32/pw'),
  use: {
    // A cached shell from a service worker would hide what the server sends.
    serviceWorkers: 'block',
    actionTimeout: 15_000,
    navigationTimeout: 20_000,
    locale: 'en-US',
    timezoneId: 'UTC',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'engine',
      grepInvert: /@mock/,
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
    {
      name: 'engine-firefox',
      grepInvert: /@mock/,
      use: { ...devices['Desktop Firefox'], baseURL: engineBaseURL },
    },
    {
      name: 'phone',
      grep: /@phone/,
      grepInvert: /@mock/,
      use: { ...devices['Pixel 7'], baseURL: engineBaseURL },
    },
    {
      name: 'mock',
      grep: /@mock/,
      use: { ...devices['Desktop Chrome'], baseURL: mockBaseURL },
    },
  ],
});
