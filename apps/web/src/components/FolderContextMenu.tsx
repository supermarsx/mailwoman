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
import {useActions} from '../keymap/index.ts';
import * as renameCss from './renameFolder.css.ts';
import * as css from './contextMenu.css.ts';
export function FolderContextMenu(props:{box:Mailbox;children:JSX.Element;onOpen:()=>void;onOpenTab?:()=>void;onHidden?:()=>void;onSettings:()=>void;onShare:()=>void;canShare:()=>boolean;onProbe:()=>void}):JSX.Element {
 const app=useApp();const [empty,setEmpty]=createSignal(false);const [deleting,setDeleting]=createSignal(false);const ability=useEmptyAbility(),canEmpty=useCanEmpty();
 const renameContext=useRenameContext(),canRename=useCanRename();const [renaming,setRenaming]=createSignal<'inline'|'dialog'|null>(null);let row:HTMLDivElement|undefined;
 // Ask for the empty-folder and rename abilities now, so the first menu opened already has them.
 onMount(()=>{void loadCatalog('folder-tabs');void loadCatalog('empty-folder');void loadCatalog('rename-folder');ability();renameContext();});
 // Rename (t65): inline in the list, a modal on phones and in the compact rail.
 function startRename():void {const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box||!canRename(box)||renaming())return;setRenaming(prefersRenameDialog(row)?'dialog':'inline');}
 function endRename():void {setRenaming(null);queueMicrotask(()=>row?.querySelector<HTMLElement>('.sidebar__box')?.focus());}
 // F2 on a focused folder; every other surface declines so the key does what it did.
 useActions({renameFolder:()=>{if(!row||!row.contains(document.activeElement)||renaming())return false;const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box||!canRename(box))return false;startRename();return true;}});
 async function selectThen(run:()=>void):Promise<void>{const owner=preferenceAccount();if(!owner)return;await app.selectMailbox(props.box.id);if(preferenceAccountIsCurrent(owner)&&app.selectedMailboxId()===props.box.id)run();}
 const context=createContextMenu({label:()=>props.box.name,scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}:${props.box.id}`,actions:()=>{
  const owner=preferenceAccount();if(!owner||owner.id!==app.accountId())return [];
  const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box)return [];
  const pinned=accountUiPreferences().folderPinned??[];
  const rows:ContextAction[]=[{id:'open',label:t('mail-context-open-folder'),icon:'folder',run:props.onOpen},
   {id:'pin',label:t(pinned.includes(box.id)?'mail-unpin':'mail-pin'),icon:'pin',run:()=>{const current=accountUiPreferences().folderPinned??[];setAccountUiPreferences({folderPinned:current.includes(box.id)?current.filter(id=>id!==box.id):[...current,box.id]});}},
   {id:'refresh',label:t('mail-context-refresh'),icon:'refresh',run:()=>selectThen(()=>{void app.refreshCurrentMailbox();})},
   {id:'settings',label:t('mail-context-folder-settings'),icon:'settings',run:props.onSettings}];
  if(box.role!=='inbox')rows.push({id:'hide',label:t('mail-context-hide-folder'),icon:'eye',run:()=>{
   const hidden=accountUiPreferences().folderHidden??[];
   if(!hidden.includes(box.id))setAccountUiPreferences({folderHidden:[...hidden,box.id]});
   props.onHidden?.();
  }});
  if(props.onOpenTab)rows.splice(1,0,{id:'open-tab',label:t('folder-tabs-open'),icon:'folder',run:props.onOpenTab});
  if(box.mailwomanSearchQuery===undefined&&props.canShare())rows.push({id:'sharing',label:t('mail-nav-sharing'),icon:'users',run:()=>selectThen(props.onShare)});
  if(canRename(box))rows.push({id:'rename',label:t('rename-folder-action'),icon:'edit',run:startRename});
  if(canDeleteFolder(box,app.mailboxes(),renameContext()))rows.push({id:'delete-folder',label:t('delete-folder-action'),icon:'delete',run:()=>{const target=app.mailboxes().find(value=>value.id===box.id);if(target&&canDeleteFolder(target,app.mailboxes(),renameContext()))setDeleting(true);}});
  // Any real folder (t45): ordinary folders move to Trash, Trash and Junk empty for good.
  if(canEmpty(box))rows.push({id:'empty',label:t('empty-folder-action'),icon:'delete',run:()=>{if(app.mailboxes().some(value=>value.id===box.id)&&canEmpty(box))setEmpty(true);}});
  return rows;
 },onError:()=>app.showToast('error',t('mail-context-failed'))});
 onCleanup(observePreferenceAccount(()=>{context.close(false);setEmpty(false);setRenaming(null);setDeleting(false);}));
 return <div ref={row} class={css.folderRow} classList={{[renameCss.editing]:renaming()==='inline'}} onContextMenu={e=>{if(renaming()==='inline')return;props.onProbe();context.open(e);}} onKeyDown={e=>{if(renaming()==='inline')return;if(e.key==='ContextMenu'||(e.shiftKey&&e.key==='F10'))props.onProbe();context.keydown(e);}}>{props.children}<Show when={renaming()==='inline'&&renameContext()}>{value=><FolderRenameField box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} context={value()} onDone={endRename}/>}</Show>{context.menu()}<Show when={deleting()&&renameContext()}>{value=><DeleteFolderDialog box={props.box} context={value()} onClose={()=>{setDeleting(false);queueMicrotask(()=>row?.querySelector<HTMLElement>('.sidebar__box')?.focus());}}/>}</Show><Show when={renaming()==='dialog'&&renameContext()}>{value=><RenameFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} context={value()} onClose={endRename}/>}</Show><Show when={empty()&&ability()}>{value=><EmptyFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} ability={value()} onClose={()=>setEmpty(false)}/>}</Show></div>;
}
