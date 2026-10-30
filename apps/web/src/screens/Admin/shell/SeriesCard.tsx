// A one-series chart in its own card (t106): the ChartCard + LineChart pair for
// a series of timed values, with the summary "Latest …, highest …" and a
// time/value table. Durations read in one unit chosen from the largest value
// when `formatSet` is given (t90).

import { onMount, type JSX } from 'solid-js';
import { formatTime as clock, loadCatalog, t } from '../../../i18n';
import { ChartCard } from './ChartCard.tsx';
import { LineChart } from './LineChart.tsx';
import { maxKnown } from './statsMath.ts';
import type { SparkPoint } from './Sparkline.tsx';

export interface SeriesCardProps {
  title: string;
  unit?: string | undefined;
  points: readonly SparkPoint[];
  format: (n: number, resolution?: number | null) => string;
  formatSet?: ((values: readonly (number | null)[]) => (n: number, resolution?: number | null) => string) | undefined;
  /** Table and summary times. */
  formatTime?: ((unixSecs: number) => string) | undefined;
  /** Axis times; defaults to `formatTime`. */
  axisTime?: ((unixSecs: number) => string) | undefined;
  /** Leads the summary, e.g. what period the chart covers. */
  about?: string | undefined;
  empty?: string | undefined;
  testId?: string | undefined;
}

export function SeriesCard(props: SeriesCardProps): JSX.Element {
  onMount(() => void loadCatalog('admin-stats'));
  const values = (): (number | null)[] => props.points.map((p) => p.value);
  const set = (): ((n: number, r?: number | null) => string) => props.formatSet?.(values()) ?? props.format;
  const time = (s: number): string => (props.formatTime ? props.formatTime(s) : clock(s * 1000));
  const absent = (): string => t('admin-shell-not-available');
  const cell = (v: number | null, r?: number | null): string => (v === null ? absent() : set()(v, r));
  const max = (): number | null => maxKnown(values());
  const last = (): SparkPoint | undefined => props.points[props.points.length - 1];
  const maxPoint = (): SparkPoint | undefined => props.points.find((p) => p.value === max());
  return (
    <ChartCard
      testId={props.testId}
      title={props.title}
      unit={props.unit}
      summary={<>{props.about ? `${props.about} ` : ''}{t('admin-shell-chart-summary', { latest: cell(last()?.value ?? null, last()?.resolution), max: cell(max(), maxPoint()?.resolution) })}</>}
      table={{
        columns: [t('admin-shell-chart-col-time'), t('admin-shell-chart-col-value')],
        numeric: [false, true],
        rows: props.points.map((p) => [time(p.t), cell(p.value, p.resolution)]),
      }}
      empty={props.points.length === 0 || max() === null ? (props.empty ?? t('admin-stats-empty-chart')) : undefined}
    >
      <LineChart times={props.points.map((p) => p.t)} series={[{ label: props.title, slot: 1, values: values() }]} format={(n) => set()(n)} formatTime={props.axisTime ?? time} />
    </ChartCard>
  );
}
