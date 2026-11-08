import {createEffect, createMemo, createSignal, For, onCleanup, onMount, Show, useContext, type Accessor, type JSX} from 'solid-js';
import {AppContext} from '../../state/context.ts';
import {QuotaDetails} from '../../components/QuotaMeter.tsx';
import {FolderIconPicker} from './FolderIconPicker.tsx';
import {FolderRoles} from './FolderRoles.tsx';
import {FolderCountPreferences} from './FolderCountPreferences.tsx';
import {PropertiesPreferences} from './PropertiesPreferences.tsx';
import {isPropertiesShortcut,openFolderProperties} from '../../components/properties/host.tsx';
import {folderDialogEnabled} from '../../components/properties/policy.ts';
import {FocusedInboxPreference} from './FocusedInboxPreference.tsx';
import {FocusedSenders} from './FocusedSenders.tsx';
import {Icon} from '../../components/Icon.tsx';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import {CreateFolderDialog} from '../../components/FolderMaintenance.tsx';
import {useRenameContext} from '../../components/RenameFolder.tsx';
import {createContextMenu, type ContextAction} from '../../components/ContextMenu.tsx';
import {createFolderLayout, serverFolderMover} from '../../components/folderLayout.ts';
import {arrangedOnly, placementForKey, type DndFolder, type KeyMove} from '../../components/folderDnd.ts';
import {treeRows} from '../../components/folderTree.ts';
import {FolderNestDialog} from '../../components/FolderNestDialog.tsx';
import {FolderMoveToDialog} from '../../components/FolderMoveToDialog.tsx';
import {FolderArrangeCard} from './FolderArrangeCard.tsx';
import {BulkDeleteFoldersDialog} from '../../components/BulkDeleteFoldersDialog.tsx';
import {deletePlan,setFoldersSubscribed} from '../../api/folderBulkDelete.ts';
import {toast} from '../../components/toast/toast.ts';
import {BulkEmptyFoldersDialog} from '../../components/BulkEmptyFoldersDialog.tsx';
import {useEmptyAbility} from '../../components/EmptyFolderDialog.tsx';
import {canEmpty} from '../../api/emptyFolder.ts';
import {applyClick} from '../../components/folderSelectionModel.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import * as dndCss from '../../components/folderDnd.css.ts';
import type {Tag} from '../../state/slices/tags.ts';
import {accountUiPreferences, accountUiPreferenceStatus, retryAccountUiPreferences, setAccountUiPreferences, type AccountUiPreferences, type FolderIcon} from '../../api/accountPreferences.ts';
import {observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {t, loadCatalog} from '../../i18n';
import {createSettingsNavigation, type SettingsNavigation} from './navigation.tsx';
import {folderIcon, folderIsHidden, folderPathLabel, shownFolderIds, type PreferenceFolder} from './folderPreferenceModel.ts';
import {ControlRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as folderCss from './FolderPreferences.css.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('properties');
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
 // One arrangement model for the sidebar and this list (components/folderLayout.ts): order, visual parents, pins, admin policy.
 const layout=createFolderLayout({
  folders:()=>props.folders() as unknown as readonly DndFolder[],prefs,update,accountId:()=>owner?.id??null,
  ...(!props.onChange&&app?{moveOnServer:serverFolderMover(app as never,ability,createConfiguredClient()),serverAvailable:()=>ability()?.manages===true}:{}),
 });
 const folders=createMemo(()=>layout.arranger.visual() as unknown as PreferenceFolder[]);
 const editing=()=>props.folders().find(folder=>navigation.route().editor?.kind==='edit'&&navigation.route().editor?.id===folder.id);
 const [query,setQuery]=createSignal(''),[selected,setSelected]=createSignal<ReadonlySet<string>>(new Set<string>());
 const [moveTo,setMoveTo]=createSignal<string[]|null>(null),[bulkDelete,setBulkDelete]=createSignal<string[]|null>(null),[bulkEmpty,setBulkEmpty]=createSignal<string[]|null>(null);
 const [anchor,setAnchor]=createSignal<string|null>(null),[rangeAdded,setRangeAdded]=createSignal<readonly string[]>([]);let wrapEl:HTMLDivElement|undefined;
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
 const filtering=()=>query().trim()!=='';
 const rowsAll=createMemo(()=>treeRows(folders() as unknown as DndFolder[],()=>true,{sorts:prefs().folderSort,collapsed:new Set(prefs().folderCollapsed??[])}));
 const tableRows=createMemo(()=>filtering()?visible().map(box=>({box:box as unknown as DndFolder,depth:0,hasChildren:false,expanded:false})):rowsAll());
 function setExpanded(id:string,expanded:boolean):void {const next=new Set(prefs().folderCollapsed??[]);if(expanded)next.delete(id);else next.add(id);update({folderCollapsed:[...next]});}
 const dnd=layout.dnd({rows:()=>rowsAll().map(row=>({id:row.box.id,depth:row.depth,hasChildren:row.hasChildren,expanded:row.expanded})),selected,scroller:()=>wrapEl,indent:()=>20,canDrag:()=>!filtering()&&current(),onExpand:id=>setExpanded(id,true)});
 const keyOk=(id:string,move:KeyMove):boolean=>current()&&placementForKey(layout.arranger.visual(),id,move,layout.policy()).ok;
 function keyMove(id:string,move:KeyMove):void {const result=placementForKey(layout.arranger.visual(),id,move,layout.policy());if(result.ok)layout.arranger.place([id],result.placement);}
 const arranged=createMemo(()=>new Set(arrangedOnly(props.folders() as unknown as DndFolder[],prefs().folderParent)));
 const starred=(id:string)=>prefs().folderStarred?.includes(id)??false;
 function star(id:string):void {const ids=(prefs().folderStarred??[]).filter(value=>value!==id);update({folderStarred:starred(id)?ids:[...ids,id]});}
 const lockedReason=(id:string)=>{const reason=layout.lockReason(id);return reason==='locked'||reason==='forced'?reason:null;};
 const lockReason=layout.lockReason;
 function pin(id:string):void {if(layout.forced().has(id))return;const ids=(prefs().folderPinned??[]).filter(value=>value!==id);update({folderPinned:pinned(id)?ids:[...ids,id]});}
 function toggle(id:string):void {if(!current())return;setSelected(value=>{const next=new Set(value);if(next.has(id))next.delete(id);else next.add(id);return next;});}
 function setVisibility(targets:readonly PreferenceFolder[],show:boolean):void {
  let hidden=[...(prefs().folderHidden??[])];
  if(show)for(const folder of targets)hidden=shownFolderIds(folder,props.folders(),hidden);
  else hidden=[...new Set([...hidden,...targets.filter(folder=>folder.role!=='inbox').map(folder=>folder.id)])];
  const inboxIds=new Set(props.folders().filter(folder=>folder.role==='inbox').map(folder=>folder.id));
  update({folderHidden:hidden.filter(id=>!inboxIds.has(id))});
 }
 /** Bulk delete and subscribe change the mail server, so they need a signed-in mailbox account that manages its folders. */
 const emptyAbility=!props.onChange&&app?useEmptyAbility():()=>null;
 const emptiable=()=>{const a=emptyAbility();return !!a&&!!app&&app.mailboxes().some(box=>selected().has(box.id)&&canEmpty(box,app.mailboxes(),a));};
 /** Click on a row checkbox: toggles it, or with Shift selects the range from the last clicked row in the order shown. */
 function pick(id:string,event:MouseEvent):void {
  if(!current())return;
  const next=applyClick({selected:selected(),anchor:anchor(),rangeAdded:rangeAdded()},tableRows().map(row=>row.box.id),id,{shift:event.shiftKey,toggle:true});
  setSelected(next.selected);setAnchor(next.anchor);setRangeAdded(next.rangeAdded);
 }
 const userFolders=()=>folders().filter(folder=>!folder.role&&(folder as unknown as {mailwomanSearchQuery?:string}).mailwomanSearchQuery===undefined);
 const serverOk=()=>!props.onChange&&!!app&&ability()?.manages===true&&current();
 const deletable=()=>app?deletePlan(app.mailboxes().filter(box=>selected().has(box.id)),app.mailboxes(),ability()).roots.length>0:false;
 async function subscribe(targets:readonly PreferenceFolder[],value:boolean):Promise<void> {
  const accountId=app?.accountId();if(!app||!owner||!accountId||!current())return;
  const ids=targets.map(folder=>folder.id);
  try{
   const result=await setFoldersSubscribed(createConfiguredClient(),owner.id,ids,value,{current:()=>preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id});
   await app.refreshFolders().catch(()=>undefined);
   if(result.failed.length)toast.error(t('settings-folders-subscribe-partial',{done:result.updated.length,failed:result.failed.length}));
   else toast.success(t(value?'settings-folders-subscribed':'settings-folders-unsubscribed',{count:result.updated.length}));
  }catch{toast.error(t('settings-folders-subscribe-failed'));}
 }
 function setPinned(targets:readonly PreferenceFolder[],value:boolean):void {
  const ids=new Set(prefs().folderPinned??[]);for(const folder of targets)if(value)ids.add(folder.id);else ids.delete(folder.id);
  update({folderPinned:[...ids]});
 }
 const context=createContextMenu({label:()=>t('settings-folders-actions'),scope:()=>`${owner?.id}:${owner?.generation}:${ownerCurrent()}:${menuFolder()}`,actions:()=>{
  const folder=folders().find(value=>value.id===menuFolder());if(!folder||!current())return [];
  const actions:ContextAction[]=[
   {id:'select',label:t('settings-folders-select',{name:folder.name}),icon:'check',checked:selected().has(folder.id),run:()=>toggle(folder.id)},
   {id:'up',label:t('settings-folders-up',{name:folder.name}),icon:'chevronUp',disabled:!keyOk(folder.id,'up'),run:()=>keyMove(folder.id,'up')},
   {id:'down',label:t('settings-folders-down',{name:folder.name}),icon:'chevronDown',disabled:!keyOk(folder.id,'down'),run:()=>keyMove(folder.id,'down')},
   {id:'indent',label:t('folder-dnd-indent',{name:folder.name}),icon:'chevron',disabled:!keyOk(folder.id,'indent'),run:()=>keyMove(folder.id,'indent')},
   {id:'outdent',label:t('folder-dnd-outdent',{name:folder.name}),icon:'chevronUp',disabled:!keyOk(folder.id,'outdent'),run:()=>keyMove(folder.id,'outdent')},
   {id:'move-to',label:t('folder-dnd-arrange-menu'),icon:'folder',disabled:lockReason(folder.id)!==null,run:()=>{setMoveTo([folder.id]);}},
   {id:'star',label:t(starred(folder.id)?'folder-star-remove':'folder-star-add'),icon:starred(folder.id)?'starOutline':'star',run:()=>star(folder.id)},
  ];
  {const live=app?.mailboxes().find(box=>box.id===folder.id);if(live&&folderDialogEnabled())actions.push({id:'properties',label:t('props-menu-properties'),icon:'info',separatorBefore:true,run:()=>{openFolderProperties(live,lockReason(folder.id)===null?{onMove:()=>setMoveTo([folder.id])}:undefined);}});}
  if(ability()?.manages&&app?.mailboxes().find(box=>box.id===folder.id)?.myRights?.mayCreateChild===true)actions.push({id:'create',label:t('folder-create'),icon:'folder',separatorBefore:true,run:()=>{setCreating({parentId:folder.id});}});
  return actions;
 }});
 onCleanup(observePreferenceAccount(()=>{setOwnerCurrent(!!owner&&preferenceAccountIsCurrent(owner));setSelected(new Set<string>());dnd.end();setCreating(undefined);context.close(false);}));
 onMount(()=>{void loadCatalog('folder-dnd');});
 createEffect(()=>{
  const ids=new Set(folders().map(folder=>folder.id));
  setSelected(value=>{const valid=new Set([...value].filter(id=>ids.has(id)));return valid.size===value.size?value:valid;});
  if(dnd.dragging().some(id=>!ids.has(id)))dnd.end();
  if(menuFolder()&&!ids.has(menuFolder()!))context.close(false);
  ability();
 });
 const rowCounts=(folder:PreferenceFolder):string=>{
  const counts=folder as PreferenceFolder&{totalEmails?:number;unreadEmails?:number};
  return typeof counts.totalEmails==='number'?t('settings-folders-counts',{unread:counts.unreadEmails??0,total:counts.totalEmails}):'';
 };
 return <Show when={editing()} keyed fallback={<div data-testid="folder-preferences">
  <SettingsCard id="folders-card-display" title={t('settings-folders-title')} description={t('settings-folders-help')}>
  <ControlRow controlId="folders-collapse-user" label={t('settings-folders-collapse-user')} help={t('settings-folders-hidden-help')} note={<p role="status" class={css.meta}>{t(`settings-folders-sync-${props.onChange?'synced':accountUiPreferenceStatus()}`)}</p>}>{describedBy=>
   <input id="folders-collapse-user" type="checkbox" checked={prefs().userFoldersCollapsed===true} disabled={!current()} aria-describedby={describedBy()} onChange={event=>update({userFoldersCollapsed:event.currentTarget.checked})}/>
  }</ControlRow>
  <Show when={!props.onChange&&app&&ability()?.manages&&current()}><button type="button" class="btn btn--ghost" onClick={()=>setCreating({})}><Icon name="folder"/>{t('folder-create')}</button></Show>
  </SettingsCard>
  <Show when={!props.onChange}><FocusedInboxPreference/></Show>
  <Show when={!props.onChange?app:undefined}>{state=><FocusedSenders senders={state().focusedSenders} onRemove={email=>state().removeSenderFocus(email)}/>}</Show>
  <Show when={!props.onChange}><FolderCountPreferences/></Show>
  <Show when={!props.onChange}><PropertiesPreferences/></Show>
  <Show when={!props.onChange&&app?.accountId()}>{accountId=><QuotaDetails accountId={accountId()} refresh={props.folders()}/>}</Show>
  <Show when={!props.onChange}><FolderRoles folders={props.folders} {...(props.onRolesSaved?{onSaved:props.onRolesSaved}:{})}/></Show>
  <FolderArrangeCard layout={layout} prefs={prefs} update={update} disabled={!current()}/>
  <SettingsCard id="folders-card-list" wide title={t('settings-folders-list-card')}>
  <div class={folderCss.tools}>
   <label class={folderCss.search}><Icon name="search"/><input class={css.input} type="search" aria-label={t('settings-folders-search')} placeholder={t('settings-folders-search')} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
   <button type="button" class="btn btn--ghost" disabled={!current()||!visible().length} onClick={()=>setSelected(value=>new Set([...value,...visible().map(folder=>folder.id)]))}>{t('settings-folders-select-visible')}</button>
   <button type="button" class="btn btn--ghost" disabled={!current()||!folders().length} onClick={()=>setSelected(new Set(folders().map(folder=>folder.id)))}>{t('settings-folders-select-all')}</button>
   <button type="button" class="btn btn--ghost" data-testid="select-user-folders" disabled={!current()||!userFolders().length} onClick={()=>{setSelected(new Set(userFolders().map(folder=>folder.id)));setAnchor(null);}}>{t('settings-folders-select-user')}</button>
  </div>
  <Show when={selectedFolders().length}><div class={folderCss.bulk} role="group" aria-label={t('settings-folders-selected',{count:selectedFolders().length})}>
   <span role="status">{t('settings-folders-selected',{count:selectedFolders().length})}</span>
   <button type="button" class="btn btn--ghost" disabled={selectedFolders().some(folder=>!!lockReason(folder.id))} onClick={()=>setMoveTo(selectedFolders().map(folder=>folder.id))}>{t('folder-dnd-selected-move')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setPinned(selectedFolders(),true)}>{t('settings-folders-pin-selected')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setPinned(selectedFolders(),false)}>{t('settings-folders-unpin-selected')}</button>
   <button type="button" class="btn btn--ghost" onClick={()=>setVisibility(selectedFolders(),true)}>{t('settings-folders-show-selected')}</button>
   <button type="button" class="btn btn--ghost" disabled={selectedFolders().every(folder=>folder.role==='inbox')} onClick={()=>setVisibility(selectedFolders(),false)}>{t('settings-folders-hide-selected')}</button>
   <Show when={serverOk()}>
    <button type="button" class="btn btn--ghost" data-testid="bulk-subscribe" onClick={()=>void subscribe(selectedFolders(),true)}>{t('settings-folders-subscribe-selected')}</button>
    <button type="button" class="btn btn--ghost" data-testid="bulk-unsubscribe" onClick={()=>void subscribe(selectedFolders(),false)}>{t('settings-folders-unsubscribe-selected')}</button>
    <button type="button" class="btn btn--ghost" data-testid="bulk-empty" disabled={!emptiable()} title={emptiable()?undefined:t('folder-ops-move-unavailable')} onClick={()=>setBulkEmpty(selectedFolders().map(folder=>folder.id))}><Icon name="delete"/>{t('folder-bar-empty')}</button>
    <button type="button" class="btn btn--ghost" data-testid="bulk-delete" disabled={!deletable()} title={deletable()?undefined:t('settings-folders-bulk-delete-none')} onClick={()=>setBulkDelete(selectedFolders().map(folder=>folder.id))}><Icon name="delete"/>{t('settings-folders-delete-selected')}</button>
   </Show>
   <button type="button" class="btn btn--ghost" onClick={()=>setSelected(new Set<string>())}>{t('settings-folders-clear')}</button>
  </div></Show>
  <p class={css.meta}>{t('settings-folders-order-help')}</p>
  <p class={folderCss.srOnly} role="status" aria-live="polite">{dnd.announcement()}</p>
  <div class={folderCss.wrap} ref={wrapEl} role="region" aria-label={t('settings-folders-table')} tabIndex={0}><table class={folderCss.table}><thead><tr><th class={folderCss.selectCell}><input type="checkbox" aria-label={t('settings-folders-select-visible')} disabled={!current()||!visible().length} checked={visible().length>0&&visible().every(folder=>selected().has(folder.id))} ref={input=>createEffect(()=>{input.indeterminate=visible().some(folder=>selected().has(folder.id))&&!visible().every(folder=>selected().has(folder.id));})} onChange={event=>{const checked=event.currentTarget.checked;setSelected(value=>{const next=new Set(value);for(const folder of visible())if(checked)next.add(folder.id);else next.delete(folder.id);return next;});}}/></th><th>{t('settings-folders-name')}</th><th class={folderCss.location}>{t('settings-folders-location')}</th><th>{t('settings-folders-actions')}</th></tr></thead><tbody>
   <For each={tableRows()}>{row=>{const folder=row.box as unknown as PreferenceFolder;const hidden=()=>folderIsHidden(folder,props.folders(),prefs());return <tr {...dnd.rowProps(folder.id)} style={{'--folder-indent':'1.25rem',...dnd.dropVars(folder.id)}} data-folder-id={folder.id} data-depth={row.depth} data-selected={selected().has(folder.id)||undefined} onKeyDown={event=>{if(folderDialogEnabled()&&isPropertiesShortcut(event)){const live=app?.mailboxes().find(box=>box.id===folder.id);if(live){event.preventDefault();openFolderProperties(live);return;}}dnd.key(event,folder.id);}} onContextMenu={event=>{setMenuFolder(folder.id);context.open(event);}}>
    <td class={folderCss.selectCell}><input type="checkbox" aria-label={t('settings-folders-select',{name:folder.name})} disabled={!current()} checked={selected().has(folder.id)} onClick={event=>pick(folder.id,event)} onChange={()=>undefined}/></td>
    <td><div class={folderCss.name} style={{'padding-inline-start':`${row.depth*1.25}rem`}}>
     <Show when={row.hasChildren} fallback={<span class={folderCss.twistySpacer}/>}><button type="button" class={`btn btn--ghost ${folderCss.twisty}`} aria-expanded={row.expanded} aria-label={t(row.expanded?'folder-ops-collapse':'folder-ops-expand',{name:folder.name})} onClick={()=>setExpanded(folder.id,!row.expanded)}><Icon name={row.expanded?'chevronDown':'chevron'}/></button></Show>
     <button type="button" class={`btn btn--ghost ${folderCss.drag}`} disabled={!current()||!!lockReason(folder.id)||filtering()} title={lockedReason(folder.id)==='locked'?t('folder-dnd-locked-hint'):lockedReason(folder.id)==='forced'?t('folder-dnd-forced-hint'):t('settings-folders-drag',{name:folder.name})} aria-label={t('settings-folders-drag',{name:folder.name})}><Icon name="sort"/></button><Icon name={folderIcon(folder,prefs())}/><span class={folderCss.folderName} title={folderPathLabel(folder)}>{folder.name}</span><Show when={pinned(folder.id)||layout.forced().has(folder.id)}><Icon name="pin"/></Show><Show when={starred(folder.id)}><span class="sidebar__box-star" aria-hidden="true"><Icon name="star"/></span><span class={folderCss.srOnly}>{t('folder-star-starred')}</span></Show><Show when={lockedReason(folder.id)}><span class={dndCss.lockIcon} data-testid="folder-lock" title={lockedReason(folder.id)==='locked'?t('folder-dnd-locked-hint'):t('folder-dnd-forced-hint')}><Icon name="lock"/><span class={folderCss.srOnly}>{t('folder-dnd-locked-row')}</span></span></Show></div>
     <div class={folderCss.metadata}><Show when={locationLabel(folder)}><span class={folderCss.mobileLocation}>{locationLabel(folder)}</span></Show><span>{t(hidden()?'settings-folders-hidden':'settings-folders-visible')}</span><Show when={rowCounts(folder)}><span>{rowCounts(folder)}</span></Show><Show when={prefs().folderLabels?.[folder.id]?.length}><span>{prefs().folderLabels?.[folder.id]?.map(id=>props.tags?.().find(tag=>tag.id===id)?.name??id).join(', ')}</span></Show><Show when={arranged().has(folder.id)}><span class={folderCss.badge} data-testid="arranged-only" title={t('folder-dnd-arranged-only-help',{path:folderPathLabel(folder)})}>{t('folder-dnd-arranged-only')}</span><Show when={!props.onChange&&ability()?.manages}><button type="button" class="btn btn--ghost" disabled={!current()} onClick={()=>void layout.arranger.applyToServer(folder.id)}>{t('folder-dnd-apply-server')}</button></Show></Show></div></td>
    <td class={folderCss.location}><span title={locationLabel(folder)}>{locationLabel(folder)||t('settings-folders-root')}</span></td>
    <td><div class={folderCss.actions}>
     <button type="button" class="btn btn--ghost" disabled={!current()} title={t('settings-folders-edit',{name:folder.name})} aria-label={t('settings-folders-edit',{name:folder.name})} onClick={()=>void navigation.navigate({section:'folders',editor:{kind:'edit',id:folder.id}})}><Icon name="edit"/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()||layout.forced().has(folder.id)} title={t('settings-folders-pin',{name:folder.name})} aria-label={t('settings-folders-pin',{name:folder.name})} aria-pressed={pinned(folder.id)||layout.forced().has(folder.id)} onClick={()=>pin(folder.id)}><Icon name="pin"/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()} title={t('folder-star-toggle',{name:folder.name})} aria-label={t('folder-star-toggle',{name:folder.name})} aria-pressed={starred(folder.id)} onClick={()=>star(folder.id)}><Icon name={starred(folder.id)?'star':'starOutline'}/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()||folder.role==='inbox'} title={t(hidden()?'settings-folders-show':'settings-folders-hide',{name:folder.name})} aria-label={t(hidden()?'settings-folders-show':'settings-folders-hide',{name:folder.name})} onClick={()=>setVisibility([folder],hidden())}><Icon name={hidden()?'eyeOff':'eye'}/></button>
     <button type="button" class="btn btn--ghost" disabled={!current()} aria-label={t('settings-folders-more',{name:folder.name})} title={t('settings-folders-more',{name:folder.name})} aria-haspopup="menu" aria-expanded={context.isOpen()&&menuFolder()===folder.id} onClick={event=>{setMenuFolder(folder.id);context.openBelow(event.currentTarget);}}><Icon name="more"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('settings-folders-up',{name:folder.name})} disabled={!keyOk(folder.id,'up')} onClick={()=>keyMove(folder.id,'up')}><Icon name="chevronUp"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('settings-folders-down',{name:folder.name})} disabled={!keyOk(folder.id,'down')} onClick={()=>keyMove(folder.id,'down')}><Icon name="chevronDown"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('folder-dnd-indent',{name:folder.name})} disabled={!keyOk(folder.id,'indent')} onClick={()=>keyMove(folder.id,'indent')}><Icon name="chevron"/></button>
     <button type="button" class={`btn btn--ghost ${folderCss.keyboardAction}`} aria-label={t('folder-dnd-outdent',{name:folder.name})} disabled={!keyOk(folder.id,'outdent')} onClick={()=>keyMove(folder.id,'outdent')}><Icon name="chevronUp"/></button>
    </div></td>
   </tr>;}}</For>
   <Show when={!visible().length}><tr><td colSpan={4} class={folderCss.empty}>{t('settings-folders-empty')}</td></tr></Show>
  </tbody></table></div>
  </SettingsCard>
  {context.menu()}
  <FolderNestDialog arranger={layout.arranger} nameOf={id=>props.folders().find(folder=>folder.id===id)?.name??''}/>
  <Show when={bulkDelete()&&serverOk()&&app?bulkDelete():null}>{ids=><BulkDeleteFoldersDialog ids={ids()} context={ability} onDone={deleted=>setSelected(value=>new Set([...value].filter(id=>!deleted.includes(id))))} onClose={()=>setBulkDelete(null)}/>}</Show>
  <Show when={bulkEmpty()&&serverOk()&&emptyAbility()}>{a=><BulkEmptyFoldersDialog ids={bulkEmpty()!} ability={a()} onClose={()=>setBulkEmpty(null)}/>}</Show>
  <Show when={moveTo()}>{ids=><FolderMoveToDialog moving={ids()} arranger={layout.arranger} policy={layout.policy} onClose={()=>setMoveTo(null)}/>}</Show>
  <Show when={creating()}>{value=><CreateFolderDialog {...(value().parentId?{parentId:value().parentId!}:{})} onClose={()=>setCreating(undefined)}/>}</Show>
 </div>}>{folder=><FolderIconEditor folder={folder} prefs={prefs} update={update} navigation={navigation} tags={props.tags??(()=>[])} local={!!props.onChange}/>}</Show>;
}
