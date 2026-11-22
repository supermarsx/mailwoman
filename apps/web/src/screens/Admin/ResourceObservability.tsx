import {createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useCatalog,t,formatDateTime,formatNumber} from '../../i18n';
import {loadResourceSnapshot,CPU_KERNELS,type CpuBackend,type ResourceSnapshot} from '../../api/admin/resources.ts';
import {Icon} from '../../components/Icon.tsx';
import {InlineLoading} from '../../components/InlineLoading.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes} from './Dashboard/format.ts';
import * as css from './admin.css.ts';
import * as resources from './resourceObservability.css.ts';

const CPU_LABELS:Record<Exclude<CpuBackend,'scalar'>,string>={sse2:'SSE2',sse3:'SSE3',ssse3:'SSSE3',sse41:'SSE4.1',sse42:'SSE4.2',avx2:'AVX2',avx512:'AVX-512F + BW'};

export function ResourceObservability():JSX.Element{
 const catalog=useCatalog('admin-database');
 const [report,setReport]=createSignal<ResourceSnapshot>();
 const [loading,setLoading]=createSignal(false),[failure,setFailure]=createSignal<string|null>(null);
 let disposed=false,request:AbortController|undefined;
 async function reload():Promise<void>{
  if(disposed||loading())return;
  setLoading(true);setFailure(null);request=new AbortController();
  try{const result=await loadResourceSnapshot(request.signal);if(!disposed)setReport(result);}
  catch(error){if(!disposed)setFailure(describeAdminError(error));}
  finally{if(!disposed)setLoading(false);}
 }
 onMount(()=>{
  void reload();
  const visible=()=>document.visibilityState!=='hidden';
  const timer=setInterval(()=>{if(visible())void reload();},10000);
  const visibility=()=>{if(visible())void reload();};
  document.addEventListener('visibilitychange',visibility);
  onCleanup(()=>{disposed=true;request?.abort();clearInterval(timer);document.removeEventListener('visibilitychange',visibility);});
 });
 const absent=()=>t('admin-shell-not-available');
 const kernelLabel=(backend:CpuBackend)=>backend==='scalar'?t('admin-resources-cpu-portable'):CPU_LABELS[backend]??backend;
 const bytes=(value:number|null)=>typeof value==='number'?formatBytes(value):absent();
 const fields=(entries:()=>[string,string][]) => <table class={resources.table}><tbody><For each={entries()}>{([label,value])=><tr><th scope="row">{label}</th><td>{value}</td></tr>}</For></tbody></table>;
 return <Show when={catalog()} fallback={<InlineLoading/>}><section class={css.section} aria-label={t('admin-resources-title')}>
  <div class={css.headingRow}><h2 class={css.heading}>{t('admin-resources-title')}</h2><button type="button" class="btn btn--ghost" disabled={loading()} onClick={()=>void reload()}><Icon name="refresh"/>{t('admin-db-refresh')}</button></div>
  <p class={css.note}>{t('admin-resources-help')}</p>
  <Feedback kind="error" as="toast" message={failure()}/>
  <Show when={report()} fallback={<Show when={loading()}><InlineLoading/></Show>}>{value=><>
   <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(value().observedAt*1000)})}</p>
   <Show when={value().cgroupDetected}><p class={css.note}>{t('admin-resources-container')}</p></Show>
   <div class={resources.grid}>
    <Show when={value().cpuCapabilities}>{cpu=><section class={resources.cpuCard} aria-label={t('admin-resources-cpu-capabilities')}><h3 class={resources.heading}>{t('admin-resources-cpu-capabilities')}</h3>{fields(()=>[
     [t('admin-resources-cpu-architecture'),cpu().architecture],
     [t('admin-resources-cpu-kernel'),kernelLabel(cpu().selectedBackend)],
     ...(cpu().highestAvailableBackend?[[t('admin-resources-cpu-highest-kernel'),kernelLabel(cpu().highestAvailableBackend!)] as [string,string]]:[]),
    ])}<h4 class={resources.heading}>{t('admin-resources-cpu-features')}</h4><p class={css.note}>{cpu().availableFeatures.length?cpu().availableFeatures.join(' · '):absent()}</p><p class={css.note}>{t(typeof cpu().sseMinBytes==='number'?'admin-resources-cpu-kernel-help-sse':'admin-resources-cpu-kernel-help',{sse:formatNumber(cpu().sseMinBytes??0),avx2:formatNumber(cpu().avx2MinBytes),avx512:formatNumber(cpu().avx512MinBytes)})}</p>
     <h4 class={resources.heading}>{t('admin-resources-cpu-usage')}</h4>
     <Show when={value().cpuUsage} fallback={<p class={css.note}>{absent()}</p>}>{usage=><table class={resources.table} aria-label={t('admin-resources-cpu-usage')}><thead><tr><th scope="col">{t('admin-resources-cpu-path')}</th><th scope="col">{t('admin-resources-cpu-calls')}</th><th scope="col">{t('admin-resources-cpu-bytes')}</th></tr></thead><tbody><For each={CPU_KERNELS.filter(backend=>usage()[backend]!==undefined)}>{backend=><tr><th scope="row">{kernelLabel(backend)}</th><td>{formatNumber(usage()[backend]?.calls??0)}</td><td>{bytes(usage()[backend]?.bytes??null)}</td></tr>}</For></tbody></table>}</Show>
     <p class={css.note}>{t('admin-resources-cpu-usage-help')}</p>
    </section>}</Show>
    <section class={resources.card} aria-label={t('admin-resources-allocation')}><h3 class={resources.heading}>{t('admin-resources-allocation')}</h3>{fields(()=>[
     [t('admin-resources-cpu-cores'),formatNumber(value().effectiveCpuCores,{maximumFractionDigits:2})],
     [t('admin-resources-memory-limit'),bytes(value().effectiveMemoryLimitBytes)],
     [t('admin-resources-memory-available'),bytes(value().effectiveMemoryAvailableBytes)],
    ])}</section>
    <section class={resources.card} aria-label={t('admin-resources-process')}><h3 class={resources.heading}>{t('admin-resources-process')}</h3>{fields(()=>[
     [t('admin-resources-process-memory'),bytes(value().processResidentBytes)],
     [t('admin-resources-process-cpu'),typeof value().processCpuPercent==='number'?formatNumber(value().processCpuPercent!/100,{style:'percent',maximumFractionDigits:1}):absent()],
    ])}<p class={css.note}>{t('admin-resources-cpu-help')}</p><Show when={value().processCpuPercent===null}><p class={css.note}>{t('admin-resources-cpu-warmup')}</p></Show></section>
    <section class={resources.card} aria-label={t('admin-resources-host')}><h3 class={resources.heading}>{t('admin-resources-host')}</h3>{fields(()=>[
     [t('admin-resources-host-total'),bytes(value().hostMemoryTotalBytes)],
     [t('admin-resources-host-available'),bytes(value().hostMemoryAvailableBytes)],
    ])}</section>
    <section class={resources.card} aria-label={t('admin-resources-filesystem')}><h3 class={resources.heading}>{t('admin-resources-filesystem')}</h3>{fields(()=>[
     [t('admin-resources-filesystem-total'),bytes(value().filesystemTotalBytes)],
     [t('admin-resources-filesystem-available'),bytes(value().filesystemAvailableBytes)],
    ])}<p class={css.note}>{t('admin-resources-filesystem-help')}</p></section>
   </div>
  </>}</Show>
  <p class={css.note}>{t('admin-resources-poll-help')}</p>
 </section></Show>;
}
