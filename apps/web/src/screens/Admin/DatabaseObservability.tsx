import {createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {formatDateTime} from '../../i18n/format.ts';
import {loadDatabaseReport,type DatabaseReport} from '../../api/admin/database.ts';
import {DatabaseSizeHistory} from './DatabaseSizeHistory.tsx';
import {SearchInput} from '../../components/SearchInput.tsx';
import {Icon} from '../../components/Icon.tsx';
import {InlineLoading} from '../../components/InlineLoading.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes,formatCount,formatMs} from './Dashboard/format.ts';
import * as css from './admin.css.ts';

export function DatabaseObservability():JSX.Element{
 void loadCatalog('admin-database');
 const [report,setReport]=createSignal<DatabaseReport>();
 const [sortGrowth,setSortGrowth]=createSignal(false);
 const [loading,setLoading]=createSignal(false),[failure,setFailure]=createSignal<string|null>(null),[query,setQuery]=createSignal('');
 async function reload(detailed=false):Promise<void>{
  if(loading())return;setLoading(true);setFailure(null);
  try{const result=await loadDatabaseReport(detailed);setReport(previous=>({...result,details:result.details??previous?.details??null}));}catch(error){setFailure(describeAdminError(error));}finally{setLoading(false);}
 }
 onMount(()=>{let active=true;void reload().then(()=>{if(active&&report())void reload(true);});onCleanup(()=>{active=false;});const timer=setInterval(()=>void reload(),60000);onCleanup(()=>clearInterval(timer));});
 const absent=()=>t('admin-shell-not-available');
 const bytes=(value:number|null|undefined)=>typeof value==='number'?formatBytes(value):absent();
 const count=(value:number|null|undefined)=>typeof value==='number'?formatCount(value):absent();
 const rows=createMemo(()=>[...(report()?.details?.tables??[])].filter(row=>row.name.toLowerCase().includes(query().toLowerCase())).sort((a,b)=>sortGrowth()?(b.growthBytes??-Infinity)-(a.growthBytes??-Infinity):Math.max(b.bytes??-1,b.payloadBytes??-1)-Math.max(a.bytes??-1,a.payloadBytes??-1)));
 const fields=(entries:()=>[string,string][]) => <table class={css.table}><tbody><For each={entries()}>{([label,value])=><tr><th scope="row">{label}</th><td>{value}</td></tr>}</For></tbody></table>;
 return <section class={css.section} aria-label={t('admin-db-title')}>
  <div class={css.headingRow}><h2 class={css.heading}>{t('admin-db-title')}</h2><button type="button" class="btn btn--ghost" disabled={loading()} onClick={()=>void reload()}><Icon name="refresh"/>{t('admin-db-refresh')}</button></div>
  <p class={css.note}>{t('admin-db-help')}</p><Feedback kind="error" as="toast" message={failure()}/>
  <Show when={report()} fallback={<InlineLoading/>}>{value=><>
   <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(value().observedAt)})}</p>
   <div class={css.grid}>
    <section class={css.card}><h3 class={css.heading}>{t('admin-db-storage')}</h3>{fields(()=>[
     [t('admin-db-file'),bytes(value().diagnostics.dbBytes)],[t('admin-db-occupied'),bytes(value().storage.occupiedBytes)],
     [t('admin-db-free'),bytes(value().storage.freeBytes)],[t('admin-db-wal'),bytes(value().diagnostics.walBytes)],
     [t('admin-db-pages'),count(value().storage.pageCount)],[t('admin-db-page-size'),bytes(value().storage.pageSize)],
    ])}<p class={css.note}>{t('admin-db-free-help')}</p></section>
    <section class={css.card}><h3 class={css.heading}>{t('admin-db-connection')}</h3>{fields(()=>[
     [t('admin-db-driver'),`${value().driver} · ${value().database.backend}`],
     [t('admin-db-version'),value().database.serverVersion??absent()],[t('admin-db-name'),value().database.name??absent()],
     [t('admin-db-host'),value().database.host?`${value().database.host}${value().database.port?`:${value().database.port}`:''}`:t('admin-db-local')],
     [t('admin-db-journal'),value().database.journalMode??absent()],
     [t('admin-db-ping'),formatMs(value().diagnostics.dbPingMs)],
     [t('admin-db-migrations'),`${value().migrations.applied} / ${value().migrations.expected}`],
    ])}</section>
    <section class={css.card}><h3 class={css.heading}>{t('admin-db-pool')}</h3>{fields(()=>[
     [t('admin-db-open'),count(value().pool.open)],[t('admin-db-idle'),count(value().pool.idle)],[t('admin-db-busy'),count(value().pool.busy)],
     [t('admin-db-limits'),`${value().pool.minimum} / ${value().pool.maximum}`],
     [t('admin-db-acquire'),formatMs(value().pool.acquireTimeoutMs)],
    ])}</section>
   </div>
   <Show when={value().growth}>{growth=><DatabaseSizeHistory samples={growth().samples}/>}</Show>
   <section class={css.card}><div class={css.headingRow}><h3 class={css.heading}>{t('admin-db-breakdown')}</h3><button type="button" class="btn" disabled={loading()} onClick={()=>void reload(true)}><Icon name="search"/>{t('admin-db-inspect')}</button></div>
    <p class={css.note}>{t('admin-db-analysis-help')}</p><Show when={loading()}><InlineLoading/></Show>
    <Show when={value().details}>{details=><>
     <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(details().observedAt)})}</p>
     <Show when={details().limited}><p class={css.note}>{t('admin-db-limited')}</p></Show>
     <Show when={details().bodies}>{bodies=><>
      {fields(()=>[[t('admin-db-body-count'),count(bodies().count)],[t('admin-db-body-bytes'),bytes(bodies().bytes)],
       [t('admin-db-live-bytes'),bytes(bodies().liveBytes??(bodies().bytes!==null&&bodies().orphanBytes!==null?bodies().bytes!-bodies().orphanBytes!:null))],[t('admin-db-orphan-count'),count(bodies().orphanCount)],[t('admin-db-orphan-bytes'),bytes(bodies().orphanBytes)]])}
      <p class={css.note}>{t('admin-db-orphan-help')}</p>
     </>}</Show>
     <label>{t('admin-db-search')}<SearchInput value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
     <div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('admin-db-table')}</th><th>{t('admin-db-payload')}</th><th>{t('admin-db-size')}</th><th>{t('admin-db-columns')}</th><th>{t('admin-db-rows')}</th><th><button class="btn btn--ghost" type="button" aria-pressed={sortGrowth()} onClick={()=>setSortGrowth(v=>!v)}>{t('admin-db-growth')}</button></th></tr></thead><tbody><For each={rows()}>{row=><tr><th scope="row"><code>{row.name}</code></th><td>{bytes(row.payloadBytes)}</td><td>{bytes(row.bytes)}</td><td><For each={[...(row.columns??[])].sort((a,b)=>b.bytes-a.bytes).slice(0,3)} fallback={<span>{absent()}</span>}>{column=><div><code>{column.name}</code> · {bytes(column.bytes)}</div>}</For></td><td>{count(row.rows)}{row.rowsEstimated?` · ${t('admin-db-estimated')}`:''}</td><td title={row.growthSince?formatDateTime(row.growthSince):undefined}>{typeof row.growthBytes==='number'?`${row.growthBytes<0?'−':row.growthBytes>0?'+':''}${bytes(Math.abs(row.growthBytes))}`:absent()}</td></tr>}</For></tbody></table></div>
    </>}</Show>
   </section>
   <Show when={value().growth}>{growth=><section class={css.card}>
    <h3 class={css.heading}>{t('admin-db-growth')}</h3>
    <p role={growth().status==='rapid-growth'?'alert':undefined} class={growth().status==='rapid-growth'?css.error:css.note}>{t(growth().status==='rapid-growth'?'admin-db-growth-alert':growth().status==='warming-up'?'admin-db-growth-warming':'admin-db-growth-normal')}</p>
    {fields(()=>[[t('admin-db-growth-rate'),growth().bytesPerMinute===null?absent():`${growth().bytesPerMinute!<0?'−':''}${bytes(Math.abs(growth().bytesPerMinute!))} / min`],[t('admin-db-growth-threshold'),`${bytes(growth().thresholdBytesPerMinute)} / min`]])}
    <p class={css.note}>{t('admin-db-write-help')}</p>
    <div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('admin-db-origin')}</th><th>{t('admin-db-writes')}</th><th>{t('admin-db-rows')}</th><th>{t('admin-db-input')}</th></tr></thead><tbody><For each={growth().writeOrigins}>{origin=><tr><th scope="row"><code>{origin.origin}</code></th><td>{count(origin.statements)}</td><td>{count(origin.affectedRows)}</td><td>{bytes(origin.inputBytes)}</td></tr>}</For></tbody></table></div>
   </section>}</Show>

  </>}</Show>
 </section>;
}
