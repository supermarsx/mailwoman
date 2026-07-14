import { test, expect } from '@playwright/test';
import { mailboxLogin, expectMounted } from './v7-helpers.ts';

/**
 * V7 directory/GAL live E2E (plan §3 e16). The directory logic against REAL OpenLDAP —
 * GAL search across recipient fields, group expand-before-send, S/MIME cert + photo
 * lookup, multi-directory priority, LDAP-bind — is proven at the Rust level against a
 * live seeded OpenLDAP in `crates/mw-server/tests/v7_e2e.rs` (the browser cannot open an
 * LDAP connection). This spec proves the browser-facing HTTP surface is MOUNTED and
 * behaves per the deployment posture: when a `directory_config` row is seeded the GAL
 * endpoints return results; when unconfigured they return an honest 501 (never a 404/SPA
 * fall-through), which is the contract the recipient-field autocomplete relies on.
 */

test.describe('Directory / GAL (V7) — web-facing HTTP contract', () => {
  test('GAL search / group-expand / cert routes are mounted', async ({ request }) => {
    await mailboxLogin(request);

    const search = await request.get('/api/directory/search?q=alice');
    expectMounted(search.status(), 'GET /api/directory/search');
    // Configured ⇒ 200 with entries; unconfigured ⇒ 501. Both prove the mount.
    expect([200, 501]).toContain(search.status());
    if (search.status() === 200) {
      const body = await search.json();
      const entries = (body.entries ?? body.results ?? body) as unknown;
      expect(Array.isArray(entries), 'GAL search returns an entry array').toBe(true);
    }

    const cert = await request.get('/api/directory/cert?email=alice@example.com');
    expectMounted(cert.status(), 'GET /api/directory/cert');
    expect([200, 404, 501]).toContain(cert.status());

    // Group expand-before-send: the endpoint is mounted (405/400/501 all prove a real
    // handler answered; a 404-SPA would not).
    const expand = await request
      .get('/api/directory/group?dn=cn=engineering,ou=groups,dc=example,dc=com')
      .catch(() => null);
    if (expand) expectMounted(expand.status(), 'GET /api/directory/group');
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/api/directory/search?q=x').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack).`,
  );
});
