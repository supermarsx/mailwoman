import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createKeysSlice, type KeysSlice, type OwnKeyDraft } from './keys.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { CAP_CRYPTO } from '../../api/crypto-types.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { __resetCryptoWorker, getCryptoWorker } from '../../crypto/index.ts';

// Under vitest `getCryptoWorker()` is the stub worker (`crypto/index.ts`): it
// returns fixed placeholder key material, so these cases pin what the SLICE does
// with a generated or imported key, not any cryptography. That the worker's
// S/MIME output really is an X.509 certificate is `crypto/wasm-smime.test.ts`
// (the committed wasm, judged by Node's own parser) and
// `crates/mw-crypto/tests/smime_generate.rs` (judged by openssl).

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_CRYPTO]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/jmap/api',
  downloadUrl: '/d',
  uploadUrl: '/u',
  eventSourceUrl: '/e',
  state: 's0',
};

interface FakeServer {
  client: Client;
  jmap: ReturnType<typeof vi.fn>;
  /** Every `CryptoKey/set` create it was sent, in order. */
  created: Array<Record<string, unknown>>;
  /** Every id a `CryptoKey/set` destroy named, in order. */
  destroyed: string[];
}

/** A fake JMAP client that records what `CryptoKey/set` is asked to create and destroy. */
function makeClient(): FakeServer {
  const created: Array<Record<string, unknown>> = [];
  const destroyed: string[] = [];
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, args, callId] = body.methodCalls[0]!;
    if (name !== 'CryptoKey/set') throw new Error(`unexpected method ${name}`);
    const a = args as { create?: Record<string, Record<string, unknown>>; destroy?: string[] };
    const made: Record<string, { id: string }> = {};
    if (a.create?.['new'] !== undefined) {
      created.push(a.create['new']);
      made['new'] = { id: `srv-${created.length}` };
    }
    destroyed.push(...(a.destroy ?? []));
    return {
      methodResponses: [['CryptoKey/set', { accountId: 'acct1', created: made, destroyed: a.destroy ?? [] }, callId]],
      sessionState: 's',
    };
  });
  const client = { session: vi.fn(async () => SESSION), jmap } as unknown as Client;
  return { client, jmap, created, destroyed };
}

function makeSlice(client: Client): { slice: KeysSlice; toasts: Array<[string, string]> } {
  const toasts: Array<[string, string]> = [];
  const ctx: SliceContext = { client, showToast: (kind, message) => void toasts.push([kind, message]) };
  return { slice: createRoot(() => createKeysSlice(ctx)), toasts };
}

const PGP_ARMOR = '-----BEGIN PGP PUBLIC KEY BLOCK-----\nx\n-----END PGP PUBLIC KEY BLOCK-----';

beforeEach(() => {
  // A fresh stub per test, so a spy on one test's worker cannot leak into the next.
  __resetCryptoWorker();
  vi.restoreAllMocks();
});

describe('keys slice — generateOwnKey', () => {
  it('files a generated key as an OpenPGP key: armor in publicKeyArmored, no certificate', async () => {
    const { client, created } = makeClient();
    const { slice, toasts } = makeSlice(client);

    const key = await slice.generateOwnKey({ kind: 'pgp', userId: 'Me <me@example.org>', passphrase: 'pw' });

    expect(key).toMatchObject({
      id: 'srv-1',
      kind: 'pgp',
      isOwn: true,
      addresses: ['me@example.org'],
      algorithm: 'ed25519',
      certPem: null,
      autocrypt: true,
      source: 'generated',
      hasPrivate: true,
    });
    expect(key.publicKeyArmored).toContain('BEGIN PGP PUBLIC KEY BLOCK');
    // The row sent to the server says the same thing.
    expect(created).toHaveLength(1);
    expect(created[0]).toMatchObject({ kind: 'pgp', algorithm: 'ed25519', certPem: null });
    expect(created[0]!['publicKeyArmored']).toBe(key.publicKeyArmored);
    expect(slice.ownKeys().map((k) => k.id)).toEqual(['srv-1']);
    expect(slice.hasVaultedKey(key.fingerprint)).toBe(true);
    expect(toasts).toEqual([['success', 'PGP key generated']]);
  });

  it('files a generated S/MIME key as a certificate, described by what the worker read from it', async () => {
    const { client, created } = makeClient();
    const worker = getCryptoWorker();
    const generate = vi.spyOn(worker, 'generateKey');
    const { slice, toasts } = makeSlice(client);

    const key = await slice.generateOwnKey({ kind: 'smime', userId: 'Me <me@example.org>', passphrase: 'pw' });

    // The worker was asked for an S/MIME key — not for a PGP key to relabel.
    expect(generate).toHaveBeenCalledWith({ kind: 'smime', userId: 'Me <me@example.org>', passphrase: 'pw' });
    const made = await generate.mock.results[0]!.value;
    expect(key).toMatchObject({
      id: 'srv-1',
      kind: 'smime',
      isOwn: true,
      publicKeyArmored: null,
      autocrypt: false,
      source: 'generated',
      hasPrivate: true,
      // Read from the worker's answer, not assumed: the stub's address differs
      // from the one typed, and its algorithm is not a constant in the slice.
      addresses: made.addresses,
      algorithm: made.algorithm,
      expiresAt: made.expiresAt,
      certPem: made.certPem,
      fingerprint: made.fingerprint,
    });
    expect(key.addresses).toEqual(['stub@example.invalid']);
    expect(key.algorithm).toBe('rsa-3072');
    expect(key.certPem).toContain('BEGIN CERTIFICATE');
    expect(key.certPem).not.toContain('PGP');
    expect(created).toHaveLength(1);
    expect(created[0]).toMatchObject({ kind: 'smime', algorithm: 'rsa-3072', publicKeyArmored: null, source: 'generated' });
    expect(created[0]!['certPem']).toBe(made.certPem);
    expect(slice.hasVaultedKey(key.fingerprint)).toBe(true);
    expect(toasts).toEqual([['success', 'S/MIME key and self-signed certificate generated']]);
  });

  // Audit §13 row 35: the old path generated an OpenPGP key, labelled it
  // `ecdsa-p256` and put the PGP armor in `certPem`. Whatever a worker answers,
  // one kind is never filed as the other, and nothing is stored.
  it.each([
    ['PGP armor instead of a certificate', { publicKeyArmored: PGP_ARMOR }],
    ['PGP armor in the certificate field', { certPem: PGP_ARMOR, addresses: ['a@b.example'], algorithm: 'rsa-3072', expiresAt: 'x' }],
    [
      'a certificate together with PGP armor',
      { certPem: '-----BEGIN CERTIFICATE-----\nx', publicKeyArmored: PGP_ARMOR, addresses: ['a@b.example'], algorithm: 'rsa-3072', expiresAt: 'x' },
    ],
    ['a certificate with no algorithm read from it', { certPem: '-----BEGIN CERTIFICATE-----\nx', addresses: ['a@b.example'], expiresAt: 'x' }],
  ])('refuses an S/MIME result that is %s', async (_label, answer) => {
    const { client, jmap } = makeClient();
    vi.spyOn(getCryptoWorker(), 'generateKey').mockResolvedValue({
      fingerprint: 'F'.repeat(64),
      keyId: 'F'.repeat(16),
      encryptedPrivateBundle: 'bundle',
      ...answer,
    });
    const { slice, toasts } = makeSlice(client);

    await expect(slice.generateOwnKey({ kind: 'smime', userId: 'me@example.org', passphrase: 'pw' })).rejects.toThrow(
      'did not return an S/MIME certificate',
    );

    expect(jmap).not.toHaveBeenCalled();
    expect(slice.keys()).toEqual([]);
    expect(slice.hasVaultedKey('F'.repeat(64))).toBe(false);
    expect(toasts).toEqual([]);
  });

  it('refuses an OpenPGP result that carries a certificate', async () => {
    const { client, jmap } = makeClient();
    vi.spyOn(getCryptoWorker(), 'generateKey').mockResolvedValue({
      fingerprint: 'F'.repeat(40),
      keyId: 'F'.repeat(16),
      encryptedPrivateBundle: 'bundle',
      publicKeyArmored: PGP_ARMOR,
      certPem: '-----BEGIN CERTIFICATE-----\nx',
    });
    const { slice } = makeSlice(client);

    await expect(slice.generateOwnKey({ kind: 'pgp', userId: 'me@example.org', passphrase: 'pw' })).rejects.toThrow(
      'did not return an OpenPGP key',
    );
    expect(jmap).not.toHaveBeenCalled();
    expect(slice.keys()).toEqual([]);
  });

  it('refuses a kind it does not generate before the worker is reached', async () => {
    const { client, jmap } = makeClient();
    const generate = vi.spyOn(getCryptoWorker(), 'generateKey');
    const { slice, toasts } = makeSlice(client);
    const forced = { kind: 'x509', userId: 'me@example.org', passphrase: 'pw' } as unknown as OwnKeyDraft;

    await expect(slice.generateOwnKey(forced)).rejects.toThrow('unknown key kind');

    expect(generate).not.toHaveBeenCalled();
    expect(jmap).not.toHaveBeenCalled();
    expect(slice.keys()).toEqual([]);
    expect(toasts).toEqual([]);
  });
});

describe('keys slice — own S/MIME key: export, request, issued certificate', () => {
  async function withSmimeKey(): Promise<FakeServer & { slice: KeysSlice; toasts: Array<[string, string]>; id: string }> {
    const server = makeClient();
    const { slice, toasts } = makeSlice(server.client);
    const key = await slice.generateOwnKey({ kind: 'smime', userId: 'me@example.org', passphrase: 'pw' });
    toasts.length = 0;
    return { ...server, slice, toasts, id: key.id };
  }

  it('exports the certificate and wrapped key as PKCS#12 bytes', async () => {
    const { slice, id } = await withSmimeKey();
    const exportP12 = vi.spyOn(getCryptoWorker(), 'exportPkcs12');
    const key = slice.keys().find((k) => k.id === id)!;

    const bytes = await slice.exportSmimePkcs12(id, 'pw');

    expect(exportP12).toHaveBeenCalledWith({
      certPem: key.certPem,
      encryptedPrivateBundle: key.encryptedPrivateBackup,
      passphrase: 'pw',
    });
    expect(new TextDecoder().decode(bytes)).toBe('STUB PKCS#12');
  });

  it('asks the worker for a certification request over the same key', async () => {
    const { slice, id } = await withSmimeKey();
    const request = vi.spyOn(getCryptoWorker(), 'certificateRequest');
    const key = slice.keys().find((k) => k.id === id)!;

    const csr = await slice.smimeCertificateRequest(id, 'pw');

    expect(request).toHaveBeenCalledWith({
      certPem: key.certPem,
      encryptedPrivateBundle: key.encryptedPrivateBackup,
      passphrase: 'pw',
    });
    expect(csr).toContain('BEGIN CERTIFICATE REQUEST');
  });

  it('does none of these for an OpenPGP key', async () => {
    const { client } = makeClient();
    const { slice } = makeSlice(client);
    const pgp = await slice.generateOwnKey({ kind: 'pgp', userId: 'me@example.org', passphrase: 'pw' });
    const exportP12 = vi.spyOn(getCryptoWorker(), 'exportPkcs12');

    await expect(slice.exportSmimePkcs12(pgp.id, 'pw')).rejects.toThrow('not an own S/MIME key');
    await expect(slice.smimeCertificateRequest(pgp.id, 'pw')).rejects.toThrow('not an own S/MIME key');
    await expect(slice.attachIssuedCertificate(pgp.id, new Uint8Array([1]), 'pw')).rejects.toThrow('not an own S/MIME key');
    expect(exportP12).not.toHaveBeenCalled();
  });

  it('replaces the certificate with an issued one: new row stored, old row destroyed', async () => {
    const { slice, toasts, created, destroyed, id } = await withSmimeKey();
    const old = slice.keys().find((k) => k.id === id)!;

    const stored = await slice.attachIssuedCertificate(id, new Uint8Array([0x30]), 'pw');

    expect(stored).toMatchObject({
      id: 'srv-2',
      kind: 'smime',
      isOwn: true,
      hasPrivate: true,
      source: 'imported',
      fingerprint: 'STUBISSUEDFINGERPRINT0000000000000000000000000000000000000000000',
      // The same wrapped private key, under the new certificate.
      encryptedPrivateBackup: old.encryptedPrivateBackup,
    });
    expect(stored.certPem).toContain('STUB ISSUED');
    expect(stored.keyHistory.map((h) => h.fingerprint)).toEqual([old.fingerprint, stored.fingerprint]);
    expect(created).toHaveLength(2);
    expect(created[1]!['certPem']).toBe(stored.certPem);
    expect(destroyed).toEqual([id]);
    expect(slice.keys().map((k) => k.id)).toEqual(['srv-2']);
    expect(slice.hasVaultedKey(stored.fingerprint)).toBe(true);
    expect(slice.hasVaultedKey(old.fingerprint)).toBe(false);
    expect(toasts).toEqual([['success', 'Certificate replaced']]);
  });

  it('keeps the held certificate when the worker refuses the issued one', async () => {
    const { slice, toasts, created, destroyed, id } = await withSmimeKey();
    vi.spyOn(getCryptoWorker(), 'attachIssuedCert').mockRejectedValue(
      new Error('the certificate is not for this private key'),
    );
    const before = slice.keys();

    await expect(slice.attachIssuedCertificate(id, new Uint8Array([0x30]), 'pw')).rejects.toThrow(
      'not for this private key',
    );

    expect(slice.keys()).toEqual(before);
    expect(created).toHaveLength(1);
    expect(destroyed).toEqual([]);
    expect(toasts).toEqual([]);
  });
});

describe('keys slice — S/MIME certificate import', () => {
  it('does not store a second row when the imported key is one already held', async () => {
    const { client, created } = makeClient();
    const { slice, toasts } = makeSlice(client);
    const generated = await slice.generateOwnKey({ kind: 'smime', userId: 'me@example.org', passphrase: 'pw' });
    toasts.length = 0;
    // The PKCS#12 that key was exported as: same certificate, so same fingerprint.
    vi.spyOn(getCryptoWorker(), 'importPkcs12').mockResolvedValue({
      certPem: generated.certPem!,
      fingerprint: generated.fingerprint,
      encryptedPrivateBundle: 'rewrapped-by-import',
      addresses: generated.addresses,
      algorithm: generated.algorithm,
      expiresAt: generated.expiresAt!,
    });

    const preview = await slice.previewPkcs12Key(new Uint8Array([1]), 'pw');
    const stored = await slice.commitImport(preview);

    expect(stored).toBe(slice.keys()[0]);
    expect(stored).toMatchObject({ id: generated.id, source: 'generated' });
    expect(created).toHaveLength(1);
    expect(slice.keys()).toHaveLength(1);
    expect(toasts).toEqual([['info', 'This key is already in your keys; nothing was added']]);
  });

  it('previews a PKCS#12 import as an S/MIME key holding the certificate, and commits it', async () => {
    const { client, created } = makeClient();
    const { slice, toasts } = makeSlice(client);

    const preview = await slice.previewPkcs12Key(new Uint8Array([1, 2, 3]), 'p12-password');
    expect(preview.key).toMatchObject({
      kind: 'smime',
      source: 'pkcs12',
      publicKeyArmored: null,
      autocrypt: false,
      // What the worker read from the certificate.
      addresses: ['stub@example.invalid'],
      algorithm: 'rsa-2048',
    });
    expect(preview.key.expiresAt).not.toBeNull();
    expect(preview.key.certPem).toContain('BEGIN CERTIFICATE');
    expect(preview.encryptedPrivateBundle).not.toBeNull();
    // A preview persists nothing.
    expect(created).toEqual([]);
    expect(slice.keys()).toEqual([]);

    const stored = await slice.commitImport(preview);

    expect(stored).toMatchObject({ id: 'srv-1', kind: 'smime', source: 'pkcs12', isOwn: true, hasPrivate: true });
    expect(stored.certPem).toBe(preview.key.certPem);
    expect(created[0]).toMatchObject({ kind: 'smime', source: 'pkcs12', publicKeyArmored: null });
    expect(created[0]!['certPem']).toBe(preview.key.certPem);
    expect(slice.ownKeys().map((k) => k.kind)).toEqual(['smime']);
    expect(slice.hasVaultedKey(stored.fingerprint)).toBe(true);
    expect(toasts).toEqual([['success', 'Key imported']]);
  });
});
