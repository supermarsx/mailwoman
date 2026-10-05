// S/MIME key generation in the committed `mw-crypto` wasm guest (t29-e2), judged
// by a parser that is not ours.
//
// The defect this replaces (audit §13 row 35) was a label: an OpenPGP key was
// generated, called `ecdsa-p256`, and its PGP armor stored as `certPem`. So
// nothing here trusts the generator's description of its own output. Node's
// `crypto.X509Certificate` (OpenSSL underneath) parses the certificate, and every
// claim the worker makes — "this is a certificate", "RSA of this size", "for this
// address", "expires then", "the private key belongs to it" — is compared with
// what Node reads from the bytes.
//
// Like wasm-crypto.test.ts this loads the committed guest with `initSync` and
// calls the exported surface directly (jsdom cannot host a Worker). The openssl
// CLI side — request verification, CMS interop, PKCS#12 with a wrong passphrase —
// is `crates/mw-crypto/tests/smime_generate.rs`.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { X509Certificate, createPrivateKey, createPublicKey } from 'node:crypto';
import { createSecureContext } from 'node:tls';
import { beforeAll, describe, expect, it } from 'vitest';
import {
  initSync,
  __init,
  generateKey,
  importPkcs12,
  exportPkcs12,
  certificateRequest,
  attachIssuedCert,
  exportBackup,
  sign,
  verify,
} from '../wasm/mw-crypto/mw_crypto.js';

const PASSPHRASE = 'correct horse battery staple';
const NAME = 'Doe, Alice + Co=1';
const EMAIL = 'alice@example.org';

interface Generated {
  publicKeyArmored?: string;
  fingerprint: string;
  keyId: string;
  encryptedPrivateBundle: string;
  certPem?: string;
  addresses?: string[];
  algorithm?: string;
  expiresAt?: string;
}

let alice: Generated;
let mallory: Generated;
let cert: X509Certificate;
let startedAt: number;

beforeAll(() => {
  // vitest runs with cwd = apps/web (the vite config dir).
  initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm/mw-crypto/mw_crypto_bg.wasm')) });
  __init();
  startedAt = Date.now();
  // No `rsaBits`: this is the size the dialog gets.
  alice = generateKey({ kind: 'smime', userId: `${NAME} <${EMAIL}>`, passphrase: PASSPHRASE }) as Generated;
  // A second, smaller key, only ever used as "somebody else's".
  mallory = generateKey({
    kind: 'smime',
    userId: 'mallory@example.org',
    passphrase: PASSPHRASE,
    rsaBits: 2048,
  }) as Generated;
  cert = new X509Certificate(alice.certPem!);
}, 600_000);

describe('a generated S/MIME key is an X.509 certificate and an RSA key, as Node reads them', () => {
  it('is a certificate, not OpenPGP material under another name', () => {
    expect(alice.certPem).toMatch(/^-----BEGIN CERTIFICATE-----\n/);
    expect(alice.publicKeyArmored).toBeUndefined();
    expect(JSON.stringify(alice)).not.toContain('PGP');
    expect(alice.encryptedPrivateBundle).toMatch(/^-----BEGIN ENCRYPTED PRIVATE KEY-----\n/);
  });

  it('carries the algorithm it is labelled with', () => {
    expect(cert.publicKey.asymmetricKeyType).toBe('rsa');
    const bits = cert.publicKey.asymmetricKeyDetails?.modulusLength;
    expect(bits).toBe(3072);
    expect(alice.algorithm).toBe(`rsa-${bits}`);
    // The explicit-size path is labelled from the key too, not from the default.
    const other = new X509Certificate(mallory.certPem!);
    expect(other.publicKey.asymmetricKeyDetails?.modulusLength).toBe(2048);
    expect(mallory.algorithm).toBe('rsa-2048');
  });

  it('is for the typed address, in the subject alternative name and the subject', () => {
    expect(cert.subjectAltName).toBe(`email:${EMAIL}`);
    expect(cert.checkEmail(EMAIL)).toBe(EMAIL);
    expect(cert.checkEmail('mallory@example.org')).toBeUndefined();
    expect(alice.addresses).toEqual([EMAIL]);
    // The name is one CN value: its `,` `+` `=` did not become DN structure.
    const subject = cert.toLegacyObject().subject as Record<string, string>;
    expect(subject['CN']).toBe(NAME);
    expect(subject['emailAddress']).toBe(EMAIL);
    expect(Object.keys(subject).sort()).toEqual(['CN', 'emailAddress']);
  });

  it('is self-signed, not a CA, and marked for email', () => {
    expect(cert.issuer).toBe(cert.subject);
    expect(cert.verify(cert.publicKey)).toBe(true);
    // (`checkIssued(cert)` is false, correctly: that OpenSSL check also wants the
    // issuer to carry keyCertSign, which a certificate that is not a CA must not
    // assert — RFC 5280 §4.2.1.3. Self-signed here means name and signature.)
    expect(cert.ca).toBe(false);
    // Node reports the extended key usage OIDs: id-kp-emailProtection, and only it.
    expect(cert.keyUsage).toEqual(['1.3.6.1.5.5.7.3.4']);
    // It does not verify under anybody else's key.
    expect(cert.verify(new X509Certificate(mallory.certPem!).publicKey)).toBe(false);
  });

  it('reports the fingerprint and expiry that the certificate has', () => {
    expect(alice.fingerprint).toBe(cert.fingerprint256.replaceAll(':', ''));
    expect(alice.keyId).toBe(alice.fingerprint.slice(0, 16));
    expect(new Date(alice.expiresAt!).getTime()).toBe(new Date(cert.validTo).getTime());
    // Two years from now (730 days), and already valid.
    const lifetimeDays = (new Date(cert.validTo).getTime() - startedAt) / 86_400_000;
    expect(lifetimeDays).toBeGreaterThan(729.9);
    expect(lifetimeDays).toBeLessThan(730.1);
    expect(new Date(cert.validFrom).getTime()).toBeLessThanOrEqual(startedAt);
  });

  it('wraps the private key of that certificate under the passphrase', () => {
    const key = createPrivateKey({ key: alice.encryptedPrivateBundle, passphrase: PASSPHRASE });
    expect(key.asymmetricKeyType).toBe('rsa');
    const fromKey = createPublicKey(key).export({ type: 'spki', format: 'der' });
    const fromCert = cert.publicKey.export({ type: 'spki', format: 'der' });
    expect(Buffer.compare(fromKey, fromCert)).toBe(0);
    expect(cert.checkPrivateKey(key)).toBe(true);
    expect(() => createPrivateKey({ key: alice.encryptedPrivateBundle, passphrase: 'wrong' })).toThrow();
    // Never in the clear.
    expect(() => createPrivateKey(alice.encryptedPrivateBundle)).toThrow();
  }, 60_000);

  it('makes a different key and serial each time', () => {
    expect(mallory.fingerprint).not.toBe(alice.fingerprint);
    expect(new X509Certificate(mallory.certPem!).serialNumber).not.toBe(cert.serialNumber);
    expect(cert.serialNumber).toMatch(/^[4-7][0-9A-F]{31}$/); // 16 octets, positive
  });
});

// Each unlock of the wrapped key is PBKDF2 with 600 000 rounds, so these take
// seconds, not milliseconds; the explicit timeouts keep a loaded machine green.
describe('what the generated key can be used for', () => {
  it('signs, and the signature verifies', () => {
    const signed = sign({
      kind: 'smime',
      data: 'signed with a generated key',
      encryptedPrivateBundle: alice.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
      detached: false,
      certPem: alice.certPem,
    }) as { signatureArmored: string };
    const verdict = verify({ kind: 'smime', data: '', signature: signed.signatureArmored }) as { status: string };
    expect(verdict.status).toBe('verified');
  }, 60_000);

  it('exports as PKCS#12 that imports back as the same certificate and key', () => {
    const { p12Base64 } = exportPkcs12({
      certPem: alice.certPem,
      encryptedPrivateBundle: alice.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
    }) as { p12Base64: string };
    const p12Bytes = Uint8Array.from(Buffer.from(p12Base64, 'base64'));
    expect(p12Bytes[0]).toBe(0x30); // a DER SEQUENCE

    // Node (OpenSSL) opens the file with the passphrase — which includes checking
    // its integrity MAC — and refuses it with another, and after one altered octet
    // in the certificate bag, which is not encrypted and so is protected by the
    // MAC alone.
    const pfx = Buffer.from(p12Bytes);
    expect(() => createSecureContext({ pfx, passphrase: PASSPHRASE })).not.toThrow();
    expect(() => createSecureContext({ pfx, passphrase: 'wrong' })).toThrow(/mac verify failure/i);
    const altered = Buffer.from(pfx);
    const at = altered.indexOf(EMAIL);
    expect(at).toBeGreaterThan(0);
    altered[at] = altered[at]! ^ 0x01;
    expect(() => createSecureContext({ pfx: altered, passphrase: PASSPHRASE })).toThrow(/mac verify failure/i);
    expect(() => importPkcs12({ p12Bytes: Uint8Array.from(altered), password: PASSPHRASE })).toThrow(
      /integrity check failed/,
    );

    const back = importPkcs12({ p12Bytes, password: PASSPHRASE }) as {
      certPem: string;
      fingerprint: string;
      encryptedPrivateBundle: string;
      addresses: string[];
      algorithm: string;
      expiresAt: string;
    };
    expect(back.fingerprint).toBe(alice.fingerprint);
    expect(back.addresses).toEqual([EMAIL]);
    expect(back.algorithm).toBe('rsa-3072');
    expect(back.expiresAt).toBe(alice.expiresAt);
    const key = createPrivateKey({ key: back.encryptedPrivateBundle, passphrase: PASSPHRASE });
    expect(cert.checkPrivateKey(key)).toBe(true);

    expect(() => importPkcs12({ p12Bytes, password: 'wrong' })).toThrow();
    expect(() =>
      exportPkcs12({ certPem: alice.certPem, encryptedPrivateBundle: alice.encryptedPrivateBundle, passphrase: 'wrong' }),
    ).toThrow();
    // A certificate is never bundled with somebody else's key.
    expect(() =>
      exportPkcs12({ certPem: mallory.certPem, encryptedPrivateBundle: alice.encryptedPrivateBundle, passphrase: PASSPHRASE }),
    ).toThrow(/not for this private key/);
  }, 60_000);

  it('produces a certification request, and only with the passphrase', () => {
    const { csrPem } = certificateRequest({
      certPem: alice.certPem,
      encryptedPrivateBundle: alice.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
    }) as { csrPem: string };
    expect(csrPem).toMatch(/^-----BEGIN CERTIFICATE REQUEST-----\n/);
    expect(csrPem).not.toContain('PRIVATE KEY');
    expect(() =>
      certificateRequest({ certPem: alice.certPem, encryptedPrivateBundle: alice.encryptedPrivateBundle, passphrase: 'wrong' }),
    ).toThrow();
  }, 60_000);

  it('accepts a certificate for the key and refuses one for another key', () => {
    const enc = new TextEncoder();
    // Precondition: a certificate that IS for this key is accepted.
    const own = attachIssuedCert({
      certBytes: enc.encode(alice.certPem!),
      encryptedPrivateBundle: alice.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
    }) as { fingerprint: string; selfIssued: boolean; addresses: string[] };
    expect(own.fingerprint).toBe(alice.fingerprint);
    expect(own.selfIssued).toBe(true);
    expect(own.addresses).toEqual([EMAIL]);
    // DER as well as PEM.
    const der = attachIssuedCert({
      certBytes: new Uint8Array(cert.raw),
      encryptedPrivateBundle: alice.encryptedPrivateBundle,
      passphrase: PASSPHRASE,
    }) as { fingerprint: string };
    expect(der.fingerprint).toBe(alice.fingerprint);

    expect(() =>
      attachIssuedCert({
        certBytes: enc.encode(mallory.certPem!),
        encryptedPrivateBundle: alice.encryptedPrivateBundle,
        passphrase: PASSPHRASE,
      }),
    ).toThrow(/not for this private key/);
  }, 60_000);

  it('is not exported as an Autocrypt Setup Message', () => {
    expect(() => exportBackup({ encryptedPrivateBundle: alice.encryptedPrivateBundle, kind: 'smime' })).toThrow(
      /OpenPGP key, not a smime key/,
    );
  });
});

describe('generation refuses what it cannot certify', () => {
  it.each([
    ['a non-ASCII address', { userId: 'älice@example.org' }],
    ['no address', { userId: 'Alice' }],
    ['a header-injection attempt', { userId: 'a@example.org\r\nBcc: x@example.org' }],
    ['an empty passphrase', { userId: EMAIL, passphrase: '' }],
    ['a 1024-bit key', { userId: EMAIL, rsaBits: 1024 }],
  ])('%s', (_label, over) => {
    expect(() => generateKey({ kind: 'smime', passphrase: PASSPHRASE, ...over })).toThrow();
  });

  it('an unknown kind', () => {
    expect(() => generateKey({ kind: 'x509', userId: EMAIL, passphrase: PASSPHRASE })).toThrow(/unknown kind/);
  });

  it('and an OpenPGP key is still an OpenPGP key', () => {
    const pgp = generateKey({ kind: 'pgp', userId: EMAIL, passphrase: PASSPHRASE }) as Generated;
    expect(pgp.publicKeyArmored).toContain('BEGIN PGP PUBLIC KEY BLOCK');
    expect(pgp.certPem).toBeUndefined();
    expect(pgp.algorithm).toBeUndefined();
  }, 60_000);
});
