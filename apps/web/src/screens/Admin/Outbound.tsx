import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import {sectionPath} from './shell/router.ts';
import { onMount, createSignal, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { AdminEgress, type EgressAdminApi } from './Egress.tsx';
import { useShell } from './shell/context.ts';
export function AdminOutbound(props: { api?: EgressAdminApi }): JSX.Element {
  const [dirty,setDirty]=createSignal(false);useDirtyGuard(dirty);
  const shell = useShell(); onMount(() => void loadCatalog('privacy'));
  return <section><h2>{t('privacy-outbound-title')}</h2><p>{t('privacy-fail-closed')}</p><AdminEgress editor={shell.router.route().rest[0]} onNavigate={id=>{void shell.router.navigate(sectionPath('egress',id?[id]:[]));}} onDirty={setDirty} {...(props.api ? { api: props.api } : {})} readOnly={!shell.can.can('egress.write')} /></section>;
}
