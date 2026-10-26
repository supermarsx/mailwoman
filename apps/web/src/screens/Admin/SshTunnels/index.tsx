import {AsyncPending} from '../../../components/AsyncState.tsx';
import { For, Show, createSignal, createEffect, on, onMount, type JSX } from 'solid-js';
import { loadCatalog,t } from '../../../i18n';
import { sshTunnelApi,type SshCredential,type SshTunnel,type SshTunnelApi,type SshTunnelConfig } from '../../../api/admin/ssh-tunnels.ts';
import { useShell } from '../shell/context.ts';
import { sectionPath } from '../shell/router.ts';
import { Feedback,describeAdminError } from '../shell/Feedback.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { SshTunnelEditor } from './Editor.tsx';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';
export function SshTunnels(props:{api?:SshTunnelApi}):JSX.Element {
 const api=props.api??sshTunnelApi;const shell=useShell();const [rows,setRows]=createSignal<SshTunnel[]>([]);const [busy,setBusy]=createSignal(false);const [loaded,setLoaded]=createSignal(false);const [error,setError]=createSignal<string>();const [deleting,setDeleting]=createSignal<SshTunnel>();
 const rest=()=>shell.router.route().rest.slice(1);const creating=()=>rest()[0]==='new';const selected=()=>rows().find(row=>row.id===rest()[0]);const canRead=()=>shell.can.can('tunnels.read');const canWrite=()=>shell.can.can('tunnels.write');const canTest=()=>shell.can.can('tunnels.test');
 const go=(segments:string[]=[]):void=>{void shell.router.navigate(sectionPath('integrations',['ssh',...segments]));};
 async function reload():Promise<void>{if(!canRead())return;setBusy(true);setError(undefined);try{setRows(await api.list());setLoaded(true);}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
 async function save(config:SshTunnelConfig,credential:SshCredential,confirmed?:boolean):Promise<void>{const saved=await api.save(config,credential,creating()?undefined:selected()?.etag,confirmed);setRows(current=>[...current.filter(row=>row.id!==saved.id),saved]);go([saved.id,'edit']);}
 onMount(()=>{void loadCatalog('admin-ssh');});
 createEffect(on(canRead,allowed=>{if(allowed)void reload();}));
 return <section class={css.section} aria-label={t('admin-ssh-title')}><Feedback kind="error" message={error()}/><Show when={busy()}><AsyncPending variant={rows().length ? 'status' : 'table'}/></Show><Show when={canRead()} fallback={<p>{t('admin-ssh-no-manage')}</p>}>
 <Show when={rest().length} fallback={<><h2>{t('admin-ssh-title')}</h2><p class={css.note}>{t('admin-ssh-description')}</p><div class={ux.pageActions}><Show when={canWrite()}><button class="btn btn--primary" onClick={()=>go(['new'])}>{t('admin-ssh-new')}</button></Show><button class="btn" disabled={busy()} onClick={()=>void reload()}>{t('admin-ssh-refresh')}</button></div><Show when={rows().length} fallback={<Show when={loaded()&&!busy()&&!error()}><p>{t('admin-ssh-empty')}</p></Show>}><div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('admin-ssh-name')}</th><th>{t('admin-ssh-host')}</th><th>{t('admin-ssh-status')}</th></tr></thead><tbody><For each={rows()}>{row=><tr><th scope="row"><button class="btn" onClick={()=>go([row.id,'edit'])}>{row.name}</button></th><td>{row.host}:{row.port}</td><td>{row.enabled?t('admin-ssh-active'):t('admin-ssh-disabled')}</td></tr>}</For></tbody></table></div></Show></>}>
 <Show when={creating()} fallback={<Show when={selected()} keyed fallback={<Show when={loaded()}><p>{t('admin-ssh-missing')}</p><button class="btn" onClick={()=>go()}>{t('admin-ssh-back')}</button></Show>}>{row=><><SshTunnelEditor initial={row} api={api} canWrite={canWrite()} canTest={canTest()} save={save} back={()=>go()}/><Show when={canWrite()}><div class={ux.pageActions}><button class="btn btn--danger" onClick={()=>setDeleting(row)}>{t('admin-ssh-remove')}</button></div></Show></>}</Show>}><Show when={canWrite()} fallback={<p>{t('admin-ssh-no-manage')}</p>}><SshTunnelEditor api={api} canWrite={canWrite()} canTest={canTest()} save={save} back={()=>go()}/></Show></Show></Show></Show>
 <ConfirmDialog open={!!deleting()} title={t('admin-ssh-remove-title')} body={t('admin-ssh-remove-detail')} typedConfirmation={deleting()?.name??''} confirmLabel={t('admin-ssh-remove')} onCancel={()=>setDeleting(undefined)} onConfirm={async()=>{const row=deleting();if(!row)return;await api.remove(row);setRows(current=>current.filter(item=>item.id!==row.id));setDeleting(undefined);go();}}/>
 </section>;
}
