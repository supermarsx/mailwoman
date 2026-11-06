import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import type { PersistedHistory, PersistedRange } from '../../api/admin/persistedMetrics.ts';

const load = vi.hoisted(() => vi.fn());
const reset = vi.hoisted(() => vi.fn());
vi.mock('../../api/admin/persistedMetrics.ts', async (original) => ({
  ...(await original<typeof import('../../api/admin/persistedMetrics.ts')>()),
  loadPersisted: load,
  resetPersisted: reset,
}));
import { PersistedStats } from './PersistedStats.tsx';

const T0 = 1_800_000_000;
function history(over: Partial<PersistedHistory> = {}, range: PersistedRange = '24h'): PersistedHistory {
  const t = [T0, T0 + 900, T0 + 1800];
  const group = { totals: { count: 90, p50: 20, p95: 400 }, series: { t, count: [30, 30, 30], p50: [18, 20, 22], p95: [380, 400, 420] }, breakdown: {} };
  return {
    source: 'imageProxy', range, from: T0, to: T0 + 2700, stepSecs: 900, counter: false, relativeError: 0.0625,
    persistence: {
      enabled: true, dimensionsEnabled: true, flushSeconds: 60, lastFlushAt: T0 + 1700, rows: 1234, historySince: T0 - 86400 * 3, instances: 2,
      retention: { minuteBucketsHours: 48, hourBucketsDays: 60, dayBucketsDays: 400 },
    },
    groups: { total: group, miss: group, hit: group },
    ...over,
  };
}

describe('saved statistics', () => {
  beforeEach(async () => {
    load.mockReset();
    reset.mockReset();
    await loadCatalog('admin-shell');
  });

  it('shows the Saved badge with the last save, the row count and the first stored bucket, and charts per group', async () => {
    load.mockResolvedValue(history());
    render(() => <PersistedStats source="imageProxy" />);
    expect(await screen.findByTestId('persisted-badge')).toHaveTextContent('Saved');
    expect(screen.getByTestId('persisted-last-flush')).toHaveTextContent(/Last saved/);
    expect(screen.getByTestId('persisted-rows')).toHaveTextContent('1,234 stored rows');
    expect(screen.getByTestId('persisted-since')).toHaveTextContent(/History since/);
    expect(screen.getByTestId('persisted-chart-image-total')).toBeInTheDocument();
    expect(screen.getByTestId('persisted-chart-image-miss')).toBeInTheDocument();
    expect(load).toHaveBeenCalledWith('imageProxy', '24h');
  });

  it('reloads for the chosen range', async () => {
    load.mockImplementation((_s: string, range: PersistedRange) => Promise.resolve(history({ range }, range)));
    render(() => <PersistedStats source="imageProxy" />);
    await screen.findByTestId('persisted-badge');
    const ranges = screen.getByTestId('persisted-ranges');
    expect([...ranges.querySelectorAll('button')].map((b) => b.dataset['range'])).toEqual(['1h', '24h', '7d', '30d', '1y']);
    expect(ranges.querySelector('[data-range="24h"]')).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(ranges.querySelector('[data-range="7d"]')!);
    await waitFor(() => expect(load).toHaveBeenCalledWith('imageProxy', '7d'));
    await waitFor(() => expect(ranges.querySelector('[data-range="7d"]')).toHaveAttribute('aria-pressed', 'true'));
  });

  it('says so when statistics are not being saved, with no badge and no charts', async () => {
    const off = history();
    off.persistence.enabled = false;
    off.groups = {};
    load.mockResolvedValue(off);
    render(() => <PersistedStats source="clientTiming" />);
    expect(await screen.findByTestId('persisted-off')).toHaveTextContent('not being saved');
    expect(screen.queryByTestId('persisted-badge')).toBeNull();
    expect(screen.queryByTestId('persisted-chart-client-usable')).toBeNull();
  });

  it('shows counter sources as counts', async () => {
    const t = [T0, T0 + 900];
    const group = (n: number) => ({ totals: { count: n }, series: { t, count: [n, 0] }, breakdown: {} });
    load.mockResolvedValue(history({ source: 'rateLimit', counter: true, groups: { allowed: group(5), queued: group(2), rejected: group(3) } }));
    render(() => <PersistedStats source="rateLimit" />);
    expect(await screen.findByTestId('persisted-chart-rate-limited')).toBeInTheDocument();
    expect(screen.getByTestId('persisted-chart-rate-allowed')).toBeInTheDocument();
  });

  it('resets after a confirmation and reloads', async () => {
    load.mockResolvedValue(history());
    reset.mockResolvedValue(12);
    render(() => <PersistedStats source="imageProxy" />);
    fireEvent.click(await screen.findByTestId('persisted-reset'));
    expect(reset).not.toHaveBeenCalled();
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('Reset these statistics?');
    const calls = load.mock.calls.length;
    fireEvent.click(within(dialog).getByRole('button', { name: 'Reset statistics' }));
    await waitFor(() => expect(reset).toHaveBeenCalledWith('imageProxy'));
    await waitFor(() => expect(load.mock.calls.length).toBeGreaterThan(calls));
  });

  it('says when the saved statistics cannot be read', async () => {
    load.mockRejectedValue(new Error('x'));
    render(() => <PersistedStats source="imageProxy" />);
    expect(await screen.findByTestId('persisted-failed')).toBeInTheDocument();
  });
});
