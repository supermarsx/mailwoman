// "Saved statistics": the median and 95th percentile (or counts) of one source
// over a chosen range, read from the stored buckets, so the history is there
// after a server restart (docs/admin/metrics-persistence.md).
//
// A range selector (last hour, 24 hours, 7 days, 30 days, 1 year), a "Saved"
// badge with the time of the last save and the stored row count, the first
// stored bucket ("history since"), line charts per group, an empty state when
// persistence is off, and "Reset statistics" (confirmed; the server writes an
// audit entry).

import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import {
  PERSISTED_RANGES,
  loadPersisted,
  resetPersisted,
  type PersistedGroup,
  type PersistedHistory,
  type PersistedRange,
  type PersistedSource,
} from '../../api/admin/persistedMetrics.ts';
import { resourceValue } from '../../components/resourceValue.ts';
import { formatDateTime, formatNumber, formatTime, loadCatalog, t } from '../../i18n';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { StatusBadge } from './shell/StatusBadge.tsx';
import { ChartCard, StatsSection, type ChartTable } from './shell/ChartCard.tsx';
import { LineChart } from './shell/LineChart.tsx';
import { formatMs } from './Dashboard/format.ts';
import { maxKnown } from './shell/statsMath.ts';
import * as stats from './shell/stats.css.ts';
import * as admin from './admin.css.ts';

interface ChartDef {
  id: string;
  /** Fluent id of the card title. */
  title: string;
  /** One group per line: its name in the response and the legend id. */
  lines: readonly { group: string; label: string }[];
}

const LATENCY = [{ group: '', label: 'mp-series-p50' }, { group: '', label: 'mp-series-p95' }] as const;
const latencyChart = (id: string, group: string): ChartDef => ({ id, title: `mp-chart-${id}`, lines: LATENCY.map((l) => ({ ...l, group })) });
const countChart = (id: string, ...groups: string[]): ChartDef => ({ id, title: `mp-chart-${id}`, lines: groups.map((g) => ({ group: g, label: `mp-series-${g}` })) });

/** The cards of each source; latency sources plot p50 and p95, counter sources plot counts. */
const CHARTS: Record<PersistedSource, readonly ChartDef[]> = {
  imageProxy: [latencyChart('image-total', 'total'), latencyChart('image-miss', 'miss'), latencyChart('image-hit', 'hit')],
  clientTiming: [
    latencyChart('client-usable', 'timeToUsable'),
    latencyChart('client-folder-cold', 'folderLoadCold'),
    latencyChart('client-folder-cached', 'folderLoadCached'),
    latencyChart('client-message-cold', 'messageLoadCold'),
    latencyChart('client-message-cached', 'messageLoadCached'),
  ],
  rateLimit: [countChart('rate-allowed', 'allowed'), countChart('rate-limited', 'queued', 'rejected')],
  requestProtection: [countChart('protection-allowed', 'allowed'), countChart('protection-limited', 'throttled', 'blocked')],
};

const rangeLabel = (r: PersistedRange): string => t(`mp-range-${r}`);

function values(group: PersistedGroup | undefined, stat: 'p50' | 'p95' | 'count'): (number | null)[] {
  if (!group) return [];
  return (stat === 'count' ? group.series.count : group.series[stat] ?? []).slice();
}

function Chart(props: { def: ChartDef; data: PersistedHistory }): JSX.Element {
  const counter = (): boolean => props.data.counter;
  const lines = (): { label: string; slot: 1 | 2; values: (number | null)[] }[] =>
    props.def.lines.map((l, i) => {
      const group = props.data.groups[l.group];
      return {
        label: t(l.label),
        slot: (i === 0 ? 1 : 2) as 1 | 2,
        values: counter() ? values(group, 'count') : values(group, i === 0 ? 'p50' : 'p95'),
      };
    });
  const times = (): number[] => props.data.groups[props.def.lines[0]?.group ?? '']?.series.t ?? [];
  const format = (n: number): string => (counter() ? formatNumber(n) : formatMs(n));
  const axis = (s: number): string => (props.data.to - props.data.from > 86400 ? formatDateTime(s * 1000, { dateStyle: 'short', timeStyle: 'short' }) : formatTime(s * 1000));
  const empty = (): boolean => lines().every((l) => maxKnown(l.values) === null || (counter() && maxKnown(l.values) === 0));
  const table = (): ChartTable => ({
    columns: [t('admin-shell-chart-col-time'), ...lines().map((l) => l.label)],
    numeric: [false, ...lines().map(() => true)],
    rows: times().map((s, i) => [formatDateTime(s * 1000), ...lines().map((l) => (l.values[i] === null || l.values[i] === undefined ? t('admin-shell-not-available') : format(l.values[i] as number)))]),
  });
  return (
    <ChartCard
      testId={`persisted-chart-${props.def.id}`}
      title={t(props.def.title)}
      unit={counter() ? t('mp-unit-count', { step: formatNumber(Math.round(props.data.stepSecs / 60)) }) : t('mp-unit-ms')}
      legend={lines().map((l) => ({ label: l.label, slot: l.slot }))}
      summary={<>{lines().map((l) => t('mp-chart-summary', { series: l.label, max: maxKnown(l.values) === null ? t('admin-shell-not-available') : format(maxKnown(l.values) as number) })).join(' ')}</>}
      table={table()}
      empty={empty() ? t('mp-chart-empty') : undefined}
    >
      <LineChart times={times()} series={lines()} format={format} formatTime={axis} whole={counter()} />
    </ChartCard>
  );
}

/** Saved statistics of one source with a range selector, the Saved badge and a reset action. */
export function PersistedStats(props: { source: PersistedSource; canReset?: boolean }): JSX.Element {
  const [ready, setReady] = createSignal(false);
  const [range, setRange] = createSignal<PersistedRange>('24h');
  const [reload, setReload] = createSignal(0);
  const [confirming, setConfirming] = createSignal(false);
  onMount(() => void loadCatalog('admin-metrics-persisted').then(() => setReady(true)));
  const [result] = createResource(
    () => (ready() ? { range: range(), reload: reload() } : undefined),
    (key) => loadPersisted(props.source, key.range).then((data) => ({ data }), () => ({ failed: true as const })),
  );
  const loaded = () => resourceValue(result);
  const data = (): PersistedHistory | undefined => {
    const v = loaded();
    return v && 'data' in v ? v.data : undefined;
  };
  const failed = (): boolean => {
    const v = loaded();
    return !!v && 'failed' in v;
  };
  const since = (d: PersistedHistory): string => (d.persistence.historySince === null ? t('admin-shell-not-available') : formatDateTime(d.persistence.historySince * 1000));
  return (
    <Show when={ready()}>
      <StatsSection kind="charts" title={t('mp-title')} testId={`persisted-${props.source}`} busy={result.loading}>
        <div role="group" aria-label={t('mp-range-label')} class={admin.note} data-testid="persisted-ranges">
          <For each={PERSISTED_RANGES}>{(r) => (
            <button type="button" class="btn btn--ghost" aria-pressed={range() === r} data-range={r} onClick={() => setRange(r)}>{rangeLabel(r)}</button>
          )}</For>
        </div>
        <Show when={failed()}><p role="status" class={admin.note} data-testid="persisted-failed">{t('mp-failed')}</p></Show>
        <Show when={data()}>{(d) => (
          <>
            <Show
              when={d().persistence.enabled}
              fallback={<p role="status" class={admin.note} data-testid="persisted-off">{t('mp-off')}</p>}
            >
              <p class={admin.note} data-testid="persisted-status">
                <StatusBadge tone="ok" status="persisted" testId="persisted-badge">{t('mp-badge')}</StatusBadge>{' '}
                <span data-testid="persisted-last-flush">
                  {d().persistence.lastFlushAt === null ? t('mp-never-saved') : t('mp-last-saved', { time: formatDateTime(d().persistence.lastFlushAt! * 1000) })}
                </span>{' '}
                <span data-testid="persisted-rows">{t('mp-rows', { count: formatNumber(d().persistence.rows) })}</span>{' '}
                <span data-testid="persisted-since">{t('mp-since', { time: since(d()) })}</span>
              </p>
              <p class={admin.note}>{t('mp-survives', { seconds: formatNumber(d().persistence.flushSeconds) })}{d().counter ? '' : ` ${t('mp-error', { error: formatNumber(Math.round(d().relativeError * 1000) / 10, { maximumFractionDigits: 1 }) })}`}</p>
              <div class={stats.charts}>
                <For each={CHARTS[props.source]}>{(def) => <Chart def={def} data={d()} />}</For>
              </div>
            </Show>
            <Show when={props.canReset !== false && (d().persistence.rows > 0 || d().persistence.enabled)}>
              <p>
                <button type="button" class="btn btn--ghost" data-testid="persisted-reset" onClick={() => setConfirming(true)}>{t('mp-reset')}</button>
              </p>
            </Show>
          </>
        )}</Show>
        <ConfirmDialog
          open={confirming()}
          title={t('mp-reset-title')}
          body={t('mp-reset-body')}
          confirmLabel={t('mp-reset-confirm')}
          testId="persisted-reset-dialog"
          onCancel={() => setConfirming(false)}
          onConfirm={async () => {
            await resetPersisted(props.source);
            setConfirming(false);
            setReload(reload() + 1);
          }}
        />
      </StatsSection>
    </Show>
  );
}
