// In-memory mock backend for the contacts surface (plan §3 e7 — mock-backed
// until e10). A `Client` whose `jmap` handles `AddressBook/*` / `ContactCard/*` /
// `ContactGroup/*` against local arrays, so the module (and its tests) exercise
// the real slice wiring end-to-end without the engine. Not shipped on the
// critical path; e10 replaces it with the same-origin `createClient()` transport.
//
// Plain module (no test-runner imports) so it typechecks/builds like any source;
// the `.test.*` files layer assertions on top.

import type { Client, Me } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_CONTACTS, type AddressBook, type ContactCard, type ContactGroup } from '../../api/pim-types.ts';

export interface ContactsSeed {
  addressBooks?: AddressBook[];
  contacts?: ContactCard[];
  groups?: ContactGroup[];
}

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_CONTACTS]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

/** A default seed: one address book, two individuals, one group. */
export function defaultSeed(): Required<ContactsSeed> {
  const mk = (over: Partial<ContactCard>): ContactCard => ({
    id: '', addressBookId: 'ab1', uid: '', kind: 'individual',
    name: { full: '', given: '', surname: '', prefix: '', suffix: '' },
    nicknames: [], organizations: [], titles: [], emails: [], phones: [],
    onlineServices: [], addresses: [], anniversaries: [], notes: '',
    photoBlobId: null, isFavorite: false, groupIds: [], pgpKey: null, smimeCert: null, etag: null,
    ...over,
  });
  return {
    addressBooks: [{ id: 'ab1', name: 'Personal', isDefault: true, carddavUrl: null, syncToken: null }],
    contacts: [
      mk({ id: 'c1', uid: 'c1', name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'ada@example.org', pref: 1 }], isFavorite: true }),
      mk({ id: 'c2', uid: 'c2', name: { full: 'Alan Turing', given: 'Alan', surname: 'Turing', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'alan@example.org', pref: 0 }], organizations: ['NPL'] }),
    ],
    groups: [{ id: 'g1', addressBookId: 'ab1', name: 'Colleagues', memberIds: ['c1'] }],
  };
}

/** Create a stateful mock `Client` seeded with contacts data. */
export function makeContactsClient(seed: ContactsSeed = {}): Client {
  const books = [...(seed.addressBooks ?? [])];
  const cards = [...(seed.contacts ?? [])];
  const groups = [...(seed.groups ?? [])];
  let counter = 0;
  const gen = (p: string): string => `${p}-${(counter += 1)}`;

  function fullCard(payload: Record<string, unknown>, id: string): ContactCard {
    const base = defaultSeed().contacts[0]!; // shape template
    return { ...base, ...(payload as Partial<ContactCard>), id, addressBookId: (payload['addressBookId'] as string) ?? books[0]?.id ?? 'default' };
  }

  function handleCall(name: string, args: Record<string, unknown>, callId: string): JmapResponse['methodResponses'][number] {
    switch (name) {
      // Note: every `get`/`query` returns *fresh copies* — a real server hands
      // back new JSON, never its live rows, so later mutations here never leak
      // into state the slice is already holding.
      case 'AddressBook/get':
        return ['AddressBook/get', { accountId: 'acct1', state: 's', list: books.map((b) => ({ ...b })), notFound: [] }, callId];
      case 'ContactGroup/get':
        return ['ContactGroup/get', { accountId: 'acct1', state: 's', list: groups.map((g) => ({ ...g, memberIds: [...g.memberIds] })), notFound: [] }, callId];
      case 'ContactCard/query':
        return ['ContactCard/query', { accountId: 'acct1', queryState: 's', ids: cards.map((c) => c.id), position: 0 }, callId];
      case 'ContactCard/get':
        return ['ContactCard/get', { accountId: 'acct1', state: 's', list: cards.map((c) => ({ ...c })), notFound: [] }, callId];
      case 'ContactCard/set': {
        const created: Record<string, { id: string }> = {};
        for (const [key, payload] of Object.entries((args['create'] as Record<string, Record<string, unknown>>) ?? {})) {
          const id = gen('srv');
          cards.push(fullCard(payload, id));
          created[key] = { id };
        }
        for (const [id, patch] of Object.entries((args['update'] as Record<string, Record<string, unknown>>) ?? {})) {
          const idx = cards.findIndex((c) => c.id === id);
          if (idx >= 0) cards[idx] = { ...cards[idx]!, ...(patch as Partial<ContactCard>) };
        }
        for (const id of (args['destroy'] as string[]) ?? []) {
          const idx = cards.findIndex((c) => c.id === id);
          if (idx >= 0) cards.splice(idx, 1);
        }
        return ['ContactCard/set', { accountId: 'acct1', oldState: 's', newState: 's2', created, updated: {}, destroyed: (args['destroy'] as string[]) ?? [], notCreated: null, notUpdated: null, notDestroyed: null }, callId];
      }
      case 'ContactGroup/set': {
        const created: Record<string, { id: string }> = {};
        for (const [key, payload] of Object.entries((args['create'] as Record<string, Record<string, unknown>>) ?? {})) {
          const id = gen('grp');
          groups.push({ id, addressBookId: (payload['addressBookId'] as string) ?? books[0]?.id ?? 'default', name: (payload['name'] as string) ?? '', memberIds: (payload['memberIds'] as string[]) ?? [] });
          created[key] = { id };
        }
        for (const [id, patch] of Object.entries((args['update'] as Record<string, Record<string, unknown>>) ?? {})) {
          const idx = groups.findIndex((g) => g.id === id);
          if (idx >= 0) groups[idx] = { ...groups[idx]!, ...(patch as Partial<ContactGroup>) };
        }
        for (const id of (args['destroy'] as string[]) ?? []) {
          const idx = groups.findIndex((g) => g.id === id);
          if (idx >= 0) groups.splice(idx, 1);
        }
        return ['ContactGroup/set', { accountId: 'acct1', created, updated: {}, destroyed: (args['destroy'] as string[]) ?? [], notCreated: null }, callId];
      }
      case 'ContactCard/merge': {
        // The slice merges client-side; the mock just tombstones the sources.
        const mergeIds = (args['mergeIds'] as string[]) ?? [];
        for (const id of mergeIds) {
          const idx = cards.findIndex((c) => c.id === id);
          if (idx >= 0) cards.splice(idx, 1);
        }
        return ['ContactCard/merge', { accountId: 'acct1', destroyed: mergeIds }, callId];
      }
      default:
        return [name, {}, callId];
    }
  }

  return {
    login: async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' }),
    logout: async () => undefined,
    me: async (): Promise<Me> => ({ username: 'me@example.org', accountId: 'acct1' }),
    session: async () => SESSION,
    jmap: async (body: JmapRequest): Promise<JmapResponse> => ({
      methodResponses: body.methodCalls.map((c) => handleCall(c[0], c[1] as Record<string, unknown>, c[2])),
      sessionState: 's',
    }),
    sanitize: async (h: string) => h,
    onNetwork: () => () => undefined,
  };
}
