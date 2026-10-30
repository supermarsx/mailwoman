// Request history (t61, t90; layout t106).
//
// One controller (`createRequestHistory`) loads the selected window, the window
// of the same length just before it (for "vs previous period") and the top
// not-found paths. The pieces render it separately so a page can lay figures,
// charts and tables out in their own sections: `PeriodPicker`, `HistoryNotes`,
// `TrafficTiles`, `TrafficCharts`, `TrafficTables`. `RequestHistory` is the
// Dashboard's composition of them in one panel.
//
// While a new window loads, the last render stays (dimmed) and the controls stay
// put; skeletons the size of the final cards stand in only for the first load.

import {createResource,createSignal,createEffect,on,onMount,For,Show,onCleanup,type JSX} from 'solid-js';
import {loadRequestHistory,loadNotFoundPaths,type RequestHistory as History,type NotFoundPaths,type RequestHistoryRange} from '../../api/admin/requestHistory.ts';
import {t,loadCatalog,formatNumber} from '../../i18n';
import {AdminProblemError} from './shell/adminApi.ts';
import {StatTile} from './shell/StatTile.tsx';
import {ChartCard,ChartSkeleton,TableCard,TableSkeleton,TileSkeleton,type ChartTable} from './shell/ChartCard.tsx';
import {LineChart,type LineSeries} from './shell/LineChart.tsx';
import {NotFoundPathsTable} from './NotFoundPaths.tsx';
import {formatDateTime,formatTime} from '../../i18n/format.ts';
import {Icon} from '../../components/Icon.tsx';
import {resourceValue} from '../../components/resourceValue.ts';
import {formatCount,formatDuration,formatMs,formatMsSet} from './Dashboard/format.ts';
import {comparable,maxKnown,previousWindow,sumKnown} from './shell/statsMath.ts';
import {HISTORY_PRESETS,relativeHistoryRange,historyLocalInput,customHistoryRange} from './RequestHistoryRange.ts';
import * as css from './RequestHistory.css.ts';
import * as charts from './Dashboard/dashboard.css.ts';
import * as stats from './shell/stats.css.ts';
import * as admin from './admin.css.ts';

/** 404s over the points where they were tracked; `null` when no point was (never zero). */
export function notFoundSummary(value:History):{total:number|null;requests:number} {
 let total:number|null=null;let requests=0;
 for(const point of value.series){if(typeof point.notFound!=='number')continue;total=(total??0)+point.notFound;requests+=point.requests??0;}
 return {total,requests};
}
/** Tracking was introduced independently of request history. */
function comparableNotFound(value:History):boolean {
 const coverage=value.coverage.notFound;
 return !!coverage&&coverage.notTracked.length===0&&comparable({observedSeconds:coverage.observedSeconds,requestedSeconds:value.coverage.requestedSeconds});
}
function latencyPeak(value:History):{value:number|null;resolution:number} {
 const peak=maxKnown(value.series.map(p=>p.p95Ms));
 return {value:peak,resolution:Math.max(0,...value.series.filter(p=>p.p95Ms===peak).map(p=>p.p95ResolutionMs??0))};
}
type PathsResult={value:NotFoundPaths}|{denied:boolean};
type Loaded={data:History;previous?:History|undefined;paths:PathsResult}|{failed:true};

export interface HistoryController {
 selection:()=>string;
 select:(value:string)=>void;
 from:()=>string;setFrom:(v:string)=>void;
 to:()=>string;setTo:(v:string)=>void;
 validation:()=>string;
 reload:()=>void;
 loading:()=>boolean;
 failed:()=>boolean;
 data:()=>History|undefined;
 /** The window before, only when the server observed most of both windows. */
 previous:()=>History|undefined;
 paths:()=>NotFoundPaths|undefined;
 pathsFailed:()=>boolean;
 maxRange:()=>number;
}

export function createRequestHistory(options:{refreshToken?:()=>unknown}={}):HistoryController {
 onMount(()=>{void loadCatalog('admin-requests');void loadCatalog('admin-stats');});
 const [selection,setSelection]=createSignal('default');
 const [request,setRequest]=createSignal<{range?:RequestHistoryRange}>({});
 const [from,setFrom]=createSignal(historyLocalInput(Date.now()/1000-3600));
 const [to,setTo]=createSignal(historyLocalInput(Date.now()/1000));
 const [validation,setValidation]=createSignal('');
 let controller:AbortController|undefined;
 onCleanup(()=>controller?.abort());
 const [result]=createResource(request,async (query):Promise<Loaded>=>{
  controller?.abort();controller=new AbortController();
  const signal=controller.signal;
  // Top paths need Observability read; a Dashboard-only reader still gets the counts.
  const paths:Promise<PathsResult>=loadNotFoundPaths(query.range,signal).then(value=>({value}),(error:unknown)=>({denied:error instanceof AdminProblemError&&error.status===403}));
  const before=(range:RequestHistoryRange)=>loadRequestHistory(previousWindow(range),signal).then(v=>comparable(v.coverage)?v:undefined,()=>undefined);
  // The window itself is always requested first, then the one before it.
  const main=loadRequestHistory(query.range,signal);
  const early=query.range?before(query.range):undefined;
  try {
   const data=await main;
   const previous=await (early??before({from:data.from,to:data.to}));
   return {data,previous:comparable(data.coverage)?previous:undefined,paths:await paths};
  }
  catch { void paths; void early; return {failed:true as const}; }
 });
 const latest=()=>resourceValue(result);
 const ok=()=>{const v=latest();return v&&'data' in v?v:undefined;};
 const maxRange=()=>ok()?.data.policy.maxRangeSeconds??2592000;
 function reload():void {
  setValidation('');
  if(selection()==='custom'){
   const parsed=customHistoryRange(from(),to(),maxRange());
   if(parsed.error){setValidation(t(`admin-history-error-${parsed.error}`));return;}
   setRequest({range:parsed.range});
  }else if(selection()==='default')setRequest({});
  else setRequest({range:relativeHistoryRange(Number(selection()))});
 }
 if(options.refreshToken)createEffect(on(options.refreshToken,()=>reload(),{defer:true}));
 return {
  selection,
  select:(value)=>{setSelection(value);setValidation('');if(value!=='custom')reload();},
  from,setFrom,to,setTo,validation,reload,
  loading:()=>result.loading,
  failed:()=>{const v=latest();return !!v&&'failed' in v;},
  data:()=>ok()?.data,
  previous:()=>ok()?.previous,
  paths:()=>{const v=ok()?.paths;return v&&'value' in v?v.value:undefined;},
  pathsFailed:()=>{const v=ok()?.paths;return v!==undefined&&'denied' in v&&!v.denied;},
  maxRange,
 };
}

/** The one period control: presets, a custom range, and reload. */
export function PeriodPicker(props:{history:HistoryController;refreshLabel?:string;onRefresh?:()=>void}):JSX.Element {
 const h=props.history;
 const label=()=>props.refreshLabel??t('admin-history-refresh');
 return <form class={css.controls} onSubmit={e=>{e.preventDefault();h.reload();props.onRefresh?.();}} data-testid="period-picker">
  <label class={css.field}><span>{t('admin-history-period')}</span><select class={css.input} aria-label={t('admin-history-period')} value={h.selection()} onChange={e=>h.select(e.currentTarget.value)}>
   <option value="default">{t('admin-history-default')}</option>
   <For each={HISTORY_PRESETS.filter(seconds=>seconds<=h.maxRange())}>{seconds=><option value={seconds}>{t(`admin-history-period-${seconds}`)}</option>}</For>
   <option value="custom">{t('admin-history-custom')}</option>
  </select></label>
  <Show when={h.selection()==='custom'}>
   <label class={css.field}><span>{t('admin-history-from')}</span><input class={css.input} type="datetime-local" required value={h.from()} onInput={e=>h.setFrom(e.currentTarget.value)}/></label>
   <label class={css.field}><span>{t('admin-history-to')}</span><input class={css.input} type="datetime-local" required value={h.to()} onInput={e=>h.setTo(e.currentTarget.value)}/></label>
  </Show>
  <button class="btn btn--ghost" type="submit" aria-label={label()} title={label()} disabled={h.loading()}><Icon name="refresh"/></button>
 </form>;
}

const timestamp=(seconds:number)=>formatDateTime(seconds*1000);

/** Range, coverage and precision notes, and load errors, for the selected window. */
export function HistoryNotes(props:{history:HistoryController;help?:boolean}):JSX.Element {
 const h=props.history;
 return <div class={stats.notes} data-history-notes>
  <Show when={h.validation()}><p role="alert" class={admin.error}>{h.validation()}</p></Show>
  <Show when={props.help!==false}><p class={admin.note}>{t('admin-history-help')}</p></Show>
  <Show when={h.failed()}><p role="alert" class={admin.error}>{t('admin-history-load-error')}</p></Show>
  <Show when={h.data()}>{value=><>
   <p class={admin.note}>{t('admin-history-range',{from:timestamp(value().from),to:timestamp(value().to),seconds:value().bucketSeconds})}</p>
   <Show when={value().coverage.observedSeconds<value().coverage.requestedSeconds}><p role="status">{t('admin-history-partial',{observed:Math.floor(value().coverage.observedSeconds/60),requested:Math.ceil(value().coverage.requestedSeconds/60)})}</p></Show>
   <p class={admin.note}>{t('admin-history-latency-resolution')}</p>
   <Show when={value().series.some(point=>point.legacyLatencyEstimate!==false && point.requests!==null && point.requests>0)}><p role="status" class={admin.note}>{t('admin-history-latency-legacy')}</p></Show>
   <Show when={(value().coverage.notFound?.notTracked.length??0)>0}><p role="status" class={admin.note} data-testid="not-found-not-tracked">{value().coverage.notFound?.trackedSince==null?t('admin-history-nf-not-tracked-window'):t('admin-history-nf-not-tracked-before',{time:timestamp(value().coverage.notFound!.trackedSince!)})}</p></Show>
  </>}</Show>
 </div>;
}

const share=(part:number|null,whole:number|null)=>part!==null&&whole!==null&&whole>0?t('admin-history-nf-share',{share:formatNumber(part/whole,{style:'percent',maximumFractionDigits:1})}):undefined;

/** The period's headline figures: totals, each with its change and a small trend. */
export function TrafficTiles(props:{history:HistoryController}):JSX.Element {
 const h=props.history;
 return <Show when={h.data()} fallback={<Show when={h.loading()}><For each={[1,2,3,4]}>{()=><TileSkeleton/>}</For></Show>}>{value=>{
  const prev=()=>h.previous();
  const requests=()=>sumKnown(value().series.map(p=>p.requests));
  const errors=()=>sumKnown(value().series.map(p=>p.errors5xx));
  const nf=()=>notFoundSummary(value());
  const p95=()=>latencyPeak(value());
  const before=<T,>(f:(v:History)=>T)=>{const p=prev();return p?f(p):null;};
  const previousPeak=()=>before(v=>{
   const current=p95();const prior=latencyPeak(v);
   if(current.value===null||prior.value===null)return null;
   // A value below its bucket resolution is a bound, not an exact measurement.
   // Nor can a smaller-than-resolution change support a precise percentage.
   const resolution=Math.max(current.resolution,prior.resolution);
   if(current.value<current.resolution||prior.value<prior.resolution||Math.abs(current.value-prior.value)<resolution)return null;
   return prior.value;
  });
  return <>
   <StatTile testId="tile-requests" label={t('admin-stats-requests-total')} value={requests()} format={formatCount}
    delta={{previous:before(v=>sumKnown(v.series.map(p=>p.requests)))}} trend={value().series.map(p=>p.requests)}/>
   <StatTile testId="tile-errors" label={t('admin-stats-errors-total')} value={errors()} format={formatCount} detail={share(errors(),requests())}
    delta={{previous:before(v=>sumKnown(v.series.map(p=>p.errors5xx))),upIsGood:false}} trend={value().series.map(p=>p.errors5xx)}/>
   <StatTile testId="tile-not-found" label={t('admin-stats-not-found')} value={nf().total} format={formatCount} unavailableReason={t('admin-history-nf-not-tracked-window')}
    detail={nf().total!==null?share(nf().total,nf().requests):undefined}
    delta={{previous:comparableNotFound(value())?before(v=>comparableNotFound(v)?notFoundSummary(v).total:null):null,upIsGood:false}} trend={value().series.map(p=>typeof p.notFound==='number'?p.notFound:null)}/>
   <StatTile testId="tile-p95" label={t('admin-stats-p95-peak')} value={p95().value} format={n=>formatMs(n,p95().resolution)}
    delta={{previous:previousPeak(),upIsGood:false}} trend={value().series.map(p=>p.p95Ms)}/>
  </>;
 }}</Show>;
}

/** Axis time labels: the time of day for windows up to a day, the date and time beyond. */
function axisTime(value:History):(s:number)=>string {
 return value.to-value.from>86400?(s=>formatDateTime(s*1000,{dateStyle:'short',timeStyle:'short'})):(s=>formatTime(s*1000));
}
const absent=()=>t('admin-shell-not-available');

function CountCard(props:{value:History;title:string;testId:string;values:(number|null)[]}):JSX.Element {
 const value=()=>props.value;
 const times=()=>value().series.map(p=>p.t);
 const table=():ChartTable=>({columns:[t('admin-shell-chart-col-time'),props.title],numeric:[false,true],rows:value().series.map((p,i)=>[timestamp(p.t),props.values[i]==null?absent():formatCount(props.values[i]!)])});
 const latest=()=>props.values[props.values.length-1]??null;
 const max=()=>maxKnown(props.values);
 const series=():LineSeries[]=>[{label:props.title,slot:1,values:props.values}];
 return <ChartCard testId={props.testId} title={props.title} unit={t('admin-stats-unit-count',{bucket:formatDuration(value().bucketSeconds)})}
  summary={<>{t('admin-stats-chart-about',{from:timestamp(value().from),to:timestamp(value().to)})} {t('admin-shell-chart-summary',{latest:latest()===null?absent():formatCount(latest()!),max:max()===null?absent():formatCount(max()!)})}</>}
  table={table()} empty={max()===null?t('admin-stats-empty-chart'):undefined}>
  <LineChart times={times()} series={series()} format={formatCount} formatTime={axisTime(value())} whole/>
 </ChartCard>;
}

function LatencyCard(props:{value:History}):JSX.Element {
 const value=()=>props.value;
 const p50=()=>value().series.map(p=>p.p50Ms);const p95=()=>value().series.map(p=>p.p95Ms);
 const all=()=>formatMsSet([...p50(),...p95()]);
 const f50=()=>formatMsSet(p50());const f95=()=>formatMsSet(p95());
 const cell=(f:(n:number,r?:number|null)=>string,v:number|null,r?:number|null)=>v===null?absent():f(v,r);
 const summary=(f:(n:number,r?:number|null)=>string,vs:(number|null)[],r:(i:number)=>number|null|undefined)=>{
  const max=maxKnown(vs);const at=max===null?-1:vs.indexOf(max);const last=vs.length-1;
  return t('admin-shell-chart-summary',{latest:cell(f,vs[last]??null,r(last)),max:max===null?absent():f(max,r(at))});
 };
 const labels=()=>({p50:t('admin-stats-series-p50'),p95:t('admin-stats-series-p95')});
 const table=():ChartTable=>({columns:[t('admin-shell-chart-col-time'),labels().p50,labels().p95],numeric:[false,true,true],
  rows:value().series.map(p=>[timestamp(p.t),cell(f50(),p.p50Ms,p.p50ResolutionMs),cell(f95(),p.p95Ms,p.p95ResolutionMs)])});
 return <ChartCard testId="chart-latency" title={t('admin-stats-latency')} unit={t('admin-stats-unit-duration',{bucket:formatDuration(value().bucketSeconds)})}
  legend={[{label:labels().p50,slot:1},{label:labels().p95,slot:2}]}
  summary={<>{t('admin-stats-chart-about',{from:timestamp(value().from),to:timestamp(value().to)})} <span data-testid="spark-p50">{labels().p50}: {summary(f50(),p50(),i=>value().series[i]?.p50ResolutionMs)} </span><span data-testid="spark-p95">{labels().p95}: {summary(f95(),p95(),i=>value().series[i]?.p95ResolutionMs)} </span></>}
  table={table()} empty={maxKnown([...p50(),...p95()])===null?t('admin-stats-empty-chart'):undefined}>
  <LineChart times={value().series.map(p=>p.t)} series={[{label:labels().p50,slot:1,values:p50()},{label:labels().p95,slot:2,values:p95()}]} format={n=>all()(n)} formatTime={axisTime(value())}/>
 </ChartCard>;
}

/** The period's charts, one card each. */
export function TrafficCharts(props:{history:HistoryController}):JSX.Element {
 const h=props.history;
 return <Show when={h.data()} fallback={<Show when={h.loading()}><For each={[1,2,3,4]}>{()=><ChartSkeleton/>}</For></Show>}>{value=><>
  <CountCard value={value()} title={t('admin-stats-requests')} testId="spark-requests" values={value().series.map(p=>p.requests)}/>
  <CountCard value={value()} title={t('admin-stats-errors')} testId="spark-errors" values={value().series.map(p=>p.errors5xx)}/>
  <CountCard value={value()} title={t('admin-stats-not-found')} testId="spark-not-found" values={value().series.map(p=>typeof p.notFound==='number'?p.notFound:null)}/>
  <LatencyCard value={value()}/>
 </>}</Show>;
}

/** The period's tables: the most frequent not-found paths. */
export function TrafficTables(props:{history:HistoryController}):JSX.Element {
 const h=props.history;
 return <>
  <Show when={h.paths()} fallback={<Show when={h.loading()&&!h.data()}><TableSkeleton/></Show>}>{list=><TableCard><NotFoundPathsTable value={list()} formatTime={timestamp}/></TableCard>}</Show>
  <Show when={h.pathsFailed()}><TableCard><p role="status" class={admin.note}>{t('admin-history-nf-paths-error')}</p></TableCard></Show>
 </>;
}

/** The Dashboard's traffic panel: period, then figures, then charts, then tables. */
export function RequestHistory(props:{refreshToken?:unknown}={}):JSX.Element {
 const history=createRequestHistory({refreshToken:()=>props.refreshToken});
 const stale=()=>history.loading()&&history.data()!==undefined;
 return <section class={charts.panel} aria-label={t('admin-history-title')} aria-busy={history.loading()} data-testid="request-history">
  <h3 class={charts.panelTitle}>{t('admin-history-title')}</h3>
  <PeriodPicker history={history}/>
  <HistoryNotes history={history}/>
  <div class={`${stats.view} ${stale()?stats.stale:''}`}>
   <div class={stats.tiles} data-stats-section="tiles"><TrafficTiles history={history}/></div>
   <div class={stats.charts} data-stats-section="charts"><TrafficCharts history={history}/></div>
   <div class={stats.tables} data-stats-section="tables"><TrafficTables history={history}/></div>
  </div>
 </section>;
}
