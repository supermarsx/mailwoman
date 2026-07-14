import { test, expect } from '@playwright/test';
import { adminLogin, uniq, V6 } from './v6-helpers.ts';

/**
 * V6 live E2E — ADMIN (plan §3 e13): provision a user + domain + quota through the
 * real /admin panel (a SEPARATE session domain), then assert an audit-log entry
 * appears and the export works. Driven against the live mw-server backed by real
 * postgres:16. The Rust harness (crates/mw-server/tests/v6_e2e.rs) additionally
 * cross-checks the audit row with a direct Postgres query.
 */
test.describe('v6 admin panel (live)', () => {
  test('provision user + domain + quota → audit entry + export', async ({ request }) => {
    // The admin surface is gated on the separate session domain.
    expect((await request.get('/admin/session')).status(), 'unauthenticated → 401').toBe(401);
    await adminLogin(request);

    const session = await (await request.get('/admin/session')).json();
    expect(session.username).toBe(V6.adminUser);

    // Unique names so re-runs against the persistent Postgres never collide.
    const u = uniq();
    const domain = `d${u}.example`;
    const username = `alice${u}`;
    const account = `${username}@${domain}`;

    // Domain upsert.
    const saveDomain = await request.put(`/admin/domains/${domain}`, {
      data: { name: domain, upstreamJson: '{}', allowlist: [], blocklist: [] },
    });
    expect(saveDomain.status(), 'domain upsert → 204').toBe(204);

    // Provision user + quota.
    const prov = await request.post('/admin/users', {
      data: { domain, username, quota: { bytesLimit: 1048576, msgLimit: 100 } },
    });
    expect(prov.status(), 'provision user → 204').toBe(204);

    // The user is listed.
    const users = await (await request.get('/admin/users')).json();
    expect(
      (users as Array<{ accountId: string }>).some((x) => x.accountId === account),
      'provisioned user listed',
    ).toBe(true);

    // The audit log recorded the provisioning.
    const audit = await (await request.get('/admin/audit?limit=50')).json();
    expect(
      (audit as Array<{ action: string }>).some((e) => e.action === 'user-provisioned'),
      'audit has user-provisioned',
    ).toBe(true);

    // Export (NDJSON) works and includes the entry.
    const exportResp = await request.get('/admin/audit/export?limit=50');
    expect(exportResp.status(), 'audit export → 200').toBe(200);
    const ndjson = await exportResp.text();
    expect(ndjson, 'export includes the entry').toContain('user-provisioned');
    for (const line of ndjson.split('\n').filter((l) => l.trim())) {
      expect(() => JSON.parse(line), 'every export line is valid JSON').not.toThrow();
    }

    // Wrong admin password fails closed.
    const bad = await request.post('/admin/login', {
      data: { username: V6.adminUser, password: 'definitely-wrong' },
    });
    expect(bad.status(), 'wrong password → 401').toBe(401);
  });
});
