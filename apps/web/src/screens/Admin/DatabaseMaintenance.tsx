import {createSignal,onMount,onCleanup,Show,type JSX} from 'solid-js';
import {useCatalog,t} from '../../i18n';
import {loadDatabaseMaintenance,type DatabaseMaintenanceReport} from '../../api/admin/databaseMaintenance.ts';
import {OnlineDatabaseConversion} from './OnlineDatabaseConversion.tsx';
import {BackgroundDatabaseMaintenance} from './BackgroundDatabaseMaintenance.tsx';
import {RetainedDatabases} from './RetainedDatabases.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {InlineLoading} from '../../components/InlineLoading.tsx';
import * as css from './admin.css.ts';

export function DatabaseMaintenance():JSX.Element{
 const catalog=useCatalog('admin-database');
 const [report,setReport]=createSignal<DatabaseMaintenanceReport>();
 const [retainedRefresh,setRetainedRefresh]=createSignal(0);
 const [error,setError]=createSignal<string|null>(null);
 let disposed=false,loading=false,request:AbortController|undefined;
 async function reload():Promise<void>{
  if(disposed||loading)return;
  loading=true;request=new AbortController();
  try{const result=await loadDatabaseMaintenance(request.signal);if(!disposed){setReport(result);setError(null);}}
  catch(cause){if(!disposed)setError(describeAdminError(cause));}
  finally{loading=false;}
 }
 onMount(()=>{
  void reload();
  const timer=setInterval(()=>{if(document.visibilityState!=='hidden')void reload();},10000);
  onCleanup(()=>{disposed=true;request?.abort();clearInterval(timer);});
 });
 return <Show when={catalog()} fallback={<InlineLoading/>}><section class={css.section} aria-label={t('admin-db-maintenance-title')}>
  <h2 class={css.heading}>{t('admin-db-maintenance-title')}</h2>
  <Feedback kind="error" as="toast" message={error()}/>
  <Show when={report()} fallback={<Show when={!error()}><InlineLoading/></Show>}>{value=><>
   <Show when={value().backend==='sqlite'}><OnlineDatabaseConversion onCompleted={()=>setRetainedRefresh(value=>value+1)}/><RetainedDatabases refreshKey={String(retainedRefresh())}/></Show>
   <BackgroundDatabaseMaintenance report={value()} onRefresh={()=>void reload()}/>
  </>}</Show>
 </section></Show>;
}
