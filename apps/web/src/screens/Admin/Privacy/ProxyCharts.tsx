// Charts of the image proxy's worker history (Privacy > Overview and Image
// proxy). The server keeps one sample every 15 s for 24 h in memory and sends the
// requested range downsampled; it is empty after a restart. Every chart is the
// shared ChartCard (summary sentence, "View as table" switch, table always in the
// document for screen readers) around the shared LineChart; the picture draws no
// animation, so reduced-motion needs nothing more.

import { For, Show, type JSX } from 'solid-js';
import { formatBytes, formatNumber, formatTime, t } from '../../../i18n';
import { ChartCard, StatsSection } from '../shell/ChartCard.tsx';
import { LineChart, type LineSeries } from '../shell/LineChart.tsx';
import { maxKnown } from '../shell/statsMath.ts';
import * as css from './privacy.css.ts';
import { HISTORY_RANGES, type HistoryRange, type PrivacyStats, type ProxyHistory } from './model.ts';

const NA = (): string => t('admin-shell-not-available');
type Fmt = (n: number) => string;
const num: Fmt = (n) => formatNumber(n, { maximumFractionDigits: 0 });
const dec: Fmt = (n) => formatNumber(n, { maximumFractionDigits: n < 10 ? 2 : 1 });
const pct: Fmt = (n) => `${formatNumber(n, { maximumFractionDigits: 1 })} %`;
const ms: Fmt = (n) => `${formatNumber(n, { maximumFractionDigits: n < 10 ? 1 : 0 })} ms`;
const perSec: Fmt = (n) => `${dec(n)}/s`;
const bytesPerSec: Fmt = (n) => `${formatBytes(n)}/s`;
const clock = (secs: number): string => formatTime(secs * 1000);

/** Sum of several aligned series; a point is null when none has a value. */
export function sumSeries(parts: readonly (readonly (number | null)[])[], length: number): (number | null)[] {
  return Array.from({ length }, (_, i) => {
    let total: number | null = null;
    for (const p of parts) {
      const v = p[i];
      if (typeof v === 'number') total = (total ?? 0) + v;
    }
    return total;
  });
}

interface SeriesDef { label: string; values: readonly (number | null)[] }

export function HistoryChart(props: { title: string; unit?: string; history: ProxyHistory; series: SeriesDef[]; format: Fmt; whole?: boolean; testId: string }): JSX.Element {
  const times = (): number[] => props.history.t;
  const latest = (s: SeriesDef): string => {
    for (let i = s.values.length - 1; i >= 0; i--) {
      const v = s.values[i];
      if (typeof v === 'number') return props.format(v);
    }
    return NA();
  };
  const highest = (s: SeriesDef): string => {
    const m = maxKnown(s.values);
    return m === null ? NA() : props.format(m);
  };
  const empty = (): string | undefined => (times().length === 0 || props.series.every((s) => maxKnown(s.values) === null) ? t('privacy-history-empty') : undefined);
  const slotOf = (i: number): 1 | 2 => (i === 0 ? 1 : 2);
  const lines = (): LineSeries[] => props.series.map((s, i) => ({ label: s.label, slot: slotOf(i), values: s.values }));
  return (
    <ChartCard
      testId={props.testId}
      title={props.title}
      unit={props.unit}
      legend={props.series.map((s, i) => ({ label: s.label, slot: slotOf(i) }))}
      summary={<>{props.series.map((s) => t('privacy-history-summary', { series: s.label, latest: latest(s), max: highest(s) })).join(' ')}</>}
      table={{
        columns: [t('admin-shell-chart-col-time'), ...props.series.map((s) => s.label)],
        numeric: [false, ...props.series.map(() => true)],
        rows: times().map((time, i) => [clock(time), ...props.series.map((s) => { const v = s.values[i]; return typeof v === 'number' ? props.format(v) : NA(); })]),
      }}
      empty={empty()}
    >
      <LineChart times={times()} series={lines()} format={props.format} formatTime={clock} whole={props.whole} />
    </ChartCard>
  );
}

/** Range selector, last-updated stamp and the "memory only" note above the charts. */
export function HistoryControls(props: { range: HistoryRange; onRange: (r: HistoryRange) => void; updatedAt: number | null; loading: boolean; onRefresh: () => void; history?: ProxyHistory | undefined }): JSX.Element {
  return (
    <div data-testid="history-controls">
      <div role="group" aria-label={t('privacy-history-range')} class={css.controls}>
        <For each={HISTORY_RANGES}>{(r) => <button type="button" class="btn btn--ghost" aria-pressed={props.range === r} onClick={() => props.onRange(r)}>{t(`privacy-history-range-${r}`)}</button>}</For>
        <button type="button" class="btn" disabled={props.loading} onClick={props.onRefresh}>{t('privacy-refresh')}</button>
        <span role="status" data-testid="history-updated">{props.updatedAt === null ? t('privacy-history-not-updated') : t('privacy-history-updated', { time: formatTime(props.updatedAt) })}</span>
      </div>
      <p>{t('privacy-history-memory-note', { interval: props.history?.sampleSecs ?? 15 })}</p>
    </div>
  );
}

const slotValues = (h: ProxyHistory, name: string): (number | null)[] => h.series[name] ?? [];

/** The charts of one view. `overview` shows the headline set; `proxy` the worker detail. */
export function ProxyCharts(props: { stats: PrivacyStats | undefined; loading: boolean; error: unknown; view: 'overview' | 'proxy' | 'traffic' }): JSX.Element {
  const history = (): ProxyHistory | undefined => props.stats?.history;
  return (
    <Show
      when={history()}
      fallback={<p role="status" data-testid="history-pending">{props.loading && !props.error ? t('admin-shell-loading') : t('privacy-history-unavailable')}</p>}
    >
      {(h) => {
        const L = (key: string): string => t(`privacy-chart-${key}`);
        const classes = (): { allowed: (number | null)[]; refused: (number | null)[]; failed: (number | null)[] }[] => Object.values(h().classes ?? {});
        const total = (pick: 'allowed' | 'refused' | 'failed'): (number | null)[] => sumSeries(classes().map((c) => c[pick]), h().t.length);
        return (
          <div aria-busy={props.loading} data-testid={`proxy-charts-${props.view}`}>
            <Show when={props.view !== 'traffic'}>
            <StatsSection title={t(props.view === 'overview' ? 'privacy-history-title' : 'privacy-history-worker-title')} kind="charts">
              <HistoryChart testId="chart-requests" title={L('requests')} unit={L('requests-unit')} history={h()} series={[{ label: L('requests'), values: slotValues(h(), 'requestsPerSec') }]} format={perSec} />
              <HistoryChart testId="chart-bytes" title={L('bytes')} unit={L('bytes-unit')} history={h()} series={[{ label: L('bytes'), values: slotValues(h(), 'bytesPerSec') }]} format={bytesPerSec} />
              <HistoryChart testId="chart-hit" title={L('hit')} unit={L('hit-unit')} history={h()} series={[{ label: L('ram'), values: slotValues(h(), 'ramHitPct') }, { label: L('disk'), values: slotValues(h(), 'diskHitPct') }]} format={pct} />
              <HistoryChart testId="chart-queue" title={L('queue')} unit={L('queue-unit')} history={h()} series={[{ label: L('active'), values: slotValues(h(), 'activeFetches') }, { label: L('queued'), values: slotValues(h(), 'queueDepth') }]} format={num} whole />
              <HistoryChart testId="chart-busy" title={L('busy')} unit={L('busy-unit')} history={h()} series={[{ label: '429', values: slotValues(h(), 'busy429') }, { label: '503', values: slotValues(h(), 'busy503') }]} format={num} whole />
              <Show when={props.view === 'overview'}>
                <HistoryChart testId="chart-trackers" title={L('trackers')} unit={L('trackers-unit')} history={h()} series={[{ label: L('trackers'), values: slotValues(h(), 'trackersReplaced') }]} format={num} whole />
              </Show>
            </StatsSection>
            </Show>
            <Show when={props.view === 'proxy'}>
              <StatsSection title={t('privacy-history-cache-title')} kind="charts">
                <HistoryChart testId="chart-size" title={L('size')} unit={L('size-unit')} history={h()} series={[{ label: L('ram'), values: slotValues(h(), 'ramBytes') }, { label: L('disk'), values: slotValues(h(), 'diskBytes') }]} format={(n) => formatBytes(n)} />
                <HistoryChart testId="chart-entries" title={L('entries')} unit={L('entries-unit')} history={h()} series={[{ label: L('ram'), values: slotValues(h(), 'ramEntries') }, { label: L('disk'), values: slotValues(h(), 'diskEntries') }]} format={num} whole />
                <HistoryChart testId="chart-wait" title={L('wait')} unit={L('wait-unit')} history={h()} series={[{ label: L('wait'), values: slotValues(h(), 'waitMs') }]} format={ms} />
                <HistoryChart testId="chart-rate" title={L('rate')} unit={L('rate-unit')} history={h()} series={[{ label: L('burst'), values: slotValues(h(), 'rateBurst') }, { label: L('accounts'), values: slotValues(h(), 'activeAccounts') }]} format={num} whole />
              </StatsSection>
            </Show>
            <Show when={props.view === 'traffic'}>
              <StatsSection title={t('privacy-history-traffic-title')} kind="charts">
                <HistoryChart testId="chart-allowed" title={L('allowed')} unit={L('traffic-unit')} history={h()} series={[{ label: t('privacy-allowed'), values: total('allowed') }]} format={num} whole />
 <HistoryChart testId="chart-trackers" title={L('trackers')} unit={L('trackers-unit')} history={h()} series={[{ label: L('trackers'), values: slotValues(h(), 'trackersReplaced') }]} format={num} whole />
                <HistoryChart testId="chart-refused" title={L('refused')} unit={L('traffic-unit')} history={h()} series={[{ label: t('privacy-refused'), values: total('refused') }, { label: t('privacy-failed'), values: total('failed') }]} format={num} whole />
              </StatsSection>
            </Show>
          </div>
        );
      }}
    </Show>
  );
}
