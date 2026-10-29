import {createEffect,createSignal,For,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {keywordFor,type Tag} from '../../state/slices/tags.ts';
import {accountUiPreferenceStatus,FOLDER_ICONS,retryAccountUiPreferences} from '../../api/accountPreferences.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {Icon,tagIconName} from '../../components/Icon.tsx';
import {ColorField} from '../../components/color/ColorPicker.tsx';
import {createConfirm,useLeaveConfirm} from '../../components/confirm.tsx';
import {loadCatalog,t} from '../../i18n';
import type {SettingsNavigation} from './navigation.tsx';
import * as css from './styles.css.ts';

function LabelEditor(props:{value?:Tag;navigation:SettingsNavigation}):JSX.Element {
  const app=useApp();const owner=preferenceAccount();
  const initial={name:props.value?.name??'',color:props.value?.color??'#2563eb',icon:props.value?.icon??'tag'};
  const [draft,setDraft]=createSignal(initial);const [saved,setSaved]=createSignal(false);const [pending,setPending]=createSignal(false);const [error,setError]=createSignal('');
  const changed=()=>!saved()&&JSON.stringify(draft())!==JSON.stringify(initial);
  const LeaveDialog=useLeaveConfirm(props.navigation,changed,()=>t(pending()?'labels-leave-pending':'labels-discard'));
  createEffect(()=>{if(pending()&&accountUiPreferenceStatus()==='synced'&&owner&&preferenceAccountIsCurrent(owner)){setSaved(true);void props.navigation.navigate({section:'labels'});}});
  function submit(event:SubmitEvent):void {
    event.preventDefault();setError('');if(!owner||!preferenceAccountIsCurrent(owner))return;
    const value={...draft(),name:draft().name.trim()};const id=props.value?.id??keywordFor(value.name);
    if(!id||id.length>255||(!props.value&&(app.tagByKeyword(id)||app.tags().length>=100))){setError(t('labels-invalid'));return;}
    if(props.value)app.updateTag(id,value);else app.addTag(value.name,value.color,value.icon);setPending(true);
  }
  return <form class={css.section} onSubmit={submit}><h3>{t(props.value?'labels-edit':'labels-new')}</h3>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <label class={css.field}><span>{t('labels-name')}</span><input class={css.input} required maxLength={128} value={draft().name} disabled={pending()} onInput={event=>setDraft(value=>({...value,name:event.currentTarget.value}))}/></label>
    <div class={css.field}><span aria-hidden="true">{t('labels-color')}</span><ColorField label={t('labels-color')} testId="label-color" value={draft().color} disabled={pending()} contrast={{against:'surface',use:'non-text'}} onChange={color=>{if(color)setDraft(value=>({...value,color}));}}/></div>
    <label class={css.field}><span>{t('labels-icon')}</span><select class={css.input} aria-label={t('labels-icon')} value={draft().icon} disabled={pending()} onChange={event=>setDraft(value=>({...value,icon:event.currentTarget.value}))}><For each={FOLDER_ICONS}>{icon=><option value={icon}>{t(`settings-folder-icon-${icon}`)}</option>}</For></select></label>
    <p class={css.meta}>{t('labels-keyword',{keyword:props.value?.id??keywordFor(draft().name)})}</p>
    <div class={css.actions}><button class={css.button} disabled={!changed()||pending()} type="submit">{t('labels-save')}</button><button class={css.ghost} type="button" onClick={()=>void props.navigation.navigate({section:'labels'})}>{t('labels-back')}</button></div>
    <LeaveDialog/>
  </form>;
}
export function Labels(props:{navigation:SettingsNavigation}):JSX.Element {
  const app=useApp();onMount(()=>{void loadCatalog('labels');void loadCatalog('settings');});
  const remove=createConfirm();
  const route=()=>props.navigation.route().editor;const selected=()=>app.tags().find(tag=>tag.id===route()?.id);
  return <section class={css.section} aria-label={t('labels-title')}><h2>{t('labels-title')}</h2><p class={css.prose}>{t('labels-help')}</p>
    <Show when={accountUiPreferenceStatus()==='saving'}><p role="status">{t('labels-saving')}</p></Show>
    <Show when={accountUiPreferenceStatus()==='error'}><p role="alert">{t('labels-sync-error')}</p><button class={css.ghost} onClick={()=>void retryAccountUiPreferences()}>{t('labels-retry')}</button></Show>
    <Show when={route()} keyed fallback={<><div class={css.actions}><button class={css.button} disabled={app.tags().length>=100} onClick={()=>void props.navigation.navigate({section:'labels',editor:{kind:'new'}})}><Icon name="plus"/>{t('labels-new')}</button></div>
      <div class="settings-table-scroll"><table class="settings-shortcut-table settings-label-table"><thead><tr><th>{t('labels-name')}</th><th>{t('labels-keyword-title')}</th><th>{t('labels-actions')}</th></tr></thead><tbody><For each={app.tags()}>{tag=><tr><td><Icon name={tagIconName(tag.icon)}/><span class="settings-label-color" style={{'border-inline-start':`4px solid ${tag.color}`}}>{tag.name}</span></td><td>{tag.id}</td><td><div class={css.actions}><button class={css.ghost} aria-label={t('labels-edit-name',{name:tag.name})} onClick={()=>void props.navigation.navigate({section:'labels',editor:{kind:'edit',id:tag.id}})}><Icon name="edit"/></button><button class={css.ghost} aria-label={t('labels-remove-name',{name:tag.name})} onClick={()=>void remove.ask({title:t('labels-remove-confirm',{name:tag.name}),confirmLabel:t('common-remove'),testId:'labels-remove-confirm'}).then(yes=>{if(yes)app.deleteTag(tag.id);})}><Icon name="delete"/></button></div></td></tr>}</For></tbody></table></div></>}>
      {editor=><Show when={editor.kind==='new'||selected()}><LabelEditor {...(selected()?{value:selected()!}:{})} navigation={props.navigation}/></Show>}
    </Show>
    <remove.Dialog/>
  </section>;
}
