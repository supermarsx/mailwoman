import {AsyncPending} from '../../../components/AsyncState.tsx';
import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { hostingApi, type HostingApi, type HostingConnection, type HostingConfig, type HostingSecret } from '../../../api/admin/hosting.ts';
import { useShell } from '../shell/context.ts';
import { sectionPath } from '../shell/router.ts';
import { Feedback } from '../shell/Feedback.tsx';
import { clearAdminError, showAdminError } from '../shell/requestErrorToast.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { HostingEditor } from './ConnectionEditor.tsx';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';
export function HostingConnections(props:{api?:HostingApi;tunnels?:{id:string;name:string;enabled:boolean}[];manage?:(connection:HostingConnection)=>JSX.Element}):JSX.Element {
 const api=props.api??hostingApi;const shell=useShell();const [rows,setRows]=createSignal<HostingConnection[]>([]);const [busy,setBusy]=createSignal(false);const [loaded,setLoaded]=createSignal(false);const [error,setError]=createSignal<string>();const [deleting,setDeleting]=createSignal<HostingConnection>();
 const rest=()=>shell.router.route().rest.slice(1);const creating=()=>rest()[0]==='new';const selected=()=>rows().find(row=>row.id===rest()[0]);const editing=()=>rest()[1]==='edit';const canWrite=()=>shell.can.can('integrations.write');
 const go=(segments:string[]=[]):void=>{void shell.router.navigate(sectionPath('integrations',['hosting',...segments]));};
 async function reload():Promise<void>{setBusy(true);setError(undefined);try{setRows(await api.list());setLoaded(true);clearAdminError('hosting-load');}catch(cause){showAdminError(cause,{key:'hosting-load',retry:reload},setError);}finally{setBusy(false);}}
 async function save(config:HostingConfig,credential:HostingSecret):Promise<void>{const saved=await api.save(config,credential,creating()?undefined:selected()?.etag);setRows(current=>[...current.filter(row=>row.id!==saved.id),saved]);go([saved.id,'edit']);}
 onMount(()=>{void loadCatalog('admin-hosting');void reload();});
 return <section class={css.section} aria-label={t('admin-hosting-title')}><Feedback kind="error" message={error()}/><Show when={busy()}><AsyncPending variant={rows().length ? 'status' : 'table'}/></Show>
 <Show when={rest().length>0} fallback={<><h2>{t('admin-hosting-title')}</h2><p class={css.note}>{t('admin-hosting-description')}</p><div class={ux.pageActions}><Show when={canWrite()}><button class="btn btn--primary" type="button" onClick={()=>go(['new'])}>{t('admin-hosting-new')}</button></Show><button class="btn" type="button" disabled={busy()} onClick={()=>void reload()}>{t('admin-hosting-refresh')}</button></div><Show when={rows().length} fallback={<Show when={loaded()&&!busy()&&!error()}><p>{t('admin-hosting-list-empty')}</p></Show>}><div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('admin-hosting-name')}</th><th>{t('admin-hosting-provider')}</th><th>{t('admin-hosting-url')}</th><th>{t('admin-hosting-enabled')}</th><th>{t('admin-hosting-open')}</th></tr></thead><tbody><For each={rows()}>{(row)=><tr><th scope="row"><button class="btn" type="button" onClick={()=>go([row.id,'edit'])}>{row.name}</button></th><td>{row.provider==='hestia'?'HestiaCP':'Virtualmin (Webmin module)'}</td><td>{row.baseUrl}</td><td>{row.enabled?t('admin-hosting-active'):t('admin-hosting-disabled')}</td><td><button class="btn" type="button" onClick={()=>go([row.id,'manage'])}>{t('admin-hosting-open')}</button></td></tr>}</For></tbody></table></div></Show></>}>
 <Show when={creating()} fallback={<Show when={selected()} keyed fallback={<Show when={loaded()}><p>{t('admin-hosting-missing')}</p><button class="btn" onClick={()=>go()}>{t('admin-hosting-back')}</button></Show>}>{(row)=><Show when={editing()} fallback={props.manage?.(row)}><HostingEditor initial={row} canWrite={canWrite()} canBind={shell.can.can('tunnels.bind')} tunnels={props.tunnels??[]} manageTunnels={()=>void shell.router.navigate(sectionPath('integrations',['ssh']))} save={save} back={()=>go()}/><Show when={canWrite()}><div class={ux.pageActions}><button class="btn btn--danger" type="button" onClick={()=>setDeleting(row)}>{t('admin-hosting-remove')}</button></div></Show></Show>}</Show>}>
 <Show when={canWrite()} fallback={<p>{t('admin-hosting-no-manage')}</p>}><HostingEditor canWrite={canWrite()} canBind={shell.can.can('tunnels.bind')} tunnels={props.tunnels??[]} manageTunnels={()=>void shell.router.navigate(sectionPath('integrations',['ssh']))} save={save} back={()=>go()}/></Show></Show></Show>
 <ConfirmDialog open={!!deleting()} title={t('admin-hosting-remove-title')} body={t('admin-hosting-remove-detail')} typedConfirmation={deleting()?.name??''} confirmLabel={t('admin-hosting-remove')} onCancel={()=>setDeleting(undefined)} onConfirm={async()=>{const row=deleting();if(!row)return;await api.remove(row);setDeleting(undefined);setRows(current=>current.filter(item=>item.id!==row.id));go();}}/>
 </section>;
}
