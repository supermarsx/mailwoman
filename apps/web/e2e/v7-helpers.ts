import { expect, type APIRequestContext } from '@playwright/test';

/**
 * Shared helpers for the V7 live E2E specs (plan §3 e16). Like the V6 specs, these
 * drive the REAL mounted mw-server surface from the browser test runner (Playwright's
 * `request` fixture = a real HTTP client with a cookie jar), against a standing V7
 * stack. The DEEP proofs live in the Rust harness `crates/mw-server/tests/v7_e2e.rs`
 * (the plugin wasmtime jail, the plugin-backed-account JMAP surface, Assist E2EE
 * redaction, directory-vs-real-OpenLDAP, RFC-3062 password change) — the browser
 * cannot load a wasm component into the host jail or query LDAP, so those proofs are
 * server-side by design. These web specs prove the browser-facing V7 HTTP contract:
 * every new surface is MOUNTED (a real handler answers, not the SPA fall-through) and
 * the web-visible governance (Assist disclosure + disabled-hides-UI, the plugin
 * registry + unsigned banner, the password policy) is served.
 *
 * Server bring-up the `v7` project assumes (coordinator/e15 wire the CI `e2e-v7` job +
 * a `v7` project in playwright.config.ts — mirrors the `v6` precedent):
 *   - mw-server in PROXY mode, admin enabled (MW_ADMIN_USER=root /
 *     MW_ADMIN_PASSWORD=hunter2), fronting a JMAP mock at MW_E2E_JMAP_URL;
 *   - OpenLDAP + the mock Assist endpoint from docker-compose.ci.yml reachable from the
 *     server (directory/assist rows seeded) — when NOT seeded, the directory/assist
 *     routes return their honest 501/Disabled, which these specs assert as the mounted
 *     contract;
 *   - the SPA served at the project baseURL.
 */

export const V7 = {
  jmapUrl: process.env['MW_E2E_JMAP_URL'] ?? 'http://mock:8181/.well-known/jmap',
  mailUser: process.env['MW_E2E_USERNAME'] ?? 'testuser@example.org',
  mailPass: process.env['MW_E2E_PASSWORD'] ?? 'testpass',
  adminUser: process.env['MW_ADMIN_USER'] ?? 'root',
  adminPass: process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2',
} as const;

/** Cookie-authenticate the mailbox session; returns the accountId. */
export async function mailboxLogin(request: APIRequestContext): Promise<string> {
  const resp = await request.post('/api/login', {
    data: { jmapUrl: V7.jmapUrl, username: V7.mailUser, password: V7.mailPass },
  });
  expect(resp.status(), 'mailbox login succeeds').toBe(200);
  return (await resp.json()).accountId as string;
}

/** Cookie-authenticate the SEPARATE admin session. */
export async function adminLogin(request: APIRequestContext): Promise<void> {
  const resp = await request.post('/admin/login', {
    data: { username: V7.adminUser, password: V7.adminPass },
  });
  expect(resp.status(), 'admin login succeeds').toBe(200);
}

/**
 * A mounted route answered with its OWN handler (not the SPA index.html
 * fall-through). Any concrete API status (200/204/400/401/403/501, …) proves the
 * route is wired; a 404 that returns the SPA HTML would mean it fell through.
 */
export function expectMounted(status: number, label: string): void {
  expect(status, `${label}: route is mounted (real handler answered)`).not.toBe(404);
}
