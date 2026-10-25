import { Show, createSignal, createEffect, on, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { hostingApi } from '../../api/admin/hosting.ts';
import { sshTunnelApi, type SshTunnel } from '../../api/admin/ssh-tunnels.ts';
import { Integrations } from './Integrations.tsx';
import { HostingConnections } from './Hosting/index.tsx';
import { HostingManage } from './Hosting/Manage.tsx';
import { SshTunnels } from './SshTunnels/index.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import * as ux from './shell/ux.css.ts';
/** One integration navigation surface; entity editors live on dedicated routes. */
export function IntegrationsRouter():JSX.Element {
 const shell=useShell();const [tunnels,setTunnels]=createSignal<SshTunnel[]>([]);const [error,setError]=createSignal<string>();
 onMount(()=>{void loadCatalog('admin-hosting');void loadCatalog('admin-ssh');});
 createEffect(on(()=>[shell.can.can('tunnels.read'),shell.router.route().rest[0]] as const,([allowed,route])=>{if(allowed&&route==='hosting'){setError(undefined);void sshTunnelApi.list().then(setTunnels).catch(cause=>setError(describeAdminError(cause)));}}));
 return <Show when={shell.router.route().rest[0]==='ssh'} fallback={<Show when={shell.router.route().rest[0]==='hosting'} fallback={<><div class={ux.pageActions}><button class="btn" type="button" onClick={()=>void shell.router.navigate(sectionPath('integrations',['hosting']))}>{t('admin-hosting-title')}</button><Show when={shell.can.can('tunnels.read')}><button class="btn" type="button" onClick={()=>void shell.router.navigate(sectionPath('integrations',['ssh']))}>{t('admin-ssh-title')}</button></Show></div><Integrations/></>}><Feedback kind="error" message={error()}/><HostingConnections tunnels={tunnels()} manage={connection=><HostingManage connection={connection} api={hostingApi}/>}/></Show>}><SshTunnels/></Show>;
}
