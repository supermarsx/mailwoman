// Horizontal bars for a few labelled counts (t32-admin.md §2.1).
//
// One measure, one colour; the label and the value are printed next to every bar,
// so nothing depends on reading the bar length. The visible list is hidden from
// assistive technology and a visually hidden table carries the same data.

import { For, Show, createUniqueId, type JSX } from 'solid-js';
import { formatNumber, t } from '../../../i18n';
import * as css from './shell.css.ts';

export interface Bar {
  label: string;
  value: number;
}

export interface BarChartProps {
  title: string;
  bars: readonly Bar[];
  format?: (n: number) => string;
  emptyText?: string;
  testId?: string;
}

export function BarChart(props: BarChartProps): JSX.Element {
  const id = createUniqueId();
  const fmt = (n: number): string => (props.format ? props.format(n) : formatNumber(n));
  const max = (): number => Math.max(0, ...props.bars.map((b) => b.value));
  /** The bar's share of the largest value, in whole percent (a class per step, no style attribute). */
  const share = (v: number): number => (max() > 0 ? Math.min(100, Math.max(0, Math.round((v / max()) * 100))) : 0);
  return (
    <section class={css.chart} aria-labelledby={`${id}-title`} data-testid={props.testId}>
      <h3 id={`${id}-title`} class={css.chartTitle}>
        {props.title}
      </h3>
      <Show when={props.bars.length > 0} fallback={<p class={css.chartSummary}>{props.emptyText}</p>}>
        <ul class={css.bars} aria-hidden="true">
          <For each={props.bars}>
            {(b) => (
              <li class={css.barRow} title={`${b.label}: ${fmt(b.value)}`}>
                <span class={css.barLabel}>{b.label}</span>
                <span class={css.barTrack}>
                  <span class={`${css.barFill} ${css.barWidth[share(b.value)]}`} data-share={share(b.value)} />
                </span>
                <span class={css.barValue}>{fmt(b.value)}</span>
              </li>
            )}
          </For>
        </ul>
        <table class={css.srOnly}>
          <caption>{props.title}</caption>
          <thead>
            <tr>
              <th scope="col">{t('admin-shell-chart-col-item')}</th>
              <th scope="col">{t('admin-shell-chart-col-value')}</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.bars}>
              {(b) => (
                <tr>
                  <td>{b.label}</td>
                  <td>{fmt(b.value)}</td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </Show>
    </section>
  );
}
