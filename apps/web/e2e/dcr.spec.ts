import { test, expect } from '@playwright/test';
import { expectMounted } from './v7-helpers.ts';

/**
 * t10 §11 — OAuth 2.0 Dynamic Client Registration (RFC 7591/7592) live E2E against the
 * REAL 26.10 server. DCR is admin/policy-gated and DEFAULT DISABLED (plan §1.5, e8).
 *
 * The browser-facing contract this spec proves:
 *   • the RFC 7591 registration endpoint is MOUNTED and CLOSED by default — a register
 *     attempt with no enabled `oauth_dcr` policy is a 403 `access_denied` (the
 *     security-critical default-off invariant);
 *   • the RFC 7592 client-config endpoints (`GET/PUT/DELETE /oauth/register/{id}`) are
 *     mounted (answer with their own handler, not the SPA fall-through).
 *
 * KNOWN GAP (flagged to the coordinator — NOT a bug): there is no HTTP route to ENABLE
 * the `oauth_dcr` policy (e8 + e13 both noted the admin toggle was never wired to a
 * route — only the `mw_store::put_oauth_dcr_policy` repo method exists). So the
 * "admin enable → mint a client" flow cannot be driven from the browser/HTTP surface;
 * the enabled-path mint (RFC 7591 issuance, redirect-host allowlist, no scope
 * escalation) is proven in the Rust harness (mw-oauth dcr tests + t10-e14). This spec
 * asserts the mounted + default-closed gate, which IS the browser-facing contract.
 */

test.describe('OAuth DCR gate on the real server (default-disabled)', () => {
  test('RFC 7591 register is mounted and 403 access_denied while the policy is disabled', async ({ request }) => {
    const resp = await request.post('/oauth/register', {
      // A well-formed RFC 7591 registration request.
      data: {
        redirect_uris: ['https://app.example.com/callback'],
        grant_types: ['authorization_code', 'refresh_token'],
        response_types: ['code'],
        token_endpoint_auth_method: 'none',
        client_name: 'e15 probe client',
      },
      failOnStatusCode: false,
    });
    expectMounted(resp.status(), 'POST /oauth/register');
    expect(resp.status(), 'default-disabled ⇒ 403').toBe(403);
    const body = await resp.json();
    expect(body.error).toBe('access_denied');
    expect(String(body.error_description)).toMatch(/disabled/i);
  });

  test('RFC 7592 client-config endpoints are mounted (handler answers, not the SPA)', async ({ request }) => {
    // With DCR disabled these still resolve through the DCR handler chain rather than
    // falling through to index.html; any concrete API status proves the mount.
    const read = await request.get('/oauth/register/does-not-exist', { failOnStatusCode: false });
    expectMounted(read.status(), 'GET /oauth/register/{id}');

    const put = await request.put('/oauth/register/does-not-exist', {
      data: { redirect_uris: ['https://app.example.com/callback'] },
      failOnStatusCode: false,
    });
    expectMounted(put.status(), 'PUT /oauth/register/{id}');

    const del = await request.delete('/oauth/register/does-not-exist', { failOnStatusCode: false });
    expectMounted(del.status(), 'DELETE /oauth/register/{id}');

    // None of these should be a 200 success while disabled / for an unknown client.
    for (const s of [read.status(), put.status(), del.status()]) {
      expect(s).not.toBe(200);
    }
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.post('/oauth/register', { data: {}, failOnStatusCode: false }).catch(() => null);
  test.skip(probe === null, `[e15 SKIP] ${testInfo.title}: no 26.10 mw-server reachable.`);
});
