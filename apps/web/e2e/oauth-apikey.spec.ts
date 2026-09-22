import { test, expect } from '@playwright/test';
import { mailboxLogin, mintKey, scope } from './v6-helpers.ts';

/**
 * V6 live E2E — OAUTH + SCOPED API KEYS (plan §3 e13): mint a scoped API key and
 * assert the /api/v1 enforcement matrix (e11b) against the live server:
 *   in-scope → 200 · out-of-scope → 403 · expired → 401 · IP-allowlist → 403 ·
 *   over-rate → 429.
 *
 * The full consent → authorization-code + PKCE → token exchange needs a SEEDED
 * `oauth_clients` row (there is no client-registration endpoint). Seeding is a
 * server-side/SQL step the CI e2e-v6 bring-up performs (or the Rust harness does
 * via psql); this browser spec proves the enforcement matrix, which is the DoD's
 * core acceptance for the scoped-key surface.
 */
test.describe('v6 scoped API keys — enforcement matrix (live)', () => {
  test('in-scope 200 / out-of-scope 403 / expired 401 / IP 403 / rate 429', async ({ request }) => {
    const account = await mailboxLogin(request);

    // GRANT: in-scope key → 200 with the JMAP list.
    const good = await mintKey(request, account, scope({ account }));
    const ok = await request.get('/api/v1/messages?limit=5', { headers: { 'x-api-key': good } });
    expect(ok.status(), 'in-scope key → 200').toBe(200);
    expect(await ok.json(), 'returns JMAP list').toHaveProperty('messages');

    // DENY: no read → 403.
    const noRead = await mintKey(request, account, scope({ account, read: false }));
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': noRead } })).status(),
      'no-read → 403',
    ).toBe(403);

    // DENY: wrong account → 403.
    const wrong = await mintKey(request, account, scope({ account: 'nobody@elsewhere.test' }));
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': wrong } })).status(),
      'wrong-account → 403',
    ).toBe(403);

    // DENY: expired → 401.
    const expired = await mintKey(
      request,
      account,
      scope({ account, expiresAt: '2000-01-01T00:00:00Z' }),
    );
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': expired } })).status(),
      'expired → 401',
    ).toBe(401);

    // DENY: source IP outside the allowlist → 403, and X-Forwarded-For cannot move
    // it. Since t20 B1 the source IP is the real peer address, refined by a
    // forwarded header ONLY when MW_FORWARDED_MODE names one and the peer is inside
    // MW_TRUSTED_PROXIES (crates/mw-server/src/scope_mw.rs). This standing server
    // sets neither, so the peer is loopback whatever the request claims.
    //
    // Until t24-e14 this spec asserted `x-forwarded-for: 10.1.2.3` → 200 against a
    // 10.0.0.0/8 allowlist. That expectation WAS the bypass t20 closed — any caller
    // could name an allowlisted address in a header it controls — so the spec is
    // what changed here, not the product. The second assertion below is the one
    // that now proves the hardening rather than passing for the wrong reason.
    const ipKey = await mintKey(request, account, scope({ account, ipAllowlist: ['10.0.0.0/8'] }));
    expect(
      (
        await request.get('/api/v1/messages', {
          headers: { 'x-api-key': ipKey, 'x-forwarded-for': '8.8.8.8' },
        })
      ).status(),
      'peer outside allowlist → 403',
    ).toBe(403);
    expect(
      (
        await request.get('/api/v1/messages', {
          headers: { 'x-api-key': ipKey, 'x-forwarded-for': '10.1.2.3' },
        })
      ).status(),
      'a spoofed X-Forwarded-For from an untrusted peer cannot enter the allowlist (t20 B1)',
    ).toBe(403);

    // GRANT: allowlist the REAL peer → 200. Without this the IP checks above would
    // all be satisfied by an allowlist that never matches anything.
    const peerKey = await mintKey(
      request,
      account,
      scope({ account, ipAllowlist: ['127.0.0.1/32', '::1/128'] }),
    );
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': peerKey } })).status(),
      'peer inside allowlist → 200',
    ).toBe(200);

    // DENY: over the per-key rate limit → 429.
    const rlKey = await mintKey(request, account, scope({ account, rateLimit: 1 }));
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': rlKey } })).status(),
      'first within rate limit → 200',
    ).toBe(200);
    expect(
      (await request.get('/api/v1/messages', { headers: { 'x-api-key': rlKey } })).status(),
      'second over rate limit → 429',
    ).toBe(429);

    // DENY: unknown key → 401.
    expect(
      (
        await request.get('/api/v1/messages', {
          headers: { 'x-api-key': 'mwk_deadbeef.notreal' },
        })
      ).status(),
      'unknown key → 401',
    ).toBe(401);
  });
});
