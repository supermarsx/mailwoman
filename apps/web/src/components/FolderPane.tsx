import {createEffect,createSignal,For,onCleanup,onMount,Show,untrack,type JSX} from 'solid-js';
import {SelectionBar,type SelectionAction} from './SelectionBar.tsx';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import * as barCss from './selectionBar.css.ts';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {createFolderView} from './folderView.ts';
import {AsyncError,AsyncPending} from './AsyncState.tsx';
import {createInitialSync} from './initialSync.ts';
import {senderLabel} from './MessageList.tsx';
import {modifierSelectionEnabled} from './mailInteractions.ts';
import {useActions} from '../keymap/index.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {mutateFolderMessages,type FolderMutation} from './folderMutations.ts';
import {startTrashJob} from './emptyTrashJob.ts';
import {EmptyFolderDialog,useCanEmpty,useEmptyAbility} from './EmptyFolderDialog.tsx';
import {loadCatalog,t} from '../i18n/index.ts';
import * as css from './folderPane.css.ts';

export function FolderPane(props:{mailboxId:string;active:boolean;/** Shown beside a message opened from it (t51): next/previous still walk this list. */companion?:boolean;onOpen:(id:string)=>void;registerRefresh?:(refresh:()=>void)=>void;beforeGlobalAction?:()=>void}):JSX.Element{
 const app=useApp();
 const owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();
 const box=()=>app.mailboxes().find(value=>value.id===props.mailboxId);
 const view=createFolderView(client,props.mailboxId,()=>!!box());
 const sync=createInitialSync(()=>props.mailboxId,()=>view.load(view.position()));
 const [busy,setBusy]=createSignal(false),[result,setResult]=createSignal(''),[failed,setFailed]=createSignal(false);
 const [confirmDelete,setConfirmDelete]=createSignal<string[]|null>(null);
 const emptyAbility=useEmptyAbility(),canEmpty=useCanEmpty();const [emptying,setEmptying]=createSignal(false);
 let refreshTimer:ReturnType<typeof setTimeout>|undefined,dirty=false;
 const admitted=()=>!!owner&&preferenceAccountIsCurrent(owner)&&view.current();
 const disabled=()=>busy()||view.loading()||!admitted();
 const role=(name:string)=>app.mailboxes().find(folder=>folder.role===name&&folder.mailwomanSearchQuery===undefined);
 const targets=()=>{const selected=[...view.selection()];if(selected.length)return selected;const focused=viewport?.ownerDocument.activeElement as HTMLElement|null;const id=focused?.closest<HTMLElement>('[data-folder-message]')?.dataset['folderMessage'];return id&&view.messages().some(email=>email.id===id)?[id]:[];};
 async function reconcile(){dirty=false;await Promise.allSettled([view.load(),app.refreshFolders()]);}
 async function run(operation:FolderMutation,ids=targets()):Promise<void>{
  if(disabled()||!ids.length||!owner)return;
  setBusy(true);setResult('');setFailed(false);
  try{
   const outcome=await mutateFolderMessages(client,{accountId:owner.id,mailboxId:props.mailboxId,virtual:box()?.mailwomanSearchQuery!==undefined,ids,operation,current:admitted,signal:controller.signal});
   if(!admitted())return;
   view.retainSelection([...outcome.refused,...outcome.uncertain]);
   setFailed(outcome.refused.length>0||outcome.uncertain.length>0);
   setResult(t('folder-tabs-result',{updated:outcome.updated.length,refused:outcome.refused.length,skipped:outcome.skipped.length,uncertain:outcome.uncertain.length}));
  }catch{if(admitted()){setFailed(true);setResult(t('folder-tabs-mutation-error'));}}
  finally{if(admitted()){await reconcile();if(admitted())setBusy(false);}}
 }
 function remove():void{
  const ids=targets();if(disabled()||!ids.length)return;
  if(box()?.role==='trash'){props.beforeGlobalAction?.();setConfirmDelete([...ids]);return;}
  const trash=role('trash');if(trash)void run({kind:'move',target:trash.id},ids);
 }
 function confirmPermanent():void{
  const ids=confirmDelete();if(!ids||!owner||disabled())return;
  if(startTrashJob(client,{accountId:owner.id,mailboxId:props.mailboxId,ids,more:false},{current:admitted,clearSelection:()=>view.retainSelection([]),refresh:async()=>{await reconcile();if(admitted())setBusy(false);}})){setBusy(true);setConfirmDelete(null);}
 }
 // The floating selection bar (t58): the same non-modal bar as the message list, with this pane's own actions.
 const barActions=():SelectionAction[]=>{
  const off=disabled(),out:SelectionAction[]=[
   {id:'read',label:t('mail-mark-read'),icon:'mailRead',disabled:off,run:()=>run({kind:'read',read:true})},
   {id:'unread',label:t('mail-mark-unread'),icon:'mail',disabled:off,run:()=>run({kind:'read',read:false})},
  ];
  const archive=role('archive');if(archive)out.push({id:'archive',label:t('mail-archive'),icon:'archive',disabled:off,run:()=>run({kind:'move',target:archive.id})});
  if(box()?.role==='trash'||role('trash'))out.push({id:'delete',label:t('mail-delete'),icon:'delete',disabled:off,run:remove});
  out.push({id:'move',label:t('selection-bar-move'),icon:'folder',testId:'folder-bulk-move',disabled:off,menu:()=>app.mailboxes().filter(folder=>folder.id!==props.mailboxId&&folder.mailwomanSearchQuery===undefined).map(folder=>({id:`move:${folder.id}`,label:folder.name,icon:'folder' as const,run:()=>run({kind:'move',target:folder.id})}))});
  return out;
 };
 function focusList():void{const rows=viewport?.querySelectorAll<HTMLButtonElement>('[data-folder-message]');if(!rows?.length)return;const last=[...view.selection()].at(-1);([...rows].find(row=>row.dataset['folderMessage']===last)??rows[0])?.focus();}
 onCleanup(()=>{controller.abort();if(refreshTimer)clearTimeout(refreshTimer);});
 onMount(()=>{const stop=app.onRealtimeChange?.(change=>{if(!admitted()||app.linkedMailboxId?.())return;const types=change.changed[owner!.id];if(!types?.Email&&!types?.Mailbox)return;dirty=true;if(refreshTimer)clearTimeout(refreshTimer);refreshTimer=setTimeout(()=>{if(props.active&&!disabled()){dirty=false;void view.load();}},250);});if(stop)onCleanup(stop);});
 let viewport:HTMLDivElement|undefined;
 let previouslyActive=props.active;
 onMount(()=>{props.registerRefresh?.(()=>{if(!disabled())void view.load();});void loadCatalog('folder-tabs');void loadCatalog('empty-folder');void view.load();});
 createEffect(()=>{if(props.active&&viewport)viewport.scrollTop=view.scroll();});
 createEffect(()=>{const active=props.active;if(active&&!previouslyActive)untrack(()=>{if(!disabled())void view.load();});previouslyActive=active;});
 createEffect(()=>{if(props.active&&!busy()&&dirty){dirty=false;void view.load();}});
 function step(delta:number):boolean{
  if(!(props.active||props.companion)||!view.current())return false;
  if(disabled())return true;
  const rows=view.messages();if(!rows.length)return true;
  const selected=[...view.selection()].at(-1),index=rows.findIndex(row=>row.id===selected);
  const row=rows[index<0?(delta>0?0:rows.length-1):Math.max(0,Math.min(rows.length-1,index+delta))];
  if(row){view.select(row.id);props.onOpen(row.id);}return true;
 }
 useActions({next:()=>step(1),previous:()=>step(-1),refresh:()=>{if(!props.active)return false;if(!disabled())void view.load();return true;},trash:()=>{if(!props.active)return false;remove();return true;},markRead:()=>{if(!props.active)return false;void run({kind:'read',read:true});return true;},markUnread:()=>{if(!props.active)return false;void run({kind:'read',read:false});return true;},archive:()=>{if(!props.active)return false;const archive=role('archive');if(archive)void run({kind:'move',target:archive.id});return true;}});
 function key(event:KeyboardEvent,id:string){
  const rows=viewport?.querySelectorAll<HTMLButtonElement>('[data-folder-message]');if(!rows)return;
  const at=[...rows].findIndex(row=>row.dataset['folderMessage']===id);
  if(event.key==='Escape'&&view.selection().size>0){event.preventDefault();view.retainSelection([]);return;}
  const next=event.key==='ArrowDown'?at+1:event.key==='ArrowUp'?at-1:event.key==='Home'?0:event.key==='End'?rows.length-1:null;
  if(next!==null){event.preventDefault();const target=rows[Math.max(0,Math.min(rows.length-1,next))];target?.focus();if(modifierSelectionEnabled()&&event.shiftKey&&target?.dataset['folderMessage'])view.select(target.dataset['folderMessage'],true);}
 }
 return <div class={css.pane}>
  <header class={css.toolbar}><h2>{box()?.name??t('folder-tabs-unavailable')}</h2><button type="button" class="btn btn--ghost" disabled={disabled()} onClick={()=>void view.load()}>{t('folder-tabs-refresh')}</button><button type="button" class="btn btn--ghost" disabled={disabled()||!view.messages().length} onClick={view.selectAll}>{t('folder-tabs-select-page')}</button><Show when={box()&&canEmpty(box()!)}><button type="button" class="btn btn--ghost" disabled={disabled()} onClick={()=>setEmptying(true)}>{t('empty-folder-action')}</button></Show></header>
  <Show when={emptying()&&box()&&emptyAbility()}>{ability=><EmptyFolderDialog box={box()!} ability={ability()} onClose={()=>setEmptying(false)} onEmptied={()=>{view.retainSelection([]);void reconcile();}}/>}</Show>
  <Show when={confirmDelete()}>{ids=><ConfirmDialog open title={t('folder-tabs-permanent-title')} body={t('folder-tabs-permanent-confirm',{count:ids().length})} confirmLabel={t('mail-trash-permanent')} cancelLabel={t('folder-tabs-cancel')} mount={viewport?.ownerDocument.body} onCancel={()=>setConfirmDelete(null)} onConfirm={confirmPermanent}/>}</Show>
  <Show when={result()}><p role={failed()?'alert':'status'}>{result()}</p></Show>
  <Show when={view.error()}><p role="alert">{t('folder-tabs-error')}</p></Show>
  <Show when={view.messages().length===0&&(view.loading()||(!view.error()&&sync.pending()))}><AsyncPending variant="list" quiet message={t('mail-loading')} bounded={!sync.pending()}/></Show>
  <Show when={!view.loading()&&!view.error()&&view.messages().length===0&&sync.failed()}><AsyncError message={t('mail-initial-sync-error')} detail={t('mail-initial-sync-error-help')} onRetry={()=>sync.retry()}/></Show>
  <div class={barCss.paneBody}>
  <div class={css.viewport} classList={{[barCss.listInset]:view.selection().size>0}} ref={viewport} onScroll={event=>view.setScroll(event.currentTarget.scrollTop)}>
   <ul class={css.list} aria-label={box()?.name??t('folder-tabs-unavailable')} aria-busy={view.loading()||(view.messages().length===0&&sync.pending())}>
    <For each={view.messages()}>{email=><li class={css.row} data-selected={view.selection().has(email.id)} data-unread={!email.keywords?.['$seen']}>
     <input type="checkbox" disabled={busy()} checked={view.selection().has(email.id)} aria-label={t('folder-tabs-select',{subject:email.subject??''})} onChange={()=>view.select(email.id,false,true)}/>
     <button type="button" disabled={busy()} class={css.message} data-folder-message={email.id} onKeyDown={event=>key(event,email.id)} onClick={event=>{const extend=modifierSelectionEnabled()&&event.shiftKey,toggle=modifierSelectionEnabled()&&(event.ctrlKey||event.metaKey);view.select(email.id,extend,toggle);if(!extend&&!toggle&&view.current())props.onOpen(email.id);}}>
      <strong>{email.subject||t('mail-no-subject')}</strong><span>{senderLabel(email.from)}</span><span class="meta">{email.preview}</span>
     </button>
    </li>}</For>
   </ul>
   <Show when={!view.loading()&&!view.error()&&view.messages().length===0&&!sync.pending()&&!sync.failed()}><p>{t('folder-tabs-empty')}</p></Show>
  </div>
  <SelectionBar count={view.selection().size} total={view.messages().length} selectAll={view.selectAll} clear={()=>view.retainSelection([])} actions={barActions()} busy={busy()} returnFocus={focusList} scope={`${props.mailboxId}:${view.position()}`}/>
  </div>
  <footer class={css.toolbar}><button type="button" class="btn" disabled={disabled()||view.position()===0} onClick={()=>void view.load(Math.max(0,view.position()-100))}>{t('folder-tabs-previous')}</button><span role="status">{t('folder-tabs-position',{start:view.messages().length?view.position()+1:0,end:view.position()+view.messages().length,total:view.total()??'?',selected:view.selection().size})}</span><button type="button" class="btn" disabled={disabled()||(view.total()===null?view.messages().length<100:view.position()+view.messages().length>=view.total()!)} onClick={()=>void view.load(view.position()+100)}>{t('folder-tabs-next')}</button></footer>
 </div>;
}
