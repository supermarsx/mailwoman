// Favorites for contacts: the store actions (optimistic, rolled back, bulk), the
// contacts module (sidebar entry, filter, list rows, bulk bar, context menu, the
// share control on the card) and the recipient suggestions ranking.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import { createContactsSlice, type ContactsSlice } from '../../state/slices/contacts.ts';
import type { SliceContext } from '../../state/slices/context.ts';
import type { Client } from '../../api/client.ts';
import type { ContactCard } from '../../api/pim-types.ts';
import { ContactsModule } from './index.tsx';
import { defaultSeed, makeContactsClient, type ContactsSeed } from './mockClient.ts';
import { favoriteSuggestions, rankSuggestions } from './autocomplete.ts';
import { pendingCompose, takePendingCompose } from '../../state/pendingCompose.ts';
import { settings } from '../../api/settings.ts';

afterEach(() => { cleanup(); vi.restoreAllMocks(); takePendingCompose(); });
beforeEach(() => localStorage.clear());

function refuseWrites(client: Client): void {
  const original = client.jmap.bind(client);
  vi.spyOn(client, 'jmap').mockImplementation(async (request, options) => request.methodCalls.some(([method]) => method === 'ContactCard/set') ? Promise.reject(new Error('offline')) : original(request, options));
}

function withSlice(seed: ContactsSeed, run: (slice: ContactsSlice, client: Client) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const client = makeContactsClient(seed);
      const ctx: SliceContext = { client, showToast: () => undefined };
      run(createContactsSlice(ctx), client).then(resolve, reject).finally(dispose);
    });
  });
}

function renderModule(seed: ContactsSeed = defaultSeed()): { app: AppState; client: Client } {
  const client = makeContactsClient(seed);
  const app = createAppState(client);
  render(() => <AppContext.Provider value={app}>{ContactsModule()}</AppContext.Provider>);
  return { app, client };
}

describe('favorites in the store', () => {
  it('flips the flag before the server answers and keeps it when the server confirms', async () => {
    await withSlice(defaultSeed(), async (slice) => {
      await slice.loadContacts();
      expect(slice.favoriteCount()).toBe(1);
      const pending = slice.toggleFavorite('c2');
      expect(slice.contactById('c2')!.isFavorite).toBe(true);
      expect(slice.favoriteCount()).toBe(2);
      expect(await pending).toBe(true);
      expect(slice.contactById('c2')!.isFavorite).toBe(true);
    });
  });
  it('rolls the flag back when the write is refused or fails', async () => {
    await withSlice(defaultSeed(), async (slice, client) => {
      await slice.loadContacts();
      refuseWrites(client);
      const pending = slice.toggleFavorite('c2');
      expect(slice.contactById('c2')!.isFavorite).toBe(true);
      expect(await pending).toBe(false);
      expect(slice.contactById('c2')!.isFavorite).toBe(false);
      expect(slice.favoriteCount()).toBe(1);
    });
  });
  it('sets and clears many contacts in one request, and reports only the ones that stuck', async () => {
    await withSlice(defaultSeed(), async (slice, client) => {
      await slice.loadContacts();
      const jmap = vi.spyOn(client, 'jmap');
      expect(await slice.setFavorite(['c1', 'c2', 'missing'], true)).toEqual(['c2']);
      const writes = jmap.mock.calls.flatMap(([request]) => request.methodCalls.filter(([method]) => method === 'ContactCard/set'));
      expect(writes).toHaveLength(1);
      expect(Object.keys((writes[0]![1] as { update: object }).update).sort()).toEqual(['c2', 'missing']);
      expect(slice.favoriteCount()).toBe(2);
      expect(await slice.setFavorite(['c1', 'c2'], false)).toEqual(['c1', 'c2']);
      expect(slice.favoriteCount()).toBe(0);
      expect(slice.contacts().every((card) => !card.isFavorite)).toBe(true);
    });
  });
  it('ignores a second request for a contact whose first is still in flight', async () => {
    await withSlice(defaultSeed(), async (slice, client) => {
      await slice.loadContacts();
      const jmap = vi.spyOn(client, 'jmap');
      const first = slice.toggleFavorite('c2'), second = slice.toggleFavorite('c2');
      expect(await first).toBe(true); expect(await second).toBe(false);
      expect(jmap.mock.calls.flatMap(([request]) => request.methodCalls.filter(([method]) => method === 'ContactCard/set'))).toHaveLength(1);
      expect(slice.contactById('c2')!.isFavorite).toBe(true);
    });
  });
  it('lists favorites through the favorites filter, within a chosen address book', async () => {
    await withSlice(defaultSeed(), async (slice) => {
      await slice.loadContacts();
      await slice.setFavorite(['c2'], true);
      slice.setFavoritesOnly(true);
      expect(slice.filteredContacts().map((card) => card.id)).toEqual(['c1', 'c2']);
      slice.selectAddressBook('other-book');
      expect(slice.filteredContacts()).toEqual([]);
    });
  });
});

describe('favorites in the contacts module', () => {
  it('shows the Favorites entry with its count and filters the list to favorites', async () => {
    renderModule();
    await screen.findByText('Alan Turing');
    const entry = screen.getByRole('button', { name: /Favorites/ });
    expect(within(entry).getByText('1')).toBeInTheDocument();
    fireEvent.click(entry);
    expect(entry).toHaveAttribute('aria-current', 'true');
    const list = screen.getByRole('list', { name: 'Contact list' });
    expect(within(list).getByText('Ada Lovelace')).toBeInTheDocument();
    expect(within(list).queryByText('Alan Turing')).toBeNull();
  });
  it('says there are no favorites instead of "no contacts" when the filter is empty', async () => {
    const seed = defaultSeed(); seed.contacts![0] = { ...seed.contacts![0]!, isFavorite: false };
    renderModule(seed);
    await screen.findByText('Alan Turing');
    fireEvent.click(screen.getByRole('button', { name: /Favorites/ }));
    expect(screen.getByText('No favorites yet. Use the star on a contact to add one.')).toBeInTheDocument();
  });
  it('flips a row star at once, updates the count, and undoes it with an error when the write fails', async () => {
    const { app, client } = renderModule();
    await screen.findByText('Alan Turing');
    refuseWrites(client);
    const toast = vi.spyOn(app, 'showToast');
    const star = screen.getByRole('button', { name: 'Favorite Alan Turing' });
    expect(star).toHaveAttribute('title', 'Add to favorites');
    star.focus();
    fireEvent.click(star);
    expect(star).toHaveAttribute('aria-pressed', 'true');
    expect(star).toHaveFocus(); // the row is kept, so keyboard focus stays on the star
    await waitFor(() => expect(star).toHaveAttribute('aria-pressed', 'false'));
    expect(star).toBe(screen.getByRole('button', { name: 'Favorite Alan Turing' }));
    expect(within(screen.getByRole('button', { name: /Favorites/ })).getByText('1')).toBeInTheDocument();
    expect(app.contactById('c2')!.isFavorite).toBe(false);
    expect(toast).not.toHaveBeenCalledWith('success', expect.anything());
  });
  it('adds and removes favorites for the selected rows in one action', async () => {
    const { app } = renderModule();
    await screen.findByText('Alan Turing');
    expect(screen.queryByRole('group', { name: 'Actions for the selected contacts' })).toBeNull();
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select Alan Turing' }));
    expect(screen.getByRole('checkbox', { name: 'Select Alan Turing' })).toBeChecked();
    expect(app.selectedContactId()).toBeNull(); // selecting a row does not open it
    const bar = screen.getByRole('group', { name: 'Actions for the selected contacts' });
    expect(within(bar).getByRole('status')).toHaveTextContent('1 selected');
    fireEvent.click(within(bar).getByRole('button', { name: 'Add to favorites' }));
    await waitFor(() => expect(app.contactById('c2')!.isFavorite).toBe(true));
    await waitFor(() => expect(screen.queryByRole('group', { name: 'Actions for the selected contacts' })).toBeNull());
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select Alan Turing' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select Ada Lovelace' }));
    fireEvent.click(within(screen.getByRole('group', { name: 'Actions for the selected contacts' })).getByRole('button', { name: 'Remove from favorites' }));
    await waitFor(() => expect(app.favoriteCount()).toBe(0));
  });
  it('clears the selection', async () => {
    renderModule();
    await screen.findByText('Alan Turing');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select Alan Turing' }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear selection' }));
    expect(screen.getByRole('checkbox', { name: 'Select Alan Turing' })).not.toBeChecked();
  });
  it('offers favorite, send email and share in the row context menu', async () => {
    const { app } = renderModule();
    const row = (await screen.findByText('Alan Turing')).closest('[role="button"]') as HTMLElement;
    fireEvent.contextMenu(row);
    const menu = await screen.findByRole('menu');
    const names = within(menu).getAllByRole('menuitem').map((item) => item.textContent!.trim());
    expect(names).toEqual(['Add to favorites', 'Send email', 'Share contact']);
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'Add to favorites' }));
    await waitFor(() => expect(app.contactById('c2')!.isFavorite).toBe(true));
  });
  it('sends an email to the contact from the context menu through the composer hand-over', async () => {
    const { app } = renderModule();
    const row = (await screen.findByText('Alan Turing')).closest('[role="button"]') as HTMLElement;
    fireEvent.contextMenu(row);
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Send email' }));
    expect(pendingCompose()?.to).toContain('alan@example.org');
    expect(app.selectedContactId()).toBeNull();
  });
  it('shares from the business card: the star, the share menu and the composer with the .vcf attached', async () => {
    const { app } = renderModule();
    fireEvent.click(await screen.findByText('Alan Turing'));
    const card = await screen.findByRole('article', { name: 'Contact Alan Turing' });
    expect(within(card).getByRole('button', { name: 'Favorite Alan Turing' })).toHaveAttribute('aria-pressed', 'false');
    vi.spyOn(app, 'accountId').mockReturnValue('acct1');
    vi.spyOn(app, 'uploadComposeBlob').mockResolvedValue({ name: 'Alan Turing.vcf', blobId: 'blob-9', size: 5, contentType: 'text/vcard' });
    fireEvent.click(within(card).getByRole('button', { name: 'Share contact' }));
    expect(within(await screen.findByRole('menu')).getAllByRole('menuitem').map((item) => item.textContent!.trim())).toEqual(['Send contact by email', 'Copy as vCard', 'Download .vcf']);
    fireEvent.click(screen.getByRole('menuitem', { name: 'Send contact by email' }));
    await waitFor(() => expect(pendingCompose()).not.toBeNull());
    expect(pendingCompose()!.attachments).toEqual([{ name: 'Alan Turing.vcf', blobId: 'blob-9', size: 5, contentType: 'text/vcard' }]);
    expect(pendingCompose()!.subject).toBe('Alan Turing');
  });
});

describe('favorites in recipient suggestions', () => {
  const cards = (): ContactCard[] => {
    const seed = defaultSeed();
    const base = seed.contacts![0]!;
    const named = (id: string, full: string, email: string, favorite: boolean): ContactCard => ({ ...base, id, uid: id, name: { full, given: full.split(' ')[0]!, surname: full.split(' ')[1] ?? '', prefix: '', suffix: '' }, emails: [{ context: 'work', value: email, pref: 1 }], isFavorite: favorite });
    return [named('a', 'Sam Exact', 'sam@example.org', false), named('b', 'Samuel Starred', 'samuel@elsewhere.org', true), named('c', 'Ann Sam', 'ann@samuel.example', true)];
  };
  it('lists a favorite ahead of a better non-favorite match when the setting is on (default)', () => {
    const ranked = rankSuggestions(cards(), 'sam', 8);
    expect(ranked.map((row) => row.cardId)).toEqual(['b', 'c', 'a']);
  });
  it('keeps plain ranking when favorites-first is off', () => {
    const ranked = rankSuggestions(cards(), 'sam', 8, false);
    expect(ranked[0]!.cardId).toBe('a');
    expect(new Set(ranked.map((row) => row.cardId))).toEqual(new Set(['a', 'b', 'c']));
  });
  it('never hides a match: favorites are only moved', () => {
    expect(rankSuggestions(cards(), 'sam', 8, true)).toHaveLength(3);
    expect(rankSuggestions(cards(), 'zzz', 8, true)).toEqual([]);
  });
  it('reads the default from the settings registry: on unless a boolean false is set', () => {
    expect(settings()['contacts.favoritesFirst']).toBeUndefined();
    expect(rankSuggestions(cards(), 'sam')[0]!.isFavorite).toBe(true);
  });
  it('offers the favorites group for an empty query: preferred address, by name', () => {
    const list = favoriteSuggestions(cards());
    expect(list.map((row) => row.cardId)).toEqual(['c', 'b']);
    expect(list.every((row) => row.isFavorite)).toBe(true);
    expect(favoriteSuggestions(cards(), 1)).toHaveLength(1);
  });
});
