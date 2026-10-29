// The most frequent not-found paths of the selected window (t61).
//
// The server has already redacted every path (no query string, address, email,
// id or token) and keeps at most `limit` pairs per hour, so this table shows only
// what it is sent. Counts of an hour that overflowed the limit are bounds, and the
// table says so instead of presenting them as exact.

import {For,Show,type JSX} from 'solid-js';
import type {NotFoundPaths} from '../../api/admin/requestHistory.ts';
import {t} from '../../i18n';
import {formatCount} from './Dashboard/format.ts';
import * as charts from './Dashboard/dashboard.css.ts';
import * as admin from './admin.css.ts';

export function NotFoundPathsTable(props:{value:NotFoundPaths;formatTime:(seconds:number)=>string}):JSX.Element {
 const count=(row:NotFoundPaths['paths'][number])=>row.minCount<row.count
  ?t('admin-history-nf-paths-range',{min:formatCount(row.minCount),max:formatCount(row.count)})
  :formatCount(row.count);
 return <section class={charts.panel} aria-label={t('admin-history-nf-paths-title')} data-testid="not-found-paths">
  <h4 class={charts.panelTitle}>{t('admin-history-nf-paths-title')}</h4>
  <p class={admin.note}>{t('admin-history-nf-paths-help',{limit:props.value.limit,from:props.formatTime(props.value.pathsFrom),to:props.formatTime(props.value.pathsTo)})}</p>
  <Show when={props.value.approximate}><p role="status" class={admin.note}>{t('admin-history-nf-paths-approximate')}</p></Show>
  <Show when={props.value.paths.length>0} fallback={<p role="status">{props.value.total===null?t('admin-history-nf-not-tracked-window'):t('admin-history-nf-paths-empty')}</p>}>
   <div class={admin.tableWrap}><table class={admin.table}>
    <thead><tr><th scope="col">{t('admin-history-nf-paths-method')}</th><th scope="col">{t('admin-history-nf-paths-path')}</th><th scope="col">{t('admin-history-nf-paths-count')}</th></tr></thead>
    <tbody><For each={props.value.paths}>{row=><tr><td>{row.method}</td><td><code dir="ltr">{row.path}</code></td><td>{count(row)}</td></tr>}</For></tbody>
   </table></div>
  </Show>
 </section>;
}
