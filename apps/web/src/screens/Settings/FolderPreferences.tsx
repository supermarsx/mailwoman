import {createEffect, createMemo, createSignal, For, onMount, Show, useContext, type Accessor, type JSX} from 'solid-js';
import {AppContext} from '../../state/context.ts';
import {QuotaDetails} from '../../components/QuotaMeter.tsx';
import {FolderRoles} from './FolderRoles.tsx';
import {FolderCountPreferences} from './FolderCountPreferences.tsx';
import {FocusedInboxPreference} from './FocusedInboxPreference.tsx';
import {FocusedSenders} from './FocusedSenders.tsx';
import {Icon} from '../../components/Icon.tsx';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import type {Tag} from '../../state/slices/tags.ts';
import {accountUiPreferences, accountUiPreferenceStatus, FOLDER_ICONS, retryAccountUiPreferences, setAccountUiPreferences, type AccountUiPreferences, type FolderIcon} from '../../api/accountPreferences.ts';
import {preferenceAccount, preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {t, loadCatalog} from '../../i18n';
import {createSettingsNavigation, type SettingsNavigation} from './navigation.tsx';
import {folderIcon, folderIsHidden, shownFolderIds, moveFolder, orderedFolders, type PreferenceFolder} from './folderPreferenceModel.ts';
import * as css from './styles.css.ts';
import * as folderCss from './FolderPreferences.css.ts';
export interface FolderPreferencesProps {
 folders: Accessor<readonly PreferenceFolder[]>;
 preferences?: Accessor<AccountUiPreferences>;
 onChange?: (patch:AccountUiPreferences)=>void;
 navigation?: SettingsNavigation;
 tags?: Accessor<readonly Tag[]>;
 onRolesSaved?:()=>Promise<void>;
}
function FolderIconEditor(props:{folder:PreferenceFolder;prefs:Accessor<AccountUiPreferences>;update:(patch:AccountUiPreferences)=>void;navigation:SettingsNavigation;tags:Accessor<readonly Tag[]>;local:boolean}):JSX.Element {
 const owner=preferenceAccount();const [pending,setPending]=createSignal(false);
 const [draft,setDraft]=createSignal<FolderIcon|''>(props.prefs().folderIcons?.[props.folder.id]??'');
 const [labels,setLabels]=createSignal(props.prefs().folderLabels?.[props.folder.id]??[]);
 const snapshot=()=>JSON.stringify([draft(),[...labels()].sort()]);const [baseline,setBaseline]=createSignal(snapshot());
 const choices=()=>[...new Set([...props.tags().map(tag=>tag.id),...labels()])];
 const LeaveDialog=useLeaveConfirm(props.navigation,()=>snapshot()!==baseline(),()=>t(pending()?'settings-folders-leave-pending':'settings-folders-discard'));
 createEffect(()=>{if(pending()&&accountUiPreferenceStatus()==='synced'&&owner&&preferenceAccountIsCurrent(owner)){setBaseline(snapshot());void props.navigation.navigate({section:'folders'});}});
 function save():void {
  if(!props.local&&(!owner||!preferenceAccountIsCurrent(owner)))return;
  const icons={...props.prefs().folderIcons};const selected=draft();
  if(selected)icons[props.folder.id]=selected;else delete icons[props.folder.id];
  const folderLabels={...props.prefs().folderLabels};if(labels().length)folderLabels[props.folder.id]=labels();else delete folderLabels[props.folder.id];
  props.update({folderIcons:icons,...(props.prefs().folderLabels?.[props.folder.id]||choices().length||Object.keys(folderLabels).length?{folderLabels}:{})});if(props.local){setBaseline(snapshot());void props.navigation.navigate({section:'folders'});}else setPending(true);
 }
 return <section class={css.section} aria-label={t('settings-folders-edit',{name:props.folder.name})}>
  <h3 class={css.heading}>{t('settings-folders-edit',{name:props.folder.name})}</h3>
  <p class={css.meta}>{t('settings-folders-icon-help')}</p>
  <label class={css.field}><span>{t('settings-folders-icon',{name:props.folder.name})}</span>
   <select class={css.select} disabled={pending()} value={draft()} onChange={event=>setDraft(event.currentTarget.value as FolderIcon|'')}>
    <option value="">{t('settings-folders-automatic')}</option><For each={FOLDER_ICONS}>{id=><option value={id}>{t(`settings-folder-icon-${id}`)}</option>}</For>
   </select>
  </label>
  <p><Icon name={draft()||folderIcon(props.folder,{})}/> {props.folder.name}</p>
  <Show when={choices().length}><fieldset class={css.section}><legend>{t('settings-folder-labels')}</legend><p class={css.meta}>{t('settings-folder-labels-help')}</p><For each={choices()}>{id=><label class={css.check}><input class={css.checkbox} type="checkbox" checked={labels().includes(id)} disabled={pending()||(!labels().includes(id)&&labels().length>=32)} onChange={event=>setLabels(value=>event.currentTarget.checked?[...value,id]:value.filter(label=>label!==id))}/>{props.tags().find(tag=>tag.id===id)?.name??id}</label>}</For></fieldset></Show>
  <Show when={pending()}><p role="status">{t(`settings-folders-sync-${accountUiPreferenceStatus()}`)}</p><Show when={accountUiPreferenceStatus()==='error'}><button class={css.ghost} onClick={()=>void retryAccountUiPreferences()}>{t('settings-folders-retry')}</button></Show></Show>
  <div class={css.actions}><button type="button" class={css.button} disabled={pending()||snapshot()===baseline()} onClick={save}>{t('settings-folders-save')}</button><button type="button" class={css.ghost} onClick={()=>void props.navigation.navigate({section:'folders'})}>{t('settings-folders-back')}</button></div>
  <LeaveDialog/>
 </section>;
}
export function FolderPreferences(props:FolderPreferencesProps):JSX.Element {
 onMount(()=>void loadCatalog('settings'));
 const navigation=props.navigation??createSettingsNavigation({section:'folders'});
 const app=useContext(AppContext);
 const prefs=()=>props.preferences?.()??accountUiPreferences();
 const update=(patch:AccountUiPreferences)=> (props.onChange??setAccountUiPreferences)(patch);
 const folders=createMemo(()=>orderedFolders(props.folders(),prefs()));
 const editing=()=>props.folders().find(folder=>navigation.route().editor?.kind==='edit'&&navigation.route().editor?.id===folder.id);
 const [dragging,setDragging]=createSignal<string|null>(null);
 const pinned=(id:string)=>prefs().folderPinned?.includes(id)??false;
 const canMove=(id:string,target?:string)=>target!==undefined&&pinned(id)===pinned(target);
 function move(id:string,target:string):void {if(canMove(id,target))update({folderOrder:moveFolder(folders().map(f=>f.id),id,target)});}
 function pin(id:string):void {const ids=(prefs().folderPinned??[]).filter(value=>value!==id);update({folderPinned:pinned(id)?ids:[...ids,id]});}
 return <Show when={editing()} keyed fallback={<section class={css.section} aria-label={t('settings-folders-title')}>
  <h2 class={css.heading}>{t('settings-folders-title')}</h2><p class={css.prose}>{t('settings-folders-help')}</p>
  <p class={css.prose}>{t('settings-folders-hidden-help')}</p>
  <label class={css.field}><input type="checkbox" checked={prefs().userFoldersCollapsed===true} onChange={event=>update({userFoldersCollapsed:event.currentTarget.checked})}/>{t('settings-folders-collapse-user')}</label>
  <p role="status" class={css.meta}>{t(`settings-folders-sync-${props.onChange?'synced':accountUiPreferenceStatus()}`)}</p>
  <Show when={!props.onChange}><FocusedInboxPreference/></Show>
  <Show when={!props.onChange?app:undefined}>{state=><FocusedSenders senders={state().focusedSenders} onRemove={email=>state().removeSenderFocus(email)}/>}</Show>
  <Show when={!props.onChange}><FolderCountPreferences/></Show>
  <Show when={!props.onChange&&app?.accountId()}>{accountId=><QuotaDetails accountId={accountId()} refresh={props.folders()}/>}</Show>
  <Show when={!props.onChange}><FolderRoles folders={props.folders} {...(props.onRolesSaved?{onSaved:props.onRolesSaved}:{})}/></Show>
  <div class={folderCss.wrap} role="region" aria-label={t('settings-folders-table')} tabIndex={0}><table class={folderCss.table}><thead><tr><th>{t('settings-folders-name')}</th><th>{t('settings-folders-actions')}</th></tr></thead><tbody>
   <For each={folders()}>{(folder,index)=><tr onDragOver={event=>{if(dragging()&&canMove(dragging()!,folder.id))event.preventDefault();}} onDrop={event=>{event.preventDefault();const id=dragging();if(id)move(id,folder.id);setDragging(null);}}>
    <td><Icon name={folderIcon(folder,prefs())}/> {folder.name}<Show when={prefs().folderLabels?.[folder.id]?.length}><small class={css.meta}>{prefs().folderLabels?.[folder.id]?.map(id=>props.tags?.().find(tag=>tag.id===id)?.name??id).join(', ')}</small></Show></td><td><div class={folderCss.actions}>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-edit',{name:folder.name})} onClick={()=>void navigation.navigate({section:'folders',editor:{kind:'edit',id:folder.id}})}><Icon name="edit"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-pin',{name:folder.name})} aria-pressed={pinned(folder.id)} onClick={()=>pin(folder.id)}><Icon name="star"/></button>
     <button type="button" class={css.ghost} disabled={folder.role==='inbox'} aria-label={t(folderIsHidden(folder,props.folders(),prefs())?'settings-folders-show':'settings-folders-hide',{name:folder.name})} onClick={()=>{
      const hidden=prefs().folderHidden??[];
      update({folderHidden:folderIsHidden(folder,props.folders(),prefs())?shownFolderIds(folder,props.folders(),hidden):[...hidden,folder.id]});
     }}><Icon name="eye"/>{t(folderIsHidden(folder,props.folders(),prefs())?'settings-folders-show-short':'settings-folders-hide-short')}</button>
     <button type="button" class={css.ghost} draggable="true" aria-label={t('settings-folders-drag',{name:folder.name})} onDragStart={event=>{setDragging(folder.id);event.dataTransfer?.setData('text/plain',folder.id);}} onDragEnd={()=>setDragging(null)}><Icon name="sort"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-up',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()-1]?.id)} onClick={()=>move(folder.id,folders()[index()-1]!.id)}><Icon name="chevronUp"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-down',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()+1]?.id)} onClick={()=>move(folder.id,folders()[index()+1]!.id)}><Icon name="chevronDown"/></button>
    </div></td>
   </tr>}</For>
  </tbody></table></div>
 </section>}>{folder=><FolderIconEditor folder={folder} prefs={prefs} update={update} navigation={navigation} tags={props.tags??(()=>[])} local={!!props.onChange}/>}</Show>;
}
