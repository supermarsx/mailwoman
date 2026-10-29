import { Show } from 'solid-js';
import { useShell } from './shell/context.ts';
import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminPrivacy, fetchLimitValue, FETCH_LIMIT_KEYS, type PrivacyStats } from './Privacy.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AdminSetting } from '../../api/admin/settings.ts';
const setting: AdminSetting = { key: 'privacy.remoteContent', family: false, kind: { type: 'enum', values: ['block','ask','proxy','direct'] }, default: 'ask', nullable: false, scopes: 'global+domain+user', userOverride: true, category: 'privacy', mapsTo: '', env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: true, globalValue: 'block', allowUserOverride: false, globalLocked: true, updatedBy: 'root', updatedAt: 1, effective: { value: 'block', source: 'global' }, envState: null, overrides: {}, etag: '"v1"' };
const stats: PrivacyStats = { cacheEntries: 3, cacheBytes: 1024, trackersReplaced: 7, traffic: [{ class:'remote_content', allowed:8, refused:2, failed:1,totalMs:100 }] };
describe('Admin Privacy', () => {
  it('shows real process counts and opens policy editors inside Privacy', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async () => stats) };
    const { navigate } = renderInShell(() => <AdminPrivacy service={api} />);
    expect(await screen.findByText('Since this server process started')).toBeInTheDocument();
    await screen.findByText('Locked by administrator policy'); await screen.findByText('1024');
    fireEvent.click(screen.getByRole('link', { name:'Edit policy' }));
    expect(navigate).toHaveBeenCalledWith('/admin#/privacy/policy/privacy.remoteContent');
    expect(screen.queryByText(/24 hours/)).toBeNull();
  });
  it('does not fetch settings without permission or fabricate zeros after diagnostic failure', async () => {
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: [setting], etag: null })), stats: vi.fn(async ():Promise<PrivacyStats> => { throw new Error('unavailable'); }) };
    renderInShell(() => { const shell = useShell(); return <Show when={shell.can.grants().kind === 'listed'}><AdminPrivacy service={api} /></Show>; }, { loadMe: holding(['egress.read']) });
    await waitFor(() => expect(api.stats).toHaveBeenCalled());
    expect(await screen.findByRole('alert')).not.toBeEmptyDOMElement();
    expect(screen.queryByText('1024')).toBeNull();
    expect(api.settings).not.toHaveBeenCalled();
  });
  it('shows the remote image fetch limits in their own subsection, in seconds, with editors (t56)', async () => {
    const limit = (key: string, value: number): AdminSetting => ({ ...setting, key, kind: { type: 'int' } as AdminSetting['kind'], default: value, scopes: 'global', userOverride: false, userVisible: false, globalValue: null, globalLocked: false, effective: { value, source: 'default' } });
    const rows = [limit('privacy.fetchTotalTimeoutMs', 20000), limit('privacy.fetchConnectTimeoutMs', 5000), limit('privacy.maxRedirects', 3), setting];
    const api = { settings: vi.fn(async () => ({ kind: 'ok' as const, data: rows, etag: null })), stats: vi.fn(async () => stats) };
    const { navigate } = renderInShell(() => <AdminPrivacy service={api} />);
    const total = await screen.findByText('Remote image total timeout');
    const section = total.closest('section') as HTMLElement;
    expect(section).toHaveTextContent('Remote image fetching');
    expect(section).toHaveTextContent('Remote image connect timeout');
    expect(section).toHaveTextContent('Remote image redirects');
    // Registry order, not the order the API returned.
    const order = [...section.querySelectorAll('strong')].map(n => n.textContent);
    expect(order).toEqual(['Remote image connect timeout', 'Remote image total timeout', 'Remote image redirects']);
    // The policy subsection keeps its own rows only.
    expect(screen.getByText('Remote content').closest('section')).not.toHaveTextContent('Remote image total timeout');
    const edit = [...section.querySelectorAll('a')].find(a => a.getAttribute('href')?.endsWith('privacy.fetchTotalTimeoutMs')) as HTMLAnchorElement;
    fireEvent.click(edit);
    expect(navigate).toHaveBeenCalledWith('/admin#/privacy/policy/privacy.fetchTotalTimeoutMs');
  });
  it('formats milliseconds as seconds and leaves counts as numbers', () => {
    expect(fetchLimitValue('privacy.fetchTotalTimeoutMs', 20000, 'en')).toBe('20 sec');
    expect(fetchLimitValue('privacy.fetchConnectTimeoutMs', 1500, 'en')).toBe('1.5 sec');
    expect(fetchLimitValue('privacy.maxConcurrentPerMessage', 6, 'en')).toBe('6');
    expect(FETCH_LIMIT_KEYS).toHaveLength(7);
  });
});
