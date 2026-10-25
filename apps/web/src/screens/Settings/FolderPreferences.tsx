import {createMemo, createSignal, For, onMount, type Accessor, type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {accountUiPreferences, accountUiPreferenceStatus, FOLDER_ICONS, setAccountUiPreferences, type AccountUiPreferences, type FolderIcon} from '../../api/accountPreferences.ts';
import {t, loadCatalog} from '../../i18n';
import {folderIcon, moveFolder, orderedFolders, type PreferenceFolder} from './folderPreferenceModel.ts';
import * as css from './styles.css.ts';
export interface FolderPreferencesProps {
 folders: Accessor<readonly PreferenceFolder[]>;
 preferences?: Accessor<AccountUiPreferences>;
 onChange?: (patch:AccountUiPreferences)=>void;
}
export function FolderPreferences(props:FolderPreferencesProps):JSX.Element {
 onMount(()=>void loadCatalog('settings'));
 const prefs=()=>props.preferences?.()??accountUiPreferences();
 const update=(patch:AccountUiPreferences)=> (props.onChange??setAccountUiPreferences)(patch);
 const folders=createMemo(()=>orderedFolders(props.folders(),prefs()));
 const [dragging,setDragging]=createSignal<string|null>(null);
 const pinned=(id:string)=>prefs().folderPinned?.includes(id)??false;
 const canMove=(id:string,target?:string)=>target!==undefined&&pinned(id)===pinned(target);
 function move(id:string,target:string):void {if(canMove(id,target))update({folderOrder:moveFolder(folders().map(f=>f.id),id,target)});}
 function pin(id:string,on:boolean):void {const ids=(prefs().folderPinned??[]).filter(value=>value!==id);update({folderPinned:on?[...ids,id]:ids});}
 function icon(id:string,value:string):void {const icons={...prefs().folderIcons};if(value)icons[id]=value as FolderIcon;else delete icons[id];update({folderIcons:icons});}
 return <section class={css.section} aria-label={t('settings-folders-title')}>
  <h2 class={css.heading}>{t('settings-folders-title')}</h2><p class={css.prose}>{t('settings-folders-help')}</p>
  <p role="status" class={css.meta}>{t(`settings-folders-sync-${props.onChange?'synced':accountUiPreferenceStatus()}`)}</p>
  <ul class={css.list}><For each={folders()}>{(folder,index)=><li class={css.item} onDragOver={event=>{if(dragging()&&canMove(dragging()!,folder.id))event.preventDefault();}} onDrop={event=>{event.preventDefault();const id=dragging();if(id)move(id,folder.id);setDragging(null);}}>
   <div class={css.itemMain}><h3 class={css.subheading}><Icon name={folderIcon(folder,prefs())}/> {folder.name}</h3>
    <label class={css.row}><input type="checkbox" checked={pinned(folder.id)} onChange={event=>pin(folder.id,event.currentTarget.checked)}/>{t('settings-folders-pin',{name:folder.name})}</label>
    <label class={css.field}><span class={css.label}>{t('settings-folders-icon',{name:folder.name})}</span><select class={css.select} value={prefs().folderIcons?.[folder.id]??''} onChange={event=>icon(folder.id,event.currentTarget.value)}><option value="">{t('settings-folders-automatic')}</option><For each={FOLDER_ICONS}>{id=><option value={id}>{t(`settings-folder-icon-${id}`)}</option>}</For></select></label>
   </div>
   <div class={css.actions}>
    <button type="button" class={css.ghost} draggable="true" aria-label={t('settings-folders-drag',{name:folder.name})} onDragStart={event=>{setDragging(folder.id);event.dataTransfer?.setData('text/plain',folder.id);}} onDragEnd={()=>setDragging(null)}><Icon name="sort"/></button>
    <button type="button" class={css.ghost} aria-label={t('settings-folders-up',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()-1]?.id)} onClick={()=>move(folder.id,folders()[index()-1]!.id)}><Icon name="chevronUp"/></button>
    <button type="button" class={css.ghost} aria-label={t('settings-folders-down',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()+1]?.id)} onClick={()=>move(folder.id,folders()[index()+1]!.id)}><Icon name="chevronDown"/></button>
   </div>
  </li>}</For></ul>
 </section>;
}
