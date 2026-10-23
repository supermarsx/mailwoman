import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { defineConfig, devices } from '@playwright/test';
import type { RestyleOptions } from './probes.ts';

/**
 * Playwright config for the t32 restyle harness (README.md in this folder).
 * Separate from apps/web/playwright.config.ts on purpose: it runs only the
 * `*.restyle.ts` files here, against a stack started by `stack.sh`.
 *
 * Projects are viewport × theme: desktop (Desktop Chrome at 1280×800) and phone
 * (Pixel 7), each in the `dark` and `light` packs. Always run with --workers=1:
 * every test signs in to the same Greenmail account.
 *
 * Environment:
 *   T32_STACK_ENV   the stack.env file `stack.sh up` wrote (base URL, SMTP port,
 *                   admin credentials); values already in the environment win
 *   T32_LABEL       output label, for example `before` or `after` (default `scratch`)
 *   T32_OUT         output root (default <repo>/.orchestration/reports/t32)
 */
const here = path.dirname(fileURLToPath(import.meta.url));

const stackEnv = process.env['T32_STACK_ENV'];
if (stackEnv !== undefined && existsSync(stackEnv)) {
  for (const line of readFileSync(stackEnv, 'utf8').split(/\r?\n/)) {
    const m = /^([A-Z0-9_]+)=(.*)$/.exec(line);
    if (m && process.env[m[1]!] === undefined) process.env[m[1]!] = m[2]!;
  }
}

process.env['T32_OUT'] ??= path.resolve(here, '../../../../.orchestration/reports/t32');
process.env['T32_LABEL'] ??= 'scratch';

const baseURL = process.env['PLAYWRIGHT_ENGINE_BASE_URL'] ?? 'http://localhost:37290';

const desktop = { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 800 } };
const phone = { ...devices['Pixel 7'] };

export default defineConfig<RestyleOptions>({
  testDir: here,
  testMatch: ['*.restyle.ts'],
  globalTeardown: path.join(here, 'teardown.ts'),
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [['list']],
  timeout: 180_000,
  expect: { timeout: 15_000 },
  outputDir: path.join(process.env['T32_OUT'], 'test-results', process.env['T32_LABEL']),
  use: {
    baseURL,
    contextOptions: { reducedMotion: 'reduce' },
    // No service worker: it keeps a cached shell between page loads, and at T32-BASE a page it
    // controls gets the server's plain-text 404 for a navigation to /admin.
    serviceWorkers: 'block',
    actionTimeout: 15_000,
    navigationTimeout: 20_000,
    locale: 'en-US',
    timezoneId: 'UTC',
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'desktop-dark', use: { ...desktop, viewportName: 'desktop', theme: 'dark' } },
    { name: 'desktop-light', use: { ...desktop, viewportName: 'desktop', theme: 'light' } },
    { name: 'phone-dark', use: { ...phone, viewportName: 'phone', theme: 'dark' } },
    { name: 'phone-light', use: { ...phone, viewportName: 'phone', theme: 'light' } },
  ],
});
