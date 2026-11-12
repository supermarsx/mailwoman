import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
import { InlineLoading } from '../../components/InlineLoading.tsx';
import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import { toast } from '../../components/toast/index.ts';
import { SignatureEditor } from './SignatureEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import type {Signature} from './types.ts';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as table from './linkedAccounts.css.ts';
import * as entity from './entityTable.css.ts';
import * as ux from '../Admin/shell/ux.css.ts';
import {Icon} from '../../components/Icon.tsx';
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
      <div class={table.header}><button type="button" class={`${css.button} ${table.addButton}`} disabled={busy()} onClick={()=>void open()}><Icon name="plus"/>{t('settings-sig-new')}</button></div>
      <ul class={table.table} aria-label={t('settings-sig-title')} data-testid="signature-list">
        <For each={signatures() ?? []}>
          {(sig) => (
            <li class={entity.row}>
              <span class={entity.avatar}><Icon name="signature"/></span>
              <div class={table.who}>
                <span class={entity.name}>
                  <span class={entity.label}>{isolate(sig.name)}</span>
                  <Show when={sig.isDefault}>
                    {' '}
                    <span class={css.badge}>{t('settings-sig-default')}</span>
                  </Show>
                </span>
                <span class={table.address}>{isolate((sig.text??sig.body).slice(0, 160))}</span>
              </div>
              <div class={table.actions}>
                <button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-edit')} data-tooltip={t('settings-edit')} onClick={()=>void open(sig.name)}><Icon name="edit"/></button>
                <button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-sig-create-variant')} data-tooltip={t('settings-sig-create-variant')} onClick={()=>void open(undefined,sig)}><Icon name="clone"/></button>
                <button type="button" class={`btn btn--danger ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-delete')} data-tooltip={t('settings-delete')} onClick={()=>void remove(sig.name)}><Icon name="delete"/></button>
              </div>
            </li>
          )}
        </For>
      </ul>
</SettingsCard>}>{route=> <Show when={route.kind==='new'} fallback={<Show when={signatures()?.find(item=>item.name===route.id)} keyed fallback={<Show when={!signatures.loading}><p>{t('settings-editor-missing')}</p><button class={css.ghost} onClick={()=>void back()}>{t('settings-editor-back')}</button></Show>}>{item=><SignatureEditor initial={item} reservedNames={(signatures()??[]).map(x=>x.name)} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/>}</Show>}><SignatureEditor {...(variant()?{seed:variant()!}:{})} reservedNames={signatures()?.map(item=>item.name)??[]} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/></Show>}</Show>;
}
export default Signatures;
