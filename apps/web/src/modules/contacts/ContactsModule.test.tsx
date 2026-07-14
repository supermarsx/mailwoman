// Component tests for the contacts module (plan §3 e7 acceptance): list/detail/
// edit, favorite toggle, import preview + CSV field mapping, merge flow, and
// group management — driven through the real store slice over the mock backend.

import { describe, it, expect, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import { ContactsModule } from './index.tsx';
import { makeContactsClient, defaultSeed, type ContactsSeed } from './mockClient.ts';

function renderModule(seed: ContactsSeed = defaultSeed()): { app: AppState } {
  const app = createAppState(makeContactsClient(seed));
  render(() => <AppContext.Provider value={app}>{ContactsModule()}</AppContext.Provider>);
  return { app };
}

describe('ContactsModule', () => {
  beforeEach(() => localStorage.clear());

  it('lists the account contacts after load', async () => {
    renderModule();
    expect(await screen.findByText('Ada Lovelace')).toBeInTheDocument();
    expect(screen.getByText('Alan Turing')).toBeInTheDocument();
  });

  it('opens a contact into the business-card detail view', async () => {
    renderModule();
    fireEvent.click(await screen.findByText('Alan Turing'));
    const card = await screen.findByRole('article', { name: 'Contact Alan Turing' });
    expect(within(card).getByText('alan@example.org')).toBeInTheDocument();
  });

  it('toggles a contact favorite from the list row', async () => {
    const { app } = renderModule();
    await screen.findByText('Alan Turing');
    const star = screen.getByRole('button', { name: 'Favorite Alan Turing' });
    expect(star).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(star);
    await waitFor(() => expect(app.contactById('c2')!.isFavorite).toBe(true));
    expect(screen.getByRole('button', { name: 'Favorite Alan Turing' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('edits a contact and reflects the new name', async () => {
    const { app } = renderModule();
    fireEvent.click(await screen.findByText('Ada Lovelace'));
    fireEvent.click(await screen.findByRole('button', { name: 'Edit' }));
    const name = await screen.findByLabelText('Full name');
    fireEvent.input(name, { target: { value: 'Ada King' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(app.contactById('c1')!.name.full).toBe('Ada King'));
    expect(await screen.findByRole('article', { name: 'Contact Ada King' })).toBeInTheDocument();
  });

  it('creates a new contact', async () => {
    const { app } = renderModule();
    await screen.findByText('Ada Lovelace');
    fireEvent.click(screen.getByRole('button', { name: 'New contact' }));
    fireEvent.input(await screen.findByLabelText('Full name'), { target: { value: 'Grace Hopper' } });
    fireEvent.input(screen.getByLabelText('Email 1'), { target: { value: 'grace@example.org' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(app.contacts().some((c) => c.name.full === 'Grace Hopper')).toBe(true));
  });

  it('imports contacts from CSV with a field mapping preview', async () => {
    const { app } = renderModule();
    await screen.findByText('Ada Lovelace');
    const before = app.contacts().length;
    fireEvent.click(screen.getByRole('button', { name: 'Import…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Import contacts' });
    fireEvent.input(within(dialog).getByLabelText('Paste vCard or CSV'), {
      target: { value: 'Name,Email\nGrace Hopper,grace@example.org\n' },
    });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Preview' }));
    // The CSV mapping surfaces a per-column selector, defaulted from the header.
    const map = await within(dialog).findByLabelText('Map column Name');
    expect(map).toHaveValue('fullName');
    expect(within(dialog).getByLabelText('Map column Email')).toHaveValue('email');
    fireEvent.click(within(dialog).getByRole('button', { name: /Import 1 contact/ }));
    await waitFor(() => expect(app.contacts().length).toBe(before + 1));
    expect(app.contacts().some((c) => c.name.full === 'Grace Hopper')).toBe(true);
  });

  it('merges two duplicate contacts through the review flow', async () => {
    const seed = defaultSeed();
    seed.contacts = [
      { ...seed.contacts[0]!, id: 'd1', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'grace@x.org', pref: 1 }] },
      { ...seed.contacts[1]!, id: 'd2', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'grace@home.org', pref: 0 }] },
    ];
    const { app } = renderModule(seed);
    await screen.findAllByText('Grace Hopper');
    fireEvent.click(screen.getByRole('button', { name: 'Find duplicates' }));
    const dialog = await screen.findByRole('dialog', { name: 'Merge duplicates' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Review merge' }));
    // Preview shows both unioned emails before committing.
    const preview = await within(dialog).findByRole('article', { name: 'Merged preview' });
    expect(within(preview).getByText('grace@x.org')).toBeInTheDocument();
    expect(within(preview).getByText('grace@home.org')).toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Merge contacts' }));
    await waitFor(() => expect(app.contacts().map((c) => c.id)).toEqual(['d1']));
  });

  // V7 (e14b): the per-contact directory Security tab, gated on a configured GAL.
  it('shows the directory security tab on a contact card when a directory is configured', async () => {
    const directoryFetcher = async (input: string): Promise<Response> => {
      if (input.includes('/api/directory/cert')) {
        return new Response(
          JSON.stringify({ certs: [{ derB64: 'AA', fingerprint: 'AB:CD:EF', notAfter: '2999-01-01' }] }),
          { status: 200, headers: { 'content-type': 'application/json' } },
        );
      }
      if (input.includes('/api/directory/photo')) {
        return new Response(JSON.stringify({ photoB64: null }), { status: 200 });
      }
      return new Response('{}', { status: 200 });
    };
    const app = createAppState(makeContactsClient(defaultSeed()), { directoryFetcher });
    app.directory.setEnabled(true);
    render(() => <AppContext.Provider value={app}>{ContactsModule()}</AppContext.Provider>);
    fireEvent.click(await screen.findByText('Alan Turing'));
    const sec = await screen.findByTestId('contact-security');
    expect(within(sec).getByText('alan@example.org')).toBeInTheDocument();
    expect(await within(sec).findByTestId('cert-row')).toBeInTheDocument();
  });

  it('hides the directory security tab when no directory is configured', async () => {
    renderModule();
    fireEvent.click(await screen.findByText('Alan Turing'));
    await screen.findByRole('article', { name: 'Contact Alan Turing' });
    expect(screen.queryByTestId('contact-security')).toBeNull();
  });

  // a11y (t8-e2): the import/merge dialogs are self-contained modals — Escape
  // closes them and focus returns to the invoking control.
  it('closes the import dialog on Escape and restores focus to the opener', async () => {
    renderModule();
    await screen.findByText('Ada Lovelace');
    const opener = screen.getByRole('button', { name: 'Import…' });
    opener.focus();
    fireEvent.click(opener);
    const dialog = await screen.findByRole('dialog', { name: 'Import contacts' });
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Import contacts' })).toBeNull());
    expect(document.activeElement).toBe(opener);
  });

  it('creates a group and adds a contact to it', async () => {
    const { app } = renderModule();
    fireEvent.click(await screen.findByText('Ada Lovelace'));
    // Create a new group from the sidebar.
    fireEvent.click(screen.getByRole('button', { name: 'New group' }));
    fireEvent.input(await screen.findByLabelText('New group name'), { target: { value: 'Friends' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(app.contactGroups().some((g) => g.name === 'Friends')).toBe(true));
    const gid = app.contactGroups().find((g) => g.name === 'Friends')!.id;
    // Toggle membership from the open contact's business card.
    fireEvent.click(await screen.findByLabelText('Friends membership'));
    await waitFor(() => expect(app.contactGroups().find((g) => g.id === gid)!.memberIds).toContain('c1'));
  });
});

