import { Show } from 'solid-js';
import { useShell } from './shell/context.ts';
import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminPrivacy, fetchLimitValue, FETCH_LIMIT_KEYS, type PrivacyStats } from './Privacy.tsx';
import { PRIVACY_TABS, canOpenPrivacyTab } from './shell/privacyNavigation.ts';
import { tabForKey } from './Privacy/model.ts';
import { renderInShell, holding } from './shell/testkit.tsx';
import { createCan, type Capability } from './shell/can.ts';
import type { AdminSetting } from '../../api/admin/settings.ts';
const setting: AdminSetting = { key: 'privacy.remoteContent', family: false, kind: { type: 'enum', values: ['block','ask','proxy','direct'] }, default: 'ask', nullable: false, scopes: 'global+domain+user', userOverride: true, category: 'privacy', mapsTo: '', env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: true, globalValue: 'block', allowUserOverride: false, globalLocked: true, updatedBy: 'root', updatedAt: 1, effective: { value: 'block', source: 'global' }, envState: null, overrides: {}, etag: '"v1"' };
const stats: PrivacyStats = { cacheEntries: 3, cacheBytes: 1024, trackersReplaced: 7, traffic: [{ class:'remote_content', allowed:8, refused:2, failed:1,totalMs:100 }] };
const history = { range: '1h', t: [1000, 1030, 1060], sampleSecs: 15, series: { requestsPerSec: [0.5, 1, 2], bytesPerSec: [100, 200, null], ramHitPct: [50, 60, 70], diskHitPct: [0, 10, 20], queueDepth: [0, 1, 2], activeFetches: [1, 2, 3], busy429: [0, 0, 1], busy503: [0, 0, 0], trackersReplaced: [0, 1, 0] }, classes: { image: { allowed: [1, 2, 3], refused: [0, 1, 0], failed: [0, 0, 1] } } };
const limit = (key: string, value: number): AdminSetting => ({ ...setting, key, kind: { type: 'int' } as AdminSetting['kind'], default: value, scopes: 'global', userOverride: false, userVisible: false, globalValue: null, globalLocked: false, effective: { value, source: 'default' } });
const route = (...rest: string[]) => () => ({ section: 'privacy', rest });
const grants = (caps: Capability[], scope: string[] | null = null) => createCan(() => ({ kind: 'listed', me: { username: 'operator', displayName: null, roles: [], capabilities: caps, domainScope: scope, totpEnrolled: false } }));

describe('Admin Privacy tabs', () => {
  it('opens on the Overview with real process counts, history charts and a last-updated stamp', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async () => ({ ...stats, history })) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route() });
    expect(await screen.findByTestId('tile-entries')).toHaveTextContent('3');
    expect(screen.getByTestId('tile-bytes')).toHaveTextContent('1 kB');
    expect(await screen.findByTestId('chart-requests')).toBeInTheDocument();
    expect(screen.getByTestId('history-updated')).toHaveTextContent(/^Updated /);
    expect(screen.getByTestId('history-controls')).toHaveTextContent('in memory only');
    expect(api.stats).toHaveBeenCalledWith('1h');
    // The same numbers are in a table for screen readers.
    const table = within(screen.getByTestId('chart-requests')).getByRole('table', { hidden: true });
    expect(table).toHaveTextContent('2/s');
    expect(screen.getByRole('navigation', { name: 'Privacy and remote content' })).toBeInTheDocument();
  });
  it('requests another history range from the selector', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [], etag: null })), stats: vi.fn(async () => ({ ...stats, history })) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route() });
    fireEvent.click(await screen.findByRole('button', { name: '6 hours' }));
    await waitFor(() => expect(api.stats).toHaveBeenCalledWith('6h'));
    expect(screen.getByRole('button', { name: '6 hours' })).toHaveAttribute('aria-pressed', 'true');
  });
  it('says the history is unavailable instead of drawing empty charts when the server sends none', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [], etag: null })), stats: vi.fn(async () => stats) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route() });
    expect(await screen.findByTestId('history-pending')).toHaveTextContent('not available');
    expect(screen.queryByTestId('chart-requests')).toBeNull();
  });
  it('shows an empty chart message for a history with no samples', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [], etag: null })), stats: vi.fn(async () => ({ ...stats, history: { t: [], series: {} } })) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route() });
    expect(await within(await screen.findByTestId('chart-requests')).findByText('No samples in this range yet.')).toBeInTheDocument();
  });
  it('every tab opens (clicking a tab navigates to its own plain path, not a doubled one) and renders its content', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting, limit('privacy.maxRedirects', 3)], etag: null })), stats: vi.fn(async () => ({ ...stats, history })) };
    const cases: [string, string][] = [['overview', 'Privacy and remote content'], ['remote', 'privacy.remoteContent'], ['image-proxy', 'Worker history'], ['fetch-limits', 'Remote image fetching'], ['traffic', 'Since this server process started']];
    for (const [tab, text] of cases) {
      const view = renderInShell(() => <AdminPrivacy service={api} />, { route: route(tab) });
      expect((await screen.findAllByText(new RegExp(text))).length).toBeGreaterThan(0);
      expect(screen.queryByText(/Something went wrong|Could not load/)).toBeNull();
      const links = screen.getAllByRole('link').filter((a) => /#\/privacy\//.test(a.getAttribute('href') ?? ''));
      expect(links.length).toBe(5);
      fireEvent.click(links[2]!);
      expect(view.navigate).toHaveBeenLastCalledWith(expect.stringMatching(/#\/privacy\/image-proxy$/));
      view.unmount();
    }
  });
  it('lists the traffic tab with the since-start tables', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async () => ({ ...stats, history })) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route('traffic') });
    expect(await screen.findByText('Since this server process started')).toBeInTheDocument();
    await screen.findByText('1024');
    expect(screen.getByText('remote content')).toBeInTheDocument();
    expect(screen.queryByText(/24 hours/)).toBeNull();
  });
  it('edits a policy in place and keeps the old policy/<key> deep link: it opens the Remote tab on that row', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async () => stats) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route('policy', 'privacy.remoteContent') });
    const card = await screen.findByTestId('inline-setting');
    expect(card).toHaveAttribute('data-setting-key', 'privacy.remoteContent');
    expect(screen.queryByRole('link', { name: 'Edit policy' })).toBeNull();
    // Locked by the administrator: the switch shows it.
    expect(within(card).getByRole('switch')).toHaveAttribute('aria-checked', 'true');
    await waitFor(() => expect(card).toContainElement(document.activeElement as HTMLElement));
  });
  it('does not fetch settings without permission or fabricate zeros after diagnostic failure', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async ():Promise<PrivacyStats> => { throw new Error('unavailable'); }) };
    renderInShell(() => { const shell = useShell(); return <Show when={shell.can.grants().kind === 'listed'}><AdminPrivacy service={api} /></Show>; }, { loadMe: holding(['egress.read']), route: route() });
    await waitFor(() => expect(api.stats).toHaveBeenCalled());
    expect((await screen.findAllByRole('alert'))[0]).not.toBeEmptyDOMElement();
    expect(screen.queryByText('1024')).toBeNull();
    expect(api.settings).not.toHaveBeenCalled();
  });
  it('shows the remote image fetch limits in their own tab, in seconds, in registry order, each editable (t56)', async () => {
    const rows = [limit('privacy.fetchTotalTimeoutMs', 20000), limit('privacy.fetchConnectTimeoutMs', 5000), limit('privacy.maxRedirects', 3),limit('privacy.fetchQueueTimeoutMs',60000),limit('privacy.imageDecodeTimeoutMs',10000),limit('privacy.maxConcurrentGlobal',32),limit('privacy.rateBurst',512),limit('privacy.cacheTtlSecs',3600),limit('privacy.imageRetryAttempts',3),limit('privacy.imageRetryDelayMs',5000), setting];
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: rows, etag: null })), stats: vi.fn(async () => stats) };
    renderInShell(() => <AdminPrivacy service={api} />, { route: route('fetch-limits') });
    await screen.findAllByTestId('inline-setting');
    const order = screen.getAllByTestId('inline-setting').map((card) => card.getAttribute('data-setting-key'));
    expect(order).toEqual(['privacy.fetchConnectTimeoutMs', 'privacy.fetchTotalTimeoutMs', 'privacy.fetchQueueTimeoutMs', 'privacy.imageDecodeTimeoutMs', 'privacy.maxRedirects', 'privacy.maxConcurrentGlobal', 'privacy.rateBurst', 'privacy.cacheTtlSecs', 'privacy.imageRetryAttempts', 'privacy.imageRetryDelayMs']);
    const total = screen.getAllByTestId('inline-setting')[1]!;
    expect(within(total).getByTestId('inline-effective')).toHaveTextContent('20 sec');
    expect(screen.getAllByTestId('inline-effective').map((n) => n.textContent).join(' ')).toContain('3,600 sec');
    expect(within(total).getByRole('spinbutton')).toBeInTheDocument();
  });
  it('reads only: a viewer without settings.write sees values and no Save', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [limit('privacy.maxRedirects', 3)], etag: null })), stats: vi.fn(async () => stats) };
    renderInShell(() => { const shell = useShell(); return <Show when={shell.can.grants().kind === 'listed'}><AdminPrivacy service={api} /></Show>; }, { loadMe: holding(['egress.read', 'settings.read']), route: route('fetch-limits') });
    const card = await screen.findByTestId('inline-setting');
    expect(within(card).getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(within(card).getByTestId('inline-readonly')).toBeInTheDocument();
  });
  it('formats milliseconds as seconds and leaves counts as numbers', () => {
    expect(fetchLimitValue('privacy.fetchTotalTimeoutMs', 20000, 'en')).toBe('20 sec');
    expect(fetchLimitValue('privacy.fetchConnectTimeoutMs', 1500, 'en')).toBe('1.5 sec');
    expect(fetchLimitValue('privacy.maxConcurrentPerMessage', 6, 'en')).toBe('6');
    expect(fetchLimitValue('privacy.imageRetryDelayMs',500,'en')).toBe('0.5 sec');
    expect(fetchLimitValue('privacy.cacheTtlSecs',10,'en')).toBe('10 sec');
    expect(FETCH_LIMIT_KEYS).toHaveLength(21);
  });
});

describe('Privacy navigation', () => {
  it('gates each tab on its capability and keeps scoped admins off the process-wide tabs', () => {
    const ids = (caps: Capability[], scope: string[] | null = null) => PRIVACY_TABS.filter((tab) => canOpenPrivacyTab(grants(caps, scope), tab)).map((tab) => tab.id);
    expect(ids(['egress.read'])).toEqual(['overview', 'image-proxy', 'traffic']);
    expect(ids(['egress.read', 'settings.read'])).toEqual(['overview', 'remote', 'image-proxy', 'fetch-limits', 'traffic']);
    expect(ids(['settings.read'])).toEqual(['remote', 'image-proxy', 'fetch-limits']);
    expect(ids(['egress.read', 'settings.read'], ['example.com'])).toEqual(['remote', 'fetch-limits']);
    expect(ids(['users.read'])).toEqual([]);
  });
  it('maps a settings key to the tab that edits it', () => {
    expect(tabForKey('privacy.remoteContent')).toBe('remote');
    expect(tabForKey('privacy.fetchTotalTimeoutMs')).toBe('fetch-limits');
    expect(tabForKey('privacy.rateBurst')).toBe('fetch-limits');
    expect(tabForKey('privacy.imageConcurrencyMode')).toBe('image-proxy');
    expect(tabForKey('privacy.imageDiskPath')).toBe('image-proxy');
    expect(tabForKey('nothing.else')).toBeUndefined();
  });
  it('shows the denied message for a tab the viewer cannot open', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [], etag: null })), stats: vi.fn(async () => stats) };
    renderInShell(() => { const shell = useShell(); return <Show when={shell.can.grants().kind === 'listed'}><AdminPrivacy service={api} /></Show>; }, { loadMe: holding(['egress.read']), route: route('fetch-limits') });
    expect(await screen.findByText('You do not have access to this tab.')).toBeInTheDocument();
  });
});
