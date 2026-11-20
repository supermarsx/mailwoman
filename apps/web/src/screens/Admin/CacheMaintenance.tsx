import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useCatalog,t,formatDateTime,formatNumber} from '../../i18n';
import {loadDatabaseMaintenance,type DatabaseMaintenanceReport} from '../../api/admin/databaseMaintenance.ts';
import {listAdminSettings,type AdminSetting} from '../../api/admin/settings.ts';
import {InlineSettingEditor} from './Settings/InlineSettingEditor.tsx';
import {useShell} from './shell/context.ts';
import {adminRequest} from './shell/adminApi.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {InlineLoading} from '../../components/InlineLoading.tsx';
import {Icon} from '../../components/Icon.tsx';
import {formatBytes} from './Dashboard/format.ts';
import * as css from './admin.css.ts';

const expiryKeys=['maintenance.database.expireCachedBodies','maintenance.database.cachedBodyRetentionDays'];
const changeKeys=['maintenance.database.changeLogRetentionDays','maintenance.database.changeLogBatchRows'];
const cleanupKeys=['maintenance.database.enabled','maintenance.database.orphanBodies','maintenance.database.graceHours','maintenance.database.incrementalVacuum'];
const advancedKeys=['maintenance.database.intervalSeconds','maintenance.database.batchRows','maintenance.database.batchMiB','maintenance.database.batchPages','maintenance.database.maxRunMs','maintenance.database.minFreeMiB','maintenance.database.maxWalMiB'];

export function CacheMaintenance():JSX.Element{
 const catalog=useCatalog('admin-cache'),databaseCatalog=useCatalog('admin-database');
 const shell=useShell();
 const [report,setReport]=createSignal<DatabaseMaintenanceReport>();
 const [settings,setSettings]=createSignal<AdminSetting[]>();
 const [busy,setBusy]=createSignal(false),[error,setError]=createSignal<string|null>(null);
 let disposed=false,loading=false,settingsRequested=false,request:AbortController|undefined;
 const expiry=()=>report()?.cachedBodyExpiry??report()?.lastReport?.cachedBodyExpiry;
 const selected=(keys:string[])=>keys.flatMap(key=>settings()?.find(row=>row.key===key)??[]);
 async function reload():Promise<void>{
  if(disposed||loading)return;
  loading=true;request=new AbortController();
  try{const result=await loadDatabaseMaintenance(request.signal);if(!disposed)setReport(result);}
  catch(cause){if(!disposed)setError(describeAdminError(cause));}
  finally{loading=false;}
 }
 async function loadSettings():Promise<void>{
  if(!shell.can.can('settings.read'))return;
  try{const result=await listAdminSettings();if(!disposed){if(result.kind==='ok')setSettings(result.data);else setError('admin-cache-settings-unavailable');}}
  catch(cause){if(!disposed)setError(describeAdminError(cause));}
 }
 async function run():Promise<void>{
  if(busy()||!shell.can.can('maintenance.run'))return;
  setBusy(true);setError(null);
  try{await adminRequest('/maintenance/database-background',{method:'POST'});await reload();}
  catch(cause){if(!disposed)setError(describeAdminError(cause));}
  finally{if(!disposed)setBusy(false);}
 }
 createEffect(()=>{if(shell.can.can('settings.read')&&!settingsRequested){settingsRequested=true;void loadSettings();}});
 onMount(()=>{
  void reload();
  const timer=setInterval(()=>{if(document.visibilityState!=='hidden')void reload();},10000);
  onCleanup(()=>{disposed=true;request?.abort();clearInterval(timer);});
 });
 const editors=(keys:string[])=><For each={selected(keys)}>{row=><InlineSettingEditor settingKey={row.key} setting={row} {...(row.key===expiryKeys[0]?{label:t('admin-cache-expiry-enabled')}:row.key===expiryKeys[1]?{label:t('admin-cache-retention-days')}:{})} noScoped onChanged={saved=>setSettings(rows=>rows?.map(value=>value.key===saved.key?saved:value))}/>}</For>;
 return <Show when={catalog()&&databaseCatalog()} fallback={<InlineLoading/>}><section class={css.section} aria-label={t('admin-cache-title')}>
  <div class={css.headingRow}><h2 class={css.heading}>{t('admin-cache-title')}</h2><button type="button" class="btn btn--ghost" disabled={busy()||report()?.running||!shell.can.can('maintenance.run')} onClick={()=>void run()}><Icon name="refresh"/>{t('admin-db-background-run')}</button></div>
  <p class={css.note}>{t('admin-cache-help')}</p>
  <Feedback kind="error" as="toast" message={error()==='admin-cache-settings-unavailable'?t('admin-cache-settings-unavailable'):error()}/>
  <section class={css.card}><h3 class={css.heading}>{t('admin-cache-expiry-status')}</h3>
   <Show when={report()?.lastReport?.walRecovery?.status==='recycled'}><p class={css.note}>{t('admin-cache-wal-recycled')}</p></Show>
   <Show when={report()?.lastReport?.walRecovery?.status==='recovery-required'}><p class={css.note}>{t('admin-cache-wal-recovery')}</p></Show>
   <Show when={expiry()} fallback={<p class={css.note}>{t('admin-cache-no-status')}</p>}>{value=><>
    <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(value().at*1000)})}</p>
    <p>{t('admin-cache-expiry-result',{scanned:formatNumber(value().scannedMessages),detached:formatNumber(value().detachedMessages),protected:formatNumber(value().protectedMessages)})}</p>
    <table class={css.table}><tbody><tr><th scope="row">{t('admin-cache-removed-payload')}</th><td>{formatBytes(value().removedBytes)}</td></tr>
     <Show when={typeof value().cleanedEnvelopes==='number'}><tr><th scope="row">{t('admin-cache-parsed-copies')}</th><td>{formatNumber(value().cleanedEnvelopes!)}</td></tr></Show>
     <Show when={typeof value().removedEnvelopeBytes==='number'}><tr><th scope="row">{t('admin-cache-parsed-bytes')}</th><td>{formatBytes(value().removedEnvelopeBytes!)}</td></tr></Show>
     <Show when={typeof value().oversizedBodies==='number'}><tr><th scope="row">{t('admin-cache-oversized')}</th><td>{formatNumber(value().oversizedBodies!)}</td></tr></Show>
    </tbody></table>
   </>}</Show>
   <Show when={report()?.changeLogRetention}>{value=><>
    <p>{t('admin-cache-change-removed')}: {formatNumber(value().removedRows)}</p>
    <Show when={value().chunkedStates}><p class={css.note}>{t('admin-cache-change-chunked')}: {formatNumber(value().chunkedStates!)}</p></Show>
   </>}</Show>
   <p class={css.note}>{t('admin-cache-expiry-help')}</p>
   <Show when={expiry()?.status==='no-refetchable-sources'}><p class={css.note}>{t('admin-cache-no-sources')}</p></Show>
   <Show when={['busy','budget','wal-pressure'].includes(expiry()?.status??'')}><p class={css.note}>{t('admin-db-background-deferred')}</p></Show>
   <p class={css.note}>{t('admin-cache-inspect-help')}</p><a href="#/observability/database">{t('admin-cache-inspect')}</a>
  </section>
  <Show when={report()?.lastReport}>{value=><section class={css.card}>
   <h3 class={css.heading}>{t('admin-db-background-title')}</h3>
   <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(value().at*1000)})}</p>
   <p>{t('admin-db-background-result',{scanned:formatNumber(value().scanned),removed:formatNumber(value().removed),bytes:formatBytes(value().removedBytes),pages:formatNumber(value().reclaimedPages)})}</p>
   <Show when={report()?.lastError||['busy','budget','wal-pressure'].includes(value().status)}><p class={css.note}>{t('admin-db-background-deferred')}</p></Show>
  </section>}</Show>
  <Show when={settings()} fallback={<Show when={shell.can.can('settings.read')&&!error()}><InlineLoading/></Show>}>
   <h3 class={css.heading}>{t('admin-cache-expiry')}</h3><p class={css.note}>{t('admin-cache-settings-help')}</p>{editors(expiryKeys)}
   <h3 class={css.heading}>{t('admin-cache-orphans')}</h3>{editors(cleanupKeys)}
   <h3 class={css.heading}>{t('admin-cache-change-log')}</h3><p class={css.note}>{t('admin-cache-change-log-help')}</p>{editors(changeKeys)}
   <details><summary>{t('admin-cache-advanced')}</summary><div class={css.section}>{editors(advancedKeys)}</div></details>
  </Show>
 </section></Show>;
}
