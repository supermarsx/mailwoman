// t107: the admin Users table — columns, server-driven search/filters/sort/paging
// (through the URL when the console shell is there), selection and bulk actions,
// column toggle and density remembered per admin, skeleton rows in place,
// keyboard row navigation and aria-sort.

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { UsersTable, Highlight, initials, parsePrefs, type UsersTableActions } from './UsersTable.tsx';
import { renderInShell } from '../shell/testkit.tsx';
import { requestQuery, viewFromQuery, viewToQuery, defaultView, dayEdge, type UserRow, type UserTablePage, type UserView } from '../../../api/admin/usersTable.ts';
import type { AdminRoute } from '../shell/router.ts';

const NOW = Date.now() / 1000;
function row(over: Partial<UserRow> & { accountId: string }): UserRow {
  const [username = over.accountId, domain = 'example.org'] = over.accountId.split('@');
  return {
    username, domain, loginAddress: over.accountId, directoryAddress: over.accountId, legacyBinding: false,
    displayName: null, status: 'active', state: 'active', deletedAt: null, lastSigninAt: null, lastSeenAt: null, createdAt: NOW - 86400 * 40,
    twoFactor: 'none', twoFactorMethods: [], adminRole: null, profile: null, linkedMailboxes: 0, quota: null,
    flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
    ...over,
  };
}
const ALICE = row({ accountId: 'alice@example.org', displayName: 'Alice Liddell', lastSeenAt: NOW - 3600, lastSigninAt: NOW - 3 * 3600, twoFactor: 'passkey', twoFactorMethods: ['passkey', 'totp'], adminRole: 'Helpdesk', profile: { id: 'p1', name: 'Staff', via: 'user' }, linkedMailboxes: 2 });
const BOB = row({ accountId: 'bob@other.test', state: 'pending' });
const CAROL = row({ accountId: 'carol@example.org', status: 'disabled', state: 'blocked', lastSigninAt: NOW - 86400 * 100 });

function pageOf(rows: UserRow[], over: Partial<UserTablePage> = {}): UserTablePage {
  return { rows, total: rows.length, unfilteredTotal: 3, offset: 0, limit: 50, facets: { domains: ['example.org', 'other.test'], roles: ['Helpdesk'], profiles: [{ id: 'p1', name: 'Staff' }, { id: 'p2', name: 'Guests' }] }, ...over };
}
function actions(over: Partial<UsersTableActions> = {}): UsersTableActions {
  return {
    load: vi.fn(async () => pageOf([ALICE, BOB, CAROL])),
    setLoginBlocked: vi.fn(async () => undefined),
    signOut: vi.fn(async () => undefined),
    deleteAccess: vi.fn(async () => undefined),
    assignProfile: vi.fn(async () => undefined),
    ...over,
  };
}

beforeEach(() => { try { localStorage.clear(); } catch { /* none */ } });

describe('Users table view model', () => {
  it('round-trips a view through the fragment query and drops unknown values', () => {
    const view: UserView = { ...defaultView(), sort: 'lastSignin', dir: 'desc', page: 2, filters: { ...defaultView().filters, q: 'ali', domain: ['example.org'], status: ['blocked', 'pending'], twoFactor: ['none'], signin: 'older:90', createdFrom: '2026-01-01', createdTo: null } };
    const query = viewToQuery(view);
    expect(query).toEqual({ q: 'ali', domain: 'example.org', status: 'blocked,pending', tfa: 'none', signin: 'older:90', created: '2026-01-01..', sort: 'lastSignin', dir: 'desc', page: '3' });
    expect(viewFromQuery(query)).toEqual(view);
    const junk = viewFromQuery({ status: 'active,bogus', tfa: 'sms', signin: 'older:x', sort: 'password', page: '-4', created: 'x..2026-13-45x' });
    expect(junk.filters.status).toEqual(['active']);
    expect(junk.filters.twoFactor).toEqual([]);
    expect(junk.filters.signin).toBeNull();
    expect(junk.sort).toBe('lastSeen');
    expect(junk.page).toBe(0);
    expect(junk.filters.createdFrom).toBeNull();
  });

  it('defaults to latest activity and preserves explicit sign-in/name sorts in shared URLs', () => {
    expect(defaultView()).toMatchObject({sort:'lastSeen',dir:'desc'});
    expect(viewToQuery(defaultView())).toEqual({});
    expect(viewFromQuery({sort:'name'})).toMatchObject({sort:'name',dir:'asc'});
    expect(viewFromQuery({sort:'lastSignin'})).toMatchObject({sort:'lastSignin',dir:'asc'});
    for(const sort of ['lastSeen','lastSignin','name'] as const)for(const dir of ['asc','desc'] as const){
      const view={...defaultView(),sort,dir};
      expect(viewFromQuery(viewToQuery(view))).toEqual(view);
    }
    expect(new URLSearchParams(requestQuery(defaultView())).get('sort')).toBe('lastSeen');
  });

  it('builds the API query with the page offset and local-day edges', () => {
    const view: UserView = { ...defaultView(), page: 1, pageSize: 25, filters: { ...defaultView().filters, twoFactor: ['passkey'], createdFrom: '2026-03-01', createdTo: '2026-03-01' } };
    const p = new URLSearchParams(requestQuery(view));
    expect(p.get('twoFactor')).toBe('passkey');
    expect(p.get('offset')).toBe('25');
    expect(p.get('limit')).toBe('25');
    expect(Number(p.get('createdTo')) - Number(p.get('createdFrom'))).toBe(86399);
    expect(dayEdge('2026-03-01', false)).toBe(new Date(2026, 2, 1).getTime() / 1000);
  });

  it('initials, highlight and stored preferences', () => {
    expect(initials({ displayName: 'Alice Liddell', username: 'alice' })).toBe('AL');
    expect(initials({ displayName: null, username: 'bob' })).toBe('BO');
    render(() => <p data-testid="h"><Highlight text="Alice ALI" needle="ali" /></p>);
    expect(screen.getByTestId('h').querySelectorAll('mark')).toHaveLength(2);
    expect(parsePrefs({ hidden: ['domain', 'user', 'nope'], density: 'compact', presets: [{ name: 'x', query: { status: 'blocked', n: 3 } }] }))
      .toEqual({ hidden: ['domain'], density: 'compact', presets: [{ name: 'x', query: { status: 'blocked' } }] });
    expect(parsePrefs('nope')).toBeNull();
  });
});

describe('Users table', () => {
  it('shows skeleton rows inside the table frame while the first page loads', async () => {
    let release!: (p: UserTablePage) => void;
    const load = vi.fn(() => new Promise<UserTablePage>((resolve) => { release = resolve; }));
    render(() => <UsersTable actions={actions({ load })} />);
    const table = await screen.findByTestId('users-table');
    expect(table).toHaveAttribute('aria-busy', 'true');
    expect(table.querySelectorAll('tbody tr[data-skeleton]').length).toBeGreaterThan(0);
    release(pageOf([ALICE]));
    expect(await screen.findByText('Alice Liddell')).toBeInTheDocument();
    expect(table.querySelectorAll('tbody tr[data-skeleton]')).toHaveLength(0);
    expect(table).toHaveAttribute('aria-busy', 'false');
  });

  it('renders the columns: person, status, 2FA, role, relative last sign-in with exact tooltip, Never, profile, linked', async () => {
    render(() => <UsersTable actions={actions()} />);
    const alice = (await screen.findByText('Alice Liddell')).closest('tr')!;
    expect(within(alice).getByText('alice@example.org')).toBeInTheDocument();
    expect(within(alice).getByText('AL')).toBeInTheDocument();
    expect(within(alice).getByText('Active')).toBeInTheDocument();
    expect(within(alice).getByText('Passkey + Authenticator app')).toBeInTheDocument();
    expect(within(alice).getByText('Helpdesk')).toBeInTheDocument();
    const time = alice.querySelector('td[data-column="lastSignin"] time')!;
    expect(time.textContent).toBe('3 hours ago');
    expect(time.getAttribute('title')).toMatch(/\d{4}/u);
    const seen=alice.querySelector('td[data-column="lastSeen"] time')!;
    expect(seen.textContent).toBe('1 hour ago');
    expect(seen.getAttribute('datetime')).not.toBe(time.getAttribute('datetime'));
    expect(seen.getAttribute('aria-label')).toMatch(/^Last seen /u);
    expect(within(alice).getByText('Staff')).toBeInTheDocument();
    expect(alice.querySelector('td[data-column="linked"]')).toHaveTextContent('2');
    const bob = screen.getByRole('link', { name: 'bob@other.test' }).closest('tr')!;
    expect(within(bob).getByText('Never signed in')).toBeInTheDocument();
    expect(bob.querySelector('td[data-column="lastSignin"]')).toHaveTextContent('Never');
    expect(bob.querySelector('td[data-column="lastSeen"]')).toHaveTextContent('No activity recorded');
    const carol=screen.getByRole('link',{name:'carol@example.org'}).closest('tr')!;
    expect(carol.querySelector('td[data-column="lastSeen"] time')).toBeNull();
    expect(screen.getByRole('link', { name: 'Configure alice@example.org' })).toBeInTheDocument();
    expect(screen.getByTestId('users-count')).toHaveTextContent('3 users');
  });

  it('sorts on the server and marks the header with aria-sort', async () => {
    const a = actions();
    render(() => <UsersTable actions={a} />);
    await screen.findByText('Alice Liddell');
    const header = screen.getByRole('columnheader', { name: /Last sign-in/u });
    expect(header).toHaveAttribute('aria-sort', 'none');
    expect(screen.getByRole('columnheader', { name: /Last seen/u })).toHaveAttribute('aria-sort', 'descending');
    expect(screen.getByRole('columnheader', { name: /User/u })).toHaveAttribute('aria-sort', 'none');
    fireEvent.click(within(header).getByRole('button'));
    await waitFor(() => expect(header).toHaveAttribute('aria-sort', 'descending'));
    const last = vi.mocked(a.load).mock.calls.at(-1)![0];
    expect(last.sort).toBe('lastSignin');
    expect(last.dir).toBe('desc');
    fireEvent.click(within(header).getByRole('button'));
    await waitFor(() => expect(header).toHaveAttribute('aria-sort', 'ascending'));
  });

  it('debounces the search, highlights matches and says how many match', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      const load = vi.fn(async (view: UserView) => (view.filters.q ? pageOf([ALICE], { total: 1 }) : pageOf([ALICE, BOB, CAROL])));
      render(() => <UsersTable actions={actions({ load })} />);
      await screen.findByText('Alice Liddell');
      const box = screen.getByRole('searchbox', { name: 'Search users' });
      fireEvent.input(box, { target: { value: 'a' } });
      fireEvent.input(box, { target: { value: 'ali' } });
      expect(load).toHaveBeenCalledTimes(1);
      await vi.advanceTimersByTimeAsync(300);
      await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
      expect(load.mock.calls[1]![0].filters.q).toBe('ali');
      await waitFor(() => expect(screen.getByTestId('users-count')).toHaveTextContent('1 of 3 users matches'));
      expect(document.querySelectorAll('mark').length).toBeGreaterThan(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it('adds, edits and removes filter chips; Clear all drops them', async () => {
    const a = actions();
    render(() => <UsersTable actions={a} />);
    await screen.findByText('Alice Liddell');
    fireEvent.click(screen.getByTestId('users-add-filter'));
    fireEvent.click(await screen.findByRole('menuitemcheckbox', { name: 'Status' }));
    const editor = await screen.findByTestId('users-filter-editor');
    fireEvent.click(within(editor).getByLabelText('Blocked'));
    fireEvent.click(within(editor).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(vi.mocked(a.load).mock.calls.at(-1)![0].filters.status).toEqual(['blocked']));
    const chip = screen.getByRole('button', { name: 'Status: Blocked' });
    expect(chip).toBeInTheDocument();

    fireEvent.click(screen.getByTestId('users-add-filter'));
    fireEvent.click(await screen.findByRole('menuitemcheckbox', { name: 'Last sign-in' }));
    const signin = await screen.findByTestId('users-filter-editor');
    fireEvent.click(within(signin).getByLabelText('Never'));
    fireEvent.click(within(signin).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(vi.mocked(a.load).mock.calls.at(-1)![0].filters.signin).toBe('never'));

    fireEvent.click(screen.getByRole('button', { name: 'Remove filter Status: Blocked' }));
    await waitFor(() => expect(vi.mocked(a.load).mock.calls.at(-1)![0].filters.status).toEqual([]));
    fireEvent.click(screen.getByRole('button', { name: 'Clear all' }));
    await waitFor(() => expect(vi.mocked(a.load).mock.calls.at(-1)![0].filters.signin).toBeNull());
  });

  it('keeps the view in the URL fragment when the console shell is there', async () => {
    const [route, setRoute] = createSignal<AdminRoute>({ section: 'users', rest: [], query: { status: 'blocked', sort: 'lastSignin', dir: 'desc' } });
    const a = actions();
    const { navigate } = renderInShell(() => <UsersTable actions={a} />, { route, navigate: (p) => { const q = p.split('?')[1] ?? ''; setRoute({ section: 'users', rest: [], query: Object.fromEntries(new URLSearchParams(q)) }); } });
    await screen.findByText('Alice Liddell');
    expect(vi.mocked(a.load).mock.calls[0]![0].filters.status).toEqual(['blocked']);
    expect(screen.getByRole('button', { name: 'Status: Blocked' })).toBeInTheDocument();
    fireEvent.click(within(screen.getByRole('columnheader', { name: /Domain/u })).getByRole('button'));
    await waitFor(() => expect(navigate).toHaveBeenCalled());
    expect(String(navigate.mock.calls.at(-1)![0])).toMatch(/#\/users\?status=blocked&sort=domain/u);
  });

  it('selects rows and blocks the eligible ones in bulk, one request each, then reloads', async () => {
    const a = actions();
    render(() => <UsersTable actions={a} />);
    await screen.findByText('Alice Liddell');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select every user on this page' }));
    const bar = screen.getByTestId('users-bulk');
    expect(bar).toHaveTextContent('3 selected');
    fireEvent.click(within(bar).getByRole('button', { name: 'Block login' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('Block login for 2 users?');
    expect(dialog).toHaveTextContent('1 selected user is skipped');
    const loads = vi.mocked(a.load).mock.calls.length;
    fireEvent.click(within(dialog).getByRole('button', { name: 'Block login' }));
    await waitFor(() => expect(a.setLoginBlocked).toHaveBeenCalledTimes(2));
    expect(a.setLoginBlocked).toHaveBeenCalledWith('alice@example.org', true);
    expect(a.setLoginBlocked).toHaveBeenCalledWith('bob@other.test', true);
    await waitFor(() => expect(vi.mocked(a.load).mock.calls.length).toBe(loads + 1));
    await waitFor(() => expect(screen.queryByTestId('users-bulk')).not.toBeInTheDocument());
  });

  it('unblocks from the row menu and assigns a profile in bulk', async () => {
    const a = actions();
    render(() => <UsersTable actions={a} />);
    await screen.findByText('Alice Liddell');
    fireEvent.click(screen.getByRole('button', { name: 'More actions for carol@example.org' }));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Unblock login' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('Unblock login for carol@example.org?');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Unblock login' }));
    await waitFor(() => expect(a.setLoginBlocked).toHaveBeenCalledWith('carol@example.org', false));

    fireEvent.click(await screen.findByRole('checkbox', { name: 'Select alice@example.org' }));
    fireEvent.click(within(screen.getByTestId('users-bulk')).getByRole('button', { name: 'Assign feature profile' }));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Guests' }));
    const confirm = await screen.findByRole('alertdialog');
    fireEvent.click(within(confirm).getByRole('button', { name: 'Change profile' }));
    await waitFor(() => expect(a.assignProfile).toHaveBeenCalledWith('alice@example.org', 'p2', 'p1'));
  });

  it('hides a column and switches density, remembered per admin', async () => {
    const first = render(() => <UsersTable actions={actions()} />);
    await screen.findByText('Alice Liddell');
    fireEvent.click(screen.getByRole('button', { name: 'Columns' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Domain' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compact rows' }));
    expect(screen.queryByRole('columnheader', { name: /^Domain/u })).not.toBeInTheDocument();
    first.unmount();
    render(() => <UsersTable actions={actions()} />);
    await screen.findByText('Alice Liddell');
    expect(screen.queryByRole('columnheader', { name: /^Domain/u })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Compact rows' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('moves between rows with the arrow keys and selects with Space', async () => {
    render(() => <UsersTable actions={actions()} />);
    await screen.findByText('Alice Liddell');
    const rows = document.querySelectorAll<HTMLTableRowElement>('tbody tr[data-row]');
    expect(rows[0]).toHaveAttribute('tabindex', '0');
    expect(rows[1]).toHaveAttribute('tabindex', '-1');
    rows[0]!.focus();
    fireEvent.keyDown(rows[0]!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(rows[1]);
    fireEvent.keyDown(rows[1]!, { key: 'End' });
    expect(document.activeElement).toBe(rows[2]);
    fireEvent.keyDown(rows[2]!, { key: ' ' });
    expect(screen.getByRole('checkbox', { name: 'Select carol@example.org' })).toBeChecked();
  });

  it('says when nothing matches and offers to clear the filters', async () => {
    const load = vi.fn(async (view: UserView) => (view.filters.status.length ? pageOf([], { total: 0 }) : pageOf([ALICE])));
    const [route] = createSignal<AdminRoute>({ section: 'users', rest: [], query: { status: 'deleted' } });
    renderInShell(() => <UsersTable actions={actions({ load })} />, { route });
    expect(await screen.findByTestId('users-empty')).toHaveTextContent('No users match');
  });
});
