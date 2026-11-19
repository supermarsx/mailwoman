import {createEffect,createSignal,For,onCleanup,onMount,Show,useContext,type JSX} from 'solid-js';
import {loadCatalog,t,formatDateTime} from '../../i18n';
import {loadOnlineConversion,startOnlineConversion,cancelOnlineConversion,type OnlineConversionReport} from '../../api/admin/onlineDatabase.ts';
import {Icon} from '../../components/Icon.tsx';
import {ShellContext} from './shell/context.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes,formatCount} from './Dashboard/format.ts';
import * as css from './admin.css.ts';
const terminal=new Set(['completed','cancelled','failed']);
const phases:Record<string,string>={preparing:'preparing',copying:'copying',verifying:'verifying','catching-up':'catching-up','waiting-for-cutover':'waiting-for-cutover',switching:'switching',publishing:'publishing','recovery-required':'recovery-required',completed:'completed',cancelled:'cancelled',failed:'failed'};
export function OnlineDatabaseConversion():JSX.Element{
 void loadCatalog('admin-database');const shell=useContext(ShellContext);
 const canRead=()=>shell?.can.can('maintenance.read')===true;
 const canRun=()=>shell?.can.can('maintenance.run')===true;
 const [report,setReport]=createSignal<OnlineConversionReport|null>(null),[loading,setLoading]=createSignal(false),[busy,setBusy]=createSignal(false),[failure,setFailure]=createSignal<string|null>(null);
 const active=()=>!!report()&&!terminal.has(report()!.phase);
 const cancellable=()=>active()&&!['switching','publishing','recovery-required'].includes(report()!.phase);
 let disposed=false,initialRequested=false,request:AbortController|undefined;
 async function reload():Promise<void>{
  if(disposed||loading()||busy()||!canRead())return;setLoading(true);request=new AbortController();
  try{const value=await loadOnlineConversion(request.signal);if(!disposed)setReport(value);}catch(error){if(!disposed)setFailure(describeAdminError(error));}finally{if(!disposed)setLoading(false);}
 }
 createEffect(()=>{if(canRead()&&!initialRequested){initialRequested=true;void reload();}});
 onMount(()=>{const visible=()=>document.visibilityState!=='hidden';const timer=setInterval(()=>{if(visible()&&active())void reload();},2000);const visibility=()=>{if(visible()&&active())void reload();};document.addEventListener('visibilitychange',visibility);onCleanup(()=>{disposed=true;request?.abort();clearInterval(timer);document.removeEventListener('visibilitychange',visibility);});});
 async function start():Promise<void>{if(busy()||loading()||active()||!canRun())return;setBusy(true);setFailure(null);try{const value=await startOnlineConversion();if(!disposed)setReport(value);}catch(error){if(!disposed)setFailure(describeAdminError(error));}finally{if(!disposed)setBusy(false);}}
 async function cancel():Promise<void>{if(busy()||loading()||!cancellable()||!canRun())return;setBusy(true);setFailure(null);try{const requested=await cancelOnlineConversion();const value=await loadOnlineConversion();if(!disposed){setReport(value);if(!requested)setFailure(t('admin-db-online-cancel-unavailable'));}}catch(error){if(!disposed)setFailure(describeAdminError(error));}finally{if(!disposed)setBusy(false);}}
 const fields=(entries:()=>[string,string][]) => <table class={css.table}><tbody><For each={entries()}>{([label,value])=><tr><th scope="row">{label}</th><td>{value}</td></tr>}</For></tbody></table>;
 return <Show when={canRead()}><section class={css.card} aria-label={t('admin-db-online-title')}>
  <div class={css.headingRow}><h3 class={css.heading}>{t('admin-db-online-title')}</h3><div class={css.headingRow}><button type="button" class="btn" disabled={busy()||loading()||active()||!canRun()} onClick={()=>void start()}><Icon name="database"/>{t('admin-db-online-start')}</button><Show when={active()}><button type="button" class="btn btn--ghost" disabled={busy()||loading()||!cancellable()||!canRun()} onClick={()=>void cancel()}><Icon name="close"/>{t('admin-db-online-cancel')}</button></Show><button type="button" class="btn btn--ghost" disabled={loading()||busy()} onClick={()=>void reload()}><Icon name="refresh"/>{t('admin-db-refresh')}</button></div></div>
  <p class={css.note}>{t('admin-db-online-help')}</p><Feedback kind="error" as="toast" message={failure()}/>
  <Show when={report()} fallback={<p class={css.note}>{t('admin-db-online-none')}</p>}>{value=><>
   <p role="status">{t('admin-db-online-phase')}: {phases[value().phase]?t(`admin-db-online-phase-${phases[value().phase]}`):value().phase}</p>
   <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(value().updatedAt*1000)})}</p>
   <Show when={value().progress} keyed>{progress=>fields(()=>[[t('admin-db-table'),`${formatCount(progress.tableIndex)} / ${formatCount(progress.tableCount)}${progress.tableName?` · ${progress.tableName}`:''}`],[t('admin-db-online-copied-rows'),formatCount(progress.copiedRows)],[t('admin-db-online-copied-bytes'),formatBytes(progress.copiedBytes)],[t('admin-db-online-pending'),formatCount(progress.pending)]])}</Show>
   <Show when={value().error}><p class={css.note}>{value().error}</p></Show>
   <Show when={value().retainedPrevious}><p class={css.note}>{t('admin-db-online-retained')}</p></Show>
  </>}</Show>
  <p class={css.note}>{t('admin-db-online-poll')}</p>
 </section></Show>;
}
