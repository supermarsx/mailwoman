import {createEffect,createSignal,onCleanup,Show,type JSX} from 'solid-js';
import {t,useCatalog,formatNumber} from '../../i18n';
import {loadPreparedMailStatus,type PreparedMailStatus} from '../../api/admin/preparedMail.ts';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {InlineSettingEditor} from './Settings/InlineSettingEditor.tsx';
import {useShell} from './shell/context.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes} from './Dashboard/format.ts';
import * as css from './admin.css.ts';
export function PreparedMailCard(props:{settings:AdminSetting[]|undefined;onChanged:(row:AdminSetting)=>void}):JSX.Element{
 const catalog=useCatalog('admin-cache'),shell=useShell();
 useCatalog('admin-settings');
 const [status,setStatus]=createSignal<PreparedMailStatus>(),[error,setError]=createSignal<string|null>(null),[busy,setBusy]=createSignal(false);
 let disposed=false;let request:AbortController|undefined;
 const refresh=async()=>{if(disposed||request||!shell.can.can('maintenance.read'))return;const controller=new AbortController();request=controller;setBusy(true);try{const result=await loadPreparedMailStatus(controller.signal);if(!disposed){setStatus(result);setError(null);}}catch(cause){if(!disposed)setError(describeAdminError(cause));}finally{request=undefined;if(!disposed)setBusy(false);}};
 createEffect(()=>{if(shell.can.can('maintenance.read'))void refresh();});
 onCleanup(()=>{disposed=true;request?.abort();});
 const row=()=>props.settings?.find(row=>row.key==='performance.preparedMail');
 const number=(value:number|null)=>value===null?t('admin-cache-prepared-unknown'):formatNumber(value);
 return <Show when={catalog()}><section class={css.card} aria-label={t('admin-cache-prepared-title')}>
  <div class={css.headingRow}><h3 class={css.heading}>{t('admin-cache-prepared-title')}</h3><button type="button" class="btn btn--ghost" disabled={busy()||!shell.can.can('maintenance.read')} onClick={()=>void refresh()}>{t('admin-cache-prepared-refresh-status')}</button></div>
  <p class={css.note}>{t('admin-cache-prepared-help')}</p><Feedback kind="error" message={error()}/>
  <Show when={status()} fallback={<p>{t('admin-cache-prepared-unknown')}</p>}>{value=><table class={css.table}><tbody>
   <tr><th scope="row">{t('admin-cache-prepared-enabled')}; {t('admin-cache-prepared-idleSeconds')}</th><td>{value().enabled===null?t('admin-cache-prepared-unknown'):t(value().enabled?'admin-settings-value-on':'admin-settings-value-off')}; {number(value().idleSeconds)}</td></tr>
   <tr><th scope="row">{t('admin-cache-prepared-memory')}</th><td>{formatBytes(value().chargedBytes)} / {value().budgetBytes===null?t('admin-cache-prepared-unknown'):formatBytes(value().budgetBytes!)}</td></tr>
   <tr><th scope="row">{t('admin-cache-prepared-residents')}</th><td>{number(value().entries)} / {number(value().maxEntries)}; {number(value().accounts)} / {number(value().maxAccounts)}</td></tr>
   <tr><th scope="row">{t('admin-cache-prepared-work')}</th><td>{formatNumber(value().admittedJobs)} / 16; {formatNumber(value().activePreparers)} / 2; {number(value().pendingBuilds)}</td></tr>
   <tr><th scope="row">{t('admin-cache-prepared-results')}</th><td>{formatNumber(value().readyCandidateHits)} / {formatNumber(value().candidateMisses)} / {formatNumber(value().completedBuilds)} / {formatNumber(value().evictions)}</td></tr>
   <tr><th scope="row">{t('admin-cache-prepared-seeds')}</th><td>{formatNumber(value().seedCalls)} / {formatNumber(value().seededRows)}; {formatNumber(value().invalidationErrors)}</td></tr>
  </tbody></table>}</Show>
  <p class={css.note}>{t('admin-cache-prepared-limits')}</p><p class={css.note}>{t('admin-cache-prepared-refresh')}</p>
  <Show when={row()}>{setting=><InlineSettingEditor settingKey="performance.preparedMail" setting={setting()} noScoped onChanged={saved=>{props.onChanged(saved);void refresh();}}/>}</Show>
 </section></Show>;
}
