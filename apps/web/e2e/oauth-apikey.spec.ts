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

    // DENY: source IP outside the allowlist → 403; inside → 200.
    const ipKey = await mintKey(request, account, scope({ account, ipAllowlist: ['10.0.0.0/8'] }));
    expect(
      (
        await request.get('/api/v1/messages', {
          headers: { 'x-api-key': ipKey, 'x-forwarded-for': '8.8.8.8' },
        })
      ).status(),
      'IP outside allowlist → 403',
    ).toBe(403);
    expect(
      (
        await request.get('/api/v1/messages', {
          headers: { 'x-api-key': ipKey, 'x-forwarded-for': '10.1.2.3' },
        })
      ).status(),
      'IP inside allowlist → 200',
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
