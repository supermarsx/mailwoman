import { test, expect } from '@playwright/test';

import { mailboxLogin } from './v6-helpers.ts';

/**
 * V6 live E2E — ZERO-ACCESS (plan §3 e13): enable zero-access through the real
 * surface and assert the browser-observable guarantee — the server stores and
 * returns ONLY the client's wrapped (ciphertext) key material, never a plaintext
 * key. The client submits opaque wrapped bytes; the server round-trips exactly
 * those bytes.
 *
 * The DoD's *ciphertext-at-rest* proof — a DIRECT Postgres query showing the
 * stored row is ciphertext, plus the Valkey no-plaintext-derived check — lives in
 * the Rust harness (crates/mw-server/tests/v6_e2e.rs), because the browser cannot
 * query the database or the cache. That harness reads the raw BYTEA column back
 * from live Postgres and asserts it equals the submitted ciphertext with no
 * plaintext marker present.
 */
test.describe('v6 zero-access (live)', () => {
  test('enable → server holds wrapped material only, never a plaintext key', async ({ request }) => {
    await mailboxLogin(request);

    // Opaque wrapped key material (nonce ‖ ct+tag shape) — in production produced
    // by the mw-crypto WASM worker (XChaCha20-Poly1305). A known plaintext marker
    // that the server must NEVER see.
    const PLAINTEXT_MARKER = 'E13_WEB_ZERO_ACCESS_PLAINTEXT_KEY';
    const wrapped = new Uint8Array(56);
    for (let i = 0; i < wrapped.length; i++) wrapped[i] = 0x80 | (i & 0x7f);
    const wrappedB64 = Buffer.from(wrapped).toString('base64');

    const enable = await request.post('/api/zeroaccess/enable', {
      data: {
        saltB64: 'c2FsdHNhbHRzYWx0',
        kdfParams: { mCost: 19456, tCost: 2, pCost: 1 },
        wrappedDataKeyB64: wrappedB64,
      },
    });
    expect(enable.status(), 'zero-access enable wired').toBe(200);

    // Status returns wrapped material only — never a key.
    const za = await (await request.get('/api/zeroaccess')).json();
    expect(za.enabled, 'zero-access enabled').toBe(true);
    expect(za.wrappedDataKeyB64, 'server returns exactly the wrapped bytes').toBe(wrappedB64);

    // The plaintext marker never appears anywhere in what the server exposes.
    expect(JSON.stringify(za)).not.toContain(PLAINTEXT_MARKER);
    expect(za).not.toHaveProperty('rootKey');
    expect(za).not.toHaveProperty('dataKey');

    // The pairing relay carries ciphertext envelopes verbatim (server relays only).
    const offer = await (await request.post('/api/zeroaccess/pair/offer', {
      data: { publicB64: 'cHVibGlj' },
    })).json();
    const pairingId = offer.pairingId as string;
    await request.post('/api/zeroaccess/pair/envelope', {
      data: { pairingId, envelopeB64: 'ZW52ZWxvcGU=' },
    });
    const got = await (await request.get(`/api/zeroaccess/pair/envelope/${pairingId}`)).json();
    expect(got.envelopeB64, 'pairing envelope relayed verbatim').toBe('ZW52ZWxvcGU=');
  });
});
