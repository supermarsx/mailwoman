import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
import { InlineLoading } from '../../components/InlineLoading.tsx';
import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import { toast } from '../../components/toast/index.ts';
import { SignatureEditor } from './SignatureEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import type {Signature} from './types.ts';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
export interface SignaturesProps { service?:SettingsService }
export function Signatures(props:SignaturesProps):JSX.Element {
 return <PersonalAccountBoundary>{()=> <OwnedSignatures {...props}/>}</PersonalAccountBoundary>;
}
function OwnedSignatures(props:SignaturesProps):JSX.Element {
 const service=props.service??new SettingsService();const navigation=useEntityNavigation('signatures');const [signatures,{refetch}]=createResource(()=>service.listSignatures());const [busy,setBusy]=createSignal(false);const [variant,setVariant]=createSignal<Signature>();
 onMount(()=>void loadCatalog('settings'));
 const editor=()=>navigation.route().section==='signatures'?navigation.route().editor:undefined;
 const open=(id?:string,source?:Signature)=>{if(source){const {rule:_rule,...content}=source;setVariant({...structuredClone(content),name:'',isDefault:false});}else setVariant(undefined);return navigation.navigate({section:'signatures',editor:id===undefined?{kind:'new'}:{kind:'edit',id}});};
 const back=async():Promise<void>=>{await navigation.navigate({section:'signatures'});};
 async function remove(id:string):Promise<void>{setBusy(true);try{await service.deleteSignature(id);await refetch();}catch{toast.error(t('settings-sig-error-generic'));}finally{setBusy(false);}}
 return <Show when={editor()} keyed fallback={<SettingsCard id="signatures-card" wide title={t('settings-sig-title')} description={t('settings-sig-intro')}><Show when={signatures.loading}><InlineLoading onRetry={()=>void refetch()}/></Show>
      <ul class={css.list} data-testid="signature-list">
        <For each={signatures() ?? []}>
          {(sig) => (
            <li class={css.item}>
              <div class={css.itemMain}>
                <span class={css.itemName}>
                  {sig.name}
                  <Show when={sig.isDefault}>
                    {' '}
                    <span class={css.badge}>{t('settings-sig-default')}</span>
                  </Show>
                </span>
                <span class={css.meta}>{sig.body.slice(0, 80)}</span>
              </div>
              <div class={css.actions}>
                <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void open(sig.name)}>
                  {t('settings-edit')}
                </button>
                <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void open(undefined,sig)}>{t('settings-sig-create-variant')}</button>
                <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(sig.name)}>
                  {t('settings-delete')}
                </button>
              </div>
            </li>
          )}
        </For>
      </ul>
<div class={css.actions}><button class={css.button} onClick={()=>void open()}>{t('settings-sig-new')}</button></div></SettingsCard>}>{route=> <Show when={route.kind==='new'} fallback={<Show when={signatures()?.find(item=>item.name===route.id)} keyed fallback={<Show when={!signatures.loading}><p>{t('settings-editor-missing')}</p><button class={css.ghost} onClick={()=>void back()}>{t('settings-editor-back')}</button></Show>}>{item=><SignatureEditor initial={item} reservedNames={(signatures()??[]).map(x=>x.name)} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/>}</Show>}><SignatureEditor {...(variant()?{seed:variant()!}:{})} reservedNames={signatures()?.map(item=>item.name)??[]} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/></Show>}</Show>;
}
export default Signatures;
