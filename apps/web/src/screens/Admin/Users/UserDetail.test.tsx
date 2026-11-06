import { createSignal } from 'solid-js';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminContext } from '../context.ts';
import { createAdminSlice, type UserSummary } from '../../../state/slices/admin.ts';
import { mockAdminApi } from '../testkit.tsx';
import { renderInShell, holding } from '../shell/testkit.tsx';
import { UserDetail } from './UserDetail.tsx';
import { resolveUserTab, USER_TABS, visibleUserTabs } from './userTabs.ts';
import { adminRequest } from '../shell/adminApi.ts';
import { listDataRequests } from '../../../api/admin/dataRequests.ts';

vi.mock('../shell/adminApi.ts', () => ({
  adminRequest: vi.fn(async () => ({ status: 200, etag: null, data: { accountId: 'acc-1', address: 'alice@example.test', label: 'Alice (alice@example.test)' } })),
}));
vi.mock('../../../api/admin/dataRequests.ts', () => ({
  listDataRequests: vi.fn(async () => ({ requests: [{ id: 'r1', kind: 'export', status: 'queued', address: 'alice@example.test', dueAt: 1_800_000_000 }], matched: 1 })),
}));
vi.mock('../Features/api.ts', () => ({ featuresApi: { load: vi.fn(async () => ({ data: { profiles: [], assignments: [], bindings: [], nativeTargets: [], userTargets: [] } })) } }));

const user = { accountId: 'acc-1', username: 'alice', domain: 'example.test', status: 'active', lastSigninAt: null, quota: { bytesLimit: 100, msgLimit: 5 }, flags: { disabled: false, forcePasswordChange: false, zeroAccess: false, remoteCacheWipe: false } } as unknown as UserSummary;
const ALL = ['users.read', 'users.flags', 'users.delete', 'users.revoke_sessions', 'settings.user_values', 'data_export.run'];

function mount(segment: string | undefined, caps: readonly string[] = ALL) {
  const [rest, setRest] = createSignal<string[]>(segment ? ['acc-1', segment] : ['acc-1']);
  const api = mockAdminApi({ listUsers: vi.fn(async () => [user]) });
  const admin = createAdminSlice(api);
  const view = renderInShell(() => <AdminContext.Provider value={admin}><UserDetail id="acc-1" /></AdminContext.Provider>, {
    loadMe: holding([...caps] as never), route: () => ({ section: 'users', rest: rest() }),
  });
  return { ...view, setRest, api };
}
const panel = (id: string) => screen.getByTestId(`user-panel-${id}`);

beforeEach(() => { vi.mocked(listDataRequests).mockClear(); vi.mocked(adminRequest).mockClear(); });

describe('user detail tab routing', () => {
  it('resolves the default, a valid and an invalid segment', () => {
    expect(resolveUserTab(undefined, USER_TABS)).toBe('overview');
    expect(resolveUserTab('sessions', USER_TABS)).toBe('sessions');
    expect(resolveUserTab('nope', USER_TABS)).toBe('overview');
    expect(resolveUserTab('compliance', USER_TABS.filter((x) => x.id !== 'compliance'))).toBe('overview');
  });
  it('opens Overview by default, with the header and one tablist', async () => {
    mount(undefined);
    expect(await screen.findByRole('heading', { name: 'Alice (alice@example.test)' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'true');
    expect(panel('overview')).not.toHaveAttribute('hidden');
    expect(panel('security')).toHaveAttribute('hidden');
    expect(screen.getAllByRole('tablist')).toHaveLength(1);
  });
  it('falls back to Overview for an unknown tab', async () => {
    mount('bogus');
    expect(await screen.findByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'true');
  });
  it('honours a deep link', async () => {
    mount('security');
    expect(await screen.findByRole('tab', { name: 'Security' })).toHaveAttribute('aria-selected', 'true');
    expect(panel('security')).not.toHaveAttribute('hidden');
    expect(panel('overview')).toHaveAttribute('hidden');
  });
  it('wires aria-controls and aria-labelledby between tabs and panels', async () => {
    mount(undefined);
    for (const t of await screen.findAllByRole('tab')) {
      const controlled = document.getElementById(t.getAttribute('aria-controls')!);
      expect(controlled).toHaveAttribute('role', 'tabpanel');
      expect(controlled).toHaveAttribute('aria-labelledby', t.id);
    }
  });
});

describe('lazy loading and permissions', () => {
  it('fetches the data requests only when Compliance is first opened, then shows a count badge', async () => {
    const m = mount(undefined);
    await screen.findByRole('tab', { name: /Compliance/ });
    await new Promise((r) => setTimeout(r, 50));
    expect(listDataRequests).not.toHaveBeenCalled();
    m.setRest(['acc-1', 'compliance']);
    await waitFor(() => expect(listDataRequests).toHaveBeenCalledTimes(1));
    expect(await screen.findByTestId('tab-badge-compliance')).toHaveTextContent('1');
    m.setRest(['acc-1', 'overview']);
    m.setRest(['acc-1', 'compliance']);
    expect(listDataRequests).toHaveBeenCalledTimes(1);
  });
  it('hides tabs the admin lacks the capability for, and falls back from a hidden deep link', async () => {
    expect(visibleUserTabs(undefined)).toHaveLength(USER_TABS.length);
    const m = mount(undefined, ['users.read']);
    await waitFor(() => expect(screen.queryByRole('tab', { name: /Compliance/ })).toBeNull());
    m.setRest(['acc-1', 'compliance']);
    expect(screen.queryByRole('tab', { name: /Compliance/ })).toBeNull();
    expect(screen.queryByRole('tab', { name: /Feature profile/ })).toBeNull();
    expect(screen.getByRole('tab', { name: 'Overview' })).toHaveAttribute('aria-selected', 'true');
    expect(listDataRequests).not.toHaveBeenCalled();
  });
});

describe('keyboard and structure', () => {
  it('uses a roving tabindex and moves with arrows, Home and End', async () => {
    const { navigate } = mount(undefined);
    const overview = await screen.findByRole('tab', { name: 'Overview' });
    const tabs = screen.getAllByRole('tab');
    expect(tabs.map((x) => x.getAttribute('tabindex'))).toEqual(['0', ...tabs.slice(1).map(() => '-1')]);
    fireEvent.keyDown(overview, { key: 'ArrowRight' });
    expect(navigate).toHaveBeenLastCalledWith(expect.stringContaining('#/users/acc-1/profile'));
    fireEvent.keyDown(overview, { key: 'ArrowLeft' });
    expect(navigate).toHaveBeenLastCalledWith(expect.stringContaining('#/users/acc-1/access'));
    fireEvent.keyDown(overview, { key: 'End' });
    expect(navigate).toHaveBeenLastCalledWith(expect.stringContaining('/access'));
    fireEvent.keyDown(overview, { key: 'Home' });
    expect(navigate).toHaveBeenLastCalledWith(expect.stringContaining('/overview'));
  });
  it('keeps the tab strip in its own horizontally scrolling box', async () => {
    mount(undefined);
    const strip = await screen.findByRole('tablist');
    expect(strip.parentElement?.className).toMatch(/scroller/);
  });
  it('renders each former field and action exactly once across all tabs', async () => {
    const m = mount(undefined);
    await screen.findByRole('tablist');
    for (const id of ['overview', 'profile', 'security', 'sessions', 'access']) { m.setRest(['acc-1', id]); await Promise.resolve(); }
    const once = (re: RegExp, role: 'checkbox' | 'button' | 'heading' = 'heading') => expect(screen.getAllByRole(role, { name: re, hidden: true })).toHaveLength(1);
    once(/^Quota/); once(/^Flags$/); once(/^Mailwoman access$/); once(/^Sessions$/);
    once(/force.*password/i, 'checkbox'); once(/zero/i, 'checkbox');
    once(/^Revoke/i, 'button');
    expect(screen.getAllByText('acc-1', { selector: 'code' })).toHaveLength(1);
  });
});
