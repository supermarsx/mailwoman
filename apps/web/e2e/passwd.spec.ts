import { test, expect } from '@playwright/test';
import { mailboxLogin, expectMounted } from './v7-helpers.ts';

/**
 * V7 password-change live E2E (plan §3 e16). The backend change paths — `Local`
 * (Argon2id re-hash) and the RFC-3062 PasswordModify exop against REAL OpenLDAP — plus
 * the re-seal / zero-access-rewrap outcome signals are proven at the Rust level in
 * `crates/mw-server/tests/v7_e2e.rs`. The zero-access key-hierarchy re-wrap CEREMONY runs
 * client-side in the crypto worker and is covered by the existing crypto/zero-access web
 * suite. This spec proves the browser-facing contract: the policy is served (so the UI
 * can display the rules before a change) and a change is routed to the configured
 * backend (a wrong current password is a 400/403 from the handler, never a 404/SPA).
 */

test.describe('Password change (V7) — web-facing HTTP contract', () => {
  test('policy is displayed and a change is routed to the backend', async ({ request }) => {
    await mailboxLogin(request);

    const policy = await request.get('/api/password/policy');
    expectMounted(policy.status(), 'GET /api/password/policy');
    expect(policy.status()).toBe(200);
    const body = await policy.json();
    // The rules the change form displays before a change.
    expect(body).toHaveProperty('minLength');
    expect(typeof body.minLength).toBe('number');

    // A change with a wrong/absent current password is rejected BY THE BACKEND
    // (400/403), proving the route + injected backend answered — not the SPA.
    const change = await request.post('/api/password', {
      data: { oldPassword: 'definitely-not-the-current', newPassword: 'A-Strong-Passw0rd!' },
    });
    expectMounted(change.status(), 'POST /api/password');
    expect([400, 403]).toContain(change.status());
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/api/password/policy').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack).`,
  );
});
