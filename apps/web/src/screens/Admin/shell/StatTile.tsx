// A single headline figure (t32-admin.md §2.2).
//
// Shows only a value the server read. `value === null` means the server cannot
// read it in this mode or on this platform: the tile says "Not available" and the
// reason, never a zero or an estimate.
//
// t106: an optional `delta` (the change from a named earlier period, coloured by
// whether up is good, with an arrow and words so colour is never alone) and an
// optional `trend` (a small line in the de-emphasis colour, hidden from assistive
// technology: the chart cards and their tables carry the series).

import { Show, createMemo, createUniqueId, type JSX } from 'solid-js';
import { formatNumber, formatPercent, loadCatalog, t } from '../../../i18n';
import { deltaOf, deltaTone } from './statsMath.ts';
import * as css from './shell.css.ts';
import * as stats from './stats.css.ts';

export interface StatTileDelta {
  /** The same figure over the earlier period; `null` when it was not measured. */
  previous: number | null;
  /** `true` when a rise is good news (sign-ins), `false` when bad (errors); omit for neutral. */
  upIsGood?: boolean | undefined;
}

export interface StatTileProps {
  label: string;
  value: number | string | null;
  /** Formats a numeric value (bytes, durations). Defaults to locale digits. */
  format?: ((n: number) => string) | undefined;
  /** Why the value is absent; shown when `value` is null. */
  unavailableReason?: string | undefined;
  /** A short line under the value, e.g. "of 33 expected". */
  detail?: string | undefined;
  /** Link to the section that explains or fixes the figure. */
  href?: string | undefined;
  linkLabel?: string | undefined;
  onLink?: ((e: MouseEvent) => void) | undefined;
  testId?: string | undefined;
  /** Change from the previous period (numeric values only). */
  delta?: StatTileDelta | undefined;
  /** A short series for the mini trend line; nulls are gaps. */
  trend?: readonly (number | null)[] | undefined;
}

const TW = 100;
const TH = 24;

function TileTrend(props: { values: readonly (number | null)[] }): JSX.Element {
  const geom = createMemo(() => {
    const vs = props.values;
    const known = vs.flatMap((v) => (v === null ? [] : [v]));
    const max = Math.max(0, ...known);
    const step = vs.length > 1 ? TW / (vs.length - 1) : 0;
    const segments: string[] = [];
    let seg: string[] = [];
    let end: { x: number; y: number } | null = null;
    vs.forEach((v, i) => {
      if (v === null) { if (seg.length) segments.push(seg.join(' ')); seg = []; return; }
      const x = vs.length > 1 ? i * step : TW / 2;
      const y = max > 0 ? TH - (v / max) * (TH - 2) - 1 : TH - 1;
      seg.push(`${x},${y}`);
      end = { x, y };
    });
    if (seg.length) segments.push(seg.join(' '));
    return { segments, end: end as { x: number; y: number } | null };
  });
  return (
    <svg class={stats.trend} viewBox={`0 0 ${TW} ${TH}`} preserveAspectRatio="none" aria-hidden="true" data-trend>
      {geom().segments.map((points) => <polyline class={stats.trendLine} points={points} />)}
      <Show when={geom().end}>{(p) => <rect class={stats.trendEnd} x={p().x - 1.5} y={p().y - 1.5} width="3" height="3" />}</Show>
    </svg>
  );
}

export function StatTile(props: StatTileProps): JSX.Element {
  const id = createUniqueId();
  const shown = (): string | null => {
    const v = props.value;
    if (v === null) return null;
    if (typeof v === 'string') return v;
    return props.format ? props.format(v) : formatNumber(v);
  };
  // A comparison may arrive after mount with the fetched statistics.
  void loadCatalog('admin-stats');
  const change = () => (typeof props.value === 'number' && props.delta ? deltaOf(props.value, props.delta.previous) : null);
  const changeText = (): string => {
    const d = change();
    if (!d) return '';
    if (d.ratio !== null) return formatPercent(Math.abs(d.ratio), { maximumFractionDigits: Math.abs(d.ratio) < 0.1 ? 1 : 0 });
    const abs = Math.abs(d.absolute);
    return props.format ? props.format(abs) : formatNumber(abs);
  };
  return (
    <figure class={css.tile} data-testid={props.testId} aria-labelledby={`${id}-label`}>
      <figcaption id={`${id}-label`} class={css.tileLabel}>
        {props.label}
      </figcaption>
      <Show
        when={shown() !== null}
        fallback={
          <>
            <p class={css.tileNa}>{t('admin-shell-not-available')}</p>
            <Show when={props.unavailableReason}>
              {(reason) => <p class={css.tileDetail}>{reason()}</p>}
            </Show>
          </>
        }
      >
        <p class={css.tileValue}>{shown()}</p>
        <Show when={change()}>
          {(d) => (
            <p class={`${stats.delta} ${stats.deltaTone[deltaTone(d(), props.delta?.upIsGood)]}`} data-delta={d().direction}>
              <span aria-hidden="true">{d().direction === 'up' ? '↑' : d().direction === 'down' ? '↓' : '→'}</span>
              <span class={css.srOnly}>{t(`admin-stats-delta-sr-${d().direction}`, { change: changeText() })}</span>
              <span aria-hidden="true">{d().direction === 'same' ? t('admin-stats-delta-same') : changeText()}</span>
              <Show when={d().direction !== 'same'}><span class={stats.deltaNote} aria-hidden="true">{t('admin-stats-delta-vs')}</span></Show>
            </p>
          )}
        </Show>
        <Show when={props.detail}>{(detail) => <p class={css.tileDetail}>{detail()}</p>}</Show>
        <Show when={props.trend && props.trend.some((v) => v !== null)}><TileTrend values={props.trend!} /></Show>
      </Show>
      <Show when={props.href}>
        {(href) => (
          <a href={href()} onClick={(e) => props.onLink?.(e)}>
            {props.linkLabel ?? props.label}
          </a>
        )}
      </Show>
    </figure>
  );
}
