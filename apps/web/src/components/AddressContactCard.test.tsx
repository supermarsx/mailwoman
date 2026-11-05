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

async function openSave(root: { findByRole: typeof screen.findByRole } = screen) {
  const toggle = await root.findByRole('button', { name: 'Save to contacts' });
  await waitFor(() => expect((toggle as HTMLButtonElement).disabled).toBe(false));
  fireEvent.click(toggle);
  return root.findByRole('button', { name: 'Save contact' });
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
    const save = await openSave();
    fireEvent.input(screen.getByLabelText('Full name'), { target: { value: 'Saved Person' } });
    fireEvent.click(save); fireEvent.click(save);
    await waitFor(() => expect(view.app.contacts().some(card => card.name.full === 'Saved Person' && card.emails[0]?.value === 'new@example.org')).toBe(true));
    const writes = jmap.mock.calls.flatMap(([request]) => request.methodCalls.filter(([method]) => method === 'ContactCard/set'));
    expect(writes).toHaveLength(1);
    expect(writes[0]![1]).toMatchObject({ create: { new: { name: { full: 'Saved Person' }, emails: [{ value: 'new@example.org' }] } } });
  });
  it('keeps the editable name and shows one error toast when a save fails', async () => {
    const view = setup(); view.open();
    const save = await openSave();
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
    await openSave();
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
    await openSave();
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
      view.open(); await openSave(within(doc.body));
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
  it('keeps the five icon actions on one row, all the same kind of control, with accessible names', async () => {
    const view = setup(); view.open();
    await screen.findByRole('button', { name: 'Save to contacts' });
    const row = screen.getByRole('group', { name: 'Contact actions' });
    const buttons = within(row).getAllByRole('button');
    expect(buttons.map(button => button.getAttribute('aria-label'))).toEqual(['Save to contacts', 'Send email', 'Favorite New Person', 'Copy contact information', 'Share contact']);
    for (const button of buttons) { expect(button).toHaveAttribute('title'); expect(button).toHaveClass('address-card__action'); expect((button.closest('.contact-share') ?? button).parentElement).toBe(row); expect(button).not.toHaveAttribute('tabindex', '-1'); }
    expect(screen.queryByRole('button', { name: 'Go to contact' })).toBeNull();
    // The menu is drawn below the row, never as another control inside it.
    fireEvent.click(screen.getByRole('button', { name: 'Share contact' }));
    expect(within(row).getAllByRole('button')).toHaveLength(5);
  });
  it('favorites a saved contact at once and keeps it when the store confirms', async () => {
    const seed = defaultSeed(), target = seed.contacts!.find(card => !card.isFavorite)!;
    const view = setup({ client: makeContactsClient(seed), address: target.emails[0]!.value });
    await view.app.loadContacts(); view.open();
    const star = await screen.findByRole('button', { name: `Favorite ${target.name.full}` });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Go to contact' })).toBeInTheDocument());
    expect(star).toHaveAttribute('aria-pressed', 'false'); expect(star).toHaveAttribute('title', 'Add to favorites');
    fireEvent.click(star);
    expect(star).toHaveAttribute('aria-pressed', 'true');
    await waitFor(() => expect(view.app.contactById(target.id)!.isFavorite).toBe(true));
    expect(star).toHaveAttribute('aria-pressed', 'true'); expect(star).toHaveAttribute('title', 'Remove from favorites');
  });
  it('rolls the star back and says so when the favorite is refused', async () => {
    const seed = defaultSeed(), target = seed.contacts!.find(card => !card.isFavorite)!;
    const view = setup({ client: makeContactsClient(seed), address: target.emails[0]!.value });
    await view.app.loadContacts(); view.open();
    const toast = vi.spyOn(view.app, 'showToast');
    const star = await screen.findByRole('button', { name: `Favorite ${target.name.full}` });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Go to contact' })).toBeInTheDocument());
    const original = view.client.jmap.bind(view.client);
    vi.spyOn(view.client, 'jmap').mockImplementation(async (request, options) => request.methodCalls.some(([method]) => method === 'ContactCard/set') ? Promise.reject(new Error('offline')) : original(request, options));
    fireEvent.click(star);
    expect(star).toHaveAttribute('aria-pressed', 'true');
    await waitFor(() => expect(toast).toHaveBeenCalledWith('error', 'Favorites could not be updated.'));
    expect(star).toHaveAttribute('aria-pressed', 'false');
    expect(view.app.contactById(target.id)!.isFavorite).toBe(false);
  });
  it('saves an unsaved address and favorites it in one step', async () => {
    const view = setup(); const jmap = vi.spyOn(view.client, 'jmap'); view.open();
    const star = await screen.findByRole('button', { name: 'Favorite New Person' });
    await waitFor(() => expect(star).toBeEnabled());
    expect(star).toHaveAttribute('title', 'Save and add to favorites');
    fireEvent.click(star);
    await waitFor(() => expect(view.app.contacts().some(card => card.emails[0]?.value === 'new@example.org' && card.isFavorite)).toBe(true));
    const writes = jmap.mock.calls.flatMap(([request]) => request.methodCalls.filter(([method]) => method === 'ContactCard/set'));
    expect(writes).toHaveLength(1);
    expect(writes[0]![1]).toMatchObject({ create: { new: { isFavorite: true, emails: [{ value: 'new@example.org' }] } } });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Favorite New Person' })).toHaveAttribute('aria-pressed', 'true'));
  });
  it('shares the vCard of an unsaved address: copy, download, and a message with the file attached', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    const createObjectURL = vi.fn().mockReturnValue('blob:vcf'), revokeObjectURL = vi.fn();
    Object.assign(URL, { createObjectURL, revokeObjectURL });
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined);
    const view = setup(); view.open();
    await screen.findByRole('button', { name: 'Save to contacts' });
    const open = () => fireEvent.click(screen.getByRole('button', { name: 'Share contact' }));
    open();
    const menu = screen.getByRole('menu', { name: 'Share contact' });
    expect(within(menu).getAllByRole('menuitem').map(item => item.textContent!.trim())).toEqual(['Send contact by email', 'Copy as vCard', 'Download .vcf']);
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'Copy as vCard' }));
    await waitFor(() => expect(writeText).toHaveBeenCalledTimes(1));
    const vcard = writeText.mock.calls[0]![0] as string;
    expect(vcard).toMatch(/^BEGIN:VCARD\r\nVERSION:4\.0\r\n/); expect(vcard).toContain('FN:New Person'); expect(vcard).toContain('EMAIL;PREF=1:new@example.org'); expect(vcard).toMatch(/REV:\d{8}T\d{6}Z/);
    expect(vcard.endsWith('END:VCARD')).toBe(true);
    await waitFor(() => expect(screen.queryByRole('menu')).toBeNull());
    open();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Download .vcf' }));
    expect(createObjectURL).toHaveBeenCalledTimes(1); expect(click).toHaveBeenCalledTimes(1);
    const link = click.mock.contexts[0] as HTMLAnchorElement; expect(link.download).toBe('New Person.vcf');
    vi.spyOn(view.app, 'accountId').mockReturnValue('acct');
    const upload = vi.spyOn(view.app, 'uploadComposeBlob').mockResolvedValue({ name: 'New Person.vcf', blobId: 'blob-1', size: 99, contentType: 'text/vcard' });
    open();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Send contact by email' }));
    await waitFor(() => expect(view.compose).toHaveBeenCalledTimes(1));
    expect(upload).toHaveBeenCalledWith('acct', 'New Person.vcf', expect.stringContaining('BEGIN:VCARD'), 'text/vcard');
    const draft = view.compose.mock.calls[0]![0];
    expect(draft.attachments).toEqual([{ name: 'New Person.vcf', blobId: 'blob-1', size: 99, contentType: 'text/vcard' }]);
    expect(draft.subject).toBe('New Person'); expect(draft.bodyText).toBe('New Person\nnew@example.org');
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('puts the vCard in the message text when the file cannot be attached, and closes the menu with Escape', async () => {
    const view = setup(); view.open();
    await screen.findByRole('button', { name: 'Save to contacts' });
    const trigger = screen.getByRole('button', { name: 'Share contact' });
    fireEvent.click(trigger);
    const menu = screen.getByRole('menu', { name: 'Share contact' });
    await waitFor(() => expect(within(menu).getAllByRole('menuitem')[0]).toHaveFocus());
    fireEvent.keyDown(menu, { key: 'ArrowDown' });
    expect(within(menu).getAllByRole('menuitem')[1]).toHaveFocus();
    fireEvent.keyDown(menu, { key: 'Escape' });
    expect(screen.queryByRole('menu')).toBeNull(); expect(trigger).toHaveFocus(); expect(screen.getByRole('dialog')).toBeInTheDocument();
    fireEvent.click(trigger);
    fireEvent.click(screen.getByRole('menuitem', { name: 'Send contact by email' }));
    await waitFor(() => expect(view.compose).toHaveBeenCalledTimes(1));
    const draft = view.compose.mock.calls[0]![0];
    expect(draft.attachments).toEqual([]); expect(draft.bodyText).toContain('BEGIN:VCARD'); expect(draft.bodyText).toContain('FN:New Person');
  });
  it('offers the system share sheet only where it can take a file, and treats a dismissed sheet as no error', async () => {
    const share = vi.fn().mockRejectedValue(new DOMException('dismissed', 'AbortError'));
    Object.defineProperty(navigator, 'share', { value: share, configurable: true });
    Object.defineProperty(navigator, 'canShare', { value: () => true, configurable: true });
    try {
      const view = setup(); const toast = vi.spyOn(view.app, 'showToast'); view.open();
      await screen.findByRole('button', { name: 'Save to contacts' });
      fireEvent.click(screen.getByRole('button', { name: 'Share contact' }));
      fireEvent.click(screen.getByRole('menuitem', { name: 'Share…' }));
      await waitFor(() => expect(share).toHaveBeenCalledTimes(1));
      const payload = share.mock.calls[0]![0] as { files: File[]; title: string };
      expect(payload.files[0]!.name).toBe('New Person.vcf'); expect(payload.files[0]!.type).toBe('text/vcard');
      expect(toast).not.toHaveBeenCalledWith('error', expect.anything());
    } finally { Reflect.deleteProperty(navigator, 'share'); Reflect.deleteProperty(navigator, 'canShare'); }
  });
  it('reserves the first slot while the lookup resolves so nothing shifts', async () => {
    const view = setup(); view.open();
    const slot = screen.getByRole('button', { name: 'Save to contacts' });
    expect(slot).toBeDisabled();
    await waitFor(() => expect(slot).toBeEnabled());
  });
  it('shows Go to contact instead of Save for a saved address and navigates to it', async () => {
    const seed = defaultSeed(), target = seed.contacts![0]!;
    const view = setup({ client: makeContactsClient(seed), address: target.emails[0]!.value });
    const select = vi.spyOn(view.app, 'selectContact'); view.open();
    const go = await screen.findByRole('button', { name: 'Go to contact' });
    expect(screen.queryByRole('button', { name: 'Save to contacts' })).toBeNull();
    fireEvent.click(go);
    expect(select).toHaveBeenCalledWith(target.id);
    expect(window.location.hash).toBe(`#/contacts/${encodeURIComponent(target.id)}`);
    expect(screen.queryByRole('dialog')).toBeNull();
  });
  it('sends mail from the icon action', async () => {
    const view = setup(); view.open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send email' }));
    expect(view.compose).toHaveBeenCalledTimes(1);
  });
  it('copies name, address, phone and organisation as plain text with feedback', async () => {
    const seed = defaultSeed();
    seed.contacts![0] = { ...seed.contacts![0]!, organizations: ['Acme'], phones: [{ value: '+351123456789', context: 'work' }] };
    const target = seed.contacts![0]!, writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    const view = setup({ client: makeContactsClient(seed), address: target.emails[0]!.value });
    const toast = vi.spyOn(view.app, 'showToast'); view.open();
    await screen.findByRole('button', { name: 'Go to contact' });
    fireEvent.click(screen.getByRole('button', { name: 'Copy contact information' }));
    await waitFor(() => expect(toast).toHaveBeenCalledWith('success', 'Contact information copied.'));
    expect(writeText).toHaveBeenCalledWith(`${target.name.full} <${target.emails[0]!.value}>
+351123456789
Acme`);
  });
  it('falls back to execCommand when the clipboard API is unavailable, and reports failure otherwise', async () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
    const exec = vi.fn().mockReturnValue(true);
    Object.defineProperty(document, 'execCommand', { value: exec, configurable: true });
    const view = setup(); const toast = vi.spyOn(view.app, 'showToast'); view.open();
    await screen.findByRole('button', { name: 'Save to contacts' });
    fireEvent.click(screen.getByRole('button', { name: 'Copy contact information' }));
    await waitFor(() => expect(toast).toHaveBeenCalledWith('success', 'Contact information copied.'));
    expect(exec).toHaveBeenCalledWith('copy');
    exec.mockReturnValue(false);
    fireEvent.click(screen.getByRole('button', { name: 'Copy contact information' }));
    await waitFor(() => expect(toast).toHaveBeenCalledWith('error', 'Could not copy the contact information.'));
  });
});
