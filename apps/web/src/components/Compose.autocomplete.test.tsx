import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
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

describe('Compose contacts autocomplete', () => {
  it('confirms typed mailboxes with Enter and keeps each recipient field ready for the next address',async()=>{
    const client=contactsClient();const app=createAppState(client);const close=vi.fn();
    render(()=><AppContext.Provider value={app}><Compose onClose={close}/></AppContext.Provider>);
    await app.loadContacts();fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
    for(const name of ['To','Cc','Bcc']){
      const input=screen.getByRole('combobox',{name}) as HTMLInputElement;input.focus();
      fireEvent.input(input,{target:{value:'new@example.org'}});
      fireEvent.keyDown(input,{key:'Enter'});
      expect(input.value).toBe('new@example.org, ');expect(input).toHaveFocus();
      expect(input.selectionStart).toBe(input.value.length);
      fireEvent.input(input,{target:{value:input.value+'"原,a"@example.org'}});
      fireEvent.keyDown(input,{key:'Enter'});
      expect(input.value).toBe('new@example.org, "原,a"@example.org, ');
      fireEvent.keyDown(input,{key:'Enter'});
      expect(input.value).toBe('new@example.org, "原,a"@example.org, ');
    }
    expect(close).not.toHaveBeenCalled();
    expect(vi.mocked(client.jmap).mock.calls.some(([request])=>request.methodCalls.some(([method])=>method==='EmailSubmission/set'))).toBe(false);
  });

  it('preserves a complete typed address over an unselected contact suggestion and leaves invalid tokens editable',async()=>{
    const app=createAppState(contactsClient());
    render(()=><AppContext.Provider value={app}><Compose onClose={()=>undefined}/></AppContext.Provider>);
    await app.loadContacts();const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;
    fireEvent.input(input,{target:{value:'alice@example.org'}});
    await screen.findByTestId('contact-suggestion');
    fireEvent.keyDown(input,{key:'Enter'});
    expect(input.value).toBe('alice@example.org, ');
    fireEvent.input(input,{target:{value:'unfinished@'}});
    fireEvent.keyDown(input,{key:'Enter'});
    expect(input.value).toBe('unfinished@');
  });

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

    fireEvent.click(suggestion);
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
    fireEvent.click(forCc);
    expect(cc.value).toBe('x@example.org, Bob Brown <bob@work.example>, ');
    expect(screen.queryByTestId('contact-suggestion')).toBeNull();

    const bcc = screen.getByLabelText('Bcc') as HTMLInputElement;
    fireEvent.input(bcc, { target: { value: 'ali' } });
    const forBcc = await screen.findByTestId('contact-suggestion');
    expect(bcc.closest('label')!.contains(forBcc)).toBe(true);
    fireEvent.click(forBcc);
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

  it('keeps keyboard focus in each recipient combobox and Enter selects instead of submitting',async()=>{
    const client=contactsClient();const app=createAppState(client);const close=vi.fn();
    render(()=><AppContext.Provider value={app}><Compose onClose={close}/></AppContext.Provider>);
    await app.loadContacts();fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
    for(const name of ['To','Cc','Bcc']){
      const input=screen.getByRole('combobox',{name}) as HTMLInputElement;input.focus();
      fireEvent.input(input,{target:{value:'a'}});
      await screen.findAllByTestId('contact-suggestion');
      expect(input).toHaveAttribute('aria-expanded','true');
      const list=screen.getByRole('listbox');expect(input.getAttribute('aria-controls')).toBe(list.id);
      fireEvent.keyDown(input,{key:'ArrowDown'});
      const selected=within(list).getByRole('option',{selected:true});expect(input.getAttribute('aria-activedescendant')).toBe(selected.id);
      expect(input).toHaveFocus();fireEvent.keyDown(input,{key:'Enter'});
      expect(input.value).toBe('Alice Adams <alice@example.org>, ');expect(input).toHaveFocus();
      expect(input).toHaveAttribute('aria-expanded','false');expect(input).not.toHaveAttribute('aria-activedescendant');
      fireEvent.keyDown(input,{key:'Enter'});
    }
    expect(close).not.toHaveBeenCalled();
    expect(vi.mocked(client.jmap).mock.calls.some(([request])=>request.methodCalls.some(([name])=>name==='EmailSubmission/set'))).toBe(false);
  });

  it('dismisses suggestions with Escape without closing the draft, then can reopen with arrows',async()=>{
    const app=createAppState(contactsClient());const close=vi.fn();
    render(()=><AppContext.Provider value={app}><Compose onClose={close}/></AppContext.Provider>);await app.loadContacts();
    const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;
    fireEvent.input(input,{target:{value:'example'}});await screen.findAllByTestId('contact-suggestion');
    fireEvent.keyDown(input,{key:'Escape'});expect(screen.queryByRole('listbox')).toBeNull();expect(close).not.toHaveBeenCalled();
    fireEvent.keyDown(input,{key:'ArrowUp'});expect(within(screen.getByRole('listbox')).getByRole('option',{selected:true})).toHaveTextContent('Bob');
    fireEvent.keyDown(input,{key:'Enter'});expect(input.value).toBe('Bob Brown <bob@work.example>, ');
  });

  it('replaces the token at the caret while preserving following recipients and quoted commas',async()=>{
    const app=createAppState(contactsClient());render(()=><AppContext.Provider value={app}><Compose onClose={()=>undefined}/></AppContext.Provider>);await app.loadContacts();
    const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;
    input.value='"Last, First" <first@example.org>, al, tail@example.org';input.setSelectionRange(36,36);
    const caret=input.value.indexOf(', tail');input.setSelectionRange(caret,caret);fireEvent.input(input);
    const option=await screen.findByTestId('contact-suggestion');fireEvent.click(option);
    expect(input.value).toBe('"Last, First" <first@example.org>, Alice Adams <alice@example.org>, tail@example.org');
    expect(input.selectionStart).toBe(input.value.indexOf(', tail'));
  });

  it('waits for IME composition to finish and never treats its Enter or Escape as a draft command',async()=>{
    const app=createAppState(contactsClient());const close=vi.fn();render(()=><AppContext.Provider value={app}><Compose onClose={close}/></AppContext.Provider>);await app.loadContacts();
    const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;
    fireEvent.compositionStart(input);fireEvent.input(input,{target:{value:'ali'}});
    fireEvent.keyDown(input,{key:'Enter',isComposing:true});fireEvent.keyDown(input,{key:'Escape',isComposing:true});
    expect(screen.queryByRole('listbox')).toBeNull();expect(input.value).toBe('ali');expect(close).not.toHaveBeenCalled();
    fireEvent.compositionEnd(input);await screen.findByTestId('contact-suggestion');fireEvent.keyDown(input,{key:'Enter'});
    expect(input.value).toBe('Alice Adams <alice@example.org>, ');
  });

  it('finds an owned contact outside the loaded book through debounced full-store autocomplete',async()=>{
    const client=contactsClient([card('outside','Zelda','Remote','Zelda.Canonical@example.org',false)]);
    const app=createAppState(client);render(()=><AppContext.Provider value={app}><Compose onClose={()=>undefined}/></AppContext.Provider>);await app.loadContacts();
    expect(app.contacts().some(c=>c.id==='outside')).toBe(false);
    const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;
    fireEvent.input(input,{target:{value:'zel'}});fireEvent.input(input,{target:{value:'zelda'}});
    const option=await screen.findByTestId('contact-suggestion');expect(option).toHaveTextContent('Zelda Remote');
    const requests=vi.mocked(client.jmap).mock.calls.map(([request])=>request.methodCalls[0]).filter(call=>call?.[0]==='ContactCard/autocomplete');
    expect(requests).toHaveLength(1);expect(requests[0]![1]).toMatchObject({accountId:'acct1',prefix:'zelda',limit:8});
    fireEvent.click(option);expect(input.value).toBe('Zelda Remote <Zelda.Canonical@example.org>, ');
  });

  it('accepts a touch click after focus moves into the suggestion without losing the pick',async()=>{
    const app=createAppState(contactsClient());render(()=><AppContext.Provider value={app}><Compose onClose={()=>undefined}/></AppContext.Provider>);await app.loadContacts();
    const input=screen.getByRole('combobox',{name:'To'}) as HTMLInputElement;input.focus();fireEvent.input(input,{target:{value:'ali'}});
    const option=await screen.findByTestId('contact-suggestion');
    fireEvent.pointerDown(option,{pointerType:'touch'});fireEvent.blur(input,{relatedTarget:option});
    expect(screen.getByRole('listbox')).toBeInTheDocument();fireEvent.pointerUp(option,{pointerType:'touch'});fireEvent.click(option);
    expect(input.value).toBe('Alice Adams <alice@example.org>, ');expect(input).toHaveFocus();
  });
});
