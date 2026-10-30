// t87-e4 (D11): the keys slice keeps a vault per mailbox and drops it on a
// switch. Loading the parent's keys and then switching to a linked child must
// leave nothing of the parent's behind: no loaded key, no vaulted bundle, no
// cached account id, no unlocked worker key.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createKeysSlice, type KeysSlice } from './keys.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { CAP_CRYPTO, type CryptoKey } from '../../api/crypto-types.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { __resetCryptoWorker, getCryptoWorker } from '../../crypto/index.ts';
import { installLinkedMailbox } from '../../api/linkedMailboxSession.ts';
import type { LinkedAccountContext } from '../../api/linkedAccountContext.ts';

function session(account: string): JmapSession {
  return {
    capabilities: {},
    accounts: { [account]: { name: account, isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
    primaryAccounts: { [CAP_CRYPTO]: account },
    username: `${account}@example.org`,
    apiUrl: '/jmap/api',
    downloadUrl: '/d',
    uploadUrl: '/u',
    eventSourceUrl: '/e',
    state: 's0',
  };
}

function ownKey(account: string, fingerprint: string): CryptoKey {
  return {
    id: `k-${account}`,
    kind: 'pgp',
    isOwn: true,
    addresses: [`${account}@example.org`],
    fingerprint,
    keyId: fingerprint.slice(0, 16),
    algorithm: 'ed25519',
    createdAt: '2026-01-01T00:00:00Z',
    expiresAt: null,
    publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nx\n-----END PGP PUBLIC KEY BLOCK-----',
    certPem: null,
    trust: 'verified',
    autocrypt: true,
    source: 'generated',
    hasPrivate: true,
    encryptedPrivateBackup: `BUNDLE-${account}`,
    verifiedAt: null,
    keyHistory: [],
  };
}

const PARENT_FP = 'PARENTFINGERPRINT0000000000000000000000';
const CHILD_FP = 'CHILDFINGERPRINT00000000000000000000000';

/** A server whose answers follow the open mailbox, like the real routed client. */
function makeServer(): { client: Client; queried: string[]; setOpen(account: string): void } {
  let open = 'parent';
  const queried: string[] = [];
  const rows: Record<string, CryptoKey[]> = {
    parent: [ownKey('parent', PARENT_FP)],
    child: [ownKey('child', CHILD_FP)],
  };
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, args] = body.methodCalls[0]!;
    if (name !== 'CryptoKey/query') throw new Error(`unexpected ${name}`);
    const acct = (args as { accountId: string }).accountId;
    queried.push(acct);
    // The routed server answers for the open mailbox only.
    const list = acct === open ? rows[acct]!.map((k) => ({ ...k })) : [];
    return {
      methodResponses: [
        ['CryptoKey/query', { accountId: acct, ids: list.map((k) => k.id) }, 'q'],
        ['CryptoKey/get', { accountId: acct, state: 's', list, notFound: [] }, 'g'],
      ],
      sessionState: 's',
    };
  });
  const client = { session: vi.fn(async () => session(open)), jmap } as unknown as Client;
  return { client, queried, setOpen: (a) => void (open = a) };
}

function makeSlice(client: Client): { slice: KeysSlice; dispose: () => void } {
  const ctx: SliceContext = { client, showToast: () => undefined };
  let dispose = (): void => undefined;
  const slice = createRoot((stop) => {
    dispose = stop;
    return createKeysSlice(ctx);
  });
  return { slice, dispose };
}

/** What `switchLinkedMailbox` does to the preference account. */
function switchTo(server: { setOpen(account: string): void }, account: string): void {
  suspendAccountPreferences();
  server.setOpen(account);
  activateAccountPreferences(account);
}

beforeEach(() => {
  __resetCryptoWorker();
  vi.restoreAllMocks();
  activateAccountPreferences('parent');
});
afterEach(() => suspendAccountPreferences());

describe('keys slice — a vault per mailbox (t87 D11)', () => {
  it('parent keys loaded, then a switch to the child: nothing of the parent is left, and signing uses no parent key', async () => {
    const server = makeServer();
    const { slice, dispose } = makeSlice(server.client);
    const lock = vi.spyOn(getCryptoWorker(), 'lockKey');
    const unlock = vi.spyOn(getCryptoWorker(), 'unlockKey');

    await slice.loadKeys();
    expect(slice.keysAccountId()).toBe('parent');
    const parentKey = slice.ownKeys()[0]!;
    expect(parentKey.fingerprint).toBe(PARENT_FP);
    const { keyRef: parentRef } = await slice.unlockOwnKey(parentKey, 'parent-pass');
    const epochBefore = slice.keysEpoch();

    switchTo(server, 'child');

    // Locked and empty for the child, before anything is loaded.
    expect(slice.keysAccountId()).toBeNull();
    expect(slice.ownKeys()).toEqual([]);
    expect(slice.keys()).toEqual([]);
    expect(slice.hasVaultedKey(PARENT_FP)).toBe(false);
    expect(slice.keysEpoch()).toBeGreaterThan(epochBefore);
    // The parent's unlocked key was locked in the worker.
    expect(lock).toHaveBeenCalledWith({ keyRef: parentRef });
    // A parent key object kept from before the switch is no longer usable.
    expect(slice.keyOwnerAccount(parentKey)).toBeNull();
    await expect(slice.unlockOwnKey(parentKey, 'parent-pass')).rejects.toThrow();

    // The child's own load asks for the child's account and gets only its keys.
    await slice.loadKeys();
    expect(server.queried.at(-1)).toBe('child');
    expect(slice.keysAccountId()).toBe('child');
    expect(slice.ownKeys().map((k) => k.fingerprint)).toEqual([CHILD_FP]);
    // Signing in the child unlocks the child's bundle, with the child's passphrase.
    unlock.mockClear();
    await slice.unlockOwnKey(slice.ownKeys()[0]!, 'child-pass');
    expect(unlock).toHaveBeenCalledTimes(1);
    expect(unlock.mock.calls[0]![0]).toEqual({ encryptedPrivateBundle: 'BUNDLE-child', passphrase: 'child-pass' });
    expect(unlock.mock.calls.some(([req]) => req.encryptedPrivateBundle === 'BUNDLE-parent')).toBe(false);
    dispose();
  });

  it('the reverse direction: child keys loaded, switch back to the parent drops them', async () => {
    const server = makeServer();
    switchTo(server, 'child');
    const { slice, dispose } = makeSlice(server.client);
    await slice.loadKeys();
    expect(slice.ownKeys().map((k) => k.fingerprint)).toEqual([CHILD_FP]);

    switchTo(server, 'parent');
    expect(slice.ownKeys()).toEqual([]);
    expect(slice.keysAccountId()).toBeNull();
    await slice.loadKeys();
    expect(slice.ownKeys().map((k) => k.fingerprint)).toEqual([PARENT_FP]);
    dispose();
  });

  it('the mailbox-change hook alone (installLinkedMailbox) drops the keys and locks the worker key', async () => {
    const server = makeServer();
    const { slice, dispose } = makeSlice(server.client);
    const lock = vi.spyOn(getCryptoWorker(), 'lockKey');
    await slice.loadKeys();
    const { keyRef } = await slice.unlockOwnKey(slice.ownKeys()[0]!, 'parent-pass');
    server.setOpen('child');
    installLinkedMailbox({ linkId: 'link-1', accountId: 'child' } as unknown as LinkedAccountContext);
    expect(slice.ownKeys()).toEqual([]);
    expect(slice.keysAccountId()).toBeNull();
    expect(lock).toHaveBeenCalledWith({ keyRef });
    installLinkedMailbox(null);
    dispose();
  });

  it('refuses a passphrase that does not open the key (another mailbox passphrase), unlocking nothing', async () => {
    const server = makeServer();
    const { slice, dispose } = makeSlice(server.client);
    await slice.loadKeys();
    vi.spyOn(getCryptoWorker(), 'sign').mockRejectedValueOnce(new Error('bad passphrase'));
    const unlock = vi.spyOn(getCryptoWorker(), 'unlockKey');
    await expect(slice.unlockOwnKey(slice.ownKeys()[0]!, 'other-mailbox-pass')).rejects.toThrow('does not unlock');
    expect(unlock).not.toHaveBeenCalled();
    dispose();
  });

  it('a load the switch overtook does not install the old mailbox keys', async () => {
    const server = makeServer();
    const { slice, dispose } = makeSlice(server.client);
    const pending = slice.loadKeys();
    switchTo(server, 'child');
    await pending;
    expect(slice.ownKeys()).toEqual([]);
    expect(slice.keysAccountId()).toBeNull();
    dispose();
  });

  it('a generated key lands in the vault of its mailbox only', async () => {
    const server = makeServer();
    const { slice, dispose } = makeSlice(server.client);
    await slice.loadKeys();
    vi.spyOn(server.client, 'jmap').mockImplementation(async (body: JmapRequest) => ({
      methodResponses: [['CryptoKey/set', { accountId: 'parent', created: { new: { id: 'srv-1' } } }, body.methodCalls[0]![2]]],
      sessionState: 's',
    }));
    const made = await slice.generateOwnKey({ kind: 'pgp', userId: 'P <parent@example.org>', passphrase: 'pw' });
    expect(slice.hasVaultedKey(made.fingerprint)).toBe(true);
    switchTo(server, 'child');
    expect(slice.hasVaultedKey(made.fingerprint)).toBe(false);
    await expect(slice.exportKeyBackup(made.fingerprint)).rejects.toThrow();
    dispose();
  });
});
