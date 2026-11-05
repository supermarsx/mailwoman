import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService, SettingsError } from './service.ts';
import { inheritedFields, type Identity } from './types.ts';
import { toast } from '../../components/toast/index.ts';
import { IdentityEditor } from './IdentityEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import { createConfirm } from '../../components/confirm.tsx';
import * as css from './styles.css.ts';
export interface IdentitiesProps { service?:SettingsService }
export function Identities(props:IdentitiesProps):JSX.Element {
 return <PersonalAccountBoundary>{()=> <OwnedIdentities {...props}/>}</PersonalAccountBoundary>;
}
function OwnedIdentities(props:IdentitiesProps):JSX.Element {
 const service=props.service??new SettingsService();const navigation=useEntityNavigation('identities');const [identities,{refetch}]=createResource(()=>service.listIdentities());const [busy,setBusy]=createSignal(false);const removal=createConfirm();const [variantBase,setVariantBase]=createSignal<Identity|undefined>();const [choosing,setChoosing]=createSignal<string>();
 onMount(()=>void loadCatalog('settings'));
 const editor=()=>navigation.route().section==='identities'?navigation.route().editor:undefined;
 const open=(id?:string,base?:Identity)=>{setVariantBase(base);return navigation.navigate({section:'identities',editor:id===undefined?{kind:'new'}:{kind:'edit',id}});};
 // Bases in server order, each followed by its variants (indented in the list). A variant whose base is gone is shown as a base.
 const grouped=():{identity:Identity;variant:boolean}[]=>{const all=identities()??[];const ids=new Set(all.map(item=>item.id));const out:{identity:Identity;variant:boolean}[]=[];for(const base of all.filter(item=>!item.variantOf||!ids.has(item.variantOf))){out.push({identity:base,variant:false});for(const child of all.filter(item=>item.variantOf===base.id))out.push({identity:child,variant:true});}return out;};
 const fieldLabel={replyTo:()=>t('settings-ident-field-replyTo'),autoBcc:()=>t('settings-ident-field-autoBcc'),organization:()=>t('settings-ident-field-organization'),pgpKeyId:()=>t('settings-ident-field-pgpKeyId'),signatureName:()=>t('settings-ident-field-signatureName')};
 const variantsOf=(id:string)=>(identities()??[]).filter(item=>item.variantOf===id);
 const baseOf=(item:Identity)=>(identities()??[]).find(other=>other.id===item.variantOf);
 const inheritText=(item:Identity)=>{const base=baseOf(item);if(!base)return '';const fields=inheritedFields(item).map(field=>fieldLabel[field]());return fields.length===0?'':t('settings-ident-inherits',{base:isolate(base.label??base.name),fields:fields.join(', ')});};
 async function makeDefault(identity:Identity):Promise<void>{setBusy(true);try{await service.makeIdentityDefault(identity.id);await refetch();toast.success(t('settings-ident-default-set',{email:identity.email}));}catch{toast.error(t('settings-ident-error-generic'));}finally{setBusy(false);}}
 const back=async():Promise<void>=>{setVariantBase(undefined);await navigation.navigate({section:'identities'});};
 async function perform(id:string,variants?:'delete'|'promote'):Promise<void>{setBusy(true);try{await service.deleteIdentity(id,variants);setChoosing(undefined);await refetch();}catch(cause){if(cause instanceof SettingsError&&cause.status===409)setChoosing(id);else toast.error(t('settings-ident-error-generic'));await refetch();}finally{setBusy(false);}}
 async function remove(identity:Identity):Promise<void>{
  // A base with variants asks what happens to them (delete or keep) instead of confirming blindly.
  if(variantsOf(identity.id).length>0){setChoosing(identity.id);return;}
  if(!await removal.ask({title:t('settings-ident-delete-title',{email:identity.email}),body:t(identity.isDefault?'settings-ident-delete-default-body':'settings-ident-delete-body'),confirmLabel:t('settings-delete'),testId:'identity-delete-confirm'}))return;await perform(identity.id);}
 return <Show when={editor()} keyed fallback={<section class={css.section} aria-label={t('settings-ident-title')}><h2 class={css.heading}>{t('settings-ident-title')}</h2><p class={css.prose}>{t('settings-ident-intro')}</p><Show when={identities.loading}><AsyncPending onRetry={()=>void refetch()}/></Show>
      <ul class={css.list} data-testid="identity-list">
        <For each={grouped()}>
          {(entry) => (
            <li class={css.item} data-testid={entry.variant?'identity-variant':'identity-base'} style={entry.variant?{'margin-inline-start':'1.5rem'}:undefined}>
              <div class={css.itemMain}>
                <span class={css.itemName}>{isolate(entry.identity.name)}<Show when={entry.identity.label}>{' '}<span class={css.badge} data-testid="identity-label">{isolate(entry.identity.label!)}</span></Show><Show when={entry.identity.isDefault}>{' '}<span class={css.badge} data-testid="identity-default-badge">{t('settings-ident-default-badge')}</span></Show></span>
                <span class={css.meta}>{isolate(entry.identity.email)}</span>
                <Show when={entry.variant&&inheritText(entry.identity)}><span class={css.meta} data-testid="identity-inherits">{inheritText(entry.identity)}</span></Show>
                <Show when={!entry.variant&&variantsOf(entry.identity.id).length>0}><span class={css.meta}>{t('settings-ident-variant-count',{count:variantsOf(entry.identity.id).length})}</span></Show>
              </div>
              <div class={css.actions}>
                <Show when={!entry.identity.isDefault}><button type="button" class={css.ghost} disabled={busy()} onClick={()=>void makeDefault(entry.identity)}>{t('settings-ident-make-default')}</button></Show>
                <Show when={!entry.variant}><button type="button" class={css.ghost} disabled={busy()} onClick={()=>void open(undefined,entry.identity)}>{t('settings-ident-create-variant')}</button></Show>
                <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void open(entry.identity.id)}>
                  {t('settings-edit')}
                </button>
                <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(entry.identity)}>
                  {t('settings-delete')}
                </button>
              </div>
              <Show when={choosing()===entry.identity.id}>
                <div role="group" aria-label={t('settings-ident-variants-choice-title',{email:entry.identity.email})} data-testid="identity-variants-choice" style={{'flex-basis':'100%'}}>
                  <p class={css.prose}>{t('settings-ident-variants-choice-body',{count:variantsOf(entry.identity.id).length})}</p>
                  <div class={css.actions}>
                    <button type="button" class={css.danger} disabled={busy()} data-testid="identity-delete-with-variants" onClick={()=>void perform(entry.identity.id,'delete')}>{t('settings-ident-delete-with-variants')}</button>
                    <button type="button" class={css.ghost} disabled={busy()} data-testid="identity-delete-keep-variants" onClick={()=>void perform(entry.identity.id,'promote')}>{t('settings-ident-delete-keep-variants')}</button>
                    <button type="button" class={css.ghost} disabled={busy()} onClick={()=>setChoosing(undefined)}>{t('settings-cancel')}</button>
                  </div>
                </div>
              </Show>
            </li>
          )}
        </For>
      </ul><Show when={!identities.loading&&(identities()?.length??0)===0}><p class={css.prose} data-testid="identity-empty">{t('settings-ident-empty')}</p></Show><removal.Dialog/>
<div class={css.actions}><button class={css.button} onClick={()=>void open()}>{t('settings-ident-new')}</button></div></section>}>{route=> <Show when={route.kind==='new'} fallback={<Show when={identities()?.find(item=>item.id===route.id)} keyed fallback={<Show when={!identities.loading}><p>{t('settings-editor-missing')}</p><button class={css.ghost} onClick={()=>void back()}>{t('settings-editor-back')}</button></Show>}>{item=><IdentityEditor initial={item} others={identities() ?? []} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/>}</Show>}><IdentityEditor {...(variantBase()?{draftVariantOf:variantBase()!}:{})} others={identities() ?? []} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/></Show>}</Show>;
}
export default Identities;
