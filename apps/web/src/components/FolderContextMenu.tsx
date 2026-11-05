import {CreateFolderDialog,CleanupFolderDialog} from './FolderMaintenance.tsx';
import {createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {loadCatalog,t} from '../i18n';
import {EmptyFolderDialog,useCanEmpty,useEmptyAbility} from './EmptyFolderDialog.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {FolderRenameField,prefersRenameDialog,RenameFolderDialog,useCanRename,useRenameContext} from './RenameFolder.tsx';
import {DeleteFolderDialog} from './DeleteFolder.tsx';
import {canDeleteFolder} from '../api/deleteFolder.ts';
import {canDeleteTree} from '../api/folderStructure.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {orderedFolders} from '../screens/Settings/folderPreferenceModel.ts';
import {descendantsOf,isSystemFolder,moveWithinSiblings,rolledUnread,siblingsOf,sortKey} from './folderTree.ts';
import {folderActivity,loadFolderActivity} from './folderActivity.ts';
import {folderSortAction} from './folderSortAction.ts';
import {runMarkAllRead} from './folderOps.ts';
import {DeleteTreeDialog,MoveFolderDialog,TransferDialog} from './FolderBulkDialogs.tsx';
import {useActions} from '../keymap/index.ts';
import * as renameCss from './renameFolder.css.ts';
import * as css from './contextMenu.css.ts';
export function FolderContextMenu(props:{box:Mailbox;children:JSX.Element;onOpen:()=>void;onOpenTab?:()=>void;onHidden?:()=>void;onSettings:()=>void;onShare:()=>void;canShare:()=>boolean;onProbe:()=>void;/** Nesting level in the folder tree (0: top). */depth?:number;/** Folders picked with the marquee; a menu opened on one of several offers bulk actions. */selection?:()=>ReadonlySet<string>}):JSX.Element {
 const [creating,setCreating]=createSignal(false),[cleaning,setCleaning]=createSignal(false);
 const app=useApp();const [empty,setEmpty]=createSignal(false);const [deleting,setDeleting]=createSignal(false);const [deletingTree,setDeletingTree]=createSignal(false),[moving,setMoving]=createSignal(false),[transfer,setTransfer]=createSignal<'copy'|'move'|null>(null);const ability=useEmptyAbility(),canEmpty=useCanEmpty();
 const renameContext=useRenameContext(),canRename=useCanRename();const [renaming,setRenaming]=createSignal<'inline'|'dialog'|null>(null);let row:HTMLDivElement|undefined;
 // Ask for the empty-folder and rename abilities now, so the first menu opened already has them.
 onMount(()=>{void loadCatalog('folder-select');void loadCatalog('folder-tabs');void loadCatalog('empty-folder');void loadCatalog('rename-folder');ability();renameContext();});
 // Rename (t65): inline in the list, a modal on phones and in the compact rail.
 function startRename():void {const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box||!canRename(box)||renaming())return;setRenaming(prefersRenameDialog(row)?'dialog':'inline');}
 function endRename():void {setRenaming(null);queueMicrotask(()=>row?.querySelector<HTMLElement>('.sidebar__box')?.focus());}
 // F2 on a focused folder; every other surface declines so the key does what it did.
 useActions({renameFolder:()=>{if(!row||!row.contains(document.activeElement)||renaming())return false;const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box||!canRename(box))return false;startRename();return true;}});
 async function selectThen(run:()=>void):Promise<void>{const owner=preferenceAccount();if(!owner)return;await app.selectMailbox(props.box.id);if(preferenceAccountIsCurrent(owner)&&app.selectedMailboxId()===props.box.id)run();}
 const client=createConfiguredClient();
 function loadActivity():void {const owner=preferenceAccount(),accountId=app.accountId();if(!owner||!accountId)return;void loadFolderActivity(client,accountId,app.mailboxes().filter(value=>!value.role&&value.mailwomanSearchQuery===undefined).map(value=>value.id),()=>preferenceAccountIsCurrent(owner)&&app.accountId()===accountId);}
 const context=createContextMenu({label:()=>props.box.name,scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}:${props.box.id}`,actions:()=>{
  const owner=preferenceAccount();if(!owner||owner.id!==app.accountId())return [];
  const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box)return [];
  const pinned=accountUiPreferences().folderPinned??[];
  const boxes=app.mailboxes(),subs=descendantsOf(boxes,box.id),system=isSystemFolder(box),ability=renameContext();
  const rows:ContextAction[]=[{id:'open',label:t('mail-context-open-folder'),icon:'folder',run:props.onOpen}];
  if(props.onOpenTab)rows.push({id:'open-tab',label:t('folder-tabs-open'),icon:'folder',run:props.onOpenTab});
  // Several folders picked with the marquee: mark all of them read at once.
  {const picked=props.selection?.()??new Set<string>();
   if(picked.size>1&&picked.has(box.id)){
    const chosen=boxes.filter(value=>picked.has(value.id)),blocked=chosen.every(value=>value.myRights?.maySetSeen===false||value.mailwomanSearchQuery!==undefined),idle=chosen.every(value=>value.unreadEmails===0);
    rows.push({id:'mark-read-selected',label:t('folder-select-mark-read',{count:chosen.length}),icon:'mailRead',disabled:blocked||idle,run:()=>{const account=preferenceAccount(),accountId=app.accountId();if(!account||!accountId||account.id!==accountId)return;void runMarkAllRead({app,client,accountId,current:()=>preferenceAccountIsCurrent(account)&&app.accountId()===accountId},t('folder-select-count',{count:chosen.length}),chosen.filter(value=>value.myRights?.maySetSeen!==false&&value.mailwomanSearchQuery===undefined).map(value=>value.id));}});}}
  // Mark all as read: this folder, or with its subfolders (a choice when there are any).
  {const own=box.unreadEmails,all=rolledUnread(boxes,box.id),blocked=box.myRights?.maySetSeen===false||box.mailwomanSearchQuery!==undefined;
   const run=(include:boolean)=>()=>{const account=preferenceAccount(),accountId=app.accountId();if(!account||!accountId||account.id!==accountId)return;void runMarkAllRead({app,client,accountId,current:()=>preferenceAccountIsCurrent(account)&&app.accountId()===accountId},box.name,include?[box.id,...subs.map(value=>value.id)]:[box.id]);};
   const title=blocked?t('folder-ops-read-unavailable'):all===0?t('folder-ops-no-unread'):undefined;
   rows.push(subs.length>0
    ?{id:'mark-read',label:t('folder-ops-mark-read'),icon:'mailRead',disabled:blocked||all===0,...(title?{title}:{}),children:[{id:'mark-read:this',label:t('folder-ops-this-folder'),icon:'mailRead',disabled:own===0,run:run(false)},{id:'mark-read:all',label:t('folder-ops-with-subfolders'),icon:'mailRead',run:run(true)}],run:()=>{}}
    :{id:'mark-read',label:t('folder-ops-mark-read'),icon:'mailRead',disabled:blocked||own===0,...(title?{title}:{}),run:run(false)});}
  rows.push({id:'pin',label:t(pinned.includes(box.id)?'mail-unpin':'mail-pin'),icon:'pin',run:()=>{const current=accountUiPreferences().folderPinned??[];setAccountUiPreferences({folderPinned:current.includes(box.id)?current.filter(id=>id!==box.id):[...current,box.id]});}},
   {id:'refresh',label:t('mail-context-refresh'),icon:'refresh',run:()=>selectThen(()=>{void app.refreshCurrentMailbox();})});
  const manages=ability?.manages===true&&box.mailwomanSearchQuery===undefined;
  if(manages&&box.myRights?.mayCreateChild===true)rows.push({id:'create-folder',label:t('folder-ops-new-subfolder'),icon:'folderPlus',separatorBefore:true,run:()=>{setCreating(true);}});
  if(canRename(box))rows.push({id:'rename',label:t('rename-folder-action'),icon:'edit',run:startRename});
  if(subs.some(value=>value.parentId===box.id))rows.push(folderSortAction(box.id,t('folder-ops-sort-subfolders'),()=>loadActivity()));
  if(canEmpty(box))rows.push({id:'cleanup',label:t('folder-cleanup'),icon:'delete',separatorBefore:true,run:()=>{setCleaning(true);}});
  // Any real folder (t45): ordinary folders move to Trash, Trash and Junk empty for good.
  if(canEmpty(box))rows.push({id:'empty',label:t('empty-folder-action'),icon:'delete',run:()=>{if(app.mailboxes().some(value=>value.id===box.id)&&canEmpty(box))setEmpty(true);}});
  rows.push({id:'settings',label:t('mail-context-folder-settings'),icon:'settings',separatorBefore:true,run:props.onSettings});
  if(box.role!=='inbox')rows.push({id:'hide',label:t('mail-context-hide-folder'),icon:'eye',run:()=>{
   const hidden=accountUiPreferences().folderHidden??[];
   if(!hidden.includes(box.id))setAccountUiPreferences({folderHidden:[...hidden,box.id]});
   props.onHidden?.();
  }});
  if(box.mailwomanSearchQuery===undefined&&props.canShare())rows.push({id:'sharing',label:t('mail-nav-sharing'),icon:'users',run:()=>selectThen(props.onShare)});
  // The second-level menu: structural and whole-folder actions.
  if(box.mailwomanSearchQuery===undefined){
   const sysTitle=t('folder-ops-system-folder');
   const siblings=siblingsOf(orderedFolders(boxes,accountUiPreferences()).filter(value=>value.mailwomanSearchQuery===undefined),box,{sorts:accountUiPreferences().folderSort,...(folderActivity(app.accountId())?{activity:folderActivity(app.accountId())!}:{})});
   const at=siblings.findIndex(value=>value.id===box.id);
   const shift=(direction:'up'|'down')=>()=>{
    const next=moveWithinSiblings(accountUiPreferences().folderOrder??[],siblings.map(value=>value.id),box.id,direction);if(!next)return;
    const parent=box.parentId&&boxes.some(value=>value.id===box.parentId)?box.parentId:null;
    setAccountUiPreferences({folderOrder:next,folderSort:{...accountUiPreferences().folderSort,[sortKey(parent)]:{by:'custom',dir:'asc'}}});
   };
   const more:ContextAction[]=[];
   const deletable=canDeleteFolder(box,boxes,ability),tree=!deletable&&subs.length>0&&canDeleteTree(box,boxes,ability);
   more.push({id:'delete-folder',label:t(tree?'folder-ops-delete-with-subfolders':'delete-folder-action'),icon:'delete',disabled:!deletable&&!tree,...(system?{title:sysTitle}:!deletable&&!tree?{title:t('folder-ops-delete-unavailable')}:{}),run:()=>{const target=app.mailboxes().find(value=>value.id===box.id);if(!target)return;if(canDeleteFolder(target,app.mailboxes(),renameContext()))setDeleting(true);else if(canDeleteTree(target,app.mailboxes(),renameContext()))setDeletingTree(true);}});
   const readable=box.myRights?.mayReadItems!==false;
   more.push({id:'copy-all',label:t('folder-ops-copy-all-menu'),icon:'copy',separatorBefore:true,disabled:!readable,...(readable?{}:{title:t('folder-ops-read-unavailable')}),run:()=>{setTransfer('copy');}},
    {id:'move-all',label:t('folder-ops-move-all-menu'),icon:'transfer',disabled:!readable||box.myRights?.mayRemoveItems===false,...(box.myRights?.mayRemoveItems===false?{title:t('folder-ops-move-unavailable')}:{}),run:()=>{setTransfer('move');}});
   const movable=ability?.manages===true&&!system&&box.myRights?.mayRename===true;
   more.push({id:'move-folder',label:t('folder-ops-move-folder-menu'),icon:'folder',separatorBefore:true,disabled:!movable,...(system?{title:sysTitle}:!movable?{title:t('folder-ops-move-unavailable')}:{}),run:()=>{if(!system&&app.mailboxes().some(value=>value.id===box.id))setMoving(true);}});
   more.push({id:'move-up',label:t('folder-ops-move-up'),icon:'chevronUp',disabled:system||at<=0,...(system?{title:sysTitle}:{}),run:shift('up')},
    {id:'move-down',label:t('folder-ops-move-down'),icon:'chevronDown',disabled:system||at<0||at>=siblings.length-1,...(system?{title:sysTitle}:{}),run:shift('down')});
   rows.push({id:'more',label:t('folder-ops-more'),icon:'more',separatorBefore:true,children:more,run:()=>{}});
  }
  return rows;
 },onError:()=>app.showToast('error',t('mail-context-failed'))});
 onCleanup(observePreferenceAccount(()=>{setDeletingTree(false);setMoving(false);setTransfer(null);context.close(false);setEmpty(false);setRenaming(null);setDeleting(false);setCreating(false);setCleaning(false);}));
 return <div ref={row} class={css.folderRow} style={{'--folder-depth':String(props.depth??0)}} data-context-open={context.isOpen()?'true':undefined} classList={{[renameCss.editing]:renaming()==='inline'}} onContextMenu={e=>{if(renaming()==='inline')return;props.onProbe();context.open(e);}} onKeyDown={e=>{if(renaming()==='inline')return;if(e.key==='ContextMenu'||(e.shiftKey&&e.key==='F10'))props.onProbe();context.keydown(e);}}>{props.children}<Show when={creating()}><CreateFolderDialog parentId={props.box.id} onClose={()=>setCreating(false)}/></Show><Show when={cleaning()}><CleanupFolderDialog box={props.box} onClose={()=>setCleaning(false)}/></Show><Show when={renaming()==='inline'&&renameContext()}>{value=><FolderRenameField box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} context={value()} onDone={endRename}/>}</Show>{context.menu()}<Show when={deletingTree()&&renameContext()}>{value=><DeleteTreeDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} context={value()} onClose={()=>setDeletingTree(false)}/>}</Show><Show when={moving()}><MoveFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} onClose={()=>setMoving(false)}/></Show><Show when={transfer()}>{mode=><TransferDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} mode={mode()} onClose={()=>setTransfer(null)}/>}</Show><Show when={deleting()&&renameContext()}>{value=><DeleteFolderDialog box={props.box} context={value()} onClose={()=>{setDeleting(false);queueMicrotask(()=>row?.querySelector<HTMLElement>('.sidebar__box')?.focus());}}/>}</Show><Show when={renaming()==='dialog'&&renameContext()}>{value=><RenameFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} context={value()} onClose={endRename}/>}</Show><Show when={empty()&&ability()}>{value=><EmptyFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} ability={value()} onClose={()=>setEmpty(false)}/>}</Show></div>;
}
