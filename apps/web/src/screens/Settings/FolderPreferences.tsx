import {createEffect, createMemo, createSignal, For, onCleanup, onMount, Show, useContext, type Accessor, type JSX} from 'solid-js';
import {AppContext} from '../../state/context.ts';
import {QuotaDetails} from '../../components/QuotaMeter.tsx';
import {FolderIconPicker} from './FolderIconPicker.tsx';
import {FolderRoles} from './FolderRoles.tsx';
import {FolderCountPreferences} from './FolderCountPreferences.tsx';
import {FocusedInboxPreference} from './FocusedInboxPreference.tsx';
import {FocusedSenders} from './FocusedSenders.tsx';
import {Icon} from '../../components/Icon.tsx';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import {CreateFolderDialog} from '../../components/FolderMaintenance.tsx';
import {useRenameContext} from '../../components/RenameFolder.tsx';
import {createContextMenu, type ContextAction} from '../../components/ContextMenu.tsx';
import type {Tag} from '../../state/slices/tags.ts';
import {accountUiPreferences, accountUiPreferenceStatus, retryAccountUiPreferences, setAccountUiPreferences, type AccountUiPreferences, type FolderIcon} from '../../api/accountPreferences.ts';
import {observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {t, loadCatalog} from '../../i18n';
import {createSettingsNavigation, type SettingsNavigation} from './navigation.tsx';
import {folderIcon, folderIsHidden, folderPathLabel, shownFolderIds, moveFolder, orderedFolders, type PreferenceFolder} from './folderPreferenceModel.ts';
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
  <div class={css.field}><span>{t('settings-folders-icon',{name:props.folder.name})}</span>
   <FolderIconPicker label={t('settings-folders-icon',{name:props.folder.name})} value={draft()} disabled={pending()} automaticIcon={folderIcon(props.folder,{})} onChange={setDraft}/>
  </div>
  <p><Icon name={draft()||folderIcon(props.folder,{})}/> {props.folder.name}</p>
  <Show when={choices().length}><fieldset class={css.section}><legend>{t('settings-folder-labels')}</legend><p class={css.meta}>{t('settings-folder-labels-help')}</p><For each={choices()}>{id=><label class={css.check}><input class={css.checkbox} type="checkbox" checked={labels().includes(id)} disabled={pending()||(!labels().includes(id)&&labels().length>=32)} onChange={event=>setLabels(value=>event.currentTarget.checked?[...value,id]:value.filter(label=>label!==id))}/>{props.tags().find(tag=>tag.id===id)?.name??id}</label>}</For></fieldset></Show>
  <Show when={pending()}><p role="status">{t(`settings-folders-sync-${accountUiPreferenceStatus()}`)}</p><Show when={accountUiPreferenceStatus()==='error'}><button class={css.ghost} onClick={()=>void retryAccountUiPreferences()}>{t('settings-folders-retry')}</button></Show></Show>
  <div class={css.actions}><button type="button" class={css.button} disabled={pending()||snapshot()===baseline()} onClick={save}>{t('settings-folders-save')}</button><button type="button" class={css.ghost} onClick={()=>void props.navigation.navigate({section:'folders'})}>{t('settings-folders-back')}</button></div>
  <LeaveDialog/>
 </section>;
}
export function FolderPreferences(props:FolderPreferencesProps):JSX.Element {
 const navigation=props.navigation??createSettingsNavigation({section:'folders'});
 const app=useContext(AppContext),owner=preferenceAccount();
 const ability=!props.onChange&&app?useRenameContext():()=>null;
 const [ownerCurrent,setOwnerCurrent]=createSignal(!!owner&&preferenceAccountIsCurrent(owner));
 const current=()=>!!props.onChange||(ownerCurrent()&&app?.accountId()===owner?.id);
 onMount(()=>{void loadCatalog('settings');if(!props.onChange&&app)void loadCatalog('mail');});
 const prefs=()=>props.preferences?.()??accountUiPreferences();
 const update=(patch:AccountUiPreferences):void=>{if(current())(props.onChange??setAccountUiPreferences)(patch);};
 const folders=createMemo(()=>orderedFolders(props.folders(),prefs()));
 const editing=()=>props.folders().find(folder=>navigation.route().editor?.kind==='edit'&&navigation.route().editor?.id===folder.id);
 const [query,setQuery]=createSignal(''),[selected,setSelected]=createSignal<ReadonlySet<string>>(new Set<string>());
 const [dragging,setDragging]=createSignal<readonly string[]>([]),[dropTarget,setDropTarget]=createSignal<string|null>(null);
 const [creating,setCreating]=createSignal<{parentId?:string}>(),[menuFolder,setMenuFolder]=createSignal<string>();
 const pinned=(id:string)=>prefs().folderPinned?.includes(id)??false;
 function parentPath(folder:PreferenceFolder):string {
  const parts:string[]=[],seen=new Set([folder.id]);let parent=props.folders().find(value=>value.id===folder.parentId);
  while(parent&&!seen.has(parent.id)){seen.add(parent.id);parts.unshift(parent.name);parent=props.folders().find(value=>value.id===parent?.parentId);}
  return parts.join(' / ');
 }
 const locationLabel=(folder:PreferenceFolder):string=>parentPath(folder)||(folderPathLabel(folder)!==folder.name?folderPathLabel(folder):'');
 const visible=createMemo(()=>{const search=query().trim().toLocaleLowerCase();return folders().filter(folder=>!search||[folderPathLabel(folder),parentPath(folder),folder.name].some(value=>value.toLocaleLowerCase().includes(search)));});
 const selectedFolders=()=>folders().filter(folder=>selected().has(folder.id));
 const canMove=(id:string,target?:string)=>current()&&target!==undefined&&id!==target&&folders().some(folder=>folder.id===id)&&folders().some(folder=>folder.id===target)&&pinned(id)===pinned(target);
 function move(id:string,target:string):void {if(canMove(id,target))update({folderOrder:moveFolder(folders().map(folder=>folder.id),id,target)});}
 function pin(id:string):void {const ids=(prefs().folderPinned??[]).filter(value=>value!==id);update({folderPinned:pinned(id)?ids:[...ids,id]});}
 function toggle(id:string):void {if(!current())return;setSelected(value=>{const next=new Set(value);if(next.has(id))next.delete(id);else next.add(id);return next;});}
 function setVisibility(targets:readonly PreferenceFolder[],show:boolean):void {
  let hidden=[...(prefs().folderHidden??[])];
  if(show)for(const folder of targets)hidden=shownFolderIds(folder,props.folders(),hidden);
  else hidden=[...new Set([...hidden,...targets.filter(folder=>folder.role!=='inbox').map(folder=>folder.id)])];
  const inboxIds=new Set(props.folders().filter(folder=>folder.role==='inbox').map(folder=>folder.id));
  update({folderHidden:hidden.filter(id=>!inboxIds.has(id))});
 }
 function setPinned(targets:readonly PreferenceFolder[],value:boolean):void {
  const ids=new Set(prefs().folderPinned??[]);for(const folder of targets)if(value)ids.add(folder.id);else ids.delete(folder.id);
  update({folderPinned:[...ids]});
 }
 function stopDragging():void {setDragging([]);setDropTarget(null);}
 function canDrop(target:string):boolean {return dragging().length>0&&!dragging().includes(target)&&dragging().every(id=>canMove(id,target));}
 function drop(target:string):void {
  if(canDrop(target)){
   const ids=folders().map(folder=>folder.id),moving=ids.filter(id=>dragging().includes(id)),remaining=ids.filter(id=>!dragging().includes(id));
   const insertion=remaining.indexOf(target)+(ids.indexOf(moving[0]!)<ids.indexOf(target)?1:0);
   remaining.splice(insertion,0,...moving);update({folderOrder:remaining});
  }
  stopDragging();
 }
 const context=createContextMenu({label:()=>t('settings-folders-actions'),scope:()=>`${owner?.id}:${owner?.generation}:${ownerCurrent()}:${menuFolder()}`,actions:()=>{
  const folder=folders().find(value=>value.id===menuFolder());if(!folder||!current())return [];
  const index=folders().findIndex(value=>value.id===folder.id),above=folders()[index-1]?.id,below=folders()[index+1]?.id;
  const actions:ContextAction[]=[
   {id:'select',label:t('settings-folders-select',{name:folder.name}),icon:'check',checked:selected().has(folder.id),run:()=>toggle(folder.id)},
   {id:'up',label:t('settings-folders-up',{name:folder.name}),icon:'chevronUp',disabled:!canMove(folder.id,above),run:()=>{if(above)move(folder.id,above);}},
   {id:'down',label:t('settings-folders-down',{name:folder.name}),icon:'chevronDown',disabled:!canMove(folder.id,below),run:()=>{if(below)move(folder.id,below);}},
  ];
  if(ability()?.manages&&app?.mailboxes().find(box=>box.id===folder.id)?.myRights?.mayCreateChild===true)actions.push({id:'create',label:t('folder-create'),icon:'folder',separatorBefore:true,run:()=>{setCreating({parentId:folder.id});}});
  return actions;
 }});
 onCleanup(observePreferenceAccount(()=>{setOwnerCurrent(!!owner&&preferenceAccountIsCurrent(owner));setSelected(new Set<string>());stopDragging();setCreating(undefined);context.close(false);}));
 createEffect(()=>{
  const ids=new Set(folders().map(folder=>folder.id));
  setSelected(value=>{const valid=new Set([...value].filter(id=>ids.has(id)));return valid.size===value.size?value:valid;});
  if(dragging().some(id=>!ids.has(id)))stopDragging();
  if(menuFolder()&&!ids.has(menuFolder()!))context.close(false);
  ability();
 });
 const rowCounts=(folder:PreferenceFolder):string=>{
  const counts=folder as PreferenceFolder&{totalEmails?:number;unreadEmails?:number};
  return typeof counts.totalEmails==='number'?t('settings-folders-counts',{unread:counts.unreadEmails??0,total:counts.totalEmails}):'';
 };
 return <Show when={editing()} keyed fallback={<section class={css.section} aria-label={t('settings-folders-title')}>
  <div class={folderCss.heading}><div><h2 class={css.heading}>{t('settings-folders-title')}</h2><p class={css.prose}>{t('settings-folders-help')}</p></div><Show when={!props.onChange&&app&&ability()?.manages&&current()}><button type="button" class="btn btn--ghost" onClick={()=>setCreating({})}><Icon name="folder"/>{t('folder-create')}</button></Show></div>
  <p class={css.prose}>{t('settings-folders-hidden-help')}</p>
  <label class={css.field}><input type="checkbox" checked={prefs().userFoldersCollapsed===true} disabled={!current()} onChange={event=>update({userFoldersCollapsed:event.currentTarget.checked})}/>{t('settings-folders-collapse-user')}</label>
  <p role="status" class={css.meta}>{t(`settings-folders-sync-${props.onChange?'synced':accountUiPreferenceStatus()}`)}</p>
  <Show when={!props.onChange}><FocusedInboxPreference/></Show>
  <Show when={!props.onChange?app:undefined}>{state=><FocusedSenders senders={state().focusedSenders} onRemove={email=>state().removeSenderFocus(email)}/>}</Show>
  <Show when={!props.onChange}><FolderCountPreferences/></Show>
  <Show when={!props.onChange&&app?.accountId()}>{accountId=><QuotaDetails accountId={accountId()} refresh={props.folders()}/>}</Show>
  <Show when={!props.onChange}><FolderRoles folders={props.folders} {...(props.onRolesSaved?{onSaved:props.onRolesSaved}:{})}/></Show>
  <div class={folderCss.tools}>
   <label class={folderCss.search}><Icon name="search"/><input class={css.input} type="search" aria-label={t('settings-folders-search')} placeholder={t('settings-folders-search')} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
   <button type="button" class="btn btn--ghost" disabled={!current()||!visible().length} onClick={()=>setSelected(value=>new Set([...value,...visible().map(folder=>folder.id)]))}>{t('settings-folders-select-visible')}</button>
   <button type="button" class="btn btn--ghost" disabled={!current()||!folders().length} onClick={()=>setSelected(new Set(folders().map(folder=>folder.id)))}>{t('settings-folders-select-all')}</button>
  </div>
  <Show when={selectedFolders().length}><div class={folderCss.bulk} role="group" aria-label={t('settings-folders-selected',{count:selectedFolders().length})}>
   <span role="status">{t('settings-folders-selected',{count:selectedFolders().length})}</span>
   <button type="button" class="btn btn--ghost" onClick={()=>setPinned(selectedFolders(),true)}>{t('settings-folders-pin-selected')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setPinned(selectedFolders(),false)}>{t('settings-folders-unpin-selected')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setVisibility(selectedFolders(),true)}>{t('settings-folders-show-selected')}</button>
   <button type="button" class="btn btn--ghost" disabled={selectedFolders().every(folder=>folder.role==='inbox')} onClick={()=>setVisibility(selectedFolders(),false)}>{t('settings-folders-hide-selected')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setSelected(new Set<string>())}>{t('settings-folders-clear')}</button>
  </div></Show>
  <p class={css.meta}>{t('settings-folders-order-help')}</p>
  <div class={folderCss.wrap} role="region" aria-label={t('settings-folders-table')} tabIndex={0}><table class={folderCss.table}><thead><tr><th class={folderCss.selectCell}><input type="checkbox" aria-label={t('settings-folders-select-visible')} disabled={!current()||!visible().length} checked={visible().length>0&&visible().every(folder=>selected().has(folder.id))} ref={input=>createEffect(()=>{input.indeterminate=visible().some(folder=>selected().has(folder.id))&&!visible().every(folder=>selected().has(folder.id));})} onChange={event=>{const checked=event.currentTarget.checked;setSelected(value=>{const next=new Set(value);for(const folder of visible())if(checked)next.add(folder.id);else next.delete(folder.id);return next;});}}/></th><th>{t('settings-folders-name')}</th><th class={folderCss.location}>{t('settings-folders-location')}</th><th>{t('settings-folders-actions')}</th></tr></thead><tbody>
   <For each={visible()}>{folder=>{const index=()=>folders().findIndex(value=>value.id===folder.id);const hidden=()=>folderIsHidden(folder,props.folders(),prefs());return <tr data-selected={selected().has(folder.id)||undefined} data-drop-target={dropTarget()===folder.id||undefined} onContextMenu={event=>{setMenuFolder(folder.id);context.open(event);}} onDragOver={event=>{if(canDrop(folder.id)){event.preventDefault();if(event.dataTransfer)event.dataTransfer.dropEffect='move';setDropTarget(folder.id);}}} onDragLeave={event=>{if(!event.currentTarget.contains(event.relatedTarget as Node|null))setDropTarget(null);}} onDrop={event=>{event.preventDefault();drop(folder.id);}}>
    <td class={folderCss.selectCell}><input type="checkbox" aria-label={t('settings-folders-select',{name:folder.name})} disabled={!current()} checked={selected().has(folder.id)} onChange={()=>toggle(folder.id)}/></td>
    <td><div class={folderCss.name}><button type="button" class={`btn btn--ghost ${folderCss.drag}`} draggable={current()} disabled={!current()} title={t('settings-folders-drag',{name:folder.name})} aria-label={t('settings-folders-drag',{name:folder.name})} onDragStart={event=>{const ids=selected().has(folder.id)?folders().filter(value=>selected().has(value.id)&&pinned(value.id)===pinned(folder.id)).map(value=>value.id):[folder.id];setDragging(ids);if(event.dataTransfer){event.dataTransfer.effectAllowed='move';event.dataTransfer.setData('text/plain',ids.join('\n'));}}} onDragEnd={stopDragging}><Icon name="sort"/></button><Icon name={folderIcon(folder,prefs())}/><span class={folderCss.folderName} title={folderPathLabel(folder)}>{folder.name}</span><Show when={pinned(folder.id)}><Icon name="pin"/></Show></div>
     <div class={folderCss.metadata}><Show when={locationLabel(folder)}><span class={folderCss.mobileLocation}>{locationLabel(folder)}</span></Show><span>{t(hidden()?'settings-folders-hidden':'settings-folders-visible')}</span><Show when={rowCounts(folder)}><span>{rowCounts(folder)}</span></Show><Show when={prefs().folderLabels?.[folder.id]?.length}><span>{prefs().folderLabels?.[folder.id]?.map(id=>props.tags?.().find(tag=>tag.id===id)?.name??id).join(', ')}</span></Show></div></td>
    <td class={folderCss.location}><span title={locationLabel(folder)}>{locationLabel(folder)||t('settings-folders-root')}</span></td>
    <td><div class={folderCss.actions}>
     <button type="button" class="btn btn--ghost" disabled={!current()} title={t('settings-folders-edit',{name:folder.name})} aria-label={t('settings-folders-edit',{name:folder.name})} onClick={()=>void navigation.navigate({section:'folders',editor:{kind:'edit',id:folder.id}})}><Icon name="edit"/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()} title={t('settings-folders-pin',{name:folder.name})} aria-label={t('settings-folders-pin',{name:folder.name})} aria-pressed={pinned(folder.id)} onClick={()=>pin(folder.id)}><Icon name="star"/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()||folder.role==='inbox'} title={t(hidden()?'settings-folders-show':'settings-folders-hide',{name:folder.name})} aria-label={t(hidden()?'settings-folders-show':'settings-folders-hide',{name:folder.name})} onClick={()=>setVisibility([folder],hidden())}><Icon name={hidden()?'eyeOff':'eye'}/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()} aria-label={t('settings-folders-more',{name:folder.name})} title={t('settings-folders-more',{name:folder.name})} aria-haspopup="menu" aria-expanded={context.isOpen()&&menuFolder()===folder.id} onClick={event=>{setMenuFolder(folder.id);context.openBelow(event.currentTarget);}}><Icon name="more"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('settings-folders-up',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()-1]?.id)} onClick={()=>move(folder.id,folders()[index()-1]!.id)}><Icon name="chevronUp"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('settings-folders-down',{name:folder.name})} disabled={!canMove(folder.id,folders()[index()+1]?.id)} onClick={()=>move(folder.id,folders()[index()+1]!.id)}><Icon name="chevronDown"/></button>
    </div></td>
   </tr>;}}</For>
   <Show when={!visible().length}><tr><td colSpan={4} class={folderCss.empty}>{t('settings-folders-empty')}</td></tr></Show>
  </tbody></table></div>
  {context.menu()}
  <Show when={creating()}>{value=><CreateFolderDialog {...(value().parentId?{parentId:value().parentId!}:{})} onClose={()=>setCreating(undefined)}/>}</Show>
 </section>}>{folder=><FolderIconEditor folder={folder} prefs={prefs} update={update} navigation={navigation} tags={props.tags??(()=>[])} local={!!props.onChange}/>}</Show>;
}
