import {createEffect,createSignal,For,onCleanup,Show,untrack,useContext,type JSX} from 'solid-js';
import {loadCatalog,t,formatDateTime} from '../../i18n';
import {loadRetainedDatabases,retireDatabase,type RetainedDatabase,type RetainedDatabaseReport} from '../../api/admin/retainedDatabases.ts';
import {InlineLoading} from '../../components/InlineLoading.tsx';
import {Icon} from '../../components/Icon.tsx';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {ShellContext} from './shell/context.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatBytes} from './Dashboard/format.ts';
import * as css from './admin.css.ts';

export function RetainedDatabases(props:{refreshKey?:string}):JSX.Element {
 void loadCatalog('admin-database');const shell=useContext(ShellContext);
 const canRead=()=>shell?.can.can('maintenance.read')===true;
 const canDelete=()=>shell?.can.can('maintenance.run')===true;
 const [report,setReport]=createSignal<RetainedDatabaseReport>(),[loading,setLoading]=createSignal(false);
 const [failure,setFailure]=createSignal<string|null>(null),[success,setSuccess]=createSignal<string|null>(null);
 const [selected,setSelected]=createSignal<{row:RetainedDatabase;active:string}|undefined>();
 let disposed=false,requested=false,lastKey:string|undefined,request:AbortController|undefined;
 async function reload():Promise<void> {
  if(disposed||loading()||!canRead())return;setLoading(true);setFailure(null);request=new AbortController();
  try {const value=await loadRetainedDatabases(request.signal);if(!disposed)setReport(value);}
  catch(error){if(!disposed)setFailure(describeAdminError(error));}finally{if(!disposed)setLoading(false);}
 }
 createEffect(()=>{const key=props.refreshKey;if(canRead()&&(!requested||key!==lastKey)){requested=true;lastKey=key;untrack(()=>void reload());}});
 onCleanup(()=>{disposed=true;request?.abort();});
 const blocked=(reason:string|null)=>t(reason==='maintenance-active'?'admin-db-retained-busy':reason==='sqlite-sidecars'?'admin-db-retained-sidecars':'admin-db-retained-unavailable');
 async function remove():Promise<void> {
  const target=selected();if(!target||!canDelete())return;
  const result=await retireDatabase({generationId:target.row.generationId,expectedActiveGeneration:target.active,fileIdentity:target.row.fileIdentity,confirmation:target.row.fileName});
  if(disposed)return;setSelected(undefined);setSuccess(t('admin-db-retained-deleted',{file:result.fileName}));await reload();
 }
 return <Show when={canRead()}><section class={css.card} aria-label={t('admin-db-retained-title')}>
  <h3 class={css.heading}>{t('admin-db-retained-title')}</h3><p class={css.note}>{t('admin-db-retained-help')}</p>
  <Feedback kind="error" as="toast" message={failure()}/><Feedback kind="success" as="toast" message={success()}/>
  <Show when={loading()}><InlineLoading/></Show>
  <Show when={report()}>{value=><Show when={value().retained.length} fallback={<p class={css.note}>{t('admin-db-retained-empty')}</p>}>
   <div class={css.tableWrap}><table class={css.table} aria-label={t('admin-db-retained-title')}><thead><tr><th scope="col">{t('admin-db-file')}</th><th scope="col">{t('admin-db-size')}</th><th scope="col">{t('admin-db-retained-modified')}</th><th scope="col">{t('admin-db-retained-actions')}</th></tr></thead><tbody><For each={value().retained}>{row=><tr>
    <th scope="row"><code>{row.fileName}</code><div class={css.note}>{row.path}</div><Show when={!row.deletable}><p class={css.note}>{blocked(row.blockedReason)}</p></Show></th>
    <td>{formatBytes(row.bytes)}</td><td>{row.modifiedAt===null?t('admin-shell-not-available'):formatDateTime(row.modifiedAt*1000)}</td>
    <td><button type="button" class={`btn btn--ghost ${css.compactIconAction}`} aria-label={t('admin-db-retained-delete',{file:row.fileName})} data-tooltip={t('admin-db-retained-delete',{file:row.fileName})} disabled={loading()||!row.deletable||!canDelete()||!value().activeGeneration} onClick={()=>setSelected({row,active:value().activeGeneration!})}><Icon name="delete"/></button></td>
   </tr>}</For></tbody></table></div>
  </Show>}</Show>
  <ConfirmDialog open={selected()!==undefined} title={t('admin-db-retained-confirm-title')} body={t('admin-db-retained-confirm-body',{file:selected()?.row.fileName??'',size:formatBytes(selected()?.row.bytes??0)})} confirmLabel={t('admin-db-retained-confirm')} {...(selected()?{typedConfirmation:selected()!.row.fileName}:{})} onCancel={()=>setSelected(undefined)} onConfirm={remove}/>
 </section></Show>;
}
