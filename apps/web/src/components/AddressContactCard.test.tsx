import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { AppContext } from '../state/context.ts';
import { createAppState, type AppState } from '../state/store.ts';
import { defaultSeed, makeContactsClient } from '../modules/contacts/mockClient.ts';
import { createAddressContactCard, contactComposeInitial } from './AddressContactCard.tsx';
import { parseRecipients } from '../api/jmap.ts';
import type { Client } from '../api/client.ts';
vi.mock('./PersonAvatar.tsx', () => ({ PersonAvatar: () => <span/> }));
afterEach(() => { cleanup(); vi.restoreAllMocks(); });

function setup(options: { client?: Client; address?: string; name?: string; onCompose?: ReturnType<typeof vi.fn>; ownerDocument?: Document; trigger?: HTMLElement } = {}) {
  const client = options.client ?? makeContactsClient(defaultSeed());
  let app!: AppState, changeScope!: (value: string) => void;
  const compose = options.onCompose ?? vi.fn();
  render(() => {
    app = createAppState(client);
    const [scope, setScope] = createSignal('account:message'); changeScope = setScope;
    const card = createAddressContactCard({ scope, active: () => true, onCompose: compose });
    return <AppContext.Provider value={app}><button onClick={event => card.open(options.address ?? 'new@example.org', options.name ?? 'New Person', options.ownerDocument ? { x: 10, y: 20, document: options.ownerDocument, ...(options.trigger ? { trigger: options.trigger } : {}) } : event.currentTarget)}>Address</button>{card.card()}</AppContext.Provider>;
  });
  return { app, client, compose, changeScope, open: () => fireEvent.click(screen.getByRole('button', { name: 'Address' })) };
}

describe('address contact card', () => {
  it('loads saved information from another book without changing contact list filters', async () => {
    const seed = defaultSeed();
    seed.contacts![0] = { ...seed.contacts![0]!, addressBookId: 'another-book', organizations: ['Saved Organization'], phones: [{ value: '+351123456789', context: 'work' }], notes: 'Saved notes' };
    const target = seed.contacts![0]!;
    const view = setup({ client: makeContactsClient(seed), address: target.emails[0]!.value });
    view.app.selectAddressBook('selected-book'); view.open();
    expect(await screen.findByText('Saved Organization')).toBeInTheDocument();
    expect(screen.getByText('+351123456789')).toBeInTheDocument();
    expect(screen.getByText('Saved notes')).toBeInTheDocument();
    expect(view.app.selectedAddressBookId()).toBe('selected-book');
    expect(screen.queryByRole('button', { name: 'Save contact' })).toBeNull();
  });
  it('persists a new contact once to the selected book using the real store API', async () => {
    const view = setup();
    const jmap = vi.spyOn(view.client, 'jmap'); view.open();
    const save = await screen.findByRole('button', { name: 'Save contact' });
    fireEvent.input(screen.getByLabelText('Full name'), { target: { value: 'Saved Person' } });
    fireEvent.click(save); fireEvent.click(save);
    await waitFor(() => expect(view.app.contacts().some(card => card.name.full === 'Saved Person' && card.emails[0]?.value === 'new@example.org')).toBe(true));
    const writes = jmap.mock.calls.flatMap(([request]) => request.methodCalls.filter(([method]) => method === 'ContactCard/set'));
    expect(writes).toHaveLength(1);
    expect(writes[0]![1]).toMatchObject({ create: { new: { name: { full: 'Saved Person' }, emails: [{ value: 'new@example.org' }] } } });
  });
  it('keeps the editable name and shows one error toast when a save fails', async () => {
    const view = setup(); view.open();
    const save = await screen.findByRole('button', { name: 'Save contact' });
    const toast = vi.spyOn(view.app, 'showToast');
    vi.spyOn(view.app, 'createContact').mockResolvedValue(null);
    fireEvent.input(screen.getByLabelText('Full name'), { target: { value: 'Keep My Draft' } });
    fireEvent.click(save);
    await waitFor(() => expect(toast).toHaveBeenCalledWith('error', 'The contact could not be saved. Try again.'));
    expect(toast).toHaveBeenCalledTimes(1);
    expect(screen.getByLabelText('Full name')).toHaveValue('Keep My Draft');
    expect(screen.getByRole('button', { name: 'Save contact' })).toBeEnabled();
  });
  it('reclamps after async content grows and preserves typed content during keyboard resize', async () => {
    let height = 60;
    const rect = vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function(this: HTMLElement) { return { x: 0, y: 0, left: 0, top: 0, width: 260, height: this.classList.contains('address-card') ? height : 24, right: 260, bottom: height, toJSON: () => ({}) }; });
    const view = setup(); view.open();
    await screen.findByRole('button', { name: 'Save contact' });
    fireEvent.input(screen.getByLabelText('Full name'), { target: { value: 'Keyboard Draft' } });
    height = 500;
    vi.spyOn(window, 'innerHeight', 'get').mockReturnValue(320);
    fireEvent.resize(window);
    const dialog = screen.getByRole('dialog');
    expect(parseFloat(dialog.style.top)).toBe(8);
    expect(dialog.style.maxHeight).toBe('304px');
    expect(screen.getByLabelText('Full name')).toHaveValue('Keyboard Draft');
    expect(screen.getByRole('dialog')).toBe(dialog);
    rect.mockRestore();
  });
  it('reclamps when a deferred contact lookup expands the card', async () => {
    let height = 60, release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; }), inner = makeContactsClient(defaultSeed());
    const client: Client = { ...inner, jmap: async request => { const result = await inner.jmap(request); await gate; return result; } };
    vi.spyOn(window, 'innerHeight', 'get').mockReturnValue(320);
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function(this: HTMLElement) {
      const card = this.classList.contains('address-card'), top = card ? 0 : 180, size = card ? height : 24;
      return { x: 0, y: top, left: 0, top, width: 260, height: size, right: 260, bottom: top + size, toJSON: () => ({}) };
    });
    const view = setup({ client }); view.open(); const dialog = screen.getByRole('dialog');
    expect(dialog.style.top).toBe('208px');
    height = 280; release();
    await screen.findByRole('button', { name: 'Save contact' });
    await waitFor(() => expect(dialog.style.top).toBe('32px'));
  });
  it('composes to an escaped address and closes on account/message scope change', async () => {
    const view = setup({ name: 'Doe, "Jane"' }); view.open();
    await screen.findByRole('button', { name: 'Send email' });
    fireEvent.click(screen.getByRole('button', { name: 'Send email' }));
    expect(view.compose).toHaveBeenCalledTimes(1);
    expect(parseRecipients(view.compose.mock.calls[0]![0].to)).toEqual([{ name: 'Doe, "Jane"', email: 'new@example.org' }]);
    expect(screen.queryByRole('dialog')).toBeNull();
    view.open(); view.changeScope('another-account:message');
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('does not construct a composer recipient from an invalid address', () => {
    expect(contactComposeInitial('bad\n@example.org')).toBeNull();
  });
  it('edits, saves, composes and restores focus in a foreign document', async () => {
    const frame = document.createElement('iframe'); document.body.append(frame);
    try {
      const doc = frame.contentDocument!, trigger = doc.createElement('button'); trigger.textContent = 'Foreign address'; doc.body.append(trigger);
      const seed = defaultSeed(); seed.addressBooks.push({ id: 'ab2', name: 'Another Book', isDefault: false, carddavUrl: null, syncToken: null });
      const view = setup({ ownerDocument: doc, trigger, client: makeContactsClient(seed) }); const jmap = vi.spyOn(view.client, 'jmap');
      view.open(); await within(doc.body).findByRole('button', { name: 'Save contact' });
      const input = within(doc.body).getByLabelText('Full name');
      fireEvent.input(input, { target: { value: 'Foreign Saved Name' } });
      fireEvent.change(within(doc.body).getByLabelText('Address book'), { target: { value: 'ab2' } });
      fireEvent.submit(doc.querySelector('form')!);
      await waitFor(() => expect(view.app.contacts().some(card => card.name.full === 'Foreign Saved Name')).toBe(true));
      const create = jmap.mock.calls.flatMap(([request]) => request.methodCalls).find(([method]) => method === 'ContactCard/set');
      expect(create?.[1]).toMatchObject({ create: { new: { name: { full: 'Foreign Saved Name' }, addressBookId: 'ab2' } } });
      fireEvent.click(within(doc.body).getByRole('button', { name: 'Send email' }));
      expect(view.compose).toHaveBeenCalledTimes(1);
      expect(parseRecipients(view.compose.mock.calls[0]![0].to)[0]).toEqual({ name: 'Foreign Saved Name', email: 'new@example.org' });
      expect(within(doc.body).queryByRole('dialog')).toBeNull();
      await waitFor(() => expect(doc.activeElement).toBe(trigger));
    } finally { cleanup(); frame.remove(); }
  });
});
