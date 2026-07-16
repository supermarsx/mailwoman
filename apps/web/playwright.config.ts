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
// V6 live full-stack E2E (plan §3 e13): a standing mw-server in PROXY mode backed by
// REAL postgres:16 + valkey:8 and fronting a JMAP mock, brought up by the CI `e2e-v6`
// job. The `v6` specs use Playwright's `request` fixture (a real HTTP client) against
// this server, so `MW_E2E_BASE_URL` points the project at it (the JMAP mock URL the
// server proxies to is passed to the login endpoint via MW_E2E_JMAP_URL, read in
// e2e/v6-helpers.ts). Defaults to :8090 for local runs.
const v6BaseURL = process.env['MW_E2E_BASE_URL'] ?? 'http://localhost:8090';
// V7 live full-stack E2E (plan §3 e16): the SAME standing-server pattern as `v6`, but
// exercising the per-V7-capability surfaces (plugin registry / directory-GAL /
// password-change / Assist governance / bridges registry). The `v7` specs use
// Playwright's `request` fixture (a real HTTP client with a cookie jar) against the live
// server the CI `e2e-v7` job builds + starts (mw-server in PROXY mode fronting a JMAP
// mock, with OpenLDAP + the mock Assist endpoint reachable). The DEEP proofs (wasm jail,
// plugin-backed JMAP surface, Assist E2EE redaction, directory-vs-real-OpenLDAP,
// RFC-3062) live in the Rust harness crates/mw-server/tests/v7_e2e.rs, which the same job
// runs alongside. Reuses MW_E2E_BASE_URL (the e2e-v7 job runs standalone). Defaults to
// :8090 for local runs.
const v7BaseURL = process.env['MW_E2E_BASE_URL'] ?? 'http://localhost:8090';

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
    {
      name: 'crypto',
      // V4 crypto/security live E2E (plan §3 e10): the SAME engine-mode server
      // (:8090) as `e2e-engine`/`e2e-pim`, but the `crypto-*.spec.ts` specs drive
      // the REAL crypto UI (key management, security panel, compose crypto/DLP,
      // max-security switch, decrypt-on-receipt) backed by the REAL WASM crypto
      // worker (mw-crypto + mw-sanitize, built by scripts/build-wasm and embedded
      // in the runtime image) and the REAL engine security surface. The DLP block
      // spec needs the engine started with a `MW_DLP_RULES` block rule (the CI
      // `e2e-crypto` job — owned by e9 — sets it on the engine bring-up; locally
      // via the docker-compose.crypto.yml override). Sibling of the `e2e` /
      // `e2e-engine` / `e2e-pim` projects; all must stay green.
      testMatch: ['crypto-*.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
    {
      name: 'push',
      // V5 browser Web Push live E2E (plan §3 e9): the SAME engine-mode server
      // (:8090) as the other engine projects, driving the real VAPID/subscribe
      // endpoints (e5) and asserting the push dispatcher delivers an OPAQUE wake
      // (no message content) to a test-controlled mock endpoint on a real
      // StateChange, then the client refetches. The server (containerized) reaches
      // the host-side mock via host.docker.internal (override MW_E2E_WAKE_HOST).
      // A dedicated project (not folded into `engine`) so its host-networking +
      // longer dispatch timing stay isolated; e8 wires an `e2e-push` job (or adds
      // `--project=push` to the engine job). See apps/web/e2e/README-push.md.
      testMatch: ['push.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
    {
      name: 'v6',
      // V6 live full-stack E2E (plan §3 e13): a standing mw-server in PROXY mode
      // backed by REAL postgres:16 + valkey:8 (not SQLite/in-memory), driving the
      // admin / OAuth+scoped-API-key / MCP / zero-access / cache-posture surfaces
      // per capability. Unlike the engine projects these specs use Playwright's
      // `request` fixture (a real HTTP client with a cookie jar) rather than page
      // navigation; `baseURL` (MW_E2E_BASE_URL) points them at the live server the
      // CI `e2e-v6` job builds + starts. The ciphertext-at-rest DIRECT Postgres
      // query + SQLite⇄Postgres backend parity live in the Rust harness
      // (crates/mw-server/tests/v6_e2e.rs), which the same job runs alongside.
      testMatch: [
        'admin.spec.ts',
        'oauth-apikey.spec.ts',
        'mcp.spec.ts',
        'zeroaccess.spec.ts',
        'cache-posture.spec.ts',
      ],
      use: { ...devices['Desktop Chrome'], baseURL: v6BaseURL },
    },
    {
      name: 'v7',
      // V7 live full-stack E2E (plan §3 e16): a standing mw-server (PROXY mode, admin on,
      // fronting a JMAP mock) with REAL OpenLDAP + the mock Assist endpoint reachable, per
      // the `v6` precedent. Like the `v6` project these specs use Playwright's `request`
      // fixture rather than page navigation; `baseURL` (MW_E2E_BASE_URL) points them at
      // the live server the CI `e2e-v7` job builds + starts. They assert the browser-
      // facing V7 HTTP contract (every surface is MOUNTED; Assist disclosure + kill
      // switch; plugin registry + unsigned banner; password policy; bridges surface as
      // account-backend-capable) and self-skip loudly when no V7 stack is reachable. The
      // deep proofs live in the Rust harness (crates/mw-server/tests/v7_e2e.rs), which the
      // same job runs alongside.
      testMatch: [
        'plugins.spec.ts',
        'directory.spec.ts',
        'passwd.spec.ts',
        'assist.spec.ts',
        'bridges.spec.ts',
      ],
      use: { ...devices['Desktop Chrome'], baseURL: v7BaseURL },
    },
    {
      name: 't12',
      // 26.12 conformance-closure live E2E (plan §2 Batch-4 e-e2e-web). The SAME
      // engine-mode server (:8090) as the `engine`/`pim`/`crypto` projects, backed by
      // the REAL WASM crypto worker + REAL mw-sanitize (native, in mw-render) + the
      // real engine. The `t12-*.spec.ts` specs drive the NEW 26.12 user surfaces:
      //   - Sieve rule builder + raw-editor round-trip (MailRule JMAP);
      //   - compose sign+encrypt AND sign-only — the wire message is byte-asserted
      //     genuinely encrypted AND (when signed) carries a valid signature (§8 gate);
      //   - the side-by-side calendar conflict resolver + distinct schedule view + role UI;
      //   - the CSS-rewrite sanitizer in a real server-rendered message.
      // Brought up by `docker compose -f docker-compose.dev.yml -f docker-compose.crypto.yml
      // up -d --build --wait greenmail mailwoman-engine`; added to the t12-conformance.yml
      // workflow by e-e2e-backend. Sibling of the other engine projects — all must stay green.
      testMatch: ['t12-*.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: engineBaseURL },
    },
    {
      name: 't10',
      // 26.10 tail live E2E (plan §3 e15). The HEADLINE UI-plugin sandbox-escape gate
      // (ui-plugin.spec.ts) is browser-only — it drives the SHIPPED opaque-origin
      // sandbox + deny-by-default broker directly, so it needs no backend and runs
      // everywhere. The masked-email + DCR admin specs use Playwright's `request`
      // fixture against the same standing 26.10 mw-server the `v6`/`v7` projects target
      // (MW_E2E_BASE_URL), and self-skip loudly when no stack is reachable. Adding a
      // `t10-*` / `ui-plugin`/`masked`/`dcr` spec slots into this project with no further
      // config edit.
      testMatch: ['ui-plugin.spec.ts', 'masked.spec.ts', 'dcr.spec.ts'],
      use: { ...devices['Desktop Chrome'], baseURL: v7BaseURL },
    },
  ],
});
