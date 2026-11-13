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
import {ControlRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as manager from './managementTable.css.ts';
import * as table from './linkedAccounts.css.ts';
import * as entity from './entityTable.css.ts';
import * as ux from '../Admin/shell/ux.css.ts';

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
  return <form onSubmit={submit}><SettingsCard id="labels-card-editor" title={t(props.value?'labels-edit':'labels-new')}>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <ControlRow controlId="label-name" label={t('labels-name')}>{()=><input id="label-name" class={css.input} required maxLength={128} value={draft().name} disabled={pending()} onInput={event=>setDraft(value=>({...value,name:event.currentTarget.value}))}/>}</ControlRow>
    <ControlRow controlId="label-color-row" group label={t('labels-color')}>{()=><ColorField label={t('labels-color')} testId="label-color" value={draft().color} disabled={pending()} contrast={{against:'surface',use:'non-text'}} onChange={color=>{if(color)setDraft(value=>({...value,color}));}}/>}</ControlRow>
    <ControlRow controlId="label-icon" label={t('labels-icon')}>{()=><select id="label-icon" class={css.input} value={draft().icon} disabled={pending()} onChange={event=>setDraft(value=>({...value,icon:event.currentTarget.value}))}><For each={FOLDER_ICONS}>{icon=><option value={icon}>{t(`settings-folder-icon-${icon}`)}</option>}</For></select>}</ControlRow>
    <p class={css.meta}>{t('labels-keyword',{keyword:props.value?.id??keywordFor(draft().name)})}</p>
    <div class={css.actions}><button class={css.button} disabled={!changed()||pending()} type="submit">{t('labels-save')}</button><button class={css.ghost} type="button" onClick={()=>void props.navigation.navigate({section:'labels'})}>{t('labels-back')}</button></div>
    <LeaveDialog/>
  </SettingsCard></form>;
}
export function Labels(props:{navigation:SettingsNavigation}):JSX.Element {
  const app=useApp();onMount(()=>{void loadCatalog('labels');void loadCatalog('settings');});
  const remove=createConfirm();
  const [query,setQuery]=createSignal('');
  const saving=()=>accountUiPreferenceStatus()==='saving';
  const shown=()=>app.tags().filter(tag=>`${tag.name} ${tag.id}`.toLocaleLowerCase().includes(query().trim().toLocaleLowerCase()));
  async function removeLabel(tag:Tag):Promise<void>{const owner=preferenceAccount();if(!owner||saving())return;const yes=await remove.ask({title:t('labels-remove-confirm',{name:tag.name}),confirmLabel:t('common-remove'),testId:'labels-remove-confirm'});if(yes&&preferenceAccountIsCurrent(owner))app.deleteTag(tag.id);}
  const route=()=>props.navigation.route().editor;const selected=()=>app.tags().find(tag=>tag.id===route()?.id);
  const syncStatus=():JSX.Element=><><Show when={accountUiPreferenceStatus()==='saving'}><p role="status">{t('labels-saving')}</p></Show><Show when={accountUiPreferenceStatus()==='error'}><p role="alert">{t('labels-sync-error')}</p><button type="button" class={css.ghost} onClick={()=>void retryAccountUiPreferences()}>{t('labels-retry')}</button></Show></>;
  return <div class={css.stack}>
    <Show when={route()} keyed fallback={<section id="labels-card" class={css.section} aria-labelledby="labels-card-title" aria-busy={saving()}>
      <div class={table.header}><div class={table.headerText}><h2 id="labels-card-title" class={css.heading}>{t('labels-title')}</h2><p class={css.meta}>{t('labels-help')}</p></div>
       <button type="button" class={`${css.button} ${table.addButton}`} disabled={saving()||app.tags().length>=100} onClick={()=>void props.navigation.navigate({section:'labels',editor:{kind:'new'}})}><Icon name="plus"/>{t('labels-new')}</button>
      </div>
      {syncStatus()}
      <div class={manager.bar}><label class={manager.search}>{t('common-search')}<input class={css.input} type="search" value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label><span class={css.meta}>{app.tags().length} / 100</span></div>
      <ul class={table.table} aria-label={t('labels-title')}><For each={shown()}>{tag=><li class={entity.row}>
       <span class={entity.avatar} style={{color:tag.color}} aria-hidden="true"><Icon name={tagIconName(tag.icon)}/></span>
       <div class={table.who}><button type="button" class={`${table.name} ${manager.title}`} disabled={saving()} onClick={()=>void props.navigation.navigate({section:'labels',editor:{kind:'edit',id:tag.id}})}>{tag.name}</button><span class={table.address}><span>{t('labels-keyword-title')}: </span><code>{tag.id}</code></span><span class={entity.detail}>{t('labels-color')}: {tag.color}</span></div>
       <div class={table.actions}><button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={saving()} aria-label={t('labels-edit-name',{name:tag.name})} title={t('labels-edit-name',{name:tag.name})} onClick={()=>void props.navigation.navigate({section:'labels',editor:{kind:'edit',id:tag.id}})}><Icon name="edit"/></button><button type="button" class={`btn btn--danger ${ux.iconAction}`} disabled={saving()} aria-label={t('labels-remove-name',{name:tag.name})} title={t('labels-remove-name',{name:tag.name})} onClick={()=>void removeLabel(tag)}><Icon name="delete"/></button></div>
      </li>}</For></ul>
    </section>}>
      {editor=><>{syncStatus()}<Show when={editor.kind==='new'||selected()}><LabelEditor {...(selected()?{value:selected()!}:{})} navigation={props.navigation}/></Show></>}
    </Show>
    <remove.Dialog/>
  </div>;
}
