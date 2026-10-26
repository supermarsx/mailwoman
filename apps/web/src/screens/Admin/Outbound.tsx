import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {sectionPath} from './shell/router.ts';
import { Show, onMount, createSignal, type JSX } from 'solid-js';
import {EgressDestinations} from './EgressDestinations.tsx';
import { loadCatalog, t } from '../../i18n';
import { AdminEgress, type EgressAdminApi } from './Egress.tsx';
import { useShell } from './shell/context.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import * as ux from './shell/ux.css.ts';
export function AdminOutbound(props: { api?: EgressAdminApi }): JSX.Element {
  const [dirty,setDirty]=createSignal(false);useDirtyGuard(dirty);
  const shell = useShell(); onMount(() => void loadCatalog('privacy'));
  const go=(editor?:string):void=>{void shell.router.navigate(sectionPath('egress',['destinations',...(editor?[editor]:[])]));};
  return <div class={ux.page}>
    <header class={ux.pageTitleBlock}><h2 class={ux.pageTitle}>{t('privacy-outbound-title')}</h2><p class={ux.pageDescription}>{t('privacy-fail-closed')}</p></header>
    <SectionTabs label={t('privacy-outbound-title')} selected={shell.router.route().rest[0]==='destinations'?'destinations':'proxies'} items={[
      {id:'proxies',label:t('admin-egress-heading'),path:sectionPath('egress')},
      {id:'destinations',label:t('admin-egress-destinations-title'),path:sectionPath('egress',['destinations'])},
    ]} navigate={path=>{void shell.router.navigate(path);}}/>
    <Show when={shell.router.route().rest[0]==='destinations'} fallback={<AdminEgress editor={shell.router.route().rest[0]} onNavigate={id=>{void shell.router.navigate(sectionPath('egress',id?[id]:[]));}} onDirty={setDirty} {...(props.api ? { api: props.api } : {})} readOnly={!shell.can.can('egress.write')} />}>
    <EgressDestinations {...(shell.router.route().rest[1]?{editor:shell.router.route().rest[1]}:{})} readOnly={!shell.can.can('egress.write')} onNavigate={go} onBack={()=>{void shell.router.navigate(sectionPath('egress'));}}/>
  </Show></div>;
}
