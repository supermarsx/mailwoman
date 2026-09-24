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
    expectMounted(search, 'GET /api/directory/search');
    // Configured ⇒ 200 with entries; unconfigured ⇒ 501. Both prove the mount.
    expect([200, 501]).toContain(search.status());
    if (search.status() === 200) {
      const body = await search.json();
      const entries = (body.entries ?? body.results ?? body) as unknown;
      expect(Array.isArray(entries), 'GAL search returns an entry array').toBe(true);
    }

    const cert = await request.get('/api/directory/cert?email=alice@example.com');
    expectMounted(cert, 'GET /api/directory/cert');
    expect([200, 404, 501]).toContain(cert.status());

    // Group expand-before-send. The DN is a PATH segment, not a query parameter:
    // the route is `GET /api/directory/group/{dn}` (crates/mw-server/src/directory.rs,
    // "The DN is path-encoded by the caller"), and that is what the real SPA client
    // calls — apps/web/src/modules/directory/service.ts encodes the DN into the path.
    // This spec asked for `/api/directory/group?dn=…`, which matches no route at all;
    // it only ever passed because the static fall-through answered 200 for every URL,
    // so the assertion was checking the fallback rather than the mount (t24-e14).
    const dn = 'cn=engineering,ou=groups,dc=example,dc=com';
    const expand = await request
      .get(`/api/directory/group/${encodeURIComponent(dn)}`)
      .catch(() => null);
    if (expand) expectMounted(expand, 'GET /api/directory/group/{dn}');
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/api/directory/search?q=x').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack).`,
  );
});
