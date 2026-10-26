import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import { toast } from '../../components/toast/index.ts';
import { IdentityEditor } from './IdentityEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import * as css from './styles.css.ts';
export interface IdentitiesProps { service?:SettingsService }
export function Identities(props:IdentitiesProps):JSX.Element {
 return <PersonalAccountBoundary>{()=> <OwnedIdentities {...props}/>}</PersonalAccountBoundary>;
}
function OwnedIdentities(props:IdentitiesProps):JSX.Element {
 const service=props.service??new SettingsService();const navigation=useEntityNavigation('identities');const [identities,{refetch}]=createResource(()=>service.listIdentities());const [busy,setBusy]=createSignal(false);
 onMount(()=>void loadCatalog('settings'));
 const editor=()=>navigation.route().section==='identities'?navigation.route().editor:undefined;
 const open=(id?:string)=>navigation.navigate({section:'identities',editor:id===undefined?{kind:'new'}:{kind:'edit',id}});
 const back=async():Promise<void>=>{await navigation.navigate({section:'identities'});};
 async function remove(id:string):Promise<void>{setBusy(true);try{await service.deleteIdentity(id);await refetch();}catch{toast.error(t('settings-ident-error-generic'));}finally{setBusy(false);}}
 return <Show when={editor()} keyed fallback={<section class={css.section} aria-label={t('settings-ident-title')}><h2 class={css.heading}>{t('settings-ident-title')}</h2><p class={css.prose}>{t('settings-ident-intro')}</p><Show when={identities.loading}><AsyncPending onRetry={()=>void refetch()}/></Show>
      <ul class={css.list} data-testid="identity-list">
        <For each={identities() ?? []}>
          {(identity) => (
            <li class={css.item}>
              <div class={css.itemMain}>
                <span class={css.itemName}>{isolate(identity.name)}</span>
                <span class={css.meta}>{isolate(identity.email)}</span>
              </div>
              <div class={css.actions}>
                <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void open(identity.id)}>
                  {t('settings-edit')}
                </button>
                <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(identity.id)}>
                  {t('settings-delete')}
                </button>
              </div>
            </li>
          )}
        </For>
      </ul>
<div class={css.actions}><button class={css.button} onClick={()=>void open()}>{t('settings-ident-new')}</button></div></section>}>{route=> <Show when={route.kind==='new'} fallback={<Show when={identities()?.find(item=>item.id===route.id)} keyed fallback={<Show when={!identities.loading}><p>{t('settings-editor-missing')}</p><button class={css.ghost} onClick={()=>void back()}>{t('settings-editor-back')}</button></Show>}>{item=><IdentityEditor initial={item} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/>}</Show>}><IdentityEditor service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/></Show>}</Show>;
}
export default Identities;
