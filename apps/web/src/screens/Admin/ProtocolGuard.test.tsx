import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { ProtocolGuardPanel } from './ProtocolGuardPanel.tsx';
import { ProtocolGuardStats } from './ProtocolGuardStats.tsx';
import { SessionWatchCard } from './SessionWatchCard.tsx';
import { RequestProtection, protectionTab } from './RequestProtection.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import { byCount, groupsCover, overallMode, wouldRefuse, GUARD_GROUPS, MODE_KEYS, ALL_GUARD_KEYS } from './protocolGuard.ts';
import type { AdminSetting } from '../../api/admin/settings.ts';
import type { ProtectionApi, ProtocolGuardRow } from '../../api/admin/requestProtection.ts';
import type { SessionWatch } from '../../api/admin/sessionWatch.ts';

vi.mock('../../api/admin/settings.ts', async (orig) => ({ ...(await orig<object>()), putGlobal: vi.fn(async (key: string, body: { globalValue: unknown }) => ({ setting: setting(key, body.globalValue as never), etag: 'e2' })) }));

function setting(key: string, value: unknown, extra: Partial<AdminSetting> = {}): AdminSetting {
  const isMode = key.endsWith('Mode');
  return {
    key, family: false, kind: isMode ? { type: 'enum', values: ['off', 'detect', 'enforce'] } : { type: 'int', min: 1, max: 100000 }, default: isMode ? 'detect' : 10, nullable: false,
    scopes: 'global', userOverride: false, category: 'security', mapsTo: key, env: null, dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false,
    globalValue: value, allowUserOverride: false, globalLocked: false, updatedBy: null, updatedAt: null, effective: { value, source: 'default' }, envState: null, overrides: {}, etag: 'e1', ...extra,
  } as unknown as AdminSetting;
}
const all = (mode: string, extra: Partial<AdminSetting> = {}): AdminSetting[] => ALL_GUARD_KEYS.map((k) => setting(k, k.endsWith('Mode') ? mode : k.endsWith('s') && !k.endsWith('Bytes') ? [] : 10, extra));
const rowsOf = (mode: 'off' | 'detect' | 'enforce', hits: Record<string, number> = {}): ProtocolGuardRow[] =>
  GUARD_GROUPS.flatMap((g) => g.classes).map((c) => ({ class: c, mode, detected: hits[c] ?? 0, enforced: 0, lastSeen: hits[c] ? Math.floor(Date.now() / 1000) - 180 : null }));
const statusApi = (rows: ProtocolGuardRow[]): ProtectionApi => ({ status: async () => ({ protocolGuard: { classes: rows, scope: 'x' } }) as never, subjects: async () => ({ subjects: [], cursor: null, total: 0 }), events: async () => ({ events: [], cursor: null, total: 0 }), clear: async () => undefined });
const route = () => ({ section: 'security', rest: ['protection', 'protocol-guard'] });

describe('protocolGuard model', () => {
  it('covers every registry key and class exactly once', () => {
    expect(new Set(ALL_GUARD_KEYS).size).toBe(ALL_GUARD_KEYS.length);
    expect(ALL_GUARD_KEYS).toHaveLength(20);
    expect(MODE_KEYS).toHaveLength(11);
    expect(groupsCover()).toBe(true);
  });
  it('sorts by count, reports mixed modes and the detect-only total', () => {
    const rows = rowsOf('detect', { body: 5, host: 9 });
    expect(byCount(rows)[0]?.class).toBe('host');
    expect(wouldRefuse(rows)).toBe(14);
    expect(overallMode(['detect', 'detect'])).toBe('detect');
    expect(overallMode(['detect', 'enforce'])).toBe('mixed');
  });
  it('opens the subtab by route and falls back to overview', () => {
    expect(protectionTab(['protection', 'protocol-guard'])).toBe('protocol-guard');
    expect(protectionTab(['protection', 'nope'])).toBe('overview');
  });
});

describe('Protocol guard subtab', () => {
  it('lists every policy in five groups with live counters and keyboard-reachable tabs', async () => {
    renderInShell(() => <RequestProtection api={statusApi(rowsOf('detect', { host: 3 }))} />, { route });
    expect(await screen.findByRole('link', { name: 'Protocol guard' })).toBeInTheDocument();
  });

  it('renders each group and the stat line beside a class', async () => {
    renderInShell(() => <ProtocolGuardPanel load={async () => ({ kind: 'ok', data: all('detect') }) as never} api={statusApi(rowsOf('detect', { host: 3 }))} />);
    for (const name of ['Framing and smuggling', 'Host and request target', 'Headers', 'Limits', 'Methods and exemptions']) expect(await screen.findByRole('heading', { name })).toBeInTheDocument();
    const row = await screen.findByTestId('protocol-row-security.protocol.hostMode');
    await waitFor(() => expect(within(row).getByText(/Detected 3, refused 0/)).toBeInTheDocument());
    expect(screen.getByTestId('protocol-would-refuse')).toHaveTextContent('3 requests');
  });

  it('asks before moving to enforce and applies on confirm', async () => {
    const { putGlobal } = await import('../../api/admin/settings.ts');
    renderInShell(() => <ProtocolGuardPanel load={async () => ({ kind: 'ok', data: all('detect') }) as never} api={statusApi(rowsOf('detect', { host: 3 }))} />);
    fireEvent.click(await screen.findByTestId('protocol-overall-enforce'));
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    expect(putGlobal).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Enforce all' }));
    await waitFor(() => expect(putGlobal).toHaveBeenCalledTimes(11));
    expect(putGlobal).toHaveBeenCalledWith('security.protocol.hostMode', { globalValue: 'enforce' }, 'e1');
  });

  it('applies off without asking', async () => {
    const { putGlobal } = await import('../../api/admin/settings.ts');
    vi.mocked(putGlobal).mockClear();
    renderInShell(() => <ProtocolGuardPanel load={async () => ({ kind: 'ok', data: all('detect') }) as never} api={statusApi([])} />);
    fireEvent.click(await screen.findByTestId('protocol-overall-off'));
    await waitFor(() => expect(putGlobal).toHaveBeenCalledTimes(11));
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('is read-only without settings.write and when every class is locked', async () => {
    renderInShell(() => <ProtocolGuardPanel load={async () => ({ kind: 'ok', data: all('detect', { globalLocked: true }) }) as never} api={statusApi([])} />, { loadMe: holding(['settings.read']) });
    expect(await screen.findByTestId('protocol-overall-enforce')).toBeDisabled();
    expect(screen.getByTestId('protocol-overall-off')).toBeDisabled();
  });

  it('says reading needs settings.read', async () => {
    renderInShell(() => <ProtocolGuardPanel load={async () => ({ kind: 'ok', data: [] }) as never} api={statusApi([])} />, { loadMe: holding(['security.read']) });
    expect(await screen.findByRole('alert')).toHaveTextContent('settings.read');
  });
});

describe('Protocol guard counters', () => {
  it('groups, sorts, dims zero rows and toggles them away', () => {
    const { container } = renderInShell(() => <ProtocolGuardStats rows={rowsOf('enforce', { host: 9, body: 2 })} />);
    const group = screen.getByTestId('protocol-group-host');
    const bodies = within(group).getAllByRole('row').slice(1);
    expect(bodies[0]).toHaveAttribute('data-class', 'host');
    expect(within(bodies[0]!).getByText('9')).toBeInTheDocument();
    expect(within(group).getAllByTestId('protocol-mode-chip')[0]).toHaveTextContent('Enforce');
    fireEvent.click(screen.getByRole('button', { name: 'Hide zero rows' }));
    expect(within(group).getAllByRole('row')).toHaveLength(2);
    expect(screen.getByTestId('protocol-group-headers')).toHaveTextContent('Nothing detected');
    expect(container.querySelector('a[href*="protocol-guard.md"]')).not.toBeNull();
  });
  it('shows an empty state without counters', () => {
    renderInShell(() => <ProtocolGuardStats rows={[]} />);
    expect(screen.getByText('No protocol guard counters yet.')).toBeInTheDocument();
  });
});

describe('Session watch layout', () => {
  const data: SessionWatch = {
    spray: { enabled: true, trackedSources: 4, sourcesAtChallengeOrAbove: 1, windowSecs: 600, challengeAt: 3, paceAt: 5, lockAt: 8 },
    spike: { enabled: true, warmedUp: true, observedBuckets: 10, warmupBuckets: 10, bucketSecs: 60, heightened: false, heightenedUntil: null, metrics: [{ metric: 'failures', baseline: 2, current: 0, floor: 10 }, { metric: 'failureRatio', baseline: 0.1, current: 0.5, floor: 0.4 }], recent: [] },
    sessionBinding: { enabled: true }, theft: { enabled: false }, countryChange: { enabled: true },
    recentAlerts: [{ ts: Math.floor(Date.now() / 1000) - 300, kind: 'impossibleTravel', action: 'revoked', fromCountry: 'PT', toCountry: 'US' }],
  } as SessionWatch;
  it('shows tiles, named metrics, percent ratios and relative alert times', async () => {
    renderInShell(() => <SessionWatchCard api={{ get: async () => data }} />);
    expect(await screen.findByText('Sources tracked')).toBeInTheDocument();
    expect(screen.getByText('Failed sign-ins')).toBeInTheDocument();
    expect(screen.getByText('Failure ratio')).toBeInTheDocument();
    expect(screen.getByText('50%')).toBeInTheDocument();
    expect(screen.getByText('Impossible travel')).toBeInTheDocument();
    const time = document.querySelector('time[datetime]') as HTMLElement;
    expect(time.getAttribute('title')).toBeTruthy();
    expect(screen.getByText(/Theft detection: off/)).toBeInTheDocument();
  });
  it('shows a loading line, then an error line', async () => {
    renderInShell(() => <SessionWatchCard api={{ get: async () => { throw new Error('x'); } }} />);
    expect(await screen.findByText('The session watch could not be loaded.')).toBeInTheDocument();
  });
});
