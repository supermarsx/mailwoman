// Line chart for one or two series that share a unit (t106).
//
// One y-axis only: two measures of different scale go in two charts. 2 px lines
// in the validated series colours, a hairline grid at zero, half and the top
// tick, round tick values, start and end times under the plot. A crosshair snaps
// to the nearest bucket and one readout lists every series at that time. The
// picture is hidden from assistive technology; the card's summary and table
// carry the same values (ChartCard.tsx).

import { For, Show, createMemo, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import type { SeriesSlot } from './ChartCard.tsx';
import { niceMax } from './statsMath.ts';
import * as css from './stats.css.ts';

export interface LineSeries {
  label: string;
  slot: SeriesSlot;
  /** One value per bucket, aligned with `times`; null marks a gap, not a zero. */
  values: readonly (number | null)[];
}

export interface LineChartProps {
  times: readonly number[];
  series: readonly LineSeries[];
  /** Formats an axis tick and a readout value. */
  format: (n: number) => string;
  formatTime: (unixSecs: number) => string;
  /** Counts: the axis ticks stay whole numbers (0, 1, 2 rather than 0, 0.5, 1). */
  whole?: boolean | undefined;
  testId?: string | undefined;
}

const W = 300;
const H = 120;

export function LineChart(props: LineChartProps): JSX.Element {
  const top = createMemo(() => {
    let max = 0;
    for (const s of props.series) for (const v of s.values) if (v !== null && v > max) max = v;
    const top = niceMax(max);
    // Whole-number ticks need an even top of at least 2 (the middle tick is top / 2).
    return props.whole ? Math.max(2, Math.ceil(top / 2) * 2) : top;
  });
  const x = (i: number): number => (props.times.length > 1 ? (i / (props.times.length - 1)) * W : W / 2);
  const y = (v: number): number => H - (v / top()) * H;
  const segments = (values: readonly (number | null)[]): string[] => {
    const out: string[] = [];
    let seg: string[] = [];
    values.forEach((v, i) => {
      if (v === null) { if (seg.length) out.push(seg.join(' ')); seg = []; return; }
      seg.push(`${x(i)},${y(v)}`);
    });
    if (seg.length) out.push(seg.join(' '));
    return out;
  };
  // A lone point between gaps has no segment to draw: mark it with a dot.
  const singles = (values: readonly (number | null)[]): number[] =>
    values.flatMap((v, i) => (v !== null && (values[i - 1] ?? null) === null && (values[i + 1] ?? null) === null ? [i] : []));

  const [hover, setHover] = createSignal<number | null>(null);
  function onMove(e: PointerEvent): void {
    const box = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const n = props.times.length;
    if (box.width === 0 || n === 0) return;
    // SVG coordinates stay left-to-right in RTL pages too.
    const i = Math.round(((e.clientX - box.left) / box.width) * (n - 1));
    setHover(Math.min(n - 1, Math.max(0, i)));
  }

  return (
    <div class={css.plot} data-testid={props.testId} data-line-chart>
      <div class={css.yAxis} aria-hidden="true">
        <span>{props.format(top())}</span>
        <span>{props.format(top() / 2)}</span>
        <span>{props.format(0)}</span>
      </div>
      <div class={css.svgWrap}>
        <svg class={css.svg} viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" aria-hidden="true" onPointerMove={onMove} onPointerLeave={() => setHover(null)}>
          <line class={css.grid} x1="0" y1="0.5" x2={W} y2="0.5" />
          <line class={css.grid} x1="0" y1={H / 2} x2={W} y2={H / 2} />
          <line class={css.grid} x1="0" y1={H - 0.5} x2={W} y2={H - 0.5} />
          <For each={props.series}>
            {(s) => (
              <>
                <For each={segments(s.values)}>{(points) => <polyline class={`${css.line} ${css.lineColour[s.slot]}`} points={points} />}</For>
                <For each={singles(s.values)}>{(i) => <line class={`${css.line} ${css.lineColour[s.slot]}`} x1={x(i)} x2={x(i) + 0.01} y1={y(s.values[i]!)} y2={y(s.values[i]!)} stroke-width="6" />}</For>
              </>
            )}
          </For>
          <Show when={hover() !== null}><line class={css.crosshair} x1={x(hover()!)} x2={x(hover()!)} y1="0" y2={H} /></Show>
          <rect class={css.hit} x="0" y="0" width={W} height={H} />
        </svg>
      </div>
      <div class={css.xAxis} aria-hidden="true">
        <Show when={props.times.length > 0}>
          <span>{props.formatTime(props.times[0]!)}</span>
          <span>{props.formatTime(props.times[props.times.length - 1]!)}</span>
        </Show>
      </div>
      <p class={css.tip} aria-hidden="true" data-chart-tip>
        <Show when={hover() !== null && props.times[hover()!] !== undefined}>
          <span class={css.tipTime}>{props.formatTime(props.times[hover()!]!)}</span>
          <For each={props.series}>
            {(s) => {
              const v = (): number | null => s.values[hover()!] ?? null;
              return (
                <span class={css.tipValue}>
                  <span class={`${css.key} ${css.keyColour[s.slot]}`} />
                  <strong>{v() === null ? t('admin-shell-not-available') : props.format(v()!)}</strong>
                  <Show when={props.series.length > 1}><span class={css.tipTime}>{s.label}</span></Show>
                </span>
              );
            }}
          </For>
        </Show>
      </p>
    </div>
  );
}
