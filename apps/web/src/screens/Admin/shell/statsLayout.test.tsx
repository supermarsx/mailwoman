// t106: figures, charts and tables in their own sections, and the pieces they use.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { StatTile } from './StatTile.tsx';
import { ChartCard } from './ChartCard.tsx';
import { LineChart } from './LineChart.tsx';
import { comparable, deltaOf, deltaTone, maxKnown, niceMax, previousWindow, sumKnown } from './statsMath.ts';
import { ObservabilityStats } from '../ObservabilityStats.tsx';
import { engineDashboard } from '../Dashboard/fixtures.ts';
import { dashboardReturning, renderInShell } from './testkit.tsx';
import { loadRequestHistory, loadNotFoundPaths, type RequestHistory } from '../../../api/admin/requestHistory.ts';

vi.mock('../../../api/admin/requestHistory.ts', () => ({ loadRequestHistory: vi.fn(), loadNotFoundPaths: vi.fn() }));

describe('statsMath', () => {
  it('sums and maximises only measured values; nothing measured is null, not zero', () => {
    expect(sumKnown([1, null, 2])).toBe(3);
    expect(sumKnown([null, undefined])).toBeNull();
    expect(maxKnown([3, null, 7])).toBe(7);
    expect(maxKnown([])).toBeNull();
  });
  it('computes the change from the previous period and its tone', () => {
    expect(deltaOf(110, 100)).toEqual({ direction: 'up', ratio: 0.1, absolute: 10 });
    expect(deltaOf(5, 0)).toEqual({ direction: 'up', ratio: null, absolute: 5 });
    expect(deltaOf(0, 0)).toEqual({ direction: 'same', ratio: 0, absolute: 0 });
    expect(deltaOf(5, null)).toBeNull();
    expect(deltaTone(deltaOf(110, 100)!, false)).toBe('bad');
    expect(deltaTone(deltaOf(90, 100)!, false)).toBe('good');
    expect(deltaTone(deltaOf(90, 100)!, undefined)).toBe('flat');
  });
  it('the previous window has the same length and only a mostly observed one compares', () => {
    expect(previousWindow({ from: 1000, to: 4600 })).toEqual({ from: -2600, to: 1000 });
    expect(comparable({ observedSeconds: 3500, requestedSeconds: 3600 })).toBe(true);
    expect(comparable({ observedSeconds: 60, requestedSeconds: 3600 })).toBe(false);
  });
  it('rounds the axis top up to a round number', () => {
    expect(niceMax(0)).toBe(1);
    expect(niceMax(7)).toBe(10);
    expect(niceMax(180)).toBe(200);
    expect(niceMax(2.2)).toBe(2.5);
  });
});

describe('StatTile delta and trend', () => {
  it('says the change in words and arrows, coloured by whether up is good', () => {
    render(() => <StatTile testId="t" label="Server errors" value={12} delta={{ previous: 10, upIsGood: false }} trend={[1, 2, null, 4]} />);
    const tile = screen.getByTestId('t');
    const delta = tile.querySelector('[data-delta]')!;
    expect(delta.getAttribute('data-delta')).toBe('up');
    expect(delta).toHaveTextContent('Up 20% on the previous period.');
    expect(delta).toHaveTextContent('vs previous period');
    const trend = tile.querySelector('svg[data-trend]')!;
    expect(trend.getAttribute('aria-hidden')).toBe('true');
    expect(trend.querySelectorAll('polyline')).toHaveLength(2);
  });
  it('no previous value, no delta; a null value never gets one', () => {
    render(() => <><StatTile testId="a" label="A" value={3} delta={{ previous: null }} /><StatTile testId="b" label="B" value={null} delta={{ previous: 2 }} /></>);
    expect(screen.getByTestId('a').querySelector('[data-delta]')).toBeNull();
    expect(screen.getByTestId('b').querySelector('[data-delta]')).toBeNull();
  });
});

describe('ChartCard', () => {
  const table = { columns: ['Time', 'Value'], numeric: [false, true], rows: [['10:00', '1'], ['10:01', '2']] };
  it('is a region named by its title and described by its summary, with a table switch', () => {
    render(() => <ChartCard title="Requests" unit="Count per 1 min" summary="Latest 2, highest 2" table={table}><div data-testid="plot" /></ChartCard>);
    const region = screen.getByRole('region', { name: 'Requests' });
    expect(region).toHaveAccessibleDescription('Latest 2, highest 2');
    expect(region).toHaveTextContent('Count per 1 min');
    expect(within(region).queryByRole('list', { name: 'Legend' })).toBeNull();
    const toggle = within(region).getByRole('button', { name: 'View as table' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    expect(region.querySelector('[data-chart-table]')!.getAttribute('data-chart-table')).toBe('hidden');
    // The hidden table is still in the document for assistive technology.
    expect(region.querySelectorAll('tbody tr')).toHaveLength(2);
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    expect(screen.queryByTestId('plot')).toBeNull();
    expect(region.querySelector('[data-chart-table]')!.getAttribute('data-chart-table')).toBe('shown');
    expect(region.querySelector('td:last-child')!.className).not.toBe('');
  });
  it('two series get a legend; an empty chart gets a friendly message instead of a plot', () => {
    render(() => <>
      <ChartCard title="Response time" summary="s" table={table} legend={[{ label: 'Median', slot: 1 }, { label: '95th percentile', slot: 2 }]}><div /></ChartCard>
      <ChartCard title="Empty" summary="s" table={table} empty="Nothing yet."><div data-testid="never" /></ChartCard>
    </>);
    const legend = within(screen.getByRole('region', { name: 'Response time' })).getByRole('list', { name: 'Legend' });
    expect(within(legend).getAllByRole('listitem').map((li) => li.textContent)).toEqual(['Median', '95th percentile']);
    const empty = screen.getByRole('region', { name: 'Empty' });
    expect(within(empty).getByRole('status')).toHaveTextContent('Nothing yet.');
    expect(screen.queryByTestId('never')).toBeNull();
    expect(within(empty).queryByRole('button')).toBeNull();
  });
});

describe('LineChart', () => {
  it('one readout lists every series at the hovered time', () => {
    render(() => <LineChart testId="lc" times={[0, 60, 120]} series={[{ label: 'Median', slot: 1, values: [1, 2, 3] }, { label: 'p95', slot: 2, values: [4, null, 6] }]} format={(n) => `${n} ms`} formatTime={(s) => `t${s}`} />);
    const chart = screen.getByTestId('lc');
    const svg = chart.querySelector('svg')!;
    expect(svg.getAttribute('aria-hidden')).toBe('true');
    svg.getBoundingClientRect = () => ({ left: 0, width: 300, top: 0, height: 120, right: 300, bottom: 120, x: 0, y: 0, toJSON: () => ({}) });
    // jsdom has no PointerEvent; a MouseEvent of the same type carries clientX.
    svg.dispatchEvent(new MouseEvent('pointermove', { bubbles: true, clientX: 150 }));
    const tip = chart.querySelector('[data-chart-tip]')!;
    expect(tip).toHaveTextContent('t60');
    expect(tip).toHaveTextContent('2 ms');
    expect(tip).toHaveTextContent('Not available');
    expect(chart).toHaveTextContent('10 ms'); // round axis top
  });
  it('a count axis keeps whole-number ticks, even when every value is zero', () => {
    render(() => <LineChart testId="z" whole times={[0, 60]} series={[{ label: 'Errors', slot: 1, values: [0, 0] }]} format={String} formatTime={String} />);
    const ticks = [...screen.getByTestId('z').firstElementChild!.children].map((el) => el.textContent);
    expect(ticks).toEqual(['2', '1', '0']);
  });
});

const history = (from: number, requests: number, observed = 3600): RequestHistory => ({
  from, to: from + 3600, bucketSeconds: 60,
  series: [{ t: from, requests, errors5xx: 1, notFound: 2, notFoundCoverage: 'complete', p50Ms: 5, p95Ms: 10, coverage: 'complete' }],
  coverage: { observedSeconds: observed, requestedSeconds: 3600, gaps: [] },
  policy: { defaultRangeSeconds: 3600, maxRangeSeconds: 2592000, retentionDays: 30 }, scope: 'server', timing: 'response-ready',
});

describe('Statistics page layout', () => {
  beforeEach(() => {
    vi.mocked(loadRequestHistory).mockReset();
    vi.mocked(loadRequestHistory).mockImplementation(async (range) => (range ? history(range.from, 80) : history(10000, 100)));
    vi.mocked(loadNotFoundPaths).mockReset();
    vi.mocked(loadNotFoundPaths).mockResolvedValue({ from: 0, to: 3600, pathsFrom: 0, pathsTo: 3600, bucketSeconds: 3600, limit: 50, total: 2, approximate: false, paths: [{ method: 'GET', path: '/x', count: 2, minCount: 2 }], coverage: { observedSeconds: 3600, trackedSince: 0, notTracked: [] }, scope: 'server' });
  });

  it('puts tiles, charts and tables in separate sections, in that order', async () => {
    renderInShell(() => <ObservabilityStats />, { loadDashboard: dashboardReturning({ kind: 'ok', data: engineDashboard() }) });
    await screen.findByTestId('spark-requests');
    await screen.findByTestId('not-found-paths');
    const sections = [...document.querySelectorAll('[data-stats-section]')].map((s) => s.getAttribute('data-stats-section'));
    expect(sections).toEqual(['tiles', 'charts', 'tables']);
    const tiles = screen.getByTestId('stats-tiles');
    const charts = screen.getByTestId('stats-charts');
    const tables = screen.getByTestId('stats-tables');
    expect(tiles.querySelectorAll('[data-chart-card]')).toHaveLength(0);
    expect(tiles.querySelectorAll('figure').length).toBeGreaterThanOrEqual(4 + 6);
    expect(charts.querySelectorAll('figure')).toHaveLength(0);
    expect(charts.querySelectorAll('[data-chart-card]').length).toBeGreaterThanOrEqual(4);
    expect(within(tables).getByTestId('not-found-paths')).toBeInTheDocument();
    expect(charts.querySelector('[data-testid="not-found-paths"]')).toBeNull();
    // One period control for the whole page.
    expect(screen.getAllByLabelText('Period')).toHaveLength(1);
  });

  it('compares each traffic total with the period before', async () => {
    renderInShell(() => <ObservabilityStats />, { loadDashboard: dashboardReturning({ kind: 'ok', data: engineDashboard() }) });
    const tile = await screen.findByTestId('tile-requests');
    await waitFor(() => expect(tile.querySelector('[data-delta]')).not.toBeNull());
    expect(vi.mocked(loadRequestHistory).mock.calls[1]![0]).toEqual({ from: 6400, to: 10000 });
    expect(tile).toHaveTextContent('Up 25% on the previous period.');
  });

  it('does not compare with a previous period the server mostly did not observe', async () => {
    vi.mocked(loadRequestHistory).mockImplementation(async (range) => (range ? history(range.from, 80, 60) : history(10000, 100)));
    renderInShell(() => <ObservabilityStats />, { loadDashboard: dashboardReturning({ kind: 'ok', data: engineDashboard() }) });
    const tile = await screen.findByTestId('tile-requests');
    await screen.findByTestId('not-found-paths');
    expect(tile.querySelector('[data-delta]')).toBeNull();
  });

  it('first load shows skeletons the shape of the cards; a period change keeps the page in place', async () => {
    let release: (v: RequestHistory) => void = () => {};
    vi.mocked(loadRequestHistory).mockImplementationOnce(() => new Promise((r) => { release = r; }));
    renderInShell(() => <ObservabilityStats />, { loadDashboard: dashboardReturning({ kind: 'ok', data: engineDashboard() }) });
    await waitFor(() => expect(document.querySelectorAll('[data-skeleton="chart"]').length).toBe(4));
    expect(document.querySelectorAll('[data-skeleton="tile"]').length).toBeGreaterThanOrEqual(4);
    release(history(10000, 100));
    const card = await screen.findByTestId('spark-requests');
    vi.mocked(loadRequestHistory).mockImplementation(() => new Promise(() => {}));
    fireEvent.change(screen.getByLabelText('Period'), { target: { value: '21600' } });
    await waitFor(() => expect(screen.getByTestId('statistics')).toHaveAttribute('aria-busy', 'true'));
    // The same card stays mounted while the next window loads: no skeleton swap.
    expect(screen.getByTestId('spark-requests')).toBe(card);
    expect(document.querySelectorAll('[data-skeleton]')).toHaveLength(0);
  });
});
