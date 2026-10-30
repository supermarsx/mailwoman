// One chart in its own card (t106): title, unit, legend, the chart, a text
// summary, and a "View as table" switch that shows the same data as a table.
//
// The card is a region named by its title and described by its summary, so a
// screen reader hears what the chart is and what it shows without the picture.
// The table is always in the document: visually hidden until the switch shows it,
// so assistive technology can read every value either way.

import { For, Show, createSignal, createUniqueId, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import * as shell from './shell.css.ts';
import * as css from './stats.css.ts';

export type SeriesSlot = 1 | 2;
export interface LegendEntry { label: string; slot: SeriesSlot }
export interface ChartTable {
  columns: readonly string[];
  /** Which columns hold numbers (aligned to the end, tabular figures). */
  numeric?: readonly boolean[];
  rows: readonly (readonly string[])[];
}

export interface ChartCardProps {
  title: string;
  /** What the values measure, e.g. "Count per 1 min". */
  unit?: string | undefined;
  /** Shown for two or more series; one series is named by the title. */
  legend?: readonly LegendEntry[] | undefined;
  /** One or two sentences: what is plotted and its latest and highest values. */
  summary: JSX.Element;
  table: ChartTable;
  /** Shown instead of the chart and table when there is nothing to plot. */
  empty?: string | undefined;
  testId?: string | undefined;
  children: JSX.Element;
}

export function ChartCard(props: ChartCardProps): JSX.Element {
  const id = createUniqueId();
  const [asTable, setAsTable] = createSignal(false);
  onMount(() => void loadCatalog('admin-stats'));
  return (
    <section class={css.card} aria-labelledby={`${id}-title`} aria-describedby={`${id}-summary`} data-testid={props.testId} data-chart-card>
      <div class={css.cardHead}>
        <div class={css.cardHeading}>
          <h4 id={`${id}-title`} class={css.cardTitle}>{props.title}</h4>
          <Show when={props.unit}>{(unit) => <p class={css.cardUnit}>{unit()}</p>}</Show>
        </div>
        <Show when={!props.empty}>
          <button type="button" class={`btn btn--ghost ${css.toggle}`} aria-pressed={asTable()} aria-controls={`${id}-table`} onClick={() => setAsTable(!asTable())}>
            {t('admin-stats-view-table')}
          </button>
        </Show>
      </div>
      <Show when={(props.legend?.length ?? 0) > 1}>
        <ul class={css.legend} aria-label={t('admin-stats-legend')}>
          <For each={props.legend}>{(entry) => <li class={css.legendItem}><span class={`${css.key} ${css.keyColour[entry.slot]}`} aria-hidden="true" />{entry.label}</li>}</For>
        </ul>
      </Show>
      <Show when={!props.empty} fallback={<p class={css.cardEmpty} role="status" id={`${id}-summary`}>{props.empty}</p>}>
        <Show when={!asTable()}>{props.children}</Show>
        <p id={`${id}-summary`} class={css.cardSummary}>{props.summary}</p>
        <div id={`${id}-table`} class={asTable() ? css.tableScroll : shell.srOnly} data-chart-table={asTable() ? 'shown' : 'hidden'}>
          <table class={css.dataTable}>
            <caption class={shell.srOnly}>{props.title}</caption>
            <thead><tr><For each={props.table.columns}>{(c, i) => <th scope="col" class={props.table.numeric?.[i()] ? css.num : undefined}>{c}</th>}</For></tr></thead>
            <tbody>
              <For each={props.table.rows}>{(row) => <tr><For each={row}>{(cell, i) => <td class={props.table.numeric?.[i()] ? css.num : undefined}>{cell}</td>}</For></tr>}</For>
            </tbody>
          </table>
        </div>
      </Show>
    </section>
  );
}

/** A card for a table or a top-N list, titled like the chart cards. */
export function TableCard(props: { title?: string; testId?: string; children: JSX.Element }): JSX.Element {
  const id = createUniqueId();
  return (
    <section class={css.card} aria-labelledby={props.title ? `${id}-title` : undefined} data-testid={props.testId} data-table-card>
      <Show when={props.title}>{(title) => <h4 id={`${id}-title`} class={css.cardTitle}>{title()}</h4>}</Show>
      {props.children}
    </section>
  );
}

/** Skeleton of a stat tile, the size of a tile with a value and a delta line. */
export function TileSkeleton(): JSX.Element {
  return (
    <div class={`${shell.tile} ${css.skeletonTile}`} aria-hidden="true" data-skeleton="tile">
      <span class={css.skeletonLine} />
      <span class={css.skeletonValue} />
      <span class={css.skeletonLine} />
    </div>
  );
}

/** Skeleton of a chart card: heading, plot area and summary at their final sizes. */
export function ChartSkeleton(): JSX.Element {
  return (
    <div class={css.card} aria-hidden="true" data-skeleton="chart">
      <span class={css.skeletonLine} />
      <span class={css.skeletonPlot} />
      <span class={css.skeletonLine} />
    </div>
  );
}

/** Skeleton of a table card: a heading and a few rows. */
export function TableSkeleton(props: { rows?: number }): JSX.Element {
  return (
    <div class={css.card} aria-hidden="true" data-skeleton="table">
      <span class={css.skeletonLine} />
      <For each={Array.from({ length: props.rows ?? 4 })}>{() => <span class={css.skeletonRow} />}</For>
    </div>
  );
}

/** A section of one kind with its heading: figures, charts or tables. */
export function StatsSection(props: { title: string; kind: 'tiles' | 'charts' | 'tables'; testId?: string; busy?: boolean; children: JSX.Element }): JSX.Element {
  const id = createUniqueId();
  return (
    <section class={css.section} aria-labelledby={`${id}-title`} aria-busy={props.busy ? 'true' : undefined} data-testid={props.testId} data-stats-section={props.kind}>
      <h3 id={`${id}-title`} class={css.sectionTitle}>{props.title}</h3>
      {props.children}
    </section>
  );
}
