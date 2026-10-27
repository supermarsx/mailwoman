import {createEffect,createSignal,For,onCleanup,onMount,Show,untrack,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {createFolderView} from './folderView.ts';
import {AsyncPending} from './AsyncState.tsx';
import {senderLabel} from './MessageList.tsx';
import {modifierSelectionEnabled} from './mailInteractions.ts';
import {useActions} from '../keymap/index.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {mutateFolderMessages,type FolderMutation} from './folderMutations.ts';
import {startTrashJob} from './emptyTrashJob.ts';
import {loadCatalog,t} from '../i18n/index.ts';
import * as css from './folderPane.css.ts';

export function FolderPane(props:{mailboxId:string;active:boolean;onOpen:(id:string)=>void;registerRefresh?:(refresh:()=>void)=>void;beforeGlobalAction?:()=>void}):JSX.Element{
 const app=useApp();
 const owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();
 const box=()=>app.mailboxes().find(value=>value.id===props.mailboxId);
 const view=createFolderView(client,props.mailboxId,()=>!!box());
 const [busy,setBusy]=createSignal(false),[result,setResult]=createSignal(''),[failed,setFailed]=createSignal(false),[destination,setDestination]=createSignal('');
 const [confirmDelete,setConfirmDelete]=createSignal<string[]|null>(null);
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
 onCleanup(()=>{controller.abort();if(refreshTimer)clearTimeout(refreshTimer);});
 onMount(()=>{const stop=app.onRealtimeChange?.(change=>{if(!admitted()||app.linkedMailboxId?.())return;const types=change.changed[owner!.id];if(!types?.Email&&!types?.Mailbox)return;dirty=true;if(refreshTimer)clearTimeout(refreshTimer);refreshTimer=setTimeout(()=>{if(props.active&&!disabled()){dirty=false;void view.load();}},250);});if(stop)onCleanup(stop);});
 let viewport:HTMLDivElement|undefined;
 let previouslyActive=props.active;
 onMount(()=>{props.registerRefresh?.(()=>{if(!disabled())void view.load();});void loadCatalog('folder-tabs');void view.load();});
 createEffect(()=>{if(props.active&&viewport)viewport.scrollTop=view.scroll();});
 createEffect(()=>{const active=props.active;if(active&&!previouslyActive)untrack(()=>{if(!disabled())void view.load();});previouslyActive=active;});
 createEffect(()=>{if(props.active&&!busy()&&dirty){dirty=false;void view.load();}});
 function step(delta:number):boolean{
  if(!props.active||!view.current())return false;
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
  const next=event.key==='ArrowDown'?at+1:event.key==='ArrowUp'?at-1:event.key==='Home'?0:event.key==='End'?rows.length-1:null;
  if(next!==null){event.preventDefault();const target=rows[Math.max(0,Math.min(rows.length-1,next))];target?.focus();if(modifierSelectionEnabled()&&event.shiftKey&&target?.dataset['folderMessage'])view.select(target.dataset['folderMessage'],true);}
 }
 return <div class={css.pane}>
  <header class={css.toolbar}><h2>{box()?.name??t('folder-tabs-unavailable')}</h2><button type="button" class="btn btn--ghost" disabled={disabled()} onClick={()=>void view.load()}>{t('folder-tabs-refresh')}</button><button type="button" class="btn btn--ghost" disabled={disabled()||!view.messages().length} onClick={view.selectAll}>{t('folder-tabs-select-page')}</button></header>
  <Show when={view.selection().size>0}><div class={css.toolbar} role="toolbar" aria-label={t('folder-tabs-actions')}>
   <button type="button" class="btn" disabled={disabled()} onClick={()=>void run({kind:'read',read:true})}>{t('mail-mark-read')}</button>
   <button type="button" class="btn" disabled={disabled()} onClick={()=>void run({kind:'read',read:false})}>{t('mail-mark-unread')}</button>
   <Show when={role('archive')}>{archive=><button type="button" class="btn" disabled={disabled()} onClick={()=>void run({kind:'move',target:archive().id})}>{t('mail-archive')}</button>}</Show>
   <Show when={box()?.role==='trash'||role('trash')}><button type="button" class="btn" disabled={disabled()} onClick={remove}>{t('mail-delete')}</button></Show>
   <select aria-label={t('folder-tabs-move-destination')} value={destination()} disabled={disabled()} onChange={event=>setDestination(event.currentTarget.value)}><option value="">{t('folder-tabs-choose-destination')}</option><For each={app.mailboxes().filter(folder=>folder.id!==props.mailboxId&&folder.mailwomanSearchQuery===undefined)}>{folder=><option value={folder.id}>{folder.name}</option>}</For></select>
   <button type="button" class="btn" disabled={disabled()||!destination()} onClick={()=>void run({kind:'move',target:destination()})}>{t('folder-tabs-move')}</button>
   <button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>view.retainSelection([])}>{t('folder-tabs-clear-selection')}</button>
  </div></Show>
  <Show when={confirmDelete()}>{ids=><PermanentConfirmation count={ids().length} onCancel={()=>setConfirmDelete(null)} onConfirm={confirmPermanent}/>}</Show>
  <Show when={result()}><p role={failed()?'alert':'status'}>{result()}</p></Show>
  <Show when={view.error()}><p role="alert">{t('folder-tabs-error')}</p></Show>
  <Show when={view.loading()&&view.messages().length===0}><AsyncPending variant="list"/></Show>
  <div class={css.viewport} ref={viewport} onScroll={event=>view.setScroll(event.currentTarget.scrollTop)}>
   <ul class={css.list} aria-label={box()?.name??t('folder-tabs-unavailable')}>
    <For each={view.messages()}>{email=><li class={css.row} data-selected={view.selection().has(email.id)} data-unread={!email.keywords?.['$seen']}>
     <input type="checkbox" disabled={busy()} checked={view.selection().has(email.id)} aria-label={t('folder-tabs-select',{subject:email.subject??''})} onChange={()=>view.select(email.id,false,true)}/>
     <button type="button" disabled={busy()} class={css.message} data-folder-message={email.id} onKeyDown={event=>key(event,email.id)} onClick={event=>{const extend=modifierSelectionEnabled()&&event.shiftKey,toggle=modifierSelectionEnabled()&&(event.ctrlKey||event.metaKey);view.select(email.id,extend,toggle);if(!extend&&!toggle&&view.current())props.onOpen(email.id);}}>
      <strong>{email.subject||t('mail-no-subject')}</strong><span>{senderLabel(email.from)}</span><span class="meta">{email.preview}</span>
     </button>
    </li>}</For>
   </ul>
   <Show when={!view.loading()&&!view.error()&&view.messages().length===0}><p>{t('folder-tabs-empty')}</p></Show>
  </div>
  <footer class={css.toolbar}><button type="button" class="btn" disabled={disabled()||view.position()===0} onClick={()=>void view.load(Math.max(0,view.position()-100))}>{t('folder-tabs-previous')}</button><span role="status">{t('folder-tabs-position',{start:view.messages().length?view.position()+1:0,end:view.position()+view.messages().length,total:view.total()??'?',selected:view.selection().size})}</span><button type="button" class="btn" disabled={disabled()||(view.total()===null?view.messages().length<100:view.position()+view.messages().length>=view.total()!)} onClick={()=>void view.load(view.position()+100)}>{t('folder-tabs-next')}</button></footer>
 </div>;
}

function PermanentConfirmation(props:{count:number;onCancel:()=>void;onConfirm:()=>void}):JSX.Element{
 let region!:HTMLDivElement;
 onMount(()=>{const previous=region.ownerDocument.activeElement as HTMLElement|null;region.querySelector<HTMLButtonElement>('button')?.focus();onCleanup(()=>previous?.focus());});
 return <div ref={region} role="alertdialog" aria-label={t('folder-tabs-permanent-title')} class={css.confirmation} onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();props.onCancel();}}}>
  <p>{t('folder-tabs-permanent-confirm',{count:props.count})}</p><div class={css.toolbar}><button type="button" class="btn" onClick={props.onCancel}>{t('folder-tabs-cancel')}</button><button type="button" class="btn btn--primary" onClick={props.onConfirm}>{t('mail-trash-permanent')}</button></div>
 </div>;
}
