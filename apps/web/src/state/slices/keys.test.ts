import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createKeysSlice, type KeysSlice, type OwnKeyDraft } from './keys.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { CAP_CRYPTO } from '../../api/crypto-types.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';

// Under vitest `getCryptoWorker()` is the stub worker (`crypto/index.ts`): it
// returns fixed placeholder key material, so these cases pin what the SLICE does
// with a generated or imported key, not any cryptography.

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

/** A fake JMAP client that records every `CryptoKey/set` create it is sent. */
function makeClient(): { client: Client; jmap: ReturnType<typeof vi.fn>; created: Array<Record<string, unknown>> } {
  const created: Array<Record<string, unknown>> = [];
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, args, callId] = body.methodCalls[0]!;
    if (name !== 'CryptoKey/set') throw new Error(`unexpected method ${name}`);
    const create = (args as { create: Record<string, Record<string, unknown>> }).create;
    created.push(create['new']!);
    return {
      methodResponses: [['CryptoKey/set', { accountId: 'acct1', created: { new: { id: `srv-${created.length}` } } }, callId]],
      sessionState: 's',
    };
  });
  const client = { session: vi.fn(async () => SESSION), jmap } as unknown as Client;
  return { client, jmap, created };
}

function makeSlice(client: Client): { slice: KeysSlice; toasts: Array<[string, string]> } {
  const toasts: Array<[string, string]> = [];
  const ctx: SliceContext = { client, showToast: (kind, message) => void toasts.push([kind, message]) };
  return { slice: createRoot(() => createKeysSlice(ctx)), toasts };
}

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

  // The path that used to generate an OpenPGP key, label it `ecdsa-p256` and put
  // the PGP armor in `certPem` (audit §13 row 35). The type no longer admits it;
  // a caller that forces it through is refused before anything is stored.
  it('refuses to generate an S/MIME key, and stores nothing', async () => {
    const { client, jmap } = makeClient();
    const { slice, toasts } = makeSlice(client);
    const forced = { kind: 'smime', userId: 'me@example.org', passphrase: 'pw' } as unknown as OwnKeyDraft;

    await expect(slice.generateOwnKey(forced)).rejects.toThrow('only OpenPGP keys can be generated');

    expect(jmap).not.toHaveBeenCalled();
    expect(slice.keys()).toEqual([]);
    expect(slice.hasVaultedKey('STUBFINGERPRINT0000000000000000000000000')).toBe(false);
    expect(toasts).toEqual([]);
  });
});

describe('keys slice — S/MIME certificate import', () => {
  it('previews a PKCS#12 import as an S/MIME key holding the certificate, and commits it', async () => {
    const { client, created } = makeClient();
    const { slice, toasts } = makeSlice(client);

    const preview = await slice.previewPkcs12Key(new Uint8Array([1, 2, 3]), 'p12-password');
    expect(preview.key).toMatchObject({ kind: 'smime', source: 'pkcs12', publicKeyArmored: null, autocrypt: false });
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
