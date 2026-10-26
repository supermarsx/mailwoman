// A one-series trend line (t32-admin.md §2.1/§2.2).
//
// Inline SVG, one series per chart (two measures of different scale get two
// charts, never two axes). The title names the series, so there is no legend.
// Pointer hover shows the bucket's time and value; the text alternative is a
// visually hidden table with every point, and a visible summary line gives the
// latest and highest values in text.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { formatNumber, formatTime, t } from '../../../i18n';
import * as css from './shell.css.ts';

export interface SparkPoint {
  /** Bucket start (Unix seconds). */
  t: number;
  value: number;
}

export interface SparklineProps {
  title: string;
  points: readonly SparkPoint[];
  /** Formats a value for the summary, hover and table. */
  format?: (n: number) => string;
  /** Formats a bucket time. */
  formatTime?: (unixSecs: number) => string;
  tone?: 'accent' | 'danger';
  /** Shown instead of the chart when there are no points. */
  emptyText?: string;
  testId?: string;
}

const W = 100;
const H = 48;

export function Sparkline(props: SparklineProps): JSX.Element {
  const id = createUniqueId();
  const fmt = (n: number): string => (props.format ? props.format(n) : formatNumber(n));
  const fmtTime = (s: number): string =>
    props.formatTime ? props.formatTime(s) : formatTime(s * 1000);

  const geom = createMemo(() => {
    const pts = props.points;
    const max = Math.max(0, ...pts.map((p) => p.value));
    const step = pts.length > 1 ? W / (pts.length - 1) : 0;
    const xy = pts.map((p, i) => ({
      x: pts.length > 1 ? i * step : W / 2,
      y: max > 0 ? H - (p.value / max) * (H - 2) - 1 : H - 1,
    }));
    return { max, step, xy };
  });
  const [hover, setHover] = createSignal<number | null>(null);

  function onMove(e: PointerEvent): void {
    const svg = e.currentTarget as SVGSVGElement;
    const box = svg.getBoundingClientRect();
    if (box.width === 0 || props.points.length === 0) return;
    // SVG coordinates remain left-to-right even when surrounding text is RTL.
    const frac = (e.clientX - box.left) / box.width;
    const i = Math.round(frac * (props.points.length - 1));
    setHover(Math.min(props.points.length - 1, Math.max(0, i)));
  }

  const latest = (): SparkPoint | undefined => props.points[props.points.length - 1];

  return (
    <section class={css.chart} aria-labelledby={`${id}-title`} data-testid={props.testId}>
      <h3 id={`${id}-title`} class={css.chartTitle}>
        {props.title}
      </h3>
      <Show when={props.points.length > 0} fallback={<p class={css.chartSummary}>{props.emptyText}</p>}>
        <svg
          class={css.chartSvg}
          viewBox={`0 0 ${W} ${H}`}
          preserveAspectRatio="none"
          aria-hidden="true"
          onPointerMove={onMove}
          onPointerLeave={() => setHover(null)}
        >
          <line class={css.chartBase} x1="0" y1={H - 0.5} x2={W} y2={H - 0.5} />
          <polyline
            class={[css.chartLine, props.tone === 'danger' ? css.chartLineDanger : ''].join(' ')}
            points={geom()
              .xy.map((p) => `${p.x},${p.y}`)
              .join(' ')}
          />
          <rect class={css.chartHit} x="0" y="0" width={W} height={H} />
          <Show when={hover() !== null && geom().xy[hover()!]}>
            {(p) => <rect class={css.chartDot} x={p().x - 1} y={p().y - 3} width="2" height="6" />}
          </Show>
        </svg>
        <p class={css.chartTip} aria-hidden="true">
          <Show when={hover() !== null && props.points[hover()!]}>
            {(p) => <span>{t('admin-shell-chart-point', { time: fmtTime(p().t), value: fmt(p().value) })}</span>}
          </Show>
        </p>
        <p class={css.chartSummary}>
          {t('admin-shell-chart-summary', {
            latest: fmt(latest()?.value ?? 0),
            max: fmt(geom().max),
          })}
        </p>
        <table class={css.srOnly}>
          <caption>{props.title}</caption>
          <thead>
            <tr>
              <th scope="col">{t('admin-shell-chart-col-time')}</th>
              <th scope="col">{t('admin-shell-chart-col-value')}</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.points}>
              {(p) => (
                <tr>
                  <td>{fmtTime(p.t)}</td>
                  <td>{fmt(p.value)}</td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </Show>
    </section>
  );
}
