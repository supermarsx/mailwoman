import {createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {loadCatalog,t} from '../i18n';
import {EmptyFolderDialog,useCanEmpty,useEmptyAbility} from './EmptyFolderDialog.tsx';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import * as css from './contextMenu.css.ts';
export function FolderContextMenu(props:{box:Mailbox;children:JSX.Element;onOpen:()=>void;onOpenTab?:()=>void;onSettings:()=>void;onShare:()=>void;canShare:()=>boolean;onProbe:()=>void}):JSX.Element {
 const app=useApp();const [empty,setEmpty]=createSignal(false);const ability=useEmptyAbility(),canEmpty=useCanEmpty();
 onMount(()=>{void loadCatalog('folder-tabs');void loadCatalog('empty-folder');});
 async function selectThen(run:()=>void):Promise<void>{const owner=preferenceAccount();if(!owner)return;await app.selectMailbox(props.box.id);if(preferenceAccountIsCurrent(owner)&&app.selectedMailboxId()===props.box.id)run();}
 const context=createContextMenu({label:()=>props.box.name,scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}:${props.box.id}`,actions:()=>{
  const owner=preferenceAccount();if(!owner||owner.id!==app.accountId())return [];
  const box=app.mailboxes().find(value=>value.id===props.box.id);if(!box)return [];
  const pinned=accountUiPreferences().folderPinned??[];
  const rows:ContextAction[]=[{id:'open',label:t('mail-context-open-folder'),icon:'folder',run:props.onOpen},
   {id:'pin',label:t(pinned.includes(box.id)?'mail-unpin':'mail-pin'),icon:'pin',run:()=>{const current=accountUiPreferences().folderPinned??[];setAccountUiPreferences({folderPinned:current.includes(box.id)?current.filter(id=>id!==box.id):[...current,box.id]});}},
   {id:'refresh',label:t('mail-context-refresh'),icon:'refresh',run:()=>selectThen(()=>{void app.refreshCurrentMailbox();})},
   {id:'settings',label:t('mail-context-folder-settings'),icon:'settings',run:props.onSettings}];
  if(props.onOpenTab)rows.splice(1,0,{id:'open-tab',label:t('folder-tabs-open'),icon:'folder',run:props.onOpenTab});
  if(box.mailwomanSearchQuery===undefined&&props.canShare())rows.push({id:'sharing',label:t('mail-nav-sharing'),icon:'users',run:()=>selectThen(props.onShare)});
  // Any real folder (t45): ordinary folders move to Trash, Trash and Junk empty for good.
  if(canEmpty(box))rows.push({id:'empty',label:t('empty-folder-action'),icon:'delete',run:()=>{if(app.mailboxes().some(value=>value.id===box.id)&&canEmpty(box))setEmpty(true);}});
  return rows;
 },onError:()=>app.showToast('error',t('mail-context-failed'))});
 onCleanup(observePreferenceAccount(()=>{context.close(false);setEmpty(false);}));
 return <div class={css.folderRow} onContextMenu={e=>{props.onProbe();context.open(e);}} onKeyDown={e=>{if(e.key==='ContextMenu'||(e.shiftKey&&e.key==='F10'))props.onProbe();context.keydown(e);}}>{props.children}{context.menu()}<Show when={empty()&&ability()}>{value=><EmptyFolderDialog box={app.mailboxes().find(item=>item.id===props.box.id)??props.box} ability={value()} onClose={()=>setEmpty(false)}/>}</Show></div>;
}
