// Functional smoke test for the committed `mw-crypto` wasm guest (t24-e13).
//
// Before this file the guest had NO test coverage at all: `getCryptoWorker()`
// returns the deterministic stub under vitest (`useRealWorker()` is false when
// `MODE === 'test'`, crypto/index.ts), and jsdom cannot host a Worker — so nothing
// in the web suite ever loaded `mw_crypto_bg.wasm`. A guest that was stale, broken,
// or built by a mismatched `wasm-bindgen` CLI would have shipped silently.
//
// Like sanitize.test.ts this loads the committed bytes with `initSync` and drives
// the real wasm directly, which jsdom CAN do. A PGP round trip through the actual
// exported surface is the strongest cheap proof that the guest and its generated
// glue are a working, mutually-consistent pair — the property that matters when
// the artefact is rebuilt, since wasm-pack regenerates `mw_crypto.js` and
// `mw_crypto_bg.wasm` together and their ABI must agree.
//
// v6 keys are Ed25519/X25519, so keygen is fast enough for a unit test.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync, __init, generateKey, encrypt, decrypt, exportPublic } from '../wasm/mw-crypto/mw_crypto.js';

const PASSPHRASE = 'correct horse battery staple';
const USER_ID = 'Test User <test@example.com>';

let key: {
  publicKeyArmored: string;
  fingerprint: string;
  keyId: string;
  encryptedPrivateBundle: string;
};

beforeAll(() => {
  // vitest runs with cwd = apps/web (the vite config dir).
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm/mw-crypto/mw_crypto_bg.wasm')) });
  __init();
  key = generateKey({ kind: 'pgp', userId: USER_ID, passphrase: PASSPHRASE });
}, 60_000);

describe('committed mw-crypto wasm guest is loadable and functional', () => {
  it('generates a v6 PGP key with a real fingerprint and a private bundle', () => {
    expect(key.publicKeyArmored).toContain('BEGIN PGP PUBLIC KEY BLOCK');
    // A PGP fingerprint is hex; v6 keys are longer than v4's 40 chars.
    expect(key.fingerprint).toMatch(/^[0-9A-Fa-f]{40,}$/);
    expect(key.keyId).not.toBe('');
    expect(key.encryptedPrivateBundle).not.toBe('');
    // The stub's placeholders must never be mistaken for a real result.
    expect(key.fingerprint).not.toContain('STUB');
    expect(key.publicKeyArmored).not.toContain('STUB');
  });

  it('round-trips encrypt → decrypt with the generated key', () => {
    const plaintext = 'decrypted E2EE body — t24-e13 round trip';
    const enc = encrypt({
      kind: 'pgp',
      plaintext,
      recipientPublicKeys: [key.publicKeyArmored],
    }) as { armoredCiphertext: string };

    expect(enc.armoredCiphertext).toContain('BEGIN PGP MESSAGE');
    // Genuinely encrypted: the plaintext must not survive in the armor.
    expect(enc.armoredCiphertext).not.toContain(plaintext);

    const dec = decrypt({
      kind: 'pgp',
      ciphertext: enc.armoredCiphertext,
      encryptedPrivateBundle: key.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
    }) as { plaintextText?: string; plaintextHtml?: string };

    expect(dec.plaintextText ?? dec.plaintextHtml).toBe(plaintext);
  }, 60_000);

  it('refuses to decrypt with the wrong passphrase', () => {
    const enc = encrypt({
      kind: 'pgp',
      plaintext: 'secret',
      recipientPublicKeys: [key.publicKeyArmored],
    }) as { armoredCiphertext: string };

    expect(() =>
      decrypt({
        kind: 'pgp',
        ciphertext: enc.armoredCiphertext,
        encryptedPrivateBundle: key.encryptedPrivateBundle,
        passphrase: 'wrong passphrase',
      }),
    ).toThrow();
  }, 60_000);

  it('exports the public key without the private bundle', () => {
    const armored = exportPublic({ keyRef: key.publicKeyArmored }) as string;
    expect(armored).toContain('BEGIN PGP PUBLIC KEY BLOCK');
    expect(armored).not.toContain('PRIVATE KEY');
  });
});
