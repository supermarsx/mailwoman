import { Show } from 'solid-js';
import { useShell } from './shell/context.ts';
import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminPrivacy, type PrivacyStats } from './Privacy.tsx';
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
});
