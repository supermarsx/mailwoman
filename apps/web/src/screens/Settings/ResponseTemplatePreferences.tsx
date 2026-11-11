import {createEffect,createSignal,createUniqueId,Show,on,onCleanup,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {TemplateLibraryEditor} from '../../components/compose/TemplateLibraryEditor.tsx';
import {validResponseTemplates,type ResponseTemplate} from '../../components/compose/responseTemplateModel.ts';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import {toast} from '../../components/toast/index.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {useSettingsNavigation} from './navigation.tsx';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {ControlRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';

export function ResponseTemplatePreferences(props: { onUse?: (row: ResponseTemplate) => void } = {}):JSX.Element{
 void loadCatalog('response-templates');
 const enabled=useSetting('mail.templates.enabled',true),allowed=useSetting('mail.templates.allowPersonal',true);
 const sidebar=useSetting('mail.templates.showInSidebar',true);
 const workspaceId=createUniqueId();
 const personal=useSetting<ResponseTemplate[]>('mail.templates.personal',[],validResponseTemplates);
 const [draft,setDraft]=createSignal<ResponseTemplate[]>([]),[baseline,setBaseline]=createSignal('[]');
 const dirty=()=>JSON.stringify(draft())!==baseline();
 onCleanup(observePreferenceAccount(()=>{setDraft([]);setBaseline('[]');}));
 createEffect(on(()=>JSON.stringify(personal.value()),value=>{if(!dirty())setDraft(JSON.parse(value) as ResponseTemplate[]);setBaseline(value);}));
 const nav=useSettingsNavigation();const Leave=nav?useLeaveConfirm(nav,dirty,()=>t('response-templates-discard')):()=>null;
 const disabled=()=>!personal.entry()||!enabled.value()||!allowed.value()||personal.locked()||personal.saving();
 let savingToast:string|undefined,disposed=false;
 onCleanup(()=>{disposed=true;if(savingToast)toast.dismiss(savingToast);});
 async function save():Promise<void>{
  if(disabled()||!dirty()||!validResponseTemplates(draft()))return;
  const owner=preferenceAccount();const id=toast.info(t('response-templates-saving'),{busy:true,duration:Infinity});savingToast=id;
  const ok=await personal.set(draft().map(row=>({...row})));
  if(disposed||owner&&!preferenceAccountIsCurrent(owner)){toast.dismiss(id);return;}
  if(ok)toast.success(t('response-templates-saved'),{id});else toast.error(personal.error(),{id});savingToast=undefined;
 }
 async function toggle(value:boolean):Promise<void>{const owner=preferenceAccount();const ok=await enabled.set(value);if(disposed||preferenceAccount()!==owner)return;if(ok)toast.success(t('response-templates-saved'));else toast.error(enabled.error());}
 return <SettingsCard id={props.onUse ? workspaceId : 'compose-card-templates'} wide title={t(props.onUse ? 'response-templates-personal' : 'response-templates-title')}>
  <Show when={!props.onUse}>
  <ControlRow controlId="response-templates-enabled" label={t('response-templates-enable')} note={<LockNote locked={enabled.locked()}/>}>{()=>
   <input id="response-templates-enabled" type="checkbox" checked={enabled.value()} disabled={!enabled.entry()||enabled.locked()||enabled.saving()} onChange={event=>void toggle(event.currentTarget.checked)}/>
  }</ControlRow>
  <ControlRow controlId="response-templates-sidebar" label={t('response-templates-sidebar')} note={<LockNote locked={sidebar.locked()}/>}>{() =>
   <input id="response-templates-sidebar" type="checkbox" checked={sidebar.value()}
     disabled={!sidebar.entry() || sidebar.locked() || sidebar.saving()}
     onChange={event => void sidebar.set(event.currentTarget.checked)}/>
  }</ControlRow>
  <Show when={sidebar.error()}><p role="alert">{sidebar.error()}</p></Show>
  </Show>
  <p class={css.meta}>{t(allowed.value()?'response-templates-personal-help':'response-templates-personal-blocked')}</p><LockNote locked={personal.locked()}/>
  <Show when={allowed.value() || !props.onUse}>
   <TemplateLibraryEditor value={draft()} onChange={setDraft} disabled={disabled()}
     {...(props.onUse && allowed.value() && enabled.value() ? {onUse: props.onUse} : {})}/>
  </Show>
  <p class={css.meta}>{t('response-templates-limits')}</p>
  <div class={css.actions}><button type="button" class={css.button} disabled={disabled()||!dirty()||!validResponseTemplates(draft())} onClick={()=>void save()}>{t('response-templates-save')}</button><button type="button" class={css.ghost} disabled={!dirty()||personal.saving()} onClick={()=>setDraft(personal.value().map(row=>({...row})))}>{t('response-templates-discard-button')}</button></div>
  <Leave/>
 </SettingsCard>;
}
