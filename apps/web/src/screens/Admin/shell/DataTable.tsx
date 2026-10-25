import { Icon } from '../../../components/Icon.tsx';
// Admin list table (t32-admin.md §2.1).
//
// Server-driven: search text, filter chips, sort and paging are reported to the
// caller, which fetches the matching page (cursor paging: the caller knows
// whether there is a previous or next page). The table renders the loading,
// error and empty states itself; the empty state names the next action
// ("No domains yet — Add a domain").
//
// Keyboard: sortable headers are buttons (`aria-sort` on the header cell); rows
// form one Tab stop and Up/Down/Home/End move between them, while the controls
// inside a row stay in the normal Tab order.

import { For, Show, createSignal, createUniqueId, onCleanup, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { Feedback } from './Feedback.tsx';
import * as css from './shell.css.ts';
import * as admin from '../admin.css.ts';

export interface Column<R> {
  id: string;
  header: string;
  cell: (row: R) => JSX.Element;
  sortable?: boolean;
}

export interface FilterChip {
  id: string;
  label: string;
  active: boolean;
}

export interface SortState {
  column: string;
  direction: 'asc' | 'desc';
}

export interface DataTableProps<R> {
  /** The table's accessible name (a visually hidden caption). */
  caption: string;
  columns: readonly Column<R>[];
  rows: readonly R[];
  rowKey: (row: R) => string;
  state: 'loading' | 'ready' | 'error';
  errorText?: string;
  onRetry?: () => void;
  emptyText: string;
  emptyAction?: { label: string; onClick: () => void };
  search?: { label: string; value: string; onSearch: (q: string) => void; debounceMs?: number };
  filters?: readonly FilterChip[];
  onToggleFilter?: (id: string) => void;
  sort?: SortState | null;
  onSort?: (column: string) => void;
  rowActions?: (row: R) => JSX.Element;
  rowActionsHeader?: string;
  paging?: { hasPrev: boolean; hasNext: boolean; onPrev: () => void; onNext: () => void };
}

export function DataTable<R>(props: DataTableProps<R>): JSX.Element {
  const id = createUniqueId();
  const [activeRow, setActiveRow] = createSignal(0);
  const [query, setQuery] = createSignal(props.search?.value ?? '');
  let timer: ReturnType<typeof setTimeout> | undefined;
  let tbody: HTMLTableSectionElement | undefined;
  onCleanup(() => clearTimeout(timer));

  function onSearchInput(q: string): void {
    setQuery(q);
    clearTimeout(timer);
    timer = setTimeout(() => props.search?.onSearch(q), props.search?.debounceMs ?? 300);
  }

  function onRowKey(e: KeyboardEvent, i: number): void {
    // Only when the row itself has focus; arrow keys inside a field are the field's.
    if (e.target !== e.currentTarget) return;
    const last = props.rows.length - 1;
    let next = i;
    if (e.key === 'ArrowDown') next = Math.min(last, i + 1);
    else if (e.key === 'ArrowUp') next = Math.max(0, i - 1);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = last;
    else return;
    e.preventDefault();
    setActiveRow(next);
    tbody?.querySelectorAll<HTMLElement>('tr')[next]?.focus();
  }

  const ariaSort = (col: Column<R>): 'ascending' | 'descending' | 'none' | undefined => {
    if (!col.sortable) return undefined;
    if (props.sort?.column !== col.id) return 'none';
    return props.sort.direction === 'asc' ? 'ascending' : 'descending';
  };

  return (
    <div class={css.field}>
      <Show when={props.search || (props.filters && props.filters.length > 0)}>
        <div class={css.tableToolbar}>
          <Show when={props.search}>
            {(search) => (
              <form
                role="search"
                class="field"
                onSubmit={(e) => {
                  e.preventDefault();
                  clearTimeout(timer);
                  search().onSearch(query());
                }}
              >
                <label for={`${id}-q`}>{search().label}</label>
                <input id={`${id}-q`} type="search" value={query()} onInput={(e) => onSearchInput(e.currentTarget.value)} />
              </form>
            )}
          </Show>
          <Show when={props.filters && props.filters.length > 0}>
            <div class={css.chips} role="group" aria-label={t('admin-shell-table-filters')}>
              <For each={props.filters}>
                {(f) => (
                  <button
                    type="button"
                    class={css.chip}
                    aria-pressed={f.active}
                    onClick={() => props.onToggleFilter?.(f.id)}
                  >
                    {f.label}
                  </button>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={props.state !== 'error'} fallback={
        <div class={[css.state, css.stateError].join(' ')}>
          <Feedback kind="error" message={props.errorText ?? t('admin-shell-table-error')} />
          <Show when={props.onRetry}>
            <button type="button" class="btn btn--ghost" onClick={() => props.onRetry?.()}>
              {t('admin-shell-retry')}
            </button>
          </Show>
        </div>
      }>
        <Show when={props.state === 'ready' || props.rows.length > 0} fallback={
          <div class={css.state}>
            <AsyncPending message={t('admin-shell-loading')} onRetry={props.onRetry} />
          </div>
        }>
          <Show when={props.rows.length > 0} fallback={
            <div class={css.state} data-testid="table-empty">
              <p class={admin.note}>{props.emptyText}</p>
              <Show when={props.emptyAction}>
                {(action) => (
                  <button type="button" class="btn btn--primary" onClick={() => action().onClick()}>
                    {action().label}
                  </button>
                )}
              </Show>
            </div>
          }>
            <div class={css.tableRefresh}>
              <Show when={props.state === 'loading'}>
                <AsyncPending message={t('admin-shell-loading')} onRetry={props.onRetry} />
              </Show>
            </div>
            <div class={admin.tableWrap}>
              <table class={admin.table} aria-busy={props.state === 'loading'}>
                <caption class={css.srOnly}>{props.caption}</caption>
                <thead>
                  <tr>
                    <For each={props.columns}>
                      {(col) => (
                        <th scope="col" aria-sort={ariaSort(col)}>
                          <Show when={col.sortable && props.onSort} fallback={col.header}>
                            <button type="button" class={css.sortButton} onClick={() => props.onSort?.(col.id)}>
                              {col.header}
                              <span aria-hidden="true">
                                <Icon name={ariaSort(col) === 'ascending' ? 'chevronUp' : ariaSort(col) === 'descending' ? 'chevronDown' : 'sort'} />
                              </span>
                            </button>
                          </Show>
                        </th>
                      )}
                    </For>
                    <Show when={props.rowActions}>
                      <th scope="col">{props.rowActionsHeader ?? t('admin-shell-table-actions')}</th>
                    </Show>
                  </tr>
                </thead>
                <tbody ref={tbody}>
                  <For each={props.rows}>
                    {(row, i) => (
                      <tr
                        class={css.rowFocus}
                        tabindex={i() === Math.min(activeRow(), props.rows.length - 1) ? 0 : -1}
                        data-row-key={props.rowKey(row)}
                        onFocus={(e) => {
                          if (e.target === e.currentTarget) setActiveRow(i());
                        }}
                        onKeyDown={(e) => onRowKey(e, i())}
                      >
                        <For each={props.columns}>{(col) => <td>{col.cell(row)}</td>}</For>
                        <Show when={props.rowActions}>
                          <td>
                            <div class={css.rowActions}>{props.rowActions?.(row)}</div>
                          </td>
                        </Show>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </Show>
      </Show>

      <Show when={props.paging}>
        {(paging) => (
          <nav class={css.pager} aria-label={t('admin-shell-table-paging', { table: props.caption })}>
            <button type="button" class="btn" disabled={!paging().hasPrev} onClick={() => paging().onPrev()}>
              {t('admin-shell-table-prev')}
            </button>
            <button type="button" class="btn" disabled={!paging().hasNext} onClick={() => paging().onNext()}>
              {t('admin-shell-table-next')}
            </button>
          </nav>
        )}
      </Show>
    </div>
  );
}
