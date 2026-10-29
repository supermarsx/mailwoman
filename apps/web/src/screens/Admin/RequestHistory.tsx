import {createResource,createSignal,createEffect,on,onMount,For,Show,onCleanup,type JSX} from 'solid-js';
import {loadRequestHistory,loadNotFoundPaths,type RequestHistory as History,type NotFoundPaths,type RequestHistoryRange} from '../../api/admin/requestHistory.ts';
import {t,loadCatalog,formatNumber} from '../../i18n';
import {AdminProblemError} from './shell/adminApi.ts';
import {StatTile} from './shell/StatTile.tsx';
import {NotFoundPathsTable} from './NotFoundPaths.tsx';
import {formatDateTime} from '../../i18n/format.ts';
import {Icon} from '../../components/Icon.tsx';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {Sparkline} from './shell/Sparkline.tsx';
import {formatCount,formatMs} from './Dashboard/format.ts';
import {HISTORY_PRESETS,relativeHistoryRange,historyLocalInput,customHistoryRange} from './RequestHistoryRange.ts';
import * as css from './RequestHistory.css.ts';
import * as charts from './Dashboard/dashboard.css.ts';
import * as admin from './admin.css.ts';

/** 404s over the points where they were tracked; `null` when no point was (never zero). */
export function notFoundSummary(value:History):{total:number|null;requests:number} {
 let total:number|null=null;let requests=0;
 for(const point of value.series){if(typeof point.notFound!=='number')continue;total=(total??0)+point.notFound;requests+=point.requests??0;}
 return {total,requests};
}
type PathsResult={value:NotFoundPaths}|{denied:boolean};
export function RequestHistory(props:{refreshToken?:unknown}={}):JSX.Element {
 onMount(()=>void loadCatalog('admin-requests'));
 const [selection,setSelection]=createSignal('default');
 const [request,setRequest]=createSignal<{range?:RequestHistoryRange}>({});
 const [from,setFrom]=createSignal(historyLocalInput(Date.now()/1000-3600));
 const [to,setTo]=createSignal(historyLocalInput(Date.now()/1000));
 const [validation,setValidation]=createSignal('');
 let controller:AbortController|undefined;
 onCleanup(()=>controller?.abort());
 const [result]=createResource(request,async query=>{
  controller?.abort();controller=new AbortController();
  const signal=controller.signal;
  // Top paths need Observability read; a Dashboard-only reader still gets the counts.
  const paths:Promise<PathsResult>=loadNotFoundPaths(query.range,signal).then(value=>({value}),(error:unknown)=>({denied:error instanceof AdminProblemError&&error.status===403}));
  try { const data=await loadRequestHistory(query.range,signal); return {data,paths:await paths}; }
  catch { void paths; return {failed:true as const}; }
 });
 const data=()=>result.latest?.data;
 const paths=()=>{const value=result.latest?.paths;return value&&'value' in value?value.value:undefined;};
 const pathsFailed=()=>{const value=result.latest?.paths;return value!==undefined&&'denied' in value&&!value.denied;};
 const maxRange=()=>data()?.policy.maxRangeSeconds??2592000;
 function reload():void {
  setValidation('');
  if(selection()==='custom'){
   const parsed=customHistoryRange(from(),to(),maxRange());
   if(parsed.error){setValidation(t(`admin-history-error-${parsed.error}`));return;}
   setRequest({range:parsed.range});
  }else if(selection()==='default')setRequest({});
  else setRequest({range:relativeHistoryRange(Number(selection()))});
 }
 createEffect(on(()=>props.refreshToken,()=>reload(),{defer:true}));
 const timestamp=(seconds:number)=>formatDateTime(seconds*1000);
 return <section class={charts.panel} aria-label={t('admin-history-title')} aria-busy={result.loading}>
  <h3 class={charts.panelTitle}>{t('admin-history-title')}</h3>
  <form class={css.controls} onSubmit={e=>{e.preventDefault();reload();}}>
   <label class={css.field}><span>{t('admin-history-period')}</span><select class={css.input} aria-label={t('admin-history-period')} value={selection()} onChange={e=>{setSelection(e.currentTarget.value);setValidation('');if(selection()!=='custom')reload();}}>
    <option value="default">{t('admin-history-default')}</option>
    <For each={HISTORY_PRESETS.filter(seconds=>seconds<=maxRange())}>{seconds=><option value={seconds}>{t(`admin-history-period-${seconds}`)}</option>}</For>
    <option value="custom">{t('admin-history-custom')}</option>
   </select></label>
   <Show when={selection()==='custom'}>
    <label class={css.field}><span>{t('admin-history-from')}</span><input class={css.input} type="datetime-local" required value={from()} onInput={e=>setFrom(e.currentTarget.value)}/></label>
    <label class={css.field}><span>{t('admin-history-to')}</span><input class={css.input} type="datetime-local" required value={to()} onInput={e=>setTo(e.currentTarget.value)}/></label>
   </Show>
   <button class="btn btn--ghost" type="submit" aria-label={t('admin-history-refresh')} title={t('admin-history-refresh')}><Icon name="refresh"/></button>
  </form>
  <Show when={validation()}><p role="alert" class={admin.error}>{validation()}</p></Show>
  <p class={admin.note}>{t('admin-history-help')}</p>
  <Show when={result.latest?.failed}><p role="alert" class={admin.error}>{t('admin-history-load-error')}</p></Show>
  <Show when={data()} fallback={<Show when={result.loading}><AsyncPending variant="page"/></Show>}>{value=><>
   <p class={admin.note}>{t('admin-history-range',{from:timestamp(value().from),to:timestamp(value().to),seconds:value().bucketSeconds})}</p>
   <Show when={value().coverage.observedSeconds<value().coverage.requestedSeconds}><p role="status">{t('admin-history-partial',{observed:Math.floor(value().coverage.observedSeconds/60),requested:Math.ceil(value().coverage.requestedSeconds/60)})}</p></Show>
   <p class={admin.note}>{t('admin-history-latency-resolution')}</p>
   <Show when={value().series.some(point=>point.legacyLatencyEstimate!==false && point.requests!==null && point.requests>0)}><p role="status" class={admin.note}>{t('admin-history-latency-legacy')}</p></Show>
   <Show when={(value().coverage.notFound?.notTracked.length??0)>0}><p role="status" class={admin.note} data-testid="not-found-not-tracked">{value().coverage.notFound?.trackedSince==null?t('admin-history-nf-not-tracked-window'):t('admin-history-nf-not-tracked-before',{time:timestamp(value().coverage.notFound!.trackedSince!)})}</p></Show>
   <div class={charts.charts}>
    <StatTile testId="tile-not-found" label={t('admin-history-nf-total')} value={notFoundSummary(value()).total} unavailableReason={t('admin-history-nf-not-tracked-window')} detail={(()=>{const {total,requests}=notFoundSummary(value());return total!==null&&requests>0?t('admin-history-nf-share',{share:formatNumber(total/requests,{style:'percent',maximumFractionDigits:1})}):undefined;})()}/>
    <Sparkline title={t('admin-history-not-found')} testId="spark-not-found" points={value().series.map(p=>({t:p.t,value:typeof p.notFound==='number'?p.notFound:null}))} format={formatCount} formatTime={timestamp}/>
   </div>
   <div class={charts.charts}>
    <Sparkline title={t('admin-history-requests')} testId="spark-requests" points={value().series.map(p=>({t:p.t,value:p.requests}))} format={formatCount} formatTime={timestamp}/>
    <Sparkline title={t('admin-history-errors')} testId="spark-errors" points={value().series.map(p=>({t:p.t,value:p.errors5xx}))} tone="danger" format={formatCount} formatTime={timestamp}/>
    <Sparkline title={t('admin-obs-p50')} points={value().series.map(p=>({t:p.t,value:p.p50Ms}))} format={formatMs} formatTime={timestamp}/>
    <Sparkline title={t('admin-dash-p95')} testId="spark-p95" points={value().series.map(p=>({t:p.t,value:p.p95Ms}))} format={formatMs} formatTime={timestamp}/>
   </div>
   <Show when={paths()}>{list=><NotFoundPathsTable value={list()} formatTime={timestamp}/>}</Show>
   <Show when={pathsFailed()}><p role="status" class={admin.note}>{t('admin-history-nf-paths-error')}</p></Show>
  </>}</Show>
 </section>;
}
