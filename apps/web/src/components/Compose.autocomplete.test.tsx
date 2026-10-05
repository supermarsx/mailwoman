import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { Compose } from './Compose.tsx';
import type { Client } from '../api/client.ts';
import { CAP_CONTACTS, type ContactCard } from '../api/pim-types.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../api/jmap-types.ts';

type Invocation = JmapResponse['methodResponses'][number];

function card(id: string, given: string, surname: string, email: string, fav: boolean): ContactCard {
  return {
    id,
    addressBookId: 'default',
    uid: id,
    kind: 'individual',
    name: { full: `${given} ${surname}`, given, surname, prefix: '', suffix: '' },
    nicknames: [],
    organizations: [],
    titles: [],
    emails: [{ context: 'work', value: email, pref: 1 }],
    phones: [],
    onlineServices: [],
    addresses: [],
    anniversaries: [],
    notes: '',
    photoBlobId: null,
    isFavorite: fav,
    groupIds: [],
    pgpKey: null,
    smimeCert: null,
    etag: null,
  } as unknown as ContactCard;
}

const CARDS = [
  card('c1', 'Alice', 'Adams', 'alice@example.org', true),
  card('c2', 'Bob', 'Brown', 'bob@work.example', false),
];

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_CONTACTS]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

function resp(...mr: Invocation[]): JmapResponse {
  return { methodResponses: mr, sessionState: 's' };
}

function contactsClient(): Client {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const name = body.methodCalls[0]?.[0];
    if (name === 'AddressBook/get') {
      return resp(['AddressBook/get', { accountId: 'acct1', state: 's', list: [{ id: 'default', name: 'Default', isDefault: true, carddavUrl: null, syncToken: null }], notFound: [] }, 'books']);
    }
    if (name === 'ContactGroup/get') {
      return resp(['ContactGroup/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, 'groups']);
    }
    if (name === 'ContactCard/query') {
      return resp(
        ['ContactCard/query', { accountId: 'acct1', ids: CARDS.map((c) => c.id) }, 'q'],
        ['ContactCard/get', { accountId: 'acct1', state: 's', list: CARDS, notFound: [] }, 'g'],
      );
    }
    if (name === 'Identity/get') {
      return resp(['Identity/get', { accountId: 'acct1', state: 's', list: [], notFound: [] }, body.methodCalls[0]?.[2] ?? 'i']);
    }
    return resp(...body.methodCalls.map((c) => [c[0], {}, c[2]] as Invocation));
  });
  return {
    login: vi.fn(async () => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async () => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };
}

describe('Compose contacts autocomplete', () => {
  it('suggests a contact by prefix and inserts it into the To field', async () => {
    const app = createAppState(contactsClient());
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={() => undefined} />
      </AppContext.Provider>
    ));
    await app.loadContacts();

    const to = screen.getByLabelText('To') as HTMLInputElement;
    fireEvent.input(to, { target: { value: 'ali' } });

    const suggestion = await screen.findByTestId('contact-suggestion');
    expect(suggestion).toHaveTextContent('Alice');

    fireEvent.mouseDown(suggestion);
    expect((screen.getByLabelText('To') as HTMLInputElement).value).toContain('alice@example.org');
    // Dropdown closes after a pick.
    expect(screen.queryByTestId('contact-suggestion')).toBeNull();
  });

  it('suggests contacts in Cc and Bcc too, and inserts the pick into the field being typed in', async () => {
    const app = createAppState(contactsClient());
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={() => undefined} />
      </AppContext.Provider>
    ));
    await app.loadContacts();
    fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));

    const cc = screen.getByLabelText('Cc') as HTMLInputElement;
    fireEvent.input(cc, { target: { value: 'x@example.org, bo' } });
    const forCc = await screen.findByTestId('contact-suggestion');
    expect(forCc).toHaveTextContent('Bob');
    // The list is the Cc field's, not To's.
    expect(cc.closest('label')!.contains(forCc)).toBe(true);
    fireEvent.mouseDown(forCc);
    expect(cc.value).toBe('x@example.org, Bob Brown <bob@work.example>, ');
    expect(screen.queryByTestId('contact-suggestion')).toBeNull();

    const bcc = screen.getByLabelText('Bcc') as HTMLInputElement;
    fireEvent.input(bcc, { target: { value: 'ali' } });
    const forBcc = await screen.findByTestId('contact-suggestion');
    expect(bcc.closest('label')!.contains(forBcc)).toBe(true);
    fireEvent.mouseDown(forBcc);
    expect(bcc.value).toBe('Alice Adams <alice@example.org>, ');
    // Neither pick went into To, and Cc kept its own.
    expect((screen.getByLabelText('To') as HTMLInputElement).value).toBe('');
    expect(cc.value).toBe('x@example.org, Bob Brown <bob@work.example>, ');
  });

  it('shows no dropdown until the user types', async () => {
    const app = createAppState(contactsClient());
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={() => undefined} />
      </AppContext.Provider>
    ));
    await app.loadContacts();
    await waitFor(() => expect(app.contacts().length).toBe(2));
    expect(screen.queryByTestId('contact-suggestion')).toBeNull();
  });
});
