import { Show, createSignal, createEffect, on, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { hostingApi } from '../../api/admin/hosting.ts';
import { sshTunnelApi, type SshTunnel } from '../../api/admin/ssh-tunnels.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { Integrations } from './Integrations.tsx';
import { HostingConnections } from './Hosting/index.tsx';
import { HostingManage } from './Hosting/Manage.tsx';
import { SshTunnels } from './SshTunnels/index.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';

/** One integration navigation surface; entity editors live on dedicated routes. */
export function IntegrationsRouter():JSX.Element {
 const shell=useShell();const [tunnels,setTunnels]=createSignal<SshTunnel[]>([]);const [error,setError]=createSignal<string>();
 onMount(()=>{void loadCatalog('admin-hosting');void loadCatalog('admin-ssh');});
 createEffect(on(()=>[shell.can.can('tunnels.read'),shell.router.route().rest[0]] as const,([allowed,route])=>{if(allowed&&route==='hosting'){setError(undefined);void sshTunnelApi.list().then(setTunnels).catch(cause=>setError(describeAdminError(cause)));}}));
 const selected=()=>shell.router.route().rest[0]??'overview';
 const tabs=()=>[['overview','admin-integrations-title'],['directory','admin-integrations-ldap'],['webhooks','admin-integrations-tab-webhooks'],['keys','admin-integrations-keys'],['hosting','admin-hosting-title'],...(shell.can.can('tunnels.read')?[['ssh','admin-ssh-title']]:[])].map(([id,label])=>({id:id!,label:t(label!),path:sectionPath('integrations',id==='overview'?[]:[id!])}));
 return <><SectionTabs label={t('admin-integrations-title')} selected={selected()} items={tabs()} navigate={path=>void shell.router.navigate(path)}/><Show when={selected()==='ssh'} fallback={<Show when={selected()==='hosting'} fallback={<Integrations category={selected()}/>}><Feedback kind="error" message={error()}/><HostingConnections tunnels={tunnels()} manage={connection=><HostingManage connection={connection} api={hostingApi}/>}/></Show>}><SshTunnels/></Show></>;
}
