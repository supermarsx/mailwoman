import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E config for Mailwoman.
 *
 * The specs drive the REAL web UI against a running stack (mw-server + a mail
 * backend). In CI each stack is brought up separately via docker compose
 * (see .github/workflows/ci.yml); this config does NOT rebuild or manage the
 * server — it assumes the app is already reachable at the project's `baseURL`.
 *
 * Two projects target the two backends the V1 UI must work against, unchanged:
 *   - `mock`   — the V0 in-repo JMAP mock (proxy mode) on :8080. Runs the
 *                original happy-path + sanitizer specs.
 *   - `engine` — V1 engine mode: mw-server driving a REAL IMAP/SMTP account
 *                (Greenmail) through mw-engine, on :8090. Runs imap-engine.spec.
 *   - `pim`    — V3 engine mode: the SAME engine-mode server (:8090), driving the
 *                four PIM modules (calendar/tasks/notes/contacts) through the real
 *                UI over the engine's auto-seeded Mailwoman-native collections.
 *                Runs the pim-*.spec.ts specs. (The CalDAV/CardDAV round-trip
 *                itself is proven at the Rust level by e11's conformance job, so
 *                these specs need no CalDAV account in the browser.)
 *
 * Select one with `--project=mock` / `--project=engine` / `--project=pim`. Each
 * project's `baseURL` can be overridden for local runs (e.g. `cargo run` / `vite`):
 *   - mock:          PLAYWRIGHT_BASE_URL or PLAYWRIGHT_MOCK_BASE_URL (default :8080)
 *   - engine / pim:  PLAYWRIGHT_ENGINE_BASE_URL (default :8090)
 */
const mockBaseURL =
  process.env['PLAYWRIGHT_MOCK_BASE_URL'] ??
  process.env['PLAYWRIGHT_BASE_URL'] ??
  'http://localhost:8080';
const engineBaseURL = process.env['PLAYWRIGHT_ENGINE_BASE_URL'] ?? 'http://localhost:8090';

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
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
  },
  projects: [
    {
      name: 'mock',
      testMatch: ['happy-path.spec.ts', 'sanitizer.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: mockBaseURL },
    },
    {
      name: 'engine',
      // V1 IMAP round-trip + the V2 modern-UX/theming specs. All target the
      // engine-mode server (:8090); a new spec added here "slots in" to the CI
      // `e2e-engine` job with no workflow edits (per e11's handoff).
      testMatch: [
        'imap-engine.spec.ts',
        'modern-ux.spec.ts',
        'theming.spec.ts',
        'realtime-push.spec.ts',
        'offline.spec.ts',
        'multiwindow.spec.ts',
        'search.spec.ts',
        'viewers.spec.ts',
        'export.spec.ts',
      ],
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
    {
      name: 'pim',
      // V3 PIM live E2E: the four modules (calendar/tasks/notes/contacts) driven
      // through the real UI against the engine-mode server (:8090), over its
      // auto-seeded native collections. Adding a `pim-*.spec.ts` slots into the CI
      // `e2e-pim` job (boots greenmail + mailwoman-engine, runs `--project=pim`).
      testMatch: ['pim-*.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
  ],
});
