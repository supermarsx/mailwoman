import { defineConfig, devices } from '@playwright/test';

// t77: last-factor.spec.ts in Chromium and Firefox (the passkey leg is
// Chromium-only). A localhost base URL: an IP is not a valid WebAuthn RP ID.
const use = { baseURL: process.env['PLAYWRIGHT_PASSKEY_BASE_URL'] ?? 'http://localhost:42277', serviceWorkers: 'block' as const, trace: 'retain-on-failure' as const };

export default defineConfig({
  testDir: '.', testMatch: 'last-factor.spec.ts', workers: 1, retries: 0,
  timeout: 240_000, reporter: 'list',
  outputDir: '../../test-results/passkeys-last-factor',
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], ...use } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'], ...use } },
  ],
});
