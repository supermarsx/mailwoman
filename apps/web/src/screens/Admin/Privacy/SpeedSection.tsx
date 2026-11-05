// "Speed" section of the Image proxy tab: how long proxied images take, where
// the time goes and whether that is healthy. The numbers come from the `latency`
// object of the privacy statistics (fixed log-scale histograms merged over the
// selected range, per server process, in memory). The verdict and its thresholds
// are decided by the server and shown as sent.

import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { formatNumber, loadCatalog, t } from '../../../i18n';
import { StatTile } from '../shell/StatTile.tsx';
import { StatusBadge, type StatusTone } from '../shell/StatusBadge.tsx';
import { ChartSkeleton, StatsSection, TableCard, TileSkeleton } from '../shell/ChartCard.tsx';
import * as css from './privacy.css.ts';
import type { LatencySummary, PrivacyStats, ProxyLatency, SpeedVerdict } from './model.ts';
import { HistoryChart } from './ProxyCharts.tsx';

/** Milliseconds as "35 ms" or "2.4 s". */
export const duration = (n: number): string => (n >= 1000 ? `${formatNumber(n / 1000, { maximumFractionDigits: 1 })} s` : `${formatNumber(n, { maximumFractionDigits: n < 10 ? 2 : n < 100 ? 1 : 0 })} ms`);
const NA = (): string => t('admin-shell-not-available');
const TONE: Record<SpeedVerdict, StatusTone> = { healthy: 'ok', slow: 'warn', degraded: 'danger', noData: 'neutral' };
const PHASES = ['queue', 'cache', 'upstream', 'decode'] as const;

const series = (l: ProxyLatency, name: string): (number | null)[] => l.series[name] ?? [];
const count = (l: ProxyLatency, outcome: string): number => l.outcomes[outcome]?.count ?? 0;
const orNA = (n: number | null | undefined): string => (n === null || n === undefined ? NA() : duration(n));
const share = (n: number | null | undefined): string => (n === null || n === undefined ? NA() : `${formatNumber(n, { maximumFractionDigits: 1 })} %`);

function Verdict(props: { latency: ProxyLatency }): JSX.Element {
  const h = () => props.latency.health;
  const th = (k: string): number => h().thresholds[k] ?? 0;
  const reason = (): string => (h().verdict === 'healthy' ? 'ok' : h().reason);
  return (
    <div data-testid="speed-health">
      <StatusBadge tone={TONE[h().verdict]} status={h().verdict} testId="speed-verdict">{t(`privacy-speed-verdict-${h().verdict}`)}</StatusBadge>
      <p data-testid="speed-reason">{t(`privacy-speed-reason-${reason()}`, { failurePct: formatNumber(props.latency.failurePct ?? 0, { maximumFractionDigits: 1 }), min: formatNumber(th('minRequests')) })}</p>
      <p>{t('privacy-speed-thresholds', {
        slowMiss: duration(th('slowMissP95Ms')), degradedMiss: duration(th('degradedMissP95Ms')), slowHit: duration(th('slowHitP95Ms')),
        slowFail: formatNumber(th('slowFailurePct')), degradedFail: formatNumber(th('degradedFailurePct')),
      })}</p>
    </div>
  );
}

function Split(props: { title: string; s: LatencySummary; testId: string }): JSX.Element {
  return (
    <TableCard title={props.title} testId={props.testId}>
      <p>{t('privacy-speed-split-count', { count: formatNumber(props.s.count) })}</p>
      <p>{t('privacy-speed-split-times', { p50: orNA(props.s.p50Ms), p95: orNA(props.s.p95Ms), max: orNA(props.s.maxMs) })}</p>
    </TableCard>
  );
}

function Body(props: { latency: ProxyLatency }): JSX.Element {
  const l = () => props.latency;
  const hist = () => ({ t: l().t, series: l().series });
  return (
    <>
      <Verdict latency={l()} />
      <div class={css.tiles}>
        <StatTile label={t('privacy-speed-median')} value={l().total.p50Ms} format={duration} unavailableReason={NA()} testId="speed-p50" />
        <StatTile label={t('privacy-speed-p95')} value={l().total.p95Ms} format={duration} unavailableReason={NA()} testId="speed-p95" />
        <StatTile label={t('privacy-speed-p99')} value={l().total.p99Ms} format={duration} unavailableReason={NA()} testId="speed-p99" />
        <StatTile label={t('privacy-speed-max')} value={l().total.maxMs} format={duration} unavailableReason={NA()} testId="speed-max" />
        <StatTile label={t('privacy-speed-requests')} value={l().total.count} testId="speed-count" />
        <StatTile label={t('privacy-speed-busy')} value={count(l(), 'busy')} testId="speed-busy" />
        <StatTile label={t('privacy-speed-timeouts')} value={count(l(), 'timeout')} testId="speed-timeouts" />
        <StatTile label={t('privacy-speed-upstream-errors')} value={count(l(), 'upstreamError')} testId="speed-upstream-errors" />
      </div>
      <div class={css.tiles}>
        <Split title={t('privacy-speed-hit')} s={l().hit} testId="speed-hit" />
        <Split title={t('privacy-speed-miss')} s={l().miss} testId="speed-miss" />
      </div>
      <HistoryChart testId="chart-speed" title={t('privacy-speed-chart')} unit={t('privacy-speed-chart-unit')} history={hist()} format={duration}
        series={[{ label: t('privacy-speed-median'), values: series(l(), 'p50Ms') }, { label: t('privacy-speed-p95'), values: series(l(), 'p95Ms') }]} />
      <HistoryChart testId="chart-speed-miss" title={t('privacy-speed-chart-miss')} unit={t('privacy-speed-chart-unit')} history={hist()} format={duration}
        series={[{ label: t('privacy-speed-median'), values: series(l(), 'missP50Ms') }, { label: t('privacy-speed-p95'), values: series(l(), 'missP95Ms') }]} />
      <TableCard title={t('privacy-speed-phases')} testId="speed-phases">
        <p>{t('privacy-speed-phases-help')}</p>
        <table>
          <thead><tr><th scope="col">{t('privacy-speed-phase')}</th><th scope="col">{t('privacy-speed-median')}</th><th scope="col">{t('privacy-speed-p95')}</th><th scope="col">{t('privacy-speed-share')}</th></tr></thead>
          <tbody>
            <For each={PHASES}>{(p) => (
              <tr data-phase={p}>
                <th scope="row">{t(`privacy-speed-phase-${p}`)}</th>
                <td>{orNA(l().phases[p]?.p50Ms)}</td>
                <td>{orNA(l().phases[p]?.p95Ms)}</td>
                <td>{share(l().phases[p]?.sharePct)}</td>
              </tr>
            )}</For>
            <tr data-phase="other"><th scope="row">{t('privacy-speed-phase-other')}</th><td>{NA()}</td><td>{NA()}</td><td>{share(l().otherPct)}</td></tr>
          </tbody>
        </table>
      </TableCard>
      <p>{t('privacy-speed-note', { error: formatNumber(Math.round((l().relativeError ?? 0.0625) * 1000) / 10, { maximumFractionDigits: 1 }) })}</p>
    </>
  );
}

/** Median and p95 over time, tiles, hit and miss, phases and a health verdict for the selected range. */
export function SpeedSection(props: { stats: PrivacyStats | undefined; loading: boolean; error: unknown }): JSX.Element {
  const [ready, setReady] = createSignal(false);
  onMount(() => void loadCatalog('admin-privacy-speed').then(() => setReady(true)));
  const latency = (): ProxyLatency | undefined => props.stats?.latency;
  return (
    <StatsSection title={ready() ? t('privacy-speed-title') : ''} kind="charts" testId="speed-section" busy={props.loading}>
      <Show when={ready()}>
        <Show
          when={latency()}
          fallback={
            <Show
              when={props.loading && !props.error}
              fallback={<p role="status" data-testid="speed-unavailable">{t('privacy-speed-unavailable')}</p>}
            >
              <div data-testid="speed-loading" role="status" aria-label={t('admin-shell-loading')}><TileSkeleton /><ChartSkeleton /></div>
            </Show>
          }
        >
          {(l) => (
            <Show
              when={l().total.count > 0}
              fallback={<p role="status" data-testid="speed-empty">{l().enabled ? t('privacy-speed-empty') : t('privacy-speed-off')}</p>}
            >
              <Show when={!l().enabled}><p role="status" data-testid="speed-off">{t('privacy-speed-off')}</p></Show>
              <Body latency={l()} />
            </Show>
          )}
        </Show>
      </Show>
    </StatsSection>
  );
}
