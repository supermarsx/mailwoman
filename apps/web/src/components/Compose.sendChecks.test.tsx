import { chooseComposeFormat } from './compose/nativeHtmlTestHarness.ts';
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { recipientValue } from './compose/recipientTestUtils.ts';
import { Compose } from './Compose.tsx';
import { clearSettings, loadSettings } from '../api/settings.ts';
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

function contactsClient(remoteCards:ContactCard[]=[]): Client {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const name = body.methodCalls[0]?.[0];
    if(name==='ContactCard/autocomplete'){
      const args=body.methodCalls[0]![1] as {prefix:string};
      return resp(['ContactCard/autocomplete',{accountId:'acct1',list:remoteCards.filter(c=>`${c.name.full} ${c.emails[0]!.value}`.toLowerCase().includes(args.prefix.toLowerCase())).map(c=>({id:c.id,name:c.name.full,email:c.emails[0]!.value,isFavorite:c.isFavorite}))},'ac']);
    }
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


const entry = (value: unknown, locked = false) => ({ value, source: 'user', locked });
const seen: Array<{ url: string; method: string }> = [];
async function configure(settings: Record<string, unknown>): Promise<void> {
  seen.length = 0;
  await loadSettings({
    refresh: true,
    fetcher: async (input, init) => {
      const method = init?.method ?? 'GET';
      seen.push({ url: String(input), method });
      if (method === 'PUT') return new Response(JSON.stringify(entry('allow')), { status: 200, headers: { 'content-type': 'application/json' } });
      return new Response(JSON.stringify({ settings }), { status: 200, headers: { 'content-type': 'application/json' } });
    },
  });
}
const POLICIES = (overrides: Record<string, unknown> = {}) => ({
  'mail.compose.checkInvalidRecipients': entry('allow'),
  'mail.compose.checkMissingAttachment': entry('allow'),
  'mail.compose.checkEmptySubject': entry('allow'),
  'mail.externalRecipientWarning': entry('off'),
  ...overrides,
});

/** Type `text` the way a person does: the last character arrives on its own. */
function typeInto(input: HTMLInputElement, text: string): void {
  fireEvent.input(input, { target: { value: text.slice(0, -1) } });
  fireEvent.input(input, { target: { value: text } });
}

async function mount() {
  const app = createAppState(contactsClient());
  const send = vi.spyOn(app, 'sendMessage').mockResolvedValue(undefined);
  render(() => <AppContext.Provider value={app}><Compose onClose={() => undefined} /></AppContext.Provider>);
  await app.loadContacts();
  return { app, send, to: screen.getByRole('combobox', { name: 'To' }) as HTMLInputElement };
}

describe('Compose recipient chips', () => {
  afterEach(() => clearSettings());
  const cardEl = () => document.querySelector<HTMLElement>('.address-card');
  const chipsOf = (field = 'to') => within(screen.getByTestId(`compose-chips-${field}`));

  it('replaces a valid address with a chip inside the box when space is typed', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'a@b.example ');
    expect(to.value).toBe('');
    expect(to.closest('.compose-rf__box')!.contains(chipsOf().getByText(/a@b\.example/))).toBe(true);
    expect(recipientValue(to)).toBe('a@b.example, ');
    expect(to).toHaveFocus();
  });

  it('shows the contact name on the chip and follows the semicolon setting', async () => {
    await configure(POLICIES({ 'mail.compose.recipientSeparator': entry('semicolon') }));
    const { to } = await mount();
    to.focus();
    typeInto(to, 'alice@example.org ');
    typeInto(to, 'x@y.example ');
    expect(chipsOf().getByText(/Alice Adams/)).toBeInTheDocument();
    expect(chipsOf().getAllByRole('listitem')).toHaveLength(2);
  });

  it('turns a typed semicolon into a chip and keeps typing after it', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'a@b.example;');
    expect(recipientValue(to)).toBe('a@b.example, ');
  });

  it('keeps invalid text as typed until it is left, then shows an invalid chip and alerts in the top bar', async () => {
    await configure(POLICIES({ 'mail.compose.checkInvalidRecipients': entry('warn') }));
    const { to } = await mount();
    to.focus();
    typeInto(to, 'nope ');
    expect(to.value).toBe('nope ');
    expect(screen.queryByTestId('compose-status-invalid')).toBeNull();
    fireEvent.blur(to);
    expect(to.value).toBe('');
    const strip = await screen.findByTestId('compose-status');
    expect(strip).toHaveAttribute('role', 'status');
    fireEvent.click(within(strip).getByRole('button', { name: /Invalid address: .*nope/ }));
    expect(to).toHaveFocus();
    expect(document.querySelector('[data-valid="false"]')).not.toBeNull();
  });

  it('does not nag on an empty draft', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('warn'), 'mail.compose.checkInvalidRecipients': entry('warn') }));
    await mount();
    const bar = screen.getByTestId('compose-status');
    for (const id of ['no-recipients', 'subject', 'invalid']) expect(within(bar).queryByTestId(`compose-status-${id}`)).toBeNull();
  });

  it('splits a pasted list into chips', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    fireEvent.paste(to, { clipboardData: { getData: () => 'a@b.example\nAnn <c@d.example>' } });
    expect(recipientValue(to)).toBe('a@b.example, Ann <c@d.example>, ');
    expect(chipsOf().getAllByRole('listitem')).toHaveLength(2);
  });

  it('removes a chip and its address', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'a@b.example ');
    typeInto(to, 'c@d.example ');
    expect(recipientValue(to)).toBe('a@b.example, c@d.example, ');
    fireEvent.click(screen.getByRole('button', { name: /Remove .*a@b\.example/ }));
    expect(recipientValue(to)).toBe('c@d.example, ');
  });

  it('leaves text alone when the setting is off', async () => {
    await configure(POLICIES({ 'mail.compose.autoChipRecipients': entry(false) }));
    const { to } = await mount();
    to.focus();
    fireEvent.input(to, { target: { value: 'a@b.example ' } });
    expect(to.value).toBe('a@b.example ');
    expect(screen.queryByTestId('compose-chips-to')).toBeNull();
  });

  it('opens the contact card of a chip on click and returns focus on Escape', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'alice@example.org ');
    const chip = chipsOf().getByRole('button', { name: /^\W*Alice Adams\W+alice@example\.org/ });
    fireEvent.click(chip);
    const card = await waitFor(() => { const el = cardEl(); expect(el).not.toBeNull(); return el!; });
    expect(card).toHaveTextContent('Alice Adams');
    expect(card).toHaveTextContent('alice@example.org');
    fireEvent.keyDown(card, { key: 'Escape' });
    await waitFor(() => expect(cardEl()).toBeNull());
    expect(chip).toHaveFocus();
  });

  it('opens the card for an unknown address and not on hover', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'stranger@nowhere.example ');
    const chip = chipsOf().getByRole('button', { name: /^\W*stranger@nowhere\.example/ });
    fireEvent.mouseEnter(chip); fireEvent.mouseOver(chip); fireEvent.pointerEnter(chip);
    expect(cardEl()).toBeNull();
    fireEvent.click(chip);
    await waitFor(() => expect(cardEl()).not.toBeNull());
    expect(cardEl()).toHaveTextContent('stranger@nowhere.example');
  });

  it('closes the card when its chip is removed, and F2 or double click edits the address', async () => {
    await configure(POLICIES());
    const { to } = await mount();
    to.focus();
    typeInto(to, 'alice@example.org ');
    fireEvent.click(chipsOf().getByRole('button', { name: /^\W*Alice Adams\W+alice@example\.org/ }));
    await waitFor(() => expect(cardEl()).not.toBeNull());
    fireEvent.click(screen.getByRole('button', { name: /Remove .*alice@example\.org/ }));
    expect(cardEl()).toBeNull();
    typeInto(to, 'b@c.example ');
    fireEvent.keyDown(chipsOf().getByRole('button', { name: /^\W*b@c\.example/ }), { key: 'F2' });
    expect(to.value).toBe('b@c.example');
    expect(screen.queryByTestId('compose-chips-to')).toBeNull();
    typeInto(to, 'b@c.example ');
    fireEvent.dblClick(chipsOf().getByRole('button', { name: /^\W*b@c\.example/ }));
    expect(to.value).toBe('b@c.example');
  });
});

describe('Compose pre-send checks', () => {
  afterEach(() => clearSettings());

  it('warns about an empty subject, goes back on Review and sends anyway on request', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('warn') }));
    const { to, send } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('The subject is empty.');
    expect(within(dialog).queryByLabelText("Don't ask again for these")).toBeNull();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Review' }));
    await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
    expect(send).not.toHaveBeenCalled();
    expect(screen.getByLabelText('Subject')).toHaveFocus();
    fireEvent.click(await screen.findByRole('button', { name: 'Send' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Send anyway' }));
    await waitFor(() => expect(send).toHaveBeenCalledOnce());
  });

  it('combines every warning in one dialog, in order', async () => {
    await configure(POLICIES({
      'mail.compose.checkEmptySubject': entry('warn'),
      'mail.compose.checkInvalidRecipients': entry('warn'),
      'mail.externalRecipientWarning': entry('warn'),
    }));
    const { to } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example, nope' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    const dialog = await screen.findByRole('alertdialog');
    const items = within(dialog).getAllByRole('listitem').map(li => li.textContent);
    expect(items).toEqual([
      expect.stringContaining('look invalid'),
      'The subject is empty.',
      expect.stringMatching(/outside trusted domains: .*b\.example/),
    ]);
  });

  it('sends without asking when the policy is allow', async () => {
    await configure(POLICIES());
    const { to, send } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await waitFor(() => expect(send).toHaveBeenCalledOnce());
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('remembers the choice for warn-remember by writing allow, and sends', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('warn-remember') }));
    const { to, send } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    fireEvent.click(await screen.findByLabelText("Don't ask again for these"));
    fireEvent.click(screen.getByRole('button', { name: 'Send anyway' }));
    await waitFor(() => expect(send).toHaveBeenCalledOnce());
    await waitFor(() => expect(seen.some(call => call.method === 'PUT' && call.url.includes('mail.compose.checkEmptySubject'))).toBe(true));
  });

  it('does not offer to remember a locked policy', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('warn-remember', true) }));
    const { to } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    await screen.findByRole('alertdialog');
    expect(screen.queryByLabelText("Don't ask again for these")).toBeNull();
  });

  it('refuses a denied check with the reason and sends nothing', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('deny') }));
    const { to, send } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Sending is blocked by your settings. Add a subject.');
    expect(send).not.toHaveBeenCalled();
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('warns when the text mentions an attachment but none is attached', async () => {
    await configure(POLICIES({ 'mail.compose.checkMissingAttachment': entry('warn') }));
    const { to, send } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Report' } });
    await chooseComposeFormat('plain');
    const body = screen.getByLabelText('Body');
    fireEvent.input(body, { target: { value: 'Please see the attached report.' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByRole('alertdialog')).toHaveTextContent('mentions an attachment, but nothing is attached');
    expect(send).not.toHaveBeenCalled();
  });

  it('flags a missing subject in the top bar once the body has text', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('warn') }));
    await mount();
    await chooseComposeFormat('plain');
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'Hello' } });
    const strip = await screen.findByTestId('compose-status');
    fireEvent.click(await within(strip).findByRole('button', { name: 'No subject' }));
    expect(screen.getByLabelText('Subject')).toHaveFocus();
  });

  it('shows a dismissible, subdued bar for recipients outside trusted domains', async () => {
    await configure(POLICIES({ 'mail.externalRecipientWarning': entry('warn'), 'mail.trustedRecipientDomains': entry(['b.example']) }));
    const { to } = await mount();
    fireEvent.input(to, { target: { value: 'a@b.example, c@other.example' } });
    const bar = await screen.findByTestId('compose-status-external');
    expect(bar).toHaveTextContent('other.example');
    expect(bar).toHaveAttribute('data-subdued');
    expect(bar).toHaveTextContent('External: 1');
    fireEvent.click(within(bar).getByRole('button', { name: /External: 1/ }));
    const detail = screen.getByTestId('compose-status-detail');
    expect(detail).toHaveTextContent('Check the To, Cc and Bcc recipients before sending confidential information.');
    fireEvent.click(within(detail).getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByTestId('compose-status-external')).toBeNull();
  });
});
