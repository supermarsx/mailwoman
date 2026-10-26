import {createMemo, createSignal, For, onCleanup, onMount, Show, type Accessor, type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {accountUiPreferences, accountUiPreferenceStatus, FOLDER_ICONS, setAccountUiPreferences, type AccountUiPreferences, type FolderIcon} from '../../api/accountPreferences.ts';
import {t, loadCatalog} from '../../i18n';
import {createSettingsNavigation, type SettingsNavigation} from './navigation.tsx';
import {folderIcon, moveFolder, orderedFolders, type PreferenceFolder} from './folderPreferenceModel.ts';
import * as css from './styles.css.ts';
export interface FolderPreferencesProps {
 folders: Accessor<readonly PreferenceFolder[]>;
 preferences?: Accessor<AccountUiPreferences>;
 onChange?: (patch:AccountUiPreferences)=>void;
 navigation?: SettingsNavigation;
}
function FolderIconEditor(props:{folder:PreferenceFolder;prefs:Accessor<AccountUiPreferences>;update:(patch:AccountUiPreferences)=>void;navigation:SettingsNavigation}):JSX.Element {
 const [draft,setDraft]=createSignal<FolderIcon|''>(props.prefs().folderIcons?.[props.folder.id]??'');
 const [baseline,setBaseline]=createSignal(draft());
 onCleanup(props.navigation.beforeLeave(()=>draft()===baseline()||window.confirm(t('settings-folders-discard'))));
 function save():void {
  const icons={...props.prefs().folderIcons};const selected=draft();
  if(selected)icons[props.folder.id]=selected;else delete icons[props.folder.id];
  props.update({folderIcons:icons});setBaseline(selected);void props.navigation.navigate({section:'folders'});
 }
 return <section class={css.section} aria-label={t('settings-folders-edit',{name:props.folder.name})}>
  <h3 class={css.heading}>{t('settings-folders-edit',{name:props.folder.name})}</h3>
  <p class={css.meta}>{t('settings-folders-icon-help')}</p>
  <label class={css.field}><span>{t('settings-folders-icon',{name:props.folder.name})}</span>
   <select class={css.select} value={draft()} onChange={event=>setDraft(event.currentTarget.value as FolderIcon|'')}>
    <option value="">{t('settings-folders-automatic')}</option><For each={FOLDER_ICONS}>{id=><option value={id}>{t(`settings-folder-icon-${id}`)}</option>}</For>
   </select>
  </label>
  <p><Icon name={draft()||folderIcon(props.folder,{})}/> {props.folder.name}</p>
  <div class={css.actions}><button type="button" class={css.button} disabled={draft()===baseline()} onClick={save}>{t('settings-folders-save')}</button><button type="button" class={css.ghost} onClick={()=>void props.navigation.navigate({section:'folders'})}>{t('settings-folders-back')}</button></div>
 </section>;
}
export function FolderPreferences(props:FolderPreferencesProps):JSX.Element {
 onMount(()=>void loadCatalog('settings'));
 const navigation=props.navigation??createSettingsNavigation({section:'folders'});
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
  <p role="status" class={css.meta}>{t(`settings-folders-sync-${props.onChange?'synced':accountUiPreferenceStatus()}`)}</p>
  <div style={{'overflow-x':'auto'}}><table class="settings-shortcut-table"><thead><tr><th>{t('settings-folders-name')}</th><th>{t('settings-folders-actions')}</th></tr></thead><tbody>
   <For each={folders()}>{(folder,index)=><tr onDragOver={event=>{if(dragging()&&canMove(dragging()!,folder.id))event.preventDefault();}} onDrop={event=>{event.preventDefault();const id=dragging();if(id)move(id,folder.id);setDragging(null);}}>
    <td><Icon name={folderIcon(folder,prefs())}/> {folder.name}</td><td><div class={css.actions}>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-edit',{name:folder.name})} onClick={()=>void navigation.navigate({section:'folders',editor:{kind:'edit',id:folder.id}})}><Icon name="edit"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-pin',{name:folder.name})} aria-pressed={pinned(folder.id)} onClick={()=>pin(folder.id)}><Icon name="star"/></button>
     <button type="button" class={css.ghost} draggable="true" aria-label={t('settings-folders-drag',{name:folder.name})} onDragStart={event=>{setDragging(folder.id);event.dataTransfer?.setData('text/plain',folder.id);}} onDragEnd={()=>setDragging(null)}><Icon name="sort"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-up',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()-1]?.id)} onClick={()=>move(folder.id,folders()[index()-1]!.id)}><Icon name="chevronUp"/></button>
     <button type="button" class={css.ghost} aria-label={t('settings-folders-down',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()+1]?.id)} onClick={()=>move(folder.id,folders()[index()+1]!.id)}><Icon name="chevronDown"/></button>
    </div></td>
   </tr>}</For>
  </tbody></table></div>
 </section>}>{folder=><FolderIconEditor folder={folder} prefs={prefs} update={update} navigation={navigation}/>}</Show>;
}
