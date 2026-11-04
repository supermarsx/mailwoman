import { describe, expect, it } from 'vitest';
import { render, screen, within } from '@solidjs/testing-library';
import { loadCatalog } from '../../../i18n';
import { ProxyCharts, sumSeries } from './ProxyCharts.tsx';
import type { PrivacyStats } from './model.ts';

const stats = (history: PrivacyStats['history']): PrivacyStats => ({ cacheEntries: 0, cacheBytes: 0, trackersReplaced: 0, traffic: [], ...(history ? { history } : {}) });

describe('proxy history charts', () => {
  it('adds series point by point and keeps a gap only where every series has one', () => {
    expect(sumSeries([[1, null, 3], [2, null, null]], 3)).toEqual([3, null, 3]);
  });

  it('draws a chart per series group with a screen-reader table of the same values', async () => {
    await loadCatalog('privacy');
    await loadCatalog('admin-shell');
    const history = { t: [100, 200], series: { requestsPerSec: [1, 2], ramHitPct: [10, 20], diskHitPct: [0, null] }, classes: { image: { allowed: [1, 2], refused: [0, 0], failed: [0, 1] } } };
    render(() => <ProxyCharts stats={stats(history)} loading={false} error={undefined} view="proxy" />);
    const card = await screen.findByTestId('chart-requests');
    expect(card.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    expect(within(card).getByRole('table', { hidden: true })).toHaveTextContent('2/s');
    expect(screen.getByTestId('chart-hit')).toHaveTextContent('Cache hit rate');
    expect(screen.getByTestId('chart-size')).toBeInTheDocument();
  });

  it('shows the empty state per chart when there are no samples', async () => {
    await loadCatalog('privacy');
    render(() => <ProxyCharts stats={stats({ t: [], series: {} })} loading={false} error={undefined} view="overview" />);
    expect(await within(await screen.findByTestId('chart-requests')).findByText('No samples in this range yet.')).toBeInTheDocument();
  });

  it('says so when the server sends no history, and while loading', async () => {
    await loadCatalog('privacy');
    const { unmount } = render(() => <ProxyCharts stats={stats(undefined)} loading={false} error={undefined} view="overview" />);
    expect(screen.getByTestId('history-pending')).toHaveTextContent('not available');
    unmount();
  });
});
