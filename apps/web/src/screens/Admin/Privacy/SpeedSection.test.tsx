import { describe, expect, it } from 'vitest';
import { render, screen, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../../i18n';
import { SpeedSection, duration } from './SpeedSection.tsx';
import type { LatencySummary, PrivacyStats, ProxyLatency, SpeedVerdict } from './model.ts';

const sum = (count: number, p50: number | null, p95: number | null): LatencySummary => ({ count, p50Ms: p50, p95Ms: p95, p99Ms: p95, maxMs: p95, meanMs: p50 });

function latency(over: Partial<ProxyLatency> = {}, verdict: SpeedVerdict = 'healthy', reason = 'ok'): ProxyLatency {
  return {
    enabled: true, t: [100, 160], stepSecs: 60, relativeError: 0.0625,
    series: { p50Ms: [12, 15], p95Ms: [120, 180], missP50Ms: [300, 320], missP95Ms: [900, 1100] },
    total: sum(120, 14, 150), hit: sum(90, 2, 20), miss: sum(30, 310, 1000),
    outcomes: { busy: sum(3, 1, 1), timeout: sum(2, 5000, 6000), upstreamError: sum(1, 10, 10) },
    phases: {
      queue: { ...sum(30, 5, 40), sharePct: 5 }, cache: { ...sum(30, 1, 2), sharePct: 1 },
      upstream: { ...sum(30, 250, 900), sharePct: 80 }, decode: { ...sum(30, 40, 120), sharePct: 12 },
    },
    otherPct: 2, failures: 6, failurePct: 5,
    health: { verdict, reason, thresholds: { minRequests: 10, slowMissP95Ms: 3000, degradedMissP95Ms: 8000, slowHitP95Ms: 250, slowFailurePct: 2, degradedFailurePct: 10 } },
    ...over,
  };
}
const stats = (l?: ProxyLatency): PrivacyStats => ({ cacheEntries: 0, cacheBytes: 0, trackersReplaced: 0, traffic: [], ...(l ? { latency: l } : {}) });

describe('image proxy speed section', () => {
  it('formats milliseconds and seconds', () => {
    expect(duration(12.5)).toMatch(/12\.5 ms/);
    expect(duration(2400)).toMatch(/2\.4 s/);
  });

  it('shows the verdict, tiles, hit/miss split, phases and both charts', async () => {
    await loadCatalog('admin-shell');
    render(() => <SpeedSection stats={stats(latency())} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-verdict')).toHaveTextContent('Healthy');
    expect(screen.getByTestId('speed-p50')).toHaveTextContent('14 ms');
    expect(screen.getByTestId('speed-p95')).toHaveTextContent('150 ms');
    expect(screen.getByTestId('speed-busy')).toHaveTextContent('3');
    expect(screen.getByTestId('speed-timeouts')).toHaveTextContent('2');
    expect(screen.getByTestId('speed-hit')).toHaveTextContent('90 requests');
    expect(screen.getByTestId('speed-miss')).toHaveTextContent('30 requests');
    const phases = screen.getByTestId('speed-phases');
    expect(within(phases).getByText('Upstream fetch').closest('tr')).toHaveTextContent('80 %');
    expect(screen.getByTestId('chart-speed')).toBeInTheDocument();
    expect(screen.getByTestId('chart-speed-miss')).toBeInTheDocument();
  });

  it('says why when the verdict is slow or degraded', async () => {
    const { unmount } = render(() => <SpeedSection stats={stats(latency({}, 'degraded', 'failureRate'))} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-verdict')).toHaveTextContent('Degraded');
    expect(screen.getByTestId('speed-reason')).toHaveTextContent('5 % of requests ended busy');
    unmount();
    render(() => <SpeedSection stats={stats(latency({}, 'slow', 'missP95'))} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-verdict')).toHaveTextContent('Slow');
    expect(screen.getByTestId('speed-reason')).toHaveTextContent('Images that had to be fetched are slow');
  });

  it('shows an empty state when nothing was measured, and says when tracking is off', async () => {
    const empty = latency({ total: sum(0, null, null), t: [], series: {} }, 'noData', 'fewRequests');
    const { unmount } = render(() => <SpeedSection stats={stats(empty)} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-empty')).toHaveTextContent('No images were served');
    unmount();
    render(() => <SpeedSection stats={stats({ ...empty, enabled: false })} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-empty')).toHaveTextContent('turned off');
  });

  it('shows a loading state, and unavailable for a server without the figures', async () => {
    const { unmount } = render(() => <SpeedSection stats={undefined} loading error={undefined} />);
    expect(await screen.findByTestId('speed-loading')).toBeInTheDocument();
    unmount();
    render(() => <SpeedSection stats={stats()} loading={false} error={undefined} />);
    expect(await screen.findByTestId('speed-unavailable')).toHaveTextContent('not available');
  });
});

describe('image proxy work panel', () => {
  const history = (series: Record<string, (number | null)[]>) => ({ t: [100, 160, 220], series });
  const withHistory = (h: ReturnType<typeof history>): PrivacyStats => ({ ...stats(latency()), history: h });

  it('charts the new series with their latest and highest values', async () => {
    render(() => <SpeedSection stats={withHistory(history({
      coalescedFetches: [0, 4, 7], highPriorityRequests: [1, 1, 2], handleCacheHitPct: [50, 75, 80],
      staleServed: [0, 3, 9], revalidated: [0, 2, 5], resized: [1, 6, 8], resizeBytesSaved: [2048, 4096, 10240],
    }))} loading={false} error={undefined} />);
    const panel = await screen.findByTestId('speed-work');
    expect(within(panel).getByTestId('chart-work-coalesced')).toBeInTheDocument();
    expect(within(panel).getByTestId('chart-work-handle')).toHaveTextContent('80 %');
    expect(within(panel).getByTestId('chart-work-stale')).toHaveTextContent('Served stale');
    expect(within(panel).getByTestId('chart-work-stale')).toHaveTextContent('Revalidated');
    expect(within(panel).getByTestId('chart-work-resized')).toHaveTextContent('8');
    expect(within(panel).getByTestId('chart-work-saved')).toHaveTextContent('10 KiB');
  });

  it('is absent when the server sent none of the series', async () => {
    render(() => <SpeedSection stats={withHistory(history({ requestsPerSec: [1, 2, 3] }))} loading={false} error={undefined} />);
    await screen.findByTestId('speed-verdict');
    expect(screen.queryByTestId('speed-work')).toBeNull();
  });

  it('says there is nothing yet for a series that has no samples', async () => {
    render(() => <SpeedSection stats={withHistory(history({ resized: [null, null, null], staleServed: [null, null, null] }))} loading={false} error={undefined} />);
    const panel = await screen.findByTestId('speed-work');
    expect(within(panel).getByTestId('chart-work-resized')).toBeInTheDocument();
  });
});
