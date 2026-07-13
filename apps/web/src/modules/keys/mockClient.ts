// In-memory mock backend for the keys surface (plan §3 e2 — mock-backed until e8).
// A `Client` whose `jmap` answers `CryptoKey/*` (query/get/set/setTrust/lookup)
// and the `ContactCard/*` calls the module touches (list + per-contact key
// association) from local arrays, so the module and its tests exercise the real
// keys store slice end-to-end without the engine. Shapes mirror `mw-mock-jmap`'s
// frozen crypto fixtures (§2.1). e8 replaces it with the same-origin transport.
//
// Plain module (no test-runner imports) so it typechecks/builds like any source.

import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import type { CryptoKey, KeyTrust } from '../../api/crypto-types.ts';
import { CAP_CRYPTO } from '../../api/crypto-types.ts';
import { CAP_CONTACTS, type ContactCard } from '../../api/pim-types.ts';

export interface KeysSeed {
  keys?: CryptoKey[];
  /** Keys returned by `CryptoKey/lookup`, keyed by the queried address. */
  lookupPool?: Record<string, CryptoKey[]>;
  contacts?: ContactCard[];
}

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_CRYPTO]: 'acct1', [CAP_CONTACTS]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
} as unknown as JmapSession;

/** A frozen own PGP key (matches the `mw-mock-jmap` fixture shape). */
export function ownPgpKey(over: Partial<CryptoKey> = {}): CryptoKey {
  return {
    id: 'key-pgp-1',
    kind: 'pgp',
    isOwn: true,
    addresses: ['me@example.org'],
    fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234',
    keyId: 'ABCD1234ABCD1234',
    algorithm: 'ed25519',
    createdAt: '2026-07-12T09:00:00Z',
    expiresAt: null,
    publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\n(mock)\n-----END PGP PUBLIC KEY BLOCK-----',
    certPem: null,
    trust: 'verified',
    autocrypt: true,
    source: 'generated',
    hasPrivate: true,
    encryptedPrivateBackup: null,
    verifiedAt: '2026-07-12T09:00:00Z',
    keyHistory: [{ fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234', seenAt: '2026-07-12T09:00:00Z' }],
    ...over,
  };
}

/** A public contact key (harvested/looked-up — no private half). */
export function contactPgpKey(over: Partial<CryptoKey> = {}): CryptoKey {
  return ownPgpKey({
    id: 'key-pgp-2',
    isOwn: false,
    addresses: ['alan@example.org'],
    fingerprint: 'EF01EF01EF01EF01EF01EF01EF01EF01EF01EF01',
    keyId: 'EF01EF01EF01EF01',
    trust: 'tofu',
    source: 'harvested',
    hasPrivate: false,
    autocrypt: false,
    ...over,
  });
}

function mkContact(over: Partial<ContactCard>): ContactCard {
  return {
    id: '', addressBookId: 'ab1', uid: '', kind: 'individual',
    name: { full: '', given: '', surname: '', prefix: '', suffix: '' },
    nicknames: [], organizations: [], titles: [], emails: [], phones: [],
    onlineServices: [], addresses: [], anniversaries: [], notes: '',
    photoBlobId: null, isFavorite: false, groupIds: [], pgpKey: null, smimeCert: null, etag: null,
    ...over,
  };
}

export function defaultKeysSeed(): Required<KeysSeed> {
  return {
    keys: [ownPgpKey()],
    lookupPool: {
      'alan@example.org': [contactPgpKey()],
    },
    contacts: [
      mkContact({ id: 'c1', uid: 'c1', name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'ada@example.org', pref: 1 }] }),
      mkContact({ id: 'c2', uid: 'c2', name: { full: 'Alan Turing', given: 'Alan', surname: 'Turing', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'alan@example.org', pref: 0 }] }),
    ],
  };
}

type Call = JmapResponse['methodResponses'][number];

/** Create a stateful mock `Client` seeded with keys + contacts. */
export function makeKeysClient(seed: KeysSeed = {}): Client {
  const keys = [...(seed.keys ?? [])];
  const pool = { ...(seed.lookupPool ?? {}) };
  const cards = [...(seed.contacts ?? [])];
  let counter = 0;

  function handle(name: string, args: Record<string, unknown>, callId: string): Call {
    switch (name) {
      case 'CryptoKey/query':
        return ['CryptoKey/query', { accountId: 'acct1', queryState: 's', ids: keys.map((k) => k.id), position: 0 }, callId];
      case 'CryptoKey/get':
        return ['CryptoKey/get', { accountId: 'acct1', state: 's', list: keys.map((k) => ({ ...k })), notFound: [] }, callId];
      case 'CryptoKey/set': {
        const created: Record<string, { id: string }> = {};
        for (const [key, payload] of Object.entries((args['create'] as Record<string, Record<string, unknown>>) ?? {})) {
          const id = `srv-${(counter += 1)}`;
          keys.push({ ...(payload as unknown as CryptoKey), id });
          created[key] = { id };
        }
        return ['CryptoKey/set', { accountId: 'acct1', oldState: 's', newState: 's2', created, updated: {}, destroyed: [] }, callId];
      }
      case 'CryptoKey/setTrust': {
        const id = args['id'] as string;
        const trust = args['trust'] as KeyTrust;
        const idx = keys.findIndex((k) => k.id === id);
        if (idx >= 0) keys[idx] = { ...keys[idx]!, trust };
        return ['CryptoKey/setTrust', { accountId: 'acct1', updated: { [id]: null } }, callId];
      }
      case 'CryptoKey/lookup': {
        const address = (args['address'] as string) ?? '';
        const found = pool[address] ?? [];
        return ['CryptoKey/lookup', { accountId: 'acct1', list: found.map((k) => ({ ...k })), notFound: found.length > 0 ? [] : [address] }, callId];
      }
      case 'AddressBook/get':
        return ['AddressBook/get', { accountId: 'acct1', state: 's', list: [{ id: 'ab1', name: 'Personal', isDefault: true, carddavUrl: null, syncToken: null }], notFound: [] }, callId];
      case 'ContactGroup/get':
        return ['ContactGroup/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, callId];
      case 'ContactCard/query':
        return ['ContactCard/query', { accountId: 'acct1', queryState: 's', ids: cards.map((c) => c.id), position: 0 }, callId];
      case 'ContactCard/get':
        return ['ContactCard/get', { accountId: 'acct1', state: 's', list: cards.map((c) => ({ ...c })), notFound: [] }, callId];
      case 'ContactCard/set': {
        for (const [id, patch] of Object.entries((args['update'] as Record<string, Record<string, unknown>>) ?? {})) {
          const idx = cards.findIndex((c) => c.id === id);
          if (idx >= 0) cards[idx] = { ...cards[idx]!, ...(patch as Partial<ContactCard>) };
        }
        return ['ContactCard/set', { accountId: 'acct1', oldState: 's', newState: 's2', created: {}, updated: {}, destroyed: [], notCreated: null, notUpdated: null, notDestroyed: null }, callId];
      }
      default:
        return [name, {}, callId];
    }
  }

  return {
    login: async () => ({ username: 'me@example.org', accountId: 'acct1' }),
    logout: async () => undefined,
    me: async () => ({ username: 'me@example.org', accountId: 'acct1' }),
    session: async () => SESSION,
    jmap: async (body: JmapRequest): Promise<JmapResponse> =>
      ({
        methodResponses: body.methodCalls.map((c) => handle(c[0], c[1] as Record<string, unknown>, c[2])),
        sessionState: 's',
      }) as unknown as JmapResponse,
    sanitize: async (h: string) => h,
    onNetwork: () => () => undefined,
  };
}

/** Expose the seeded card list for assertions (association writes land here). */
export function cardsFrom(client: Client): Promise<ContactCard[]> {
  return client
    .jmap({ using: [], methodCalls: [['ContactCard/get', {}, 'g']] })
    .then((r) => (r.methodResponses[0]![1] as { list: ContactCard[] }).list);
}
