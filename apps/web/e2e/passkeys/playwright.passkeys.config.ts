import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: '.', testMatch: '*.spec.ts', workers: 1, retries: 0,
  timeout: 120_000, reporter: 'list',
  outputDir: '../../test-results/passkeys',
  use: { ...devices['Desktop Chrome'], baseURL: process.env['PLAYWRIGHT_PASSKEY_BASE_URL'] ?? 'http://localhost:42202', serviceWorkers: 'block', trace: 'retain-on-failure' },
});
