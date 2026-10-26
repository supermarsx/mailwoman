import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {sectionPath} from './shell/router.ts';
import { Show, onMount, createSignal, type JSX } from 'solid-js';
import {EgressDestinations} from './EgressDestinations.tsx';
import { loadCatalog, t } from '../../i18n';
import { AdminEgress, type EgressAdminApi } from './Egress.tsx';
import { useShell } from './shell/context.ts';
export function AdminOutbound(props: { api?: EgressAdminApi }): JSX.Element {
  const [dirty,setDirty]=createSignal(false);useDirtyGuard(dirty);
  const shell = useShell(); onMount(() => void loadCatalog('privacy'));
  const go=(editor?:string):void=>{void shell.router.navigate(sectionPath('egress',['destinations',...(editor?[editor]:[])]));};
  return <Show when={shell.router.route().rest[0]==='destinations'} fallback={<section><h2>{t('privacy-outbound-title')}</h2><p>{t('privacy-fail-closed')}</p><button class="btn" onClick={()=>go()}>{t('admin-egress-destinations-title')}</button><AdminEgress editor={shell.router.route().rest[0]} onNavigate={id=>{void shell.router.navigate(sectionPath('egress',id?[id]:[]));}} onDirty={setDirty} {...(props.api ? { api: props.api } : {})} readOnly={!shell.can.can('egress.write')} /></section>}>
    <EgressDestinations {...(shell.router.route().rest[1]?{editor:shell.router.route().rest[1]}:{})} readOnly={!shell.can.can('egress.write')} onNavigate={go} onBack={()=>{void shell.router.navigate(sectionPath('egress'));}}/>
  </Show>;
}
