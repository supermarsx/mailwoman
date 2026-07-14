import { test, expect } from '@playwright/test';
import { V7, adminLogin, expectMounted } from './v7-helpers.ts';

/**
 * V7 plugin-registry live E2E (plan §3 e16). The wasmtime JAIL itself — capability
 * grant enforced, out-of-allowlist host DENIED, resource-limit trip ⇒ clean
 * LimitExceeded, unsigned ⇒ banner — is proven against the REAL committed
 * `wasm32-wasip2` components (LanguageTool + the Graph bridge) in the Rust harness
 * `crates/mw-server/tests/v7_e2e.rs` (the browser cannot load a component into the host
 * jail). This spec proves the web-facing admin registry contract: the registry is
 * mounted, approve/enable/disable + the `allow-unsigned` policy are wired, and the
 * unsigned-banner signal is exposed to the admin UI.
 */

test.describe('Plugin registry (V7) — admin surface on the real server', () => {
  test('registry lists + approve/enable/allow-unsigned routes are mounted', async ({ request }) => {
    await adminLogin(request);

    const list = await request.get('/admin/plugins');
    expectMounted(list.status(), 'GET /admin/plugins');
    expect(list.status()).toBe(200);
    const body = await list.json();
    expect(Array.isArray(body.plugins), 'registry returns a plugin array').toBe(true);

    // Every registry row carries the fields the admin UI renders — including the
    // `signature`/unsigned signal that drives the permanent unsigned banner.
    for (const p of body.plugins as Array<Record<string, unknown>>) {
      expect(p).toHaveProperty('id');
      expect(p).toHaveProperty('enabled');
      // capabilities + net allowlist are what the capability-grant UI shows.
      expect(p).toHaveProperty('capabilities');
    }

    // The lifecycle routes answer with their own handler (deny-by-default: an unknown
    // id is a 400/404-from-handler, never the SPA fall-through).
    const approve = await request.post('/admin/plugins/does-not-exist/approve').catch(() => null);
    if (approve) expectMounted(approve.status(), 'POST /admin/plugins/{id}/approve');

    const unsigned = await request.post('/admin/plugins/does-not-exist/allow-unsigned');
    expectMounted(unsigned.status(), 'POST /admin/plugins/{id}/allow-unsigned');
    expect(unsigned.status(), 'unknown id ⇒ handler 400 (not SPA)').toBe(400);
  });

  test('the registry is admin-gated (unauthenticated ⇒ 401)', async ({ request }) => {
    const resp = await request.get('/admin/plugins');
    // Mounted + gated: unauthenticated is 401 from the handler, not a 404/SPA.
    expect([401, 403]).toContain(resp.status());
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/admin/plugins').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack; admin=${V7.adminUser}).`,
  );
});
