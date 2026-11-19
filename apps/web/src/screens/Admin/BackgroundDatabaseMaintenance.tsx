import {createSignal,createEffect,on,Show,useContext,type JSX} from 'solid-js';
import {t,loadCatalog,formatDateTime} from '../../i18n';
import type {BackgroundDatabaseReport} from '../../api/admin/database.ts';
import {adminRequest} from './shell/adminApi.ts';
import {ShellContext} from './shell/context.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes,formatCount} from './Dashboard/format.ts';
import {Icon} from '../../components/Icon.tsx';
import * as css from './admin.css.ts';
export function BackgroundDatabaseMaintenance(props:{report:BackgroundDatabaseReport;onRefresh?:()=>void}):JSX.Element{
 void loadCatalog('admin-database');const shell=useContext(ShellContext);
 const [latest,setLatest]=createSignal<BackgroundDatabaseReport>(),[busy,setBusy]=createSignal(false),[error,setError]=createSignal<string|null>(null);
 createEffect(on(()=>props.report,()=>setLatest(undefined)));
 const report=()=>latest()??props.report;
 async function run():Promise<void>{if(busy())return;setBusy(true);setError(null);try{setLatest((await adminRequest<BackgroundDatabaseReport>('/maintenance/database-background',{method:'POST'})).data);props.onRefresh?.();}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
 return <section class={css.card} data-testid="background-database-maintenance">
  <div class={css.headingRow}><h3 class={css.heading}>{t('admin-db-background-title')}</h3><div class={css.headingRow}><a class="btn btn--ghost" href="#/settings/maintenance.database.enabled">{t('admin-db-background-configure')}</a><button type="button" class="btn btn--ghost" disabled={busy()||report().running||!shell?.can.can('maintenance.run')} onClick={()=>void run()}><Icon name="refresh"/>{t('admin-db-background-run')}</button></div></div>
  <p class={css.note}>{t('admin-db-background-help')}</p><Feedback kind="error" as="toast" message={error()}/>
  <Show when={report().lastError}><p class={css.note}>{t('admin-db-background-deferred')}</p></Show>
  <Show when={report().running}><p role="status">{t('common-loading')}</p></Show>
  <Show when={report().lastReport} keyed>{last=><>
   <p class={css.note}>{t('admin-db-observed',{time:formatDateTime(last.at*1000)})}</p>
   <p>{t('admin-db-background-result',{scanned:formatCount(last.scanned),removed:formatCount(last.removed),bytes:formatBytes(last.removedBytes),pages:formatCount(last.reclaimedPages)})}</p>
   <Show when={last.vacuumStatus==='offline-conversion-required'}><p class={css.note}>{t('admin-db-background-conversion')} <code>./deploy.sh maintenance --apply --enable-incremental-vacuum</code></p></Show>
   <Show when={last.status==='busy'||last.status==='budget'||last.status==='wal-pressure'||last.vacuumStatus==='wal-pressure'||last.vacuumStatus==='budget'}><p class={css.note}>{t('admin-db-background-deferred')} <code>{last.status} / {last.vacuumStatus}</code></p></Show>
  </>}</Show>
  <Show when={report().nextRunAt}><p class={css.note}>{t('admin-db-background-next',{time:formatDateTime(report().nextRunAt!*1000)})}</p></Show>
 </section>;
}
