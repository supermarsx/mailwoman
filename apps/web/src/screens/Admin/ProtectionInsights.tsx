// Request protection: graphs and counters for the Aggregate and Adaptive
// subtabs and a compact summary for the Overview. The data is the server's
// per-minute decision counts (`GET /bans/protection/history?range=`), kept in
// memory since the server started: counts only, never an address or an account.
// One range selector at screen level drives every card. Charts are the console's
// shared ChartCard (summary, "View as table" switch, legend line keys), LineChart
// and BarRows, so colour is never the only encoding and tooltips and the
// crosshair come with them.

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { PROTECTION_RANGES, type ProtectionApi, type ProtectionHistory, type ProtectionRange, type ProtectionStatus } from '../../api/admin/requestProtection.ts';
import { formatDateTime, formatNumber, formatPercent, formatRelative, t } from '../../i18n';
import { formatTime } from '../../i18n/format.ts';
import { resourceValue } from '../../components/resourceValue.ts';
import { StatTile } from './shell/StatTile.tsx';
import { BarRows } from './shell/BarChart.tsx';
import { LineChart } from './shell/LineChart.tsx';
import { ChartCard, ChartSkeleton, TileSkeleton } from './shell/ChartCard.tsx';
import { lastSimulation } from './simulationMemory.ts';
import { formatSeconds } from './protectionRules.ts';
import * as stats from './shell/stats.css.ts';
import * as admin from './admin.css.ts';
import * as css from './protocolGuard.css.ts';

const KNOWN_CLASSES = ['missing', 'uncommon', 'automation', 'search', 'ai-training', 'ai-search', 'ai-user', 'custom'];
const kebab = (rule: string): string => rule.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
export const classLabel = (name: string): string => (KNOWN_CLASSES.includes(name) ? t(`admin-protect-sim-uaclass-${name}`) : name === 'other' ? t('admin-insights-class-other') : name);
export const ruleLabel = (rule: string): string => t(`admin-protection-reason-${kebab(rule)}`);
const limited = (o: { throttled: number; blocked: number }): number => o.throttled + o.blocked;
const share = (part: number, whole: number): number => (whole > 0 ? part / whole : 0);

/** The selected range, shared by every card of the screen. */
export interface ProtectionHistoryController {
  range: () => ProtectionRange;
  setRange: (r: ProtectionRange) => void;
  data: () => ProtectionHistory | null | undefined;
  loading: () => boolean;
  failed: () => boolean;
  supported: () => boolean;
  reload: () => void;
}
export function createProtectionHistory(api: () => ProtectionApi, initial: ProtectionRange = '1h'): ProtectionHistoryController {
  const [range, setRange] = createSignal<ProtectionRange>(initial);
  const [failed, setFailed] = createSignal(false);
  const [resource, { refetch }] = createResource(range, async (r): Promise<ProtectionHistory | null> => {
    const load = api().history;
    if (!load) return null;
    try {
      setFailed(false);
      return await load.call(api(), r);
    } catch {
      setFailed(true);
      return null;
    }
  });
  return {
    range,
    setRange,
    data: () => resourceValue(resource),
    loading: () => resource.loading,
    failed,
    supported: () => api().history !== undefined,
    reload: () => void refetch(),
  };
}

export function RangeSelect(props: { history: ProtectionHistoryController }): JSX.Element {
  return (
    <label class={css.rangeField} data-testid="protection-range">
      <span>{t('admin-insights-range')}</span>
      <select aria-label={t('admin-insights-range')} value={props.history.range()} onChange={(e) => props.history.setRange(e.currentTarget.value as ProtectionRange)}>
        <For each={PROTECTION_RANGES}>{(r) => <option value={r} selected={props.history.range() === r}>{t(`admin-insights-range-${r}`)}</option>}</For>
      </select>
    </label>
  );
}

/** Loading, error and empty lines shared by both subtabs. */
function Gate(props: { history: ProtectionHistoryController; children: (h: ProtectionHistory) => JSX.Element }): JSX.Element {
  const h = props.history;
  return (
    <Show when={h.supported()} fallback={<p class={admin.note} role="status">{t('admin-insights-unsupported')}</p>}>
      <Show when={h.data()} fallback={
        <Show when={h.loading()} fallback={<p class={admin.note} role="alert" data-testid="insights-error">{t(h.failed() ? 'admin-insights-error' : 'admin-insights-unsupported')}</p>}>
          <div class={stats.tiles} aria-busy="true"><For each={[1, 2, 3, 4]}>{() => <TileSkeleton />}</For></div>
          <ChartSkeleton />
        </Show>
      }>
        {(data) => props.children(data())}
      </Show>
    </Show>
  );
}

const axisTime = (): ((s: number) => string) => (s) => formatTime(s * 1000);
const stamp = (secs: number): string => formatDateTime(secs * 1000);
function sinceNote(d: ProtectionHistory): JSX.Element {
  return <p class={admin.note} data-testid="insights-scope">{t('admin-insights-scope', { from: stamp(d.start), step: Math.round(d.stepSecs / 60) })}</p>;
}
const topOf = <T,>(rows: readonly T[], value: (r: T) => number): T | undefined => rows.filter((r) => value(r) > 0).sort((a, b) => value(b) - value(a))[0];

export function AggregateInsights(props: { history: ProtectionHistoryController }): JSX.Element {
  return (
    <section aria-label={t('admin-insights-aggregate')} data-testid="aggregate-insights">
      <Gate history={props.history}>
        {(d) => {
          const topBudget = topOf(d.budgets, limited);
          const topClass = topOf(d.userAgents, limited);
          const budgetName = (b: string): string => t(`admin-insights-budget-${b}`);
          return (
            <>
              <div class={stats.tiles} data-testid="aggregate-tiles">
                <StatTile label={t('admin-insights-evaluated')} value={d.totals.evaluated} testId="agg-evaluated" />
                <StatTile label={t('admin-insights-allowed')} value={d.totals.allowed} testId="agg-allowed" />
                <StatTile label={t('admin-insights-throttled')} value={d.totals.throttled} testId="agg-throttled" />
                <StatTile label={t('admin-insights-blocked')} value={d.totals.blocked} testId="agg-blocked" />
                <StatTile label={t('admin-insights-limited-share')} value={formatPercent(share(limited(d.totals), d.totals.evaluated))} detail={t('admin-insights-limited-share-help')} testId="agg-share" />
                <StatTile label={t('admin-insights-top-budget')} value={topBudget ? budgetName(topBudget.budget) : t('admin-insights-none')} detail={topBudget ? t('admin-insights-limited-count', { count: formatNumber(limited(topBudget)) }) : undefined} testId="agg-top-budget" />
                <StatTile label={t('admin-insights-top-class')} value={topClass ? classLabel(topClass.class) : t('admin-insights-none')} detail={topClass ? t('admin-insights-limited-count', { count: formatNumber(limited(topClass)) }) : undefined} testId="agg-top-class" />
              </div>
              {sinceNote(d)}
              <div class={stats.charts}>
                <ChartCard
                  title={t('admin-insights-over-time')}
                  unit={t('admin-insights-per-step', { minutes: Math.round(d.stepSecs / 60) })}
                  legend={[{ label: t('admin-insights-allowed'), slot: 1 }, { label: t('admin-insights-limited'), slot: 2 }]}
                  summary={t('admin-insights-over-time-summary', { allowed: formatNumber(d.totals.allowed), throttled: formatNumber(d.totals.throttled), blocked: formatNumber(d.totals.blocked) })}
                  table={{
                    columns: [t('admin-insights-time'), t('admin-insights-allowed'), t('admin-insights-throttled'), t('admin-insights-blocked')],
                    numeric: [false, true, true, true],
                    rows: d.points.map((p) => [stamp(p.t), formatNumber(p.allowed), formatNumber(p.throttled), formatNumber(p.blocked)]),
                  }}
                  empty={d.totals.evaluated === 0 ? t('admin-insights-empty') : undefined}
                  testId="aggregate-chart-time"
                >
                  <LineChart times={d.points.map((p) => p.t)} series={[{ label: t('admin-insights-allowed'), slot: 1, values: d.points.map((p) => p.allowed) }, { label: t('admin-insights-limited'), slot: 2, values: d.points.map((p) => p.throttled + p.blocked) }]} format={formatNumber} formatTime={axisTime()} whole />
                </ChartCard>
                <ChartCard
                  title={t('admin-insights-budgets')}
                  unit={t('admin-insights-budgets-unit')}
                  summary={t('admin-insights-budgets-summary')}
                  table={{
                    columns: [t('admin-insights-budget'), t('admin-insights-allowed'), t('admin-insights-throttled'), t('admin-insights-blocked'), t('admin-insights-limited-share')],
                    numeric: [false, true, true, true, true],
                    rows: d.budgets.map((b) => [budgetName(b.budget), formatNumber(b.allowed), formatNumber(b.throttled), formatNumber(b.blocked), formatPercent(share(limited(b), b.allowed + limited(b)))]),
                  }}
                  empty={d.budgets.every((b) => b.allowed + limited(b) === 0) ? t('admin-insights-empty') : undefined}
                  testId="aggregate-chart-budgets"
                >
                  <BarRows bars={d.budgets.map((b) => ({ label: budgetName(b.budget), value: Math.round(share(limited(b), b.allowed + limited(b)) * 1000) / 10 }))} format={(n) => formatPercent(n / 100)} />
                </ChartCard>
                <ChartCard
                  title={t('admin-insights-classes')}
                  unit={t('admin-insights-classes-unit')}
                  summary={t('admin-insights-classes-summary')}
                  table={{
                    columns: [t('admin-insights-class'), t('admin-insights-allowed'), t('admin-insights-throttled'), t('admin-insights-blocked')],
                    numeric: [false, true, true, true],
                    rows: d.userAgents.map((u) => [classLabel(u.class), formatNumber(u.allowed), formatNumber(u.throttled), formatNumber(u.blocked)]),
                  }}
                  empty={d.userAgents.length === 0 ? t('admin-insights-empty-classes') : undefined}
                  testId="aggregate-chart-classes"
                >
                  <BarRows bars={d.userAgents.map((u) => ({ label: classLabel(u.class), value: u.allowed + u.throttled + u.blocked }))} format={formatNumber} />
                </ChartCard>
              </div>
            </>
          );
        }}
      </Gate>
    </section>
  );
}

function SimulationLine(): JSX.Element {
  const last = lastSimulation;
  return (
    <div class={admin.card} data-testid="insights-simulation">
      <h3 class={admin.heading}>{t('admin-insights-simulation')}</h3>
      <Show when={last()} fallback={<p class={admin.note}>{t('admin-insights-simulation-none')}</p>}>
        {(l) => {
          const sim = l().result;
          return (
            <p class={admin.note}>
              {sim.off ? t('admin-protect-sim-off') : sim.exempt ? t('admin-protect-sim-exempt') : sim.action === 'allow' ? t('admin-protect-sim-allow') : t(sim.enforced ? 'admin-protect-sim-enforced' : 'admin-protect-sim-observed', { action: t(`admin-protect-sim-action-${sim.action}`), duration: formatSeconds(sim.durationSecs), rule: t(`admin-protection-reason-${sim.reason ?? 'request-rate'}`) })}{' '}
              <time datetime={new Date(l().at).toISOString()} title={formatDateTime(l().at)}>{formatRelative(new Date(l().at).toISOString())}</time>
            </p>
          );
        }}
      </Show>
    </div>
  );
}

export function AdaptiveInsights(props: { history: ProtectionHistoryController; status?: ProtectionStatus | undefined; objects?: number | null | undefined }): JSX.Element {
  return (
    <section aria-label={t('admin-insights-adaptive')} data-testid="adaptive-insights">
      <Gate history={props.history}>
        {(d) => {
          const restrictions = d.actions.throttleEnforced + d.actions.blockEnforced;
          const observed = d.actions.throttleObserved + d.actions.blockObserved;
          const tierTotal = d.tiers.unknown + d.tiers.known + d.tiers.authenticated;
          const mode = props.status?.policy.adaptiveMode;
          const topRule = d.rules[0];
          const tiers = [
            { id: 'unknown', value: d.tiers.unknown },
            { id: 'known', value: d.tiers.known },
            { id: 'authenticated', value: d.tiers.authenticated },
          ];
          return (
            <>
              <div class={stats.tiles} data-testid="adaptive-tiles">
                <StatTile label={t('admin-insights-mode')} value={mode ? t(`admin-protection-mode-${mode}`) : t('admin-insights-none')} testId="ada-mode" />
                <StatTile label={t('admin-insights-objects')} value={props.objects ?? null} unavailableReason={t('admin-insights-objects-unavailable')} detail={props.status ? t('admin-insights-objects-capacity', { capacity: formatNumber(props.status.subjectCapacity) }) : undefined} testId="ada-objects" />
                <StatTile label={t('admin-insights-restrictions')} value={restrictions} detail={t('admin-insights-restrictions-help')} testId="ada-restrictions" />
                <StatTile label={t('admin-insights-observed')} value={observed} detail={t('admin-insights-observed-help')} testId="ada-observed" />
                <StatTile label={t('admin-insights-top-rule')} value={topRule ? ruleLabel(topRule.rule) : t('admin-insights-none')} detail={topRule ? t('admin-insights-times', { count: formatNumber(topRule.count) }) : undefined} testId="ada-top-rule" />
                <For each={tiers}>{(x) => <StatTile label={t('admin-insights-tier-requests', { tier: t(`admin-protect-sim-tier-${x.id}`) })} value={x.value} testId={`ada-tier-${x.id}`} />}</For>
              </div>
              {sinceNote(d)}
              <p class={admin.note}>{t('admin-insights-no-deescalation')}</p>
              <div class={stats.charts}>
                <ChartCard
                  title={t('admin-insights-actions-time')}
                  unit={t('admin-insights-per-step', { minutes: Math.round(d.stepSecs / 60) })}
                  legend={[{ label: t('admin-insights-throttles'), slot: 1 }, { label: t('admin-insights-blocks'), slot: 2 }]}
                  summary={t('admin-insights-actions-summary', { throttles: formatNumber(d.actions.throttleObserved + d.actions.throttleEnforced), blocks: formatNumber(d.actions.blockObserved + d.actions.blockEnforced), enforced: formatNumber(restrictions) })}
                  table={{
                    columns: [t('admin-insights-time'), t('admin-insights-throttles'), t('admin-insights-blocks')],
                    numeric: [false, true, true],
                    rows: d.points.map((p) => [stamp(p.t), formatNumber(p.actions.throttleObserved + p.actions.throttleEnforced), formatNumber(p.actions.blockObserved + p.actions.blockEnforced)]),
                  }}
                  empty={restrictions + observed === 0 ? t('admin-insights-empty-actions') : undefined}
                  testId="adaptive-chart-actions"
                >
                  <LineChart times={d.points.map((p) => p.t)} series={[{ label: t('admin-insights-throttles'), slot: 1, values: d.points.map((p) => p.actions.throttleObserved + p.actions.throttleEnforced) }, { label: t('admin-insights-blocks'), slot: 2, values: d.points.map((p) => p.actions.blockObserved + p.actions.blockEnforced) }]} format={formatNumber} formatTime={axisTime()} whole />
                </ChartCard>
                <ChartCard
                  title={t('admin-insights-tiers')}
                  unit={t('admin-insights-tiers-unit')}
                  summary={t('admin-insights-tiers-summary')}
                  table={{
                    columns: [t('admin-insights-tier'), t('admin-insights-requests'), t('admin-insights-share')],
                    numeric: [false, true, true],
                    rows: tiers.map((x) => [t(`admin-protect-sim-tier-${x.id}`), formatNumber(x.value), formatPercent(share(x.value, tierTotal))]),
                  }}
                  empty={tierTotal === 0 ? t('admin-insights-empty') : undefined}
                  testId="adaptive-chart-tiers"
                >
                  <BarRows bars={tiers.map((x) => ({ label: t(`admin-protect-sim-tier-${x.id}`), value: x.value }))} format={(n) => `${formatNumber(n)} · ${formatPercent(share(n, tierTotal))}`} />
                </ChartCard>
                <ChartCard
                  title={t('admin-insights-rules')}
                  unit={t('admin-insights-rules-unit')}
                  summary={t('admin-insights-rules-summary')}
                  table={{ columns: [t('admin-insights-rule'), t('admin-insights-times-col')], numeric: [false, true], rows: d.rules.map((r) => [ruleLabel(r.rule), formatNumber(r.count)]) }}
                  empty={d.rules.length === 0 ? t('admin-insights-empty-rules') : undefined}
                  testId="adaptive-chart-rules"
                >
                  <BarRows bars={d.rules.map((r) => ({ label: ruleLabel(r.rule), value: r.count }))} format={formatNumber} />
                </ChartCard>
              </div>
              <SimulationLine />
            </>
          );
        }}
      </Gate>
    </section>
  );
}

/** Overview: four tiles, one line, from the same history. */
export function InsightsSummary(props: { history: ProtectionHistoryController }): JSX.Element {
  return (
    <section class={admin.card} aria-label={t('admin-insights-summary')} data-testid="insights-summary">
      <h3 class={admin.heading}>{t('admin-insights-summary')}</h3>
      <Gate history={props.history}>
        {(d) => (
          <>
            <div class={stats.tiles}>
              <StatTile label={t('admin-insights-evaluated')} value={d.totals.evaluated} />
              <StatTile label={t('admin-insights-limited')} value={limited(d.totals)} detail={t('admin-insights-share-of', { share: formatPercent(share(limited(d.totals), d.totals.evaluated)) })} />
              <StatTile label={t('admin-insights-restrictions')} value={d.actions.throttleEnforced + d.actions.blockEnforced} />
              <StatTile label={t('admin-insights-observed')} value={d.actions.throttleObserved + d.actions.blockObserved} />
            </div>
            {sinceNote(d)}
          </>
        )}
      </Gate>
    </section>
  );
}
