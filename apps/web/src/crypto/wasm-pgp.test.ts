// The OpenPGP core exports of the committed mw-crypto wasm guest (t32-e37), driven
// directly like wasm-crypto.test.ts: key profiles and inspection, key management,
// PGP/MIME with header protection, message analysis, Autocrypt and attached keys.
// The Rust behind each export is tested in crates/mw-crypto/tests/pgp_*.rs; this
// file proves the guest exports them with the shapes contracts/crypto.ts declares.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import {
  initSync,
  __init,
  addSubkey,
  analyzeMessage,
  autocryptHeader,
  autocryptRecommend,
  changePassphrase,
  decrypt,
  detectAttachedKeys,
  encrypt,
  exportBackupFull,
  generateKey,
  inspectKey,
  parseAutocrypt,
  pgpMimeEncrypt,
  pgpMimeOpen,
  pgpMimeSign,
  pgpMimeVerify,
  restoreBackup,
  revokeKey,
  setKeyExpiry,
  sign,
  unlockKey,
  verifyPgp,
  wkdUrls,
} from '../wasm/mw-crypto/mw_crypto.js';
import type {
  AnalyzeMessageResult,
  AutocryptHeader,
  BuiltEntity,
  DecryptResult,
  EncryptResult,
  GenerateKeyResult,
  ManagedKey,
  OpenedMessage,
  PgpKeyInfo,
  VerifyPgpResult,
} from '../contracts/crypto.ts';

const PW = 'wasm pgp passphrase';
const HEADERS: [string, string][] = [
  ['From', 'Ann <ann@example.com>'],
  ['To', 'Bob <bob@example.com>'],
  ['Subject', 'Wasm subject'],
];
const PAYLOAD = 'Content-Type: text/plain; charset=utf-8\r\n\r\nbody through the guest\r\n';

let ann: GenerateKeyResult;
let bob: GenerateKeyResult;

beforeAll(() => {
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm/mw-crypto/mw_crypto_bg.wasm')) });
  __init();
  ann = generateKey({ kind: 'pgp', userId: 'Ann <ann@example.com>', passphrase: PW }) as GenerateKeyResult;
  bob = generateKey({
    kind: 'pgp',
    userId: 'Bob <bob@example.com>',
    passphrase: PW,
    profile: 'compatible',
  }) as GenerateKeyResult;
}, 60_000);

describe('mw-crypto wasm guest: OpenPGP core (t32-e37)', () => {
  it('inspects both key profiles', () => {
    const a = inspectKey({ armored: ann.publicKeyArmored }) as PgpKeyInfo;
    expect(a.version).toBe(6);
    expect(a.state).toBe('valid');
    expect(a.seipdV2).toBe(true);
    expect(a.userIds[0]?.address).toBe('ann@example.com');
    const b = inspectKey({ armored: bob.publicKeyArmored }) as PgpKeyInfo;
    expect(b.version).toBe(4);
    expect(b.algorithm).toBe('eddsa-legacy');
    expect(b.seipdV2).toBe(false);
  });

  it('picks SEIPDv1 for a mixed recipient list and reports the verification', () => {
    const enc = encrypt({
      kind: 'pgp',
      plaintext: 'mixed',
      recipientPublicKeys: [ann.publicKeyArmored!, bob.publicKeyArmored!],
    }) as EncryptResult;
    expect(enc.seipdVersion).toBe(1);
    const dec = decrypt({
      kind: 'pgp',
      ciphertext: enc.armoredCiphertext,
      encryptedPrivateBundle: bob.encryptedPrivateBundle,
      passphrase: PW,
      signerPublicKeys: [ann.publicKeyArmored],
    }) as DecryptResult & { plaintextText: string };
    expect(dec.plaintextText).toBe('mixed');
    expect(dec.verification?.state).toBe('not-signed');
    expect(dec.aead).toBe(false);
  }, 30_000);

  it('manages a key: expiry, subkey, passphrase, revocation, backup', () => {
    const at = Math.floor(Date.now() / 1000) + 86_400 * 30;
    const withExpiry = setKeyExpiry({ encryptedPrivateBundle: ann.encryptedPrivateBundle, passphrase: PW, expiresAt: at }) as ManagedKey;
    expect((inspectKey({ armored: withExpiry.publicKeyArmored }) as PgpKeyInfo).expiresAt).not.toBeNull();
    const withSub = addSubkey({
      encryptedPrivateBundle: withExpiry.encryptedPrivateBundle,
      passphrase: PW,
      purpose: 'encrypt',
    }) as ManagedKey;
    expect((inspectKey({ armored: withSub.publicKeyArmored }) as PgpKeyInfo).subkeys).toHaveLength(2);
    const changed = changePassphrase({
      encryptedPrivateBundle: withSub.encryptedPrivateBundle,
      oldPassphrase: PW,
      newPassphrase: 'new one',
    }) as { encryptedPrivateBundle: string };
    expect(() => setKeyExpiry({ encryptedPrivateBundle: changed.encryptedPrivateBundle, passphrase: PW, expiresAt: null })).toThrow();
    const revoked = revokeKey({
      encryptedPrivateBundle: changed.encryptedPrivateBundle,
      passphrase: 'new one',
      reason: 'retired',
      message: 'replaced',
    }) as ManagedKey;
    const info = inspectKey({ armored: revoked.publicKeyArmored }) as PgpKeyInfo;
    expect(info.state).toBe('revoked');
    expect(info.revocation?.reason).toBe('retired');

    const backup = exportBackupFull({ encryptedPrivateBundle: ann.encryptedPrivateBundle, from: 'ann@example.com' }) as {
      setupCode: string;
      payload: string;
      mime: string;
    };
    expect(backup.setupCode).toMatch(/^\d{4}(-\d{4}){8}$/);
    expect(backup.mime).toContain('Autocrypt-Setup-Message: v1');
    const restored = restoreBackup({ payload: backup.payload, setupCode: backup.setupCode }) as {
      encryptedPrivateBundle: string;
    };
    expect(restored.encryptedPrivateBundle).toContain('BEGIN PGP PRIVATE KEY');
  }, 60_000);

  it('builds and opens PGP/MIME with protected headers', () => {
    const built = pgpMimeEncrypt({
      payload: PAYLOAD,
      headers: HEADERS,
      recipientPublicKeys: [ann.publicKeyArmored, bob.publicKeyArmored],
      encryptedPrivateBundle: ann.encryptedPrivateBundle,
      passphrase: PW,
      gossip: [['bob@example.com', bob.publicKeyArmored!]],
    }) as BuiltEntity;
    expect(built.outerSubject).toBe('[...]');
    expect(built.entity).not.toContain('Wasm subject');
    expect((analyzeMessage({ raw: built.entity }) as AnalyzeMessageResult).kind).toBe('pgp-encrypted');

    const opened = pgpMimeOpen({
      raw: new TextEncoder().encode(built.entity),
      encryptedPrivateBundle: bob.encryptedPrivateBundle,
      passphrase: PW,
      signerPublicKeys: [ann.publicKeyArmored],
    }) as OpenedMessage;
    expect(opened.verification.state).toBe('valid');
    expect(opened.protected?.scheme).toBe('cipher');
    expect(opened.protected?.headers).toContainEqual(['Subject', 'Wasm subject']);
    expect(opened.entity).toContain('body through the guest');
    expect(opened.gossip.map((g) => g.addr)).toEqual(['bob@example.com']);

    const signed = pgpMimeSign({
      payload: PAYLOAD,
      headers: HEADERS,
      encryptedPrivateBundle: ann.encryptedPrivateBundle,
      passphrase: PW,
    }) as BuiltEntity;
    const verified = pgpMimeVerify({ raw: signed.entity, signerPublicKeys: [ann.publicKeyArmored] }) as OpenedMessage;
    expect(verified.verification.state).toBe('valid');
    const tampered = pgpMimeVerify({
      raw: signed.entity.replace('body through the guest', 'body changed in transit'),
      signerPublicKeys: [ann.publicKeyArmored],
    }) as OpenedMessage;
    expect(tampered.verification.state).toBe('bad-signature');
  }, 60_000);

  it('verifies a detached signature with the full state', () => {
    const { signatureArmored } = sign({
      kind: 'pgp',
      data: 'signed data',
      encryptedPrivateBundle: ann.encryptedPrivateBundle,
      passphrase: PW,
      detached: true,
    }) as { signatureArmored: string };
    const good = verifyPgp({ data: 'signed data', signature: signatureArmored, signerPublicKeys: [ann.publicKeyArmored] }) as VerifyPgpResult;
    expect(good.verification.state).toBe('valid');
    expect(good.verification.signerFingerprint).toBe(ann.fingerprint);
    expect(good.verification.signature.status).toBe('verified');
    const bad = verifyPgp({ data: 'signed dat4', signature: signatureArmored, signerPublicKeys: [ann.publicKeyArmored] }) as VerifyPgpResult;
    expect(bad.verification.state).toBe('bad-signature');
    expect(bad.verification.detail).toBe('tampered');
    expect(() => verifyPgp({ data: 'x', signature: 'not a signature', signerPublicKeys: [] })).toThrow();
  });

  it('handles Autocrypt, attached keys and WKD URLs', () => {
    const value = autocryptHeader({ publicKeyArmored: ann.publicKeyArmored, addr: 'ann@example.com', preferEncrypt: true }) as string;
    const parsed = parseAutocrypt({ value }) as AutocryptHeader;
    expect(parsed.addr).toBe('ann@example.com');
    expect(parsed.preferEncrypt).toBe(true);
    expect(parsed.key.fingerprint).toBe(ann.fingerprint);
    const rec = autocryptRecommend({
      peers: [{ lastSeen: 10, autocryptTimestamp: 10, hasAutocryptKey: true, hasGossipKey: false, preferEncrypt: true }, null],
      ownPreferEncrypt: true,
    }) as { perRecipient: string[]; message: string };
    expect(rec.perRecipient).toEqual(['encrypt', 'disable']);
    expect(rec.message).toBe('disable');

    const keys = detectAttachedKeys({ contentType: 'application/pgp-keys', filename: 'key.asc', bytes: ann.publicKeyArmored }) as
      | { fingerprint: string }[]
      | null;
    expect(keys?.map((k) => k.fingerprint)).toEqual([ann.fingerprint]);
    expect(detectAttachedKeys({ contentType: 'image/png', bytes: new Uint8Array([1, 2]) })).toBeNull();

    expect(wkdUrls({ email: 'Joe.Doe@Example.ORG' })).toEqual([
      'https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe',
      'https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe',
    ]);
  });
  it('verifies mail signed by either of two keys for one sender (key rotation)', () => {
    const annNew = generateKey({ kind: 'pgp', userId: 'Ann <ann@example.com>', passphrase: PW }) as GenerateKeyResult;
    const both = [ann.publicKeyArmored!, annNew.publicKeyArmored!];
    for (const [signer, label] of [[ann, 'old'], [annNew, 'new']] as const) {
      const { keyRef } = unlockKey({ kind: 'pgp', encryptedPrivateBundle: signer.encryptedPrivateBundle, passphrase: PW }) as {
        keyRef: string;
      };
      const enc = encrypt({
        kind: 'pgp',
        plaintext: `signed by the ${label} key`,
        recipientPublicKeys: [bob.publicKeyArmored!],
        signWithKeyRef: keyRef,
      }) as EncryptResult;
      const dec = decrypt({
        kind: 'pgp',
        ciphertext: enc.armoredCiphertext,
        encryptedPrivateBundle: bob.encryptedPrivateBundle,
        passphrase: PW,
        signerPublicKeys: both,
      }) as DecryptResult & { plaintextText: string };
      expect(dec.plaintextText).toBe(`signed by the ${label} key`);
      expect(dec.signature.status).toBe('verified');
      expect(dec.verification?.signerFingerprint).toBe(signer.fingerprint);
    }
    // The old behaviour (only the first key passed) cannot verify the new key's mail.
    const { keyRef } = unlockKey({ kind: 'pgp', encryptedPrivateBundle: annNew.encryptedPrivateBundle, passphrase: PW }) as {
      keyRef: string;
    };
    const enc = encrypt({ kind: 'pgp', plaintext: 'x', recipientPublicKeys: [bob.publicKeyArmored!], signWithKeyRef: keyRef }) as EncryptResult;
    const onlyOld = decrypt({
      kind: 'pgp',
      ciphertext: enc.armoredCiphertext,
      encryptedPrivateBundle: bob.encryptedPrivateBundle,
      passphrase: PW,
      signerPublicKeys: [ann.publicKeyArmored!],
    }) as DecryptResult;
    expect(onlyOld.signature.status).toBe('unverified-key');
  }, 60_000);
});

