// Admin > Observability > Statistics > Client load times.
//
// Browser-measured page load figures from signed-in sessions only, sampled and
// aggregated server side into fixed histograms (docs/admin/observability.md).
// They follow the page's period control. Percentiles are withheld below the
// admin-set minimum sample count; every figure is a histogram approximation
// (within one bin, about 22% worst case), which the note says.

import {createResource,For,Show,type JSX} from 'solid-js';
import {loadClientTiming,CLIENT_TIMING_DEVICES,CLIENT_TIMING_METRICS,type ClientTimingHistory,type ClientTimingMetric,type ClientTimingStats} from '../../api/admin/clientTiming.ts';
import {t} from '../../i18n';
import {formatDateTime,formatTime} from '../../i18n/format.ts';
import {resourceValue} from '../../components/resourceValue.ts';
import {StatTile} from './shell/StatTile.tsx';
import {ChartCard,TableCard,TileSkeleton,type ChartTable} from './shell/ChartCard.tsx';
import {LineChart} from './shell/LineChart.tsx';
import {formatCount,formatDuration,formatMs,formatMsSet} from './Dashboard/format.ts';
import {maxKnown} from './shell/statsMath.ts';
import type {HistoryController} from './RequestHistory.tsx';
import * as stats from './shell/stats.css.ts';
import * as admin from './admin.css.ts';

export type ClientTimingController={
 data:()=>ClientTimingHistory|undefined;
 loading:()=>boolean;
 failed:()=>boolean;
};

/** Loads the browser figures for the window the request history resolved. */
export function createClientTiming(history:HistoryController,enabled:()=>boolean):ClientTimingController {
 const source=()=>{const d=history.data();return enabled()&&d?{from:d.from,to:d.to}:undefined;};
 const [result]=createResource(source,range=>loadClientTiming(range).then(data=>({data}),()=>({failed:true as const})));
 const value=()=>{const v=resourceValue(result);return v;};
 return {
  data:()=>{const v=value();return v&&'data' in v?v.data:undefined;},
  loading:()=>result.loading,
  failed:()=>{const v=value();return !!v&&'failed' in v;},
 };
}

const metricLabel=(m:ClientTimingMetric)=>t(`admin-ct-metric-${m}`);
const deviceLabel=(d:string)=>t(`admin-ct-device-${d}`);
const timestamp=(seconds:number)=>formatDateTime(seconds*1000);
const absent=()=>t('admin-shell-not-available');

function reason(stat:ClientTimingStats,min:number):string {
 return stat.count===0?t('admin-ct-no-samples'):t('admin-ct-few-samples',{count:stat.count,min});
}

function Tile(props:{value:ClientTimingHistory;metric:ClientTimingMetric;stat:'p50Ms'|'p95Ms'}):JSX.Element {
 const all=()=>props.value.metrics[props.metric].all;
 return <StatTile testId={`ct-${props.metric}-${props.stat}`}
  label={t('admin-ct-tile',{metric:metricLabel(props.metric),stat:t(props.stat==='p50Ms'?'admin-stats-series-p50':'admin-stats-series-p95')})}
  value={all()[props.stat]} format={n=>formatMs(n)} unavailableReason={reason(all(),props.value.policy.minSamples)}
  detail={t('admin-ct-samples',{count:all().count})}/>;
}

const HEADLINE:readonly ClientTimingMetric[]=['timeToUsable','fcp','lcp','load'];

/** Key figures: median and 95th percentile of each headline metric. */
export function ClientTimingTiles(props:{timing:ClientTimingController}):JSX.Element {
 return <Show when={props.timing.data()} fallback={<Show when={props.timing.loading()}><For each={[1,2,3,4]}>{()=><TileSkeleton/>}</For></Show>}>{value=><>
  <For each={HEADLINE}>{metric=><>
   <Tile value={value()} metric={metric} stat="p50Ms"/>
   <Tile value={value()} metric={metric} stat="p95Ms"/>
  </>}</For>
 </>}</Show>;
}

function axisTime(value:ClientTimingHistory):(s:number)=>string {
 return value.to-value.from>86400?(s=>formatDateTime(s*1000,{dateStyle:'short',timeStyle:'short'})):(s=>formatTime(s*1000));
}

function UsableCard(props:{value:ClientTimingHistory}):JSX.Element {
 const value=()=>props.value;
 const series=()=>value().metrics.timeToUsable.series;
 const p50=()=>series().map(p=>p.p50Ms);const p95=()=>series().map(p=>p.p95Ms);
 const f=()=>formatMsSet([...p50(),...p95()]);
 const labels=()=>({p50:t('admin-stats-series-p50'),p95:t('admin-stats-series-p95')});
 const cell=(v:number|null)=>v===null?absent():f()(v);
 const table=():ChartTable=>({columns:[t('admin-shell-chart-col-time'),labels().p50,labels().p95,t('admin-ct-col-samples')],numeric:[false,true,true,true],
  rows:series().map(p=>[timestamp(p.t),cell(p.p50Ms),cell(p.p95Ms),formatCount(p.count)])});
 const max50=()=>maxKnown(p50());const max95=()=>maxKnown(p95());
 return <ChartCard testId="chart-client-usable" title={t('admin-ct-chart-usable')} unit={t('admin-stats-unit-duration',{bucket:formatDuration(value().bucketSeconds)})}
  legend={[{label:labels().p50,slot:1},{label:labels().p95,slot:2}]}
  summary={<>{t('admin-stats-chart-about',{from:timestamp(value().from),to:timestamp(value().to)})} {labels().p50}: {max50()===null?absent():f()(max50()!)}. {labels().p95}: {max95()===null?absent():f()(max95()!)}. {t('admin-ct-chart-note',{min:value().policy.minSamples})}</>}
  table={table()} empty={max50()===null&&max95()===null?t('admin-ct-empty-chart',{min:value().policy.minSamples}):undefined}>
  <LineChart times={series().map(p=>p.t)} series={[{label:labels().p50,slot:1,values:p50()},{label:labels().p95,slot:2,values:p95()}]} format={n=>f()(n)} formatTime={axisTime(value())}/>
 </ChartCard>;
}

export function ClientTimingCharts(props:{timing:ClientTimingController}):JSX.Element {
 return <Show when={props.timing.data()}>{value=><UsableCard value={value()}/>}</Show>;
}

/** Every metric by device class: samples, median and 95th percentile. */
export function ClientTimingTables(props:{timing:ClientTimingController}):JSX.Element {
 return <Show when={props.timing.data()}>{value=>{
  const rows=()=>CLIENT_TIMING_METRICS.flatMap(metric=>CLIENT_TIMING_DEVICES.flatMap(device=>{
   const s=value().metrics[metric].devices[device];
   return s&&s.count>0?[{metric,device,stat:s}]:[];
  }));
  const ms=(n:number|null)=>n===null?absent():formatMs(n);
  return <TableCard title={t('admin-ct-table-title')} testId="ct-by-device">
   <Show when={rows().length>0} fallback={<p role="status" class={admin.note}>{t('admin-ct-empty-table')}</p>}>
    <div class={admin.tableWrap}><table class={admin.table}>
     <caption class={stats.cardUnit}>{t('admin-ct-table-caption')}</caption>
     <thead><tr><th scope="col">{t('admin-ct-col-metric')}</th><th scope="col">{t('admin-ct-col-device')}</th><th scope="col">{t('admin-ct-col-samples')}</th><th scope="col">{t('admin-stats-series-p50')}</th><th scope="col">{t('admin-stats-series-p95')}</th></tr></thead>
     <tbody><For each={rows()}>{row=><tr>
      <td>{metricLabel(row.metric)}</td><td>{deviceLabel(row.device)}</td><td>{formatCount(row.stat.count)}</td><td>{ms(row.stat.p50Ms)}</td><td>{ms(row.stat.p95Ms)}</td>
     </tr>}</For></tbody>
    </table></div>
   </Show>
  </TableCard>;
 }}</Show>;
}

/** What the figures are, and when they are off or unavailable. */
export function ClientTimingNote(props:{timing:ClientTimingController}):JSX.Element {
 return <>
  <Show when={props.timing.failed()}><p role="status" class={admin.note} data-testid="ct-failed">{t('admin-ct-failed')}</p></Show>
  <Show when={props.timing.data()}>{value=><>
   <Show when={!value().policy.enabled}><p role="status" class={admin.note} data-testid="ct-disabled">{t('admin-ct-disabled')}</p></Show>
   <p class={admin.note} data-testid="ct-note">{t('admin-ct-note',{rate:value().policy.sampleRatePercent,error:Math.round(value().resolution.maxRelativeError*100)})}</p>
  </>}</Show>
 </>;
}
