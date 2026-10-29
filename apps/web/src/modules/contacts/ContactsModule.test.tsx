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
  it('shows an initial loading skeleton rather than an empty contact list',()=>{const app={...createAppState(makeContactsClient(defaultSeed())),contactsLoading:()=>true,loadContacts:async()=>undefined};render(()=><AppContext.Provider value={app}><ContactsModule/></AppContext.Provider>);expect(document.querySelector('[data-skeleton="list"]')).not.toBeNull();});
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
    expect(screen.queryByRole('list', {name:'Contacts'})).not.toBeInTheDocument();
    expect(screen.queryByRole('searchbox')).not.toBeInTheDocument();
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

  it('renames an existing group in its exclusive editor',async()=>{
    const {app}=renderModule();await screen.findByText('Ada Lovelace');fireEvent.click(screen.getByRole('button',{name:'Edit group Colleagues'}));
    expect(screen.queryByRole('searchbox',{name:'Search contacts'})).not.toBeInTheDocument();expect(screen.getByRole('heading',{name:'Edit contact group'})).toBeInTheDocument();
    fireEvent.input(screen.getByLabelText('Group name'),{target:{value:'Team'}});expect(screen.getByLabelText('Address books')).toBeDisabled();fireEvent.click(screen.getByRole('button',{name:'Save'}));
    await waitFor(()=>expect(app.contactGroups().find(group=>group.id==='g1')?.name).toBe('Team'));expect(app.contactGroups().find(group=>group.id==='g1')?.memberIds).toEqual(['c1']);
  });

  // t39 R6: import, merge and membership are pages of their own (pages.tsx).
  it('imports contacts from CSV on the import page with a field mapping and leaves out unticked rows', async () => {
    const { app } = renderModule();
    await screen.findByText('Ada Lovelace');
    const before = app.contacts().length;
    fireEvent.click(screen.getByRole('button', { name: 'Import…' }));
    const page = await screen.findByTestId('contacts-import-page');
    expect(within(page).getByRole('heading', { level: 1, name: 'Import contacts' })).toHaveFocus();
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(screen.queryByRole('searchbox', { name: 'Search contacts' })).toBeNull();
    expect(within(page).getByRole('navigation', { name: 'Source' })).toBeInTheDocument();
    fireEvent.input(within(page).getByLabelText('Paste vCard or CSV'), {
      target: { value: 'Name,Email\nGrace Hopper,grace@example.org\nKatherine Johnson,kj@example.org\n' },
    });
    fireEvent.click(within(page).getByRole('button', { name: 'Preview' }));
    expect(await within(page).findByLabelText('Map column Name')).toHaveValue('fullName');
    expect(within(page).getByLabelText('Map column Email')).toHaveValue('email');
    const preview = within(page).getByRole('list', { name: 'Import preview' });
    expect(within(preview).getAllByRole('checkbox')).toHaveLength(2);
    fireEvent.click(within(preview).getAllByRole('checkbox')[1]!);
    fireEvent.click(within(page).getByRole('button', { name: 'Import 1 contact' }));
    expect(await within(page).findByRole('status')).toHaveTextContent('Imported 1 contact');
    expect(app.contacts().length).toBe(before + 1);
    expect(app.contacts().some((c) => c.name.full === 'Grace Hopper')).toBe(true);
    expect(app.contacts().some((c) => c.name.full === 'Katherine Johnson')).toBe(false);
    fireEvent.click(within(page).getByRole('button', { name: 'Done' }));
    expect(await screen.findByRole('searchbox', { name: 'Search contacts' })).toBeInTheDocument();
  });

  it('merges a duplicate set on the merge page, keeping the chosen card', async () => {
    const seed = defaultSeed();
    seed.contacts = [
      { ...seed.contacts[0]!, id: 'd1', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'grace@x.org', pref: 1 }] },
      { ...seed.contacts[1]!, id: 'd2', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'grace@home.org', pref: 0 }] },
    ];
    const { app } = renderModule(seed);
    await screen.findAllByText('Grace Hopper');
    fireEvent.click(screen.getByRole('button', { name: 'Find duplicates' }));
    const page = await screen.findByTestId('contacts-merge-page');
    const sets = within(page).getByRole('navigation', { name: 'Possible duplicates' });
    expect(within(sets).getByRole('button', { name: /Grace Hopper/ })).toHaveAttribute('aria-current', 'true');
    const keep = within(page).getAllByRole('radio');
    expect(keep).toHaveLength(2);
    expect(keep[0]).toBeChecked();
    fireEvent.click(keep[1]!);
    const preview = within(page).getByRole('region', { name: 'Merged preview' });
    expect(within(preview).getByText('grace@x.org')).toBeInTheDocument();
    expect(within(preview).getByText('grace@home.org')).toBeInTheDocument();
    fireEvent.click(within(page).getByRole('button', { name: 'Merge contacts' }));
    await waitFor(() => expect(app.contacts().map((c) => c.id)).toEqual(['d2']));
    expect(await within(page).findByRole('status')).toHaveTextContent('Merged into Grace Hopper.');
    expect(within(page).getByText('No duplicates found.')).toBeInTheDocument();
  });

  it('manages the members of a group on the membership page with the multi-select', async () => {
    const { app } = renderModule();
    await screen.findByText('Ada Lovelace');
    fireEvent.click(screen.getByRole('button', { name: 'Group membership' }));
    const page = await screen.findByTestId('contacts-membership-page');
    expect(within(page).getByRole('button', { name: /Colleagues/ })).toHaveAttribute('aria-current', 'true');
    const picker = within(page).getByRole('combobox', { name: 'Members' });
    expect(within(page).getByRole('button', { name: 'Remove Ada Lovelace' })).toBeInTheDocument();
    fireEvent.input(picker, { target: { value: 'alan' } });
    fireEvent.keyDown(picker, { key: 'Enter' });
    await waitFor(() => expect(app.contactGroups().find((g) => g.id === 'g1')!.memberIds).toEqual(['c1', 'c2']));
    await waitFor(() => expect(picker).toBeEnabled());
    fireEvent.click(within(page).getByRole('button', { name: 'Remove Ada Lovelace' }));
    await waitFor(() => expect(app.contactGroups().find((g) => g.id === 'g1')!.memberIds).toEqual(['c2']));
    expect(within(page).getByText('1 member')).toBeInTheDocument();
    fireEvent.click(within(page).getByRole('button', { name: 'Back to contacts' }));
    expect(await screen.findByRole('searchbox', { name: 'Search contacts' })).toBeInTheDocument();
  });

  it('creates a group and adds the open contact to it from the business card', async () => {
    const { app } = renderModule();
    fireEvent.click(await screen.findByText('Ada Lovelace'));
    fireEvent.click(screen.getByRole('button', { name: 'New group' }));
    expect(screen.queryByRole('searchbox',{name:'Search contacts'})).not.toBeInTheDocument();
    fireEvent.input(await screen.findByLabelText('New group name'), { target: { value: 'Friends' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(app.contactGroups().some((g) => g.name === 'Friends')).toBe(true));
    const gid = app.contactGroups().find((g) => g.name === 'Friends')!.id;
    const card = await screen.findByRole('article', { name: 'Contact Ada Lovelace' });
    expect(within(card).queryByRole('checkbox')).toBeNull();
    const groups = within(card).getByRole('combobox', { name: 'Member of' });
    fireEvent.keyDown(groups, { key: 'ArrowDown' });
    fireEvent.click(within(card).getByRole('option', { name: 'Friends' }));
    await waitFor(() => expect(app.contactGroups().find((g) => g.id === gid)!.memberIds).toContain('c1'));
    fireEvent.click(within(card).getByRole('button', { name: 'Manage group membership' }));
    expect(await screen.findByTestId('contacts-membership-page')).toBeInTheDocument();
  });
});
