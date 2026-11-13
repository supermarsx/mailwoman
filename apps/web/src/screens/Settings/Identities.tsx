import { PersonalAccountBoundary } from './PersonalAccountBoundary.tsx';
import { InlineLoading } from '../../components/InlineLoading.tsx';
import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService, SettingsError } from './service.ts';
import { inheritedFields, type Identity } from './types.ts';
import { toast } from '../../components/toast/index.ts';
import { IdentityEditor } from './IdentityEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import { createConfirm } from '../../components/confirm.tsx';
import * as css from './styles.css.ts';
import * as table from './linkedAccounts.css.ts';
import * as entity from './entityTable.css.ts';
import * as ux from '../Admin/shell/ux.css.ts';
import {Icon} from '../../components/Icon.tsx';
import { RectificationBanner } from './RectificationBanner.tsx';
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
 return <><RectificationBanner/><Show when={editor()} keyed fallback={<section id="identities-card" class={css.section} aria-labelledby="identities-card-title" aria-busy={busy()}>
      <div class={table.header}><div class={table.headerText}><h2 id="identities-card-title" class={css.heading}>{t('settings-ident-title')}</h2><p class={css.meta}>{t('settings-ident-intro')}</p></div><button type="button" class={`${css.button} ${table.addButton}`} disabled={busy()} onClick={()=>void open()}><Icon name="plus"/>{t('settings-ident-new')}</button></div>
      <Show when={identities.loading}><InlineLoading onRetry={()=>void refetch()}/></Show>
      <ul class={table.table} aria-label={t('settings-ident-title')} data-testid="identity-list">
        <For each={grouped()}>
          {(entry) => (
            <li class={`${entity.row} ${entry.variant?entity.variant:''}`} data-testid={entry.variant?'identity-variant':'identity-base'}>
              <span class={entity.avatar}><Icon name={entry.variant?'clone':'person'}/></span>
              <div class={table.who}>
                <span class={entity.name}><span class={entity.label}>{isolate(entry.identity.name)}</span><Show when={entry.identity.label}><span class={css.badge} data-testid="identity-label">{isolate(entry.identity.label!)}</span></Show><Show when={entry.identity.isDefault}><span class={css.badge} data-testid="identity-default-badge">{t('settings-ident-default-badge')}</span></Show></span>
                <span class={table.address} title={entry.identity.email}>{isolate(entry.identity.email)}</span>
                <Show when={entry.variant&&inheritText(entry.identity)}><span class={entity.detail} data-testid="identity-inherits">{inheritText(entry.identity)}</span></Show>
                <Show when={!entry.variant&&variantsOf(entry.identity.id).length>0}><span class={entity.detail}>{t('settings-ident-variant-count',{count:variantsOf(entry.identity.id).length})}</span></Show>
              </div>
              <div class={table.actions}>
                <button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={busy()||entry.identity.isDefault===true} aria-pressed={entry.identity.isDefault===true} aria-label={t(entry.identity.isDefault?'settings-ident-default-badge':'settings-ident-make-default')} data-tooltip={t(entry.identity.isDefault?'settings-ident-default-badge':'settings-ident-make-default')} onClick={()=>void makeDefault(entry.identity)}><Icon name={entry.identity.isDefault?'star':'starOutline'}/></button>
                <Show when={!entry.variant}><button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-ident-create-variant')} data-tooltip={t('settings-ident-create-variant')} onClick={()=>void open(undefined,entry.identity)}><Icon name="clone"/></button></Show>
                <button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-edit')} data-tooltip={t('settings-edit')} onClick={()=>void open(entry.identity.id)}><Icon name="edit"/></button>
                <button type="button" class={`btn btn--danger ${ux.iconAction}`} disabled={busy()} aria-label={t('settings-delete')} data-tooltip={t('settings-delete')} onClick={()=>void remove(entry.identity)}><Icon name="delete"/></button>
              </div>
              <Show when={choosing()===entry.identity.id}>
                <div class={entity.choice} role="group" aria-label={t('settings-ident-variants-choice-title',{email:entry.identity.email})} data-testid="identity-variants-choice">
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
</section>}>{route=> <Show when={route.kind==='new'} fallback={<Show when={identities()?.find(item=>item.id===route.id)} keyed fallback={<Show when={!identities.loading}><p>{t('settings-editor-missing')}</p><button class={css.ghost} onClick={()=>void back()}>{t('settings-editor-back')}</button></Show>}>{item=><IdentityEditor initial={item} others={identities() ?? []} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/>}</Show>}><IdentityEditor {...(variantBase()?{draftVariantOf:variantBase()!}:{})} others={identities() ?? []} service={service} onSaved={async()=>{await refetch();await back();}} onCancel={back}/></Show>}</Show></>;
}
export default Identities;
