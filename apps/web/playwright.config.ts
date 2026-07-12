import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E config for Mailwoman.
 *
 * The specs drive the REAL web UI against a running stack (mw-server + a JMAP
 * backend). In CI the stack is brought up separately via docker compose
 * (see .github/workflows/ci.yml); this config does NOT rebuild or manage the
 * server — it assumes the app is already reachable at `baseURL`.
 *
 * `baseURL` defaults to the compose-published address and can be overridden
 * with PLAYWRIGHT_BASE_URL for local runs against `cargo run` / `vite`.
 */
const baseURL = process.env['PLAYWRIGHT_BASE_URL'] ?? 'http://localhost:8080';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 2 : 0,
  workers: process.env['CI'] ? 1 : undefined,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  timeout: 30_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
});
