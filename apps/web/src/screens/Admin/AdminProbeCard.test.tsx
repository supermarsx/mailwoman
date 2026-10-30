import { beforeAll, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import { AdminProbeCard, validateProbe } from './AdminProbeCard.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AdminProbeApi, AdminProbeOverview, AdminProbePolicy, AdminProbeSource } from '../../api/admin/adminProbe.ts';

beforeAll(async () => { await loadCatalog('admin'); await loadCatalog('admin-security'); await loadCatalog('admin-settings'); });

const policy: AdminProbePolicy = {
  enabled: true, graceAllowance: 10, windowSecs: 600, blockAfter: 10, tarpitDelayMs: 1500, blockSecs: 900, maxBlockSecs: 86_400,
  repeatWindowSecs: 86_400, blockStatus: '429', signals: ['probe-path', 'slug-guess', 'admin-signin-failed', 'admin-unauthenticated'],
  probePaths: ['/wp-admin', '/.env'], allowlist: [], auditIp: 'masked', fail2banLog: true,
};
const blocked: AdminProbeSource = { id: 'a'.repeat(32), source: '203.0.113.0/24', score: 31.5, graceAllowance: 10, signals: 25, lastSignal: 'probe-path', lastSeen: '2026-10-30T01:00:00Z', strikes: 1, blocked: true, blockedAt: '2026-10-30T01:00:00Z', blockedUntil: '2026-10-30T01:15:00Z', retryAfterSecs: 840 };
function overview(p: AdminProbePolicy = policy): AdminProbeOverview {
  return {
    policy: p,
    stats: { since: '2026-10-30T00:00:00Z', blocksTotal: 3, blocksLast24h: 2, activeBlocks: 1, refusedRequests: 40, tarpittedRequests: 12, clearedBySignin: 1, signals: { 'probe-path': 25 }, trackedSources: 4, capacity: 10_000, evictedSources: 0 },
    topOffenders: [blocked, { ...blocked, id: 'b'.repeat(32), source: '198.51.100.0/24', blocked: false, blockedAt: null, blockedUntil: null, retryAfterSecs: null, score: 4 }],
    scope: 'instance', propagationSecs: 30,
  };
}
function api(overrides: Partial<AdminProbeApi> = {}): AdminProbeApi {
  return {
    overview: vi.fn(async () => ({ data: overview(), etag: '"p1"' })),
    save: vi.fn(async (values) => ({ data: overview({ ...policy, ...Object.fromEntries(Object.entries(values).map(([k, v]) => [k, v ?? (policy as unknown as Record<string, unknown>)[k]])) } as AdminProbePolicy), etag: '"p2"' })),
    blocks: vi.fn(async () => [blocked]),
    unblock: vi.fn(async () => undefined),
    ...overrides,
  };
}

describe('Admin probe protection card', () => {
  it('explains the policy, warns that a block also stops the mail app, and shows counters, blocks and offenders', async () => {
    renderInShell(() => <AdminProbeCard api={api()} />);
    const card = await screen.findByTestId('admin-probe-card');
    expect(await within(card).findByTestId('admin-probe-summary')).toHaveTextContent('Grace: 10 misses per 10 min are forgiven.');
    expect(within(card).getByTestId('admin-probe-mail-warning')).toHaveTextContent(/the mail app included.*Never-blocked networks/);
    expect(within(card).getByTestId('admin-probe-counters')).toHaveTextContent('Requests refused40');
    expect(within(within(card).getByTestId('admin-probe-blocks')).getByText('203.0.113.0/24')).toBeInTheDocument();
    expect(within(within(card).getByTestId('admin-probe-offenders')).getByText('198.51.100.0/24')).toBeInTheDocument();
  });

  it('unblocks after confirmation and reloads', async () => {
    const a = api();
    renderInShell(() => <AdminProbeCard api={a} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Unblock 203.0.113.0/24' }));
    expect(a.unblock).not.toHaveBeenCalled();
    fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Unblock' }));
    await waitFor(() => expect(a.unblock).toHaveBeenCalledWith('a'.repeat(32)));
    await waitFor(() => expect(a.overview).toHaveBeenCalledTimes(2));
  });

  it('saves one setting in place with the policy ETag, validates first, and resets to the default', async () => {
    const a = api();
    renderInShell(() => <AdminProbeCard api={a} />);
    const row = await screen.findByTestId('admin-probe-row-block-secs');
    const input = within(row).getByRole('spinbutton', { name: 'First block (seconds)' });
    fireEvent.input(input, { target: { value: '100000' } });
    fireEvent.click(within(row).getByRole('button', { name: 'Save First block (seconds)' }));
    expect(await within(row).findByRole('alert')).toHaveTextContent('The longest block must be at least as long as the first block.');
    expect(a.save).not.toHaveBeenCalled();
    fireEvent.input(input, { target: { value: '1800' } });
    fireEvent.click(within(row).getByRole('button', { name: 'Save First block (seconds)' }));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith({ blockSecs: 1800 }, '"p1"'));
    expect(await screen.findByText('First block (seconds) saved.')).toBeInTheDocument();
    const allow = screen.getByTestId('admin-probe-row-allowlist');
    fireEvent.input(within(allow).getByRole('textbox', { name: 'Never-blocked networks' }), { target: { value: '203.0.113.0/24\n2001:db8:42::/48' } });
    fireEvent.click(within(allow).getByRole('button', { name: 'Save Never-blocked networks' }));
    await waitFor(() => expect(a.save).toHaveBeenLastCalledWith({ allowlist: ['203.0.113.0/24', '2001:db8:42::/48'] }, '"p2"'));
    const blockRow = screen.getByTestId('admin-probe-row-block-secs');
    await waitFor(() => expect(within(blockRow).getByRole('button', { name: 'Reset First block (seconds) to its default' })).not.toBeDisabled());
    fireEvent.click(within(blockRow).getByRole('button', { name: 'Reset First block (seconds) to its default' }));
    await waitFor(() => expect(a.save).toHaveBeenLastCalledWith({ blockSecs: null }, '"p2"'));
  });

  it('is read-only without bans.write', async () => {
    renderInShell(() => <AdminProbeCard api={api()} />, { loadMe: holding(['bans.read']) });
    expect(await screen.findByTestId('admin-probe-readonly')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^Unblock/ })).toBeNull();
    expect(within(screen.getByTestId('admin-probe-row-grace-allowance')).getByRole('spinbutton')).toBeDisabled();
  });

  it('client checks mirror the server bounds', () => {
    expect(validateProbe('graceAllowance', 1001, policy)).toMatch(/1,000|1000/);
    expect(validateProbe('probePaths', ['wp-admin'], policy)).toMatch(/starts with \//);
    expect(validateProbe('windowSecs', 600, policy)).toBeNull();
  });
});
