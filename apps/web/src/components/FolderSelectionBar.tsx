import {createMemo,createSignal,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {deletePlan,setFoldersSubscribed} from '../api/folderBulkDelete.ts';
import {canEmpty} from '../api/emptyFolder.ts';
import {t,loadCatalog} from '../i18n';
import {useRenameContext} from './RenameFolder.tsx';
import {useEmptyAbility} from './EmptyFolderDialog.tsx';
import {BulkDeleteFoldersDialog} from './BulkDeleteFoldersDialog.tsx';
import {BulkEmptyFoldersDialog} from './BulkEmptyFoldersDialog.tsx';
import {runMarkAllRead} from './folderOps.ts';
import {Icon} from './Icon.tsx';
import {toast} from './toast/toast.ts';
import {clearSelectedFolders} from './folderSelection.ts';
import * as css from './folderSelectionBar.css.ts';
/**
 * The floating bar over the sidebar's folder list while folders are selected: Delete, Empty, Mark all read, Move to,
 * Pin, Star, Subscribe and Unsubscribe for the whole set. What the account or the administrator does not allow is
 * disabled with its reason as the tooltip (system folders are never deleted; Empty needs the backend's ability).
 */
export function FolderSelectionBar(props:{ids:ReadonlySet<string>;onMove:(ids:string[])=>void}):JSX.Element {
 const app=useApp(),ability=useRenameContext(),emptyAbility=useEmptyAbility();
 void loadCatalog('folder-dnd');void loadCatalog('folder-select');
 const [deleting,setDeleting]=createSignal<string[]|null>(null),[emptying,setEmptying]=createSignal<string[]|null>(null);
 const chosen=createMemo(()=>app.mailboxes().filter(box=>props.ids.has(box.id)));
 const manages=()=>ability()?.manages===true;
 const deletable=()=>deletePlan(chosen(),app.mailboxes(),ability()).roots.length>0;
 const emptiable=()=>{const a=emptyAbility();return !!a&&chosen().some(box=>canEmpty(box,app.mailboxes(),a));};
 const prefs=()=>accountUiPreferences();
 const allIn=(list:readonly string[]|undefined)=>chosen().length>0&&chosen().every(box=>list?.includes(box.id));
 const toggle=(key:'folderPinned'|'folderStarred')=>{
  const current=prefs()[key]??[],ids=chosen().map(box=>box.id);
  setAccountUiPreferences({[key]:allIn(current)?current.filter(id=>!ids.includes(id)):[...new Set([...current,...ids])]});
 };
 async function subscribe(value:boolean):Promise<void>{
  const owner=preferenceAccount();if(!owner||!manages())return;
  try{
   const result=await setFoldersSubscribed(createConfiguredClient(),owner.id,chosen().map(box=>box.id),value,{current:()=>preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id});
   await app.refreshFolders().catch(()=>undefined);
   if(result.failed.length)toast.error(t('settings-folders-subscribe-partial',{done:result.updated.length,failed:result.failed.length}));
   else toast.success(t(value?'settings-folders-subscribed':'settings-folders-unsubscribed',{count:result.updated.length}));
  }catch{toast.error(t('settings-folders-subscribe-failed'));}
 }
 function markRead():void{
  const owner=preferenceAccount(),accountId=app.accountId();if(!owner||!accountId||owner.id!==accountId)return;
  const targets=chosen().filter(box=>box.myRights?.maySetSeen!==false&&box.mailwomanSearchQuery===undefined).map(box=>box.id);
  void runMarkAllRead({app,client:createConfiguredClient(),accountId,current:()=>preferenceAccountIsCurrent(owner)&&app.accountId()===accountId},t('folder-select-count',{count:targets.length}),targets);
 }
 const btn=(testId:string,label:string,icon:Parameters<typeof Icon>[0]['name'],run:()=>void,disabled=false,title?:string)=>
  <button type="button" class={`btn btn--ghost ${css.action}`} data-testid={testId} disabled={disabled} {...(title?{title}:{})} onClick={run}><Icon name={icon}/><span>{label}</span></button>;
 return <div class={css.bar} role="toolbar" aria-label={t('folder-bar-label')} data-testid="folder-selection-bar">
  <span class={css.count} role="status">{t('folder-select-count',{count:props.ids.size})}</span>
  {btn('bar-delete',t('settings-folders-delete-selected'),'delete',()=>setDeleting([...props.ids]),!manages()||!deletable(),!manages()?t('folder-ops-delete-unavailable'):!deletable()?t('settings-folders-bulk-delete-none'):undefined)}
  {btn('bar-empty',t('folder-bar-empty'),'delete',()=>setEmptying([...props.ids]),!emptiable(),emptiable()?undefined:t('folder-ops-move-unavailable'))}
  {btn('bar-read',t('folder-bar-mark-read'),'mailRead',markRead)}
  {btn('bar-move',t('folder-dnd-arrange-menu'),'folder',()=>props.onMove([...props.ids]))}
  {btn('bar-pin',t(allIn(prefs().folderPinned)?'folder-bar-unpin':'folder-bar-pin'),'pin',()=>toggle('folderPinned'))}
  {btn('bar-star',t(allIn(prefs().folderStarred)?'folder-bar-unstar':'folder-bar-star'),'star',()=>toggle('folderStarred'))}
  {btn('bar-subscribe',t('settings-folders-subscribe-selected'),'check',()=>void subscribe(true),!manages())}
  {btn('bar-unsubscribe',t('settings-folders-unsubscribe-selected'),'close',()=>void subscribe(false),!manages())}
  {btn('bar-clear',t('settings-folders-clear'),'close',clearSelectedFolders)}
  <Show when={deleting()}>{ids=><BulkDeleteFoldersDialog ids={ids()} context={ability} onDone={()=>clearSelectedFolders()} onClose={()=>setDeleting(null)}/>}</Show>
  <Show when={emptying()&&emptyAbility()}>{a=><BulkEmptyFoldersDialog ids={emptying()!} ability={a()} onClose={()=>setEmptying(null)}/>}</Show>
 </div>;
}
