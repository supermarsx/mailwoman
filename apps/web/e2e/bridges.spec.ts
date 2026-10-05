import { test, expect } from '@playwright/test';
import { adminLogin, expectMounted } from './v7-helpers.ts';

/**
 * V7 bridges live E2E (plan §3 e16). The headline bridge proof — a REAL
 * `wasm32-wasip2` Graph-bridge component loaded in the wasmtime jail, its
 * `as_account_backend()` registered on the engine, serving the JMAP surface through the
 * SAME dispatch an IMAP account uses against recorded Graph fixtures — is proven in the
 * Rust harness `crates/mw-server/tests/v7_e2e.rs`
 * (`plugin_backed_account_serves_mailboxes_through_engine_jmap`; the deeper per-message
 * sync is captured by an ESCALATED, currently-ignored reproduction there). The browser
 * cannot load a wasm component into the host jail, so the bridge account-backend proof
 * is server-side by design. This spec proves the web-facing contract: bridge plugins
 * surface in the admin registry as account-backend-capable, approvable plugins.
 */

test.describe('Bridges (V7) — registry surface on the real server', () => {
  test('the three bridges register as account-backend plugins and are not reported as running', async ({
    request,
  }) => {
    await adminLogin(request);
    const bridges = ['bridge-ews', 'bridge-gmail', 'bridge-graph'];
    for (const id of bridges) await request.post(`/admin/plugins/${id}/uninstall`);

    for (const id of bridges) {
      const registered = await request.post('/admin/plugins', { data: { id } });
      expectMounted(registered, 'POST /admin/plugins');
      expect(registered.status(), `${id} registers`).toBe(201);
      const plugin = (await registered.json()).plugin as Record<string, unknown>;
      expect(plugin.role).toBe('account-backend');
      expect(plugin.capabilities as string[]).toContain('account-backend');
      // Registration grants nothing and loads nothing.
      expect(plugin.granted).toEqual([]);
      expect(plugin.loaded).toBe(false);
    }

    const list = await request.get('/admin/plugins');
    expect(list.status()).toBe(200);
    const listed = ((await list.json()).plugins as Array<Record<string, unknown>>).filter((p) =>
      String(p.id).startsWith('bridge-'),
    );
    expect(listed.map((p) => p.id)).toEqual(bridges);

    for (const id of bridges) {
      const removed = await request.post(`/admin/plugins/${id}/uninstall`);
      expect(removed.status()).toBe(200);
    }
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/admin/plugins').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack).`,
  );
});
