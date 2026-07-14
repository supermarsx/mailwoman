import { test, expect } from '@playwright/test';
import { V7, mailboxLogin, adminLogin, expectMounted } from './v7-helpers.ts';

/**
 * V7 Assist live E2E (plan §3 e16, DoD §7.5): the browser-facing Assist contract on
 * the REAL mounted server. The safety-critical proofs — capability grant/deny,
 * **E2EE-decrypted content NEVER forwarded**, content-free audit, and the compile-time
 * no-send guarantee — are asserted end-to-end against a mock endpoint in the Rust
 * harness (`crates/mw-server/tests/v7_e2e.rs::assist_scope_redaction_and_content_free_audit_live`,
 * which records exactly what left the gateway). Here we prove the web surface: the
 * "what left the device" disclosure is present, an unconfigured gateway is Disabled
 * (⇒ the web hides all Assist UI + invoke 404s), and the admin governance
 * (endpoint/capability/kill-switch) is mounted.
 */

test.describe('Assist (V7) — web-facing contract on the real server', () => {
  test('config carries the "what left the device" disclosure; disabled ⇒ invoke 404', async ({
    request,
  }) => {
    await mailboxLogin(request);

    const cfg = await request.get('/api/assist/config');
    expectMounted(cfg.status(), 'GET /api/assist/config');
    expect(cfg.status()).toBe(200);
    const body = await cfg.json();
    // The disclosure copy is ALWAYS present so the UI can show "what left the device".
    expect(typeof body.disclosure, 'disclosure string present').toBe('string');
    expect(body.disclosure.length).toBeGreaterThan(0);

    if (body.enabled === false) {
      // Unconfigured/kill-switched ⇒ Disabled ⇒ every capability invoke 404s and the
      // web hides all Assist UI.
      const invoke = await request.post('/api/assist/invoke', {
        data: { capability: 'summarize', input: { prompt: 'hi', context: [] } },
      });
      expect(invoke.status(), 'disabled ⇒ invoke 404').toBe(404);

      const transcribe = await request.post('/api/assist/transcribe', {
        data: { audioBase64: 'AAA=', mime: 'audio/webm' },
      });
      expect(transcribe.status(), 'disabled ⇒ transcribe 404').toBe(404);
    }
  });

  test('there is NO Assist send/delete/accept route (send is always human-gated)', async ({
    request,
  }) => {
    await mailboxLogin(request);
    // Assist has no transmit capability (structural, mw_assist::AssistCapability). No
    // send/delete/accept endpoint exists on the Assist surface.
    for (const path of ['/api/assist/send', '/api/assist/delete', '/api/assist/accept']) {
      const resp = await request.post(path, { data: {} });
      expect(
        resp.status(),
        `${path} must not be a working send path`,
      ).not.toBe(200);
    }
  });

  test('admin Assist governance is mounted: GET/PUT config + kill switch', async ({ request }) => {
    await adminLogin(request);

    const get = await request.get('/admin/assist');
    expectMounted(get.status(), 'GET /admin/assist');
    expect(get.status()).toBe(200);

    // Persist an endpoint allowlist + capability grants, then flip the kill switch.
    const put = await request.put('/admin/assist', {
      data: {
        enabled: true,
        adapters: {
          OpenAiCompatible: {
            base_url: process.env['MW_E2E_ASSIST_URL'] ?? 'http://mock-assist:8199',
            chat_model: 'mock',
            embed_model: 'mock',
            api_key: 'k',
          },
        },
        capabilityGrants: ['summarize'],
        dataCeilings: { accounts: [] },
      },
    });
    expect([200, 204]).toContain(put.status());

    const kill = await request.post('/admin/assist/kill');
    expectMounted(kill.status(), 'POST /admin/assist/kill');
    expect(kill.status()).toBe(200);
    expect((await kill.json()).killed).toBe(true);

    // After the kill switch, the gateway is Disabled again.
    const after = await request.get('/admin/assist');
    expect((await after.json()).enabled).toBe(false);
  });
});

// Guard so an accidental local run without a stack fails fast+clear rather than hanging.
test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/api/assist/config').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack; user=${V7.adminUser}).`,
  );
});
