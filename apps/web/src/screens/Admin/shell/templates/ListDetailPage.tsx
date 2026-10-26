// List + detail page template (t32-cfg UX3): users, domains, bans, sessions, jobs.
//
// The list is the shell's `DataTable` (search, filter chips, cursor paging, the
// loading / error / empty states). Each row has an Open link to its detail at
// `#/<section>/<id>`, so a detail is linkable and Back works.
// Details and registration forms occupy an exclusive page at every viewport.
// Back restores the list; opening a detail focuses its heading.

import { Show, createEffect, on, useContext, type JSX } from 'solid-js';
import { t } from '../../../../i18n';
import { DataTable, type DataTableProps } from '../DataTable.tsx';
import { ShellContext } from '../context.ts';
import { Icon } from '../icons.tsx';
import { hrefFor, sectionPath, type AdminRoute } from '../router.ts';
import { AdminPage } from './PageFrame.tsx';
import * as css from '../shell.css.ts';
import * as ux from '../ux.css.ts';

export interface ListDetailPageProps<R> {
  title: string;
  description?: string;
  actions?: JSX.Element;
  /** The section id: details live at `#/<section>/<id>`. */
  section: string;
  /** The current route; `rest[0]` is the open item. */
  route: AdminRoute;
  /** The list: everything `DataTable` takes. The Open link is added to the row actions. */
  table: DataTableProps<R>;
  /** The open item's heading. */
  detailTitle: (id: string) => string;
  /** The open item's content. */
  renderDetail: (id: string) => JSX.Element;
  /** The Open link's text for a row (default "Open"); its accessible name adds the row key. */
  openLabel?: (row: R) => string;
  compactActions?: boolean;
  /** Navigate in place. Default: the shell router. */
  navigate?: (path: string) => void;
  testId?: string;
}

export function ListDetailPage<R>(props: ListDetailPageProps<R>): JSX.Element {
  const shell = useContext(ShellContext);
  let detailHeading: HTMLHeadingElement | undefined;
  const selected = (): string | undefined => props.route.rest[0];
  const go = (path: string): void => {
    if (props.navigate) props.navigate(path);
    else void shell?.router.navigate(path);
  };
  const inPlace = (e: MouseEvent, path: string): void => {
    if (e.defaultPrevented || e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    go(path);
  };

  createEffect(
    on(selected, (id, prev) => {
      if (id !== undefined && id !== prev) queueMicrotask(() => {
        detailHeading?.closest<HTMLElement>('[data-testid="list-detail"]')?.scrollTo?.({ top: 0 });
        detailHeading?.focus({ preventScroll: window.matchMedia?.('(min-width: 1200px)').matches ?? false });
      });
    }),
  );

  const rowActions = (row: R): JSX.Element => {
    const key = props.table.rowKey(row);
    const path = sectionPath(props.section, [key]);
    return (
      <>
        <a
          class={`btn btn--ghost ${props.compactActions ? ux.iconAction : ''}`}
          title={`${props.openLabel?.(row) ?? t('admin-ux-list-open')} ${key}`}
          href={hrefFor(path)}
          aria-current={selected() === key ? 'true' : undefined}
          onClick={(e) => inPlace(e, path)}
        >
          <Show when={props.compactActions} fallback={props.openLabel?.(row) ?? t('admin-ux-list-open')}>
            <Icon name="edit" /><span class={css.srOnly}>{props.openLabel?.(row) ?? t('admin-ux-list-open')}</span>
          </Show>
          <span class={css.srOnly}> {key}</span>
        </a>
        {props.table.rowActions?.(row)}
      </>
    );
  };

  return (
    <AdminPage
      title={props.title}
      titleId={`${props.section}-page-title`}
      {...(props.description ? { description: props.description } : {})}
      actions={selected() ? undefined : props.actions}
      testId={props.testId}
    >
      <div class={ux.listDetail} data-detail-open={selected() !== undefined ? '' : undefined}>
        <Show when={!selected()}><div class={ux.listPane}>
          <DataTable {...props.table} rowActions={rowActions} />
        </div></Show>
        <Show when={selected()}>
          {(id) => (
            <section class={ux.detailPane} aria-labelledby={`${props.section}-detail-title`} data-testid="list-detail">
              <a class={ux.backLink} href={hrefFor(sectionPath(props.section))} onClick={(e) => inPlace(e, sectionPath(props.section))}>
                <Icon name="chevron" class={ux.backIcon} />
                {t('admin-ux-list-back')}
              </a>
              <div class={ux.detailHead}>
                <h3 id={`${props.section}-detail-title`} ref={detailHeading} tabindex="-1" class={ux.detailTitle}>
                  {props.detailTitle(id())}
                </h3>
                <a class={`btn btn--ghost ${ux.iconAction} ${ux.detailClose}`} href={hrefFor(sectionPath(props.section))}
                  aria-label={t('common-close')} title={t('common-close')} onClick={(e) => inPlace(e, sectionPath(props.section))}>
                  <Icon name="close" />
                </a>
              </div>
              {props.renderDetail(id())}
            </section>
          )}
        </Show>
      </div>
    </AdminPage>
  );
}
