import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {t,useCatalog,formatNumber,formatDateTime} from '../../i18n';
import {loadPublicAssetCache,clearPublicAssetCache,type PublicAssetCacheStatus} from '../../api/admin/publicAssets.ts';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {InlineSettingEditor} from './Settings/InlineSettingEditor.tsx';
import {useShell} from './shell/context.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes} from './Dashboard/format.ts';
import * as css from './admin.css.ts';
export function PublicAssetCacheCard(props:{settings:AdminSetting[]|undefined;onChanged:(row:AdminSetting)=>void}):JSX.Element{
 const shell=useShell(),catalog=useCatalog('admin-cache');
 const [status,setStatus]=createSignal<PublicAssetCacheStatus>();
 const [busy,setBusy]=createSignal(false),[error,setError]=createSignal<string|null>(null);
 let disposed=false,loading=false;const requests=new Set<AbortController>();
 async function request(clear=false):Promise<void>{
  if(disposed||loading||!shell.can.can(clear?'maintenance.run':'maintenance.read'))return;
  loading=true;setBusy(true);const controller=new AbortController();requests.add(controller);
  try{const value=await (clear?clearPublicAssetCache:loadPublicAssetCache)(controller.signal);if(!disposed){setStatus(value);setError(null);}}
  catch(cause){if(!disposed)setError(describeAdminError(cause));}
  finally{requests.delete(controller);loading=false;if(!disposed)setBusy(false);}
 }
 createEffect(()=>{if(shell.can.can('maintenance.read'))void request();});
 onMount(()=>{const timer=setInterval(()=>{if(document.visibilityState!=='hidden')void request();},10000);onCleanup(()=>{disposed=true;clearInterval(timer);for(const request of requests)request.abort();});});
 const rows=()=>props.settings?.filter(row=>row.key.startsWith('web.assets.cache.'))??[];
 return <Show when={catalog()}><section class={css.card} aria-label={t('admin-cache-assets-title')}>
  <div class={css.headingRow}><h3 class={css.heading}>{t('admin-cache-assets-title')}</h3><button type="button" class="btn btn--ghost" disabled={busy()||!shell.can.can('maintenance.run')} onClick={()=>void request(true)}>{t('admin-cache-assets-clear')}</button></div>
  <p class={css.note}>{t('admin-cache-assets-help')}</p>
  <Feedback kind="error" message={error()}/>
  <Show when={status()} fallback={<p class={css.note}>{t('admin-cache-no-status')}</p>}>{value=><>
   <Show when={value().configurationStale}><p role="status" class={css.note}>{t('admin-cache-assets-stale')}</p></Show>
   <Show when={value().lastConfigAt!==null}><p class={css.note}>{t('admin-cache-assets-applied',{time:formatDateTime(value().lastConfigAt!*1000)})}</p></Show>
   <table class={css.table}><tbody>
    <tr><th scope="row">{t('admin-cache-assets-resident')}</th><td>{formatBytes(value().residentOwnedBytes)} / {formatBytes(value().config.budgetMiB*1024*1024)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-entries')}</th><td>{formatNumber(value().entries)} / {formatNumber(value().config.maxEntries)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-file')}</th><td>{formatBytes(value().effectiveMaxFileBytes)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-loads')}</th><td>{formatNumber(value().runningLoads)} / {formatNumber(value().maxRunningLoads)}; {formatNumber(value().queuedUnique)} / {formatNumber(value().maxQueuedUnique)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-hits')}</th><td>{formatNumber(value().counters.hits)} / {formatNumber(value().counters.misses)} / {formatNumber(value().counters.coalesced)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-evictions')}</th><td>{formatNumber(value().counters.evictions)} / {formatNumber(value().counters.expired)} / {formatNumber(value().counters.oversize)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-failures')}</th><td>{formatNumber(value().counters.busy)} / {formatNumber(value().counters.loadFailures)}</td></tr>
    <tr><th scope="row">{t('admin-cache-assets-preload')}</th><td>{value().effectivePreload}; {formatNumber(value().counters.preloadLoaded)} / {formatNumber(value().counters.preloadSkipped)}</td></tr>
   </tbody></table>
  </>}</Show>
  <p class={css.note}>{t('admin-cache-assets-clear-help')}</p>
  <For each={rows()}>{row=><InlineSettingEditor settingKey={row.key} setting={row} noScoped onChanged={saved=>{props.onChanged(saved);void request();}}/>}</For>
 </section></Show>;
}
