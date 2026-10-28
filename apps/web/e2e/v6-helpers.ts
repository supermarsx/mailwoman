import { expect, type APIRequestContext } from '@playwright/test';
import { postSignIn } from './helpers.ts';

/**
 * Shared helpers for the V6 live E2E specs (plan §3 e13). These specs drive the
 * REAL mounted mw-server surface from the browser test runner (Playwright's
 * `request` fixture = a real HTTP client with a cookie jar), against a live stack
 * brought up by the CI `e2e-v6` job (mw-server + a JMAP mock, backed by real
 * postgres:16 + valkey:8).
 *
 * Server bring-up the `v6` project assumes (coordinator/e12 wire the CI job +
 * playwright.config `v6` project):
 *   - mw-server in PROXY mode, admin enabled with MW_ADMIN_USER=root /
 *     MW_ADMIN_PASSWORD=hunter2, db on the postgres:16 DSN, redis on valkey:8;
 *   - a JMAP mock reachable from the server at MW_E2E_JMAP_URL;
 *   - the SPA served at the project baseURL.
 *
 * The headline zero-access *ciphertext-at-rest* proof (a DIRECT Postgres query on
 * the stored row) + the SQLite↔Postgres backend-parity run live in the Rust
 * harness crates/mw-server/tests/v6_e2e.rs — the browser cannot query the DB, so
 * that proof lives server-side by design.
 */

// The JMAP server is the stack's MW_UPSTREAM_DEFAULT; a sign-in never names one (t32-h9).
export const V6 = {
  mailUser: process.env['MW_E2E_USERNAME'] ?? 'testuser@example.org',
  mailPass: process.env['MW_E2E_PASSWORD'] ?? 'testpass',
  adminUser: process.env['MW_ADMIN_USER'] ?? 'root',
  adminPass: process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2',
} as const;

/** Where the stack under test serves the admin console and its API: its
 *  `MW_ADMIN_PATH`, given to the specs as `MW_E2E_ADMIN_PATH` (t32-h15). */
export const ADMIN_PATH = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';

/** An admin route under {@link ADMIN_PATH}: `adminPath('/users')`, or the console
 *  page itself with no argument. */
export function adminPath(suffix = ''): string {
  return `${ADMIN_PATH}${suffix}`;
}

/** Fixed PKCE pair: verifier + BASE64URL-NOPAD(SHA256(verifier)) challenge. */
export const PKCE = {
  verifier: 'e13pkceverifier0123456789abcdefghijklmnopqrstuvwxyzABCDEF',
  challenge: 'daNkxhCFTwqdR-SivcCAKzFQqFpXxfZcC0bYsvxGEbw',
} as const;

/** Cookie-authenticate the mailbox session against the real server; returns the accountId. */
export async function mailboxLogin(request: APIRequestContext): Promise<string> {
  // postSignIn waits out the per-IP sign-in cooldown that a spec's deliberate
  // failed sign-in starts (signin.retryCooldownSecs, failed attempts only since 63d344c0).
  const resp = await postSignIn(request, '/api/login', { username: V6.mailUser, password: V6.mailPass });
  expect(resp.status(), 'mailbox login succeeds').toBe(200);
  const body = await resp.json();
  return body.accountId as string;
}

/** Cookie-authenticate the SEPARATE admin session domain. */
export async function adminLogin(request: APIRequestContext): Promise<void> {
  const resp = await postSignIn(request, adminPath('/login'), { username: V6.adminUser, password: V6.adminPass });
  expect(resp.status(), 'admin login succeeds').toBe(200);
}

/** A scoped-key wire Scope (matches mw_oauth::Scope serde). */
export function scope(opts: {
  account: string;
  read?: boolean;
  mail?: boolean;
  ipAllowlist?: string[];
  expiresAt?: string | null;
  rateLimit?: number | null;
}): Record<string, unknown> {
  const mail = opts.mail ?? true;
  return {
    read: opts.read ?? true,
    send: false,
    delete: false,
    accounts: { subset: [opts.account] },
    folders: 'all',
    mail,
    pim: !mail,
    ip_allowlist: opts.ipAllowlist ?? [],
    expires_at: opts.expiresAt ?? null,
    rate_limit: opts.rateLimit ?? null,
    mcp_tools: [],
    unattended_send: false,
  };
}

/** Mint a scoped API key; returns the shown-once `mwk_…` token. */
export async function mintKey(
  request: APIRequestContext,
  account: string,
  scopeObj: Record<string, unknown>,
): Promise<string> {
  // Minting a key needs a fresh password confirmation (step-up, ApiKeyCreate,
  // 7e9af0d0); without it the server answers 403.
  const stepUp = await request.post('/api/security/step-up', { data: { password: V6.mailPass } });
  expect(stepUp.status(), 'step-up before minting a key').toBe(200);
  const resp = await request.post('/api/keys', {
    data: { label: 'e13-web', accountId: account, scope: scopeObj },
  });
  expect(resp.status(), 'key mint accepted').toBe(200);
  const body = await resp.json();
  const token = body.displayToken as string;
  expect(token, 'key shown once as mwk_…').toMatch(/^mwk_/);
  return token;
}

/** A unique suffix so specs re-run against the persistent Postgres without colliding. */
export function uniq(): string {
  return `${Date.now().toString(36)}${Math.floor(Math.random() * 1e6).toString(36)}`;
}
