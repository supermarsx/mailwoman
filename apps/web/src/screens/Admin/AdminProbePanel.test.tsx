import { beforeAll, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import { AdminProbePanel, validateProbe, evidenceRows, PROBE_GROUPS, PROBE_GROUP_KEYS, PROBE_FIELDS } from './AdminProbePanel.tsx';
import { AdminProbeCard } from './AdminProbeCard.tsx';
import { RequestProtection, protectionTab } from './RequestProtection.tsx';
import type { AdminSetting } from '../../api/admin/settings.ts';
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
  it('shows readable block reasons in top offenders and a safe fallback for a watched source without evidence', async () => {
    const data=overview();
    data.topOffenders=[blocked,{...blocked,id:'c'.repeat(32),source:'198.51.100.0/24',lastSignal:'admin-signin-failed'},{...blocked,id:'d'.repeat(32),source:'192.0.2.0/24',lastSignal:null,blocked:false,blockedAt:null,blockedUntil:null,retryAfterSecs:null}];
    renderInShell(() => <AdminProbePanel api={api({overview:async()=>({data,etag:'"p1"'})})} />);
    const table=await screen.findByTestId('admin-probe-offenders');
    expect(within(table).getByRole('columnheader',{name:'Reason'})).toBeInTheDocument();
    const probe=within(table).getByRole('row',{name:/203\.0\.113\.0\/24/});
    expect(within(probe).getByRole('cell',{name:'A known scanner path'})).toBeInTheDocument();
    expect(probe).toHaveAttribute('data-fired','true');
    const signin=within(table).getByRole('row',{name:/198\.51\.100\.0\/24/});
    expect(within(signin).getByRole('cell',{name:'A failed admin sign-in'})).toBeInTheDocument();
    expect(signin).toHaveAttribute('data-fired','true');
    const watched=within(table).getByRole('row',{name:/192\.0\.2\.0\/24/});
    expect(within(watched).getByRole('cell',{name:'—'})).toBeInTheDocument();
    expect(watched).toHaveAttribute('data-fired','false');
    expect(table).not.toHaveTextContent('admin-probe-signal-');
  });
  it('explains the policy, warns that a block also stops the mail app, and shows counters, blocks and offenders', async () => {
    renderInShell(() => <AdminProbePanel api={api()} />);
    const card = await screen.findByTestId('admin-probe-card');
    expect(await within(card).findByTestId('admin-probe-summary')).toHaveTextContent('Grace: 10 misses per 10 min are forgiven.');
    expect(within(card).getByTestId('admin-probe-mail-warning')).toHaveTextContent(/the mail app included.*Never-blocked networks/);
    expect(screen.getByTestId('admin-probe-counters')).toHaveTextContent('40Requests refused');
    expect(within(screen.getByTestId('admin-probe-blocks')).getByText('203.0.113.0/24')).toBeInTheDocument();
    expect(within(screen.getByTestId('admin-probe-offenders')).getByText('198.51.100.0/24')).toBeInTheDocument();
  });

  it('unblocks after confirmation and reloads', async () => {
    const a = api();
    renderInShell(() => <AdminProbePanel api={a} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Unblock 203.0.113.0/24' }));
    expect(a.unblock).not.toHaveBeenCalled();
    fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Unblock' }));
    await waitFor(() => expect(a.unblock).toHaveBeenCalledWith('a'.repeat(32)));
    await waitFor(() => expect(a.overview).toHaveBeenCalledTimes(2));
  });

  it('saves one setting in place with the policy ETag, validates first, and resets to the default', async () => {
    const a = api();
    renderInShell(() => <AdminProbePanel api={a} />);
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
    renderInShell(() => <AdminProbePanel api={api()} />, { loadMe: holding(['bans.read']) });
    expect(await screen.findByTestId('admin-probe-readonly')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^Unblock/ })).toBeNull();
    expect(within(await screen.findByTestId('admin-probe-row-grace-allowance')).getByRole('spinbutton')).toBeDisabled();
  });

  it('client checks mirror the server bounds', () => {
    expect(validateProbe('graceAllowance', 1001, policy)).toMatch(/1,000|1000/);
    expect(validateProbe('probePaths', ['wp-admin'], policy)).toMatch(/starts with \//);
    expect(validateProbe('windowSecs', 600, policy)).toBeNull();
  });
});

function geo(key: string, value: unknown, extra: Partial<AdminSetting> = {}): AdminSetting {
  return {
    key, family: false, kind: { type: 'int', min: 1, max: 100 }, default: 3, nullable: false, scopes: 'global', userOverride: false, category: 'security', mapsTo: key, env: null,
    dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: value, allowUserOverride: false, globalLocked: false, updatedBy: null, updatedAt: null,
    effective: { value, source: 'default' }, envState: null, overrides: {}, etag: 'e1', ...extra,
  } as unknown as AdminSetting;
}
const geoKeys = PROBE_GROUPS.flatMap((g) => g.registry);

describe('Admin probe subtab', () => {
  it('shows every admin-probe policy once, in five named groups', () => {
    expect(PROBE_GROUPS.map((g) => g.id)).toEqual(['detection', 'responses', 'countries', 'exemptions', 'logging']);
    expect([...PROBE_GROUP_KEYS].sort()).toEqual(PROBE_FIELDS.map((f) => f.key).sort());
    expect(new Set(geoKeys).size).toBe(geoKeys.length);
    expect(geoKeys).toContain('security.trust.adminGeoCountries');
  });
  it('renders the groups with editors, the registry rows and the evidence table', async () => {
    renderInShell(() => <AdminProbePanel api={api()} load={async () => ({ kind: 'ok', data: geoKeys.map((k) => geo(k, 3)) }) as never} />);
    for (const name of ['Detection and evidence', 'Responses and bans', 'Countries and trust tiers', 'Exemptions and allowlists', 'Audit and logging']) expect(await screen.findByRole('heading', { name })).toBeInTheDocument();
    expect(await screen.findByTestId('admin-probe-geo-security.trust.adminGeoThreshold')).toBeInTheDocument();
    const evidence = within(screen.getByTestId('admin-probe-evidence'));
    expect(evidence.getAllByRole('row')[1]).toHaveAttribute('data-signal', 'probe-path');
    expect(screen.getByTestId('admin-probe-evidence').querySelector('time[title]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Hide zero rows' }));
    expect(evidence.getAllByRole('row')).toHaveLength(2);
  });
  it('orders evidence by count and picks the latest time per type', () => {
    const rows = evidenceRows({ ...overview().stats, signals: { 'slug-guess': 7, 'probe-path': 2 } }, [blocked]);
    expect(rows[0]?.signal).toBe('slug-guess');
    expect(rows.find((r) => r.signal === 'probe-path')?.lastSeen).toBe('2026-10-30T01:00:00Z');
  });
  it('asks before turning protection on, and turns it off at once', async () => {
    const off = overview({ ...policy, enabled: false });
    const a = api({ overview: async () => ({ data: off, etag: '"p1"' }) });
    renderInShell(() => <AdminProbePanel api={a} />);
    fireEvent.click(await screen.findByTestId('admin-probe-mode-on'));
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    expect(a.save).not.toHaveBeenCalled();
    fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Turn on' }));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith({ enabled: true }, '"p1"'));
  });
  it('turns off without a dialog', async () => {
    const a = api();
    renderInShell(() => <AdminProbePanel api={a} />);
    fireEvent.click(await screen.findByTestId('admin-probe-mode-off'));
    await waitFor(() => expect(a.save).toHaveBeenCalledWith({ enabled: false }, '"p1"'));
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });
  it('is read-only without bans.write and says reading country settings needs settings.read', async () => {
    renderInShell(() => <AdminProbePanel api={api()} />, { loadMe: holding(['bans.read']) });
    expect(await screen.findByTestId('admin-probe-mode-on')).toBeDisabled();
    expect(await screen.findAllByText('Reading settings needs the settings.read permission.')).not.toHaveLength(0);
  });
  it('opens by route, lists the tab, and the Overview keeps a compact card linking to it', async () => {
    expect(protectionTab(['protection', 'admin-probe'])).toBe('admin-probe');
    renderInShell(() => <RequestProtection probeApi={api()} api={{ status: async () => ({}) as never, subjects: async () => ({ subjects: [], cursor: null, total: 0 }), events: async () => ({ events: [], cursor: null, total: 0 }), clear: async () => undefined }} />, { route: () => ({ section: 'security', rest: ['protection', 'admin-probe'] }) });
    expect(await screen.findByRole('link', { name: 'Admin probe' })).toHaveAttribute('aria-current', 'page');
    expect(await screen.findByTestId('admin-probe-panel')).toBeInTheDocument();
  });
  it('shows the compact summary with an Edit policies link and no editors', async () => {
    const open = vi.fn();
    renderInShell(() => <AdminProbeCard api={api()} onOpenEditor={open} />);
    const card = await screen.findByTestId('admin-probe-summary-card');
    expect(await within(card).findByTestId('admin-probe-summary')).toBeInTheDocument();
    expect(within(card).queryByRole('spinbutton')).toBeNull();
    fireEvent.click(within(card).getByRole('button', { name: 'Edit policies' }));
    expect(open).toHaveBeenCalled();
  });
});
