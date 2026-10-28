// "Empty folder…" (t45): count, confirm, then empty in the background.
//
// The confirmation is the shared ConfirmDialog (t42). It names the folder, gives
// the count the server just reported and says what happens: moved to Trash, or
// deleted permanently (always said plainly in Trash, Junk, and in an account
// with no Trash). A permanent empty of more than TYPED_CONFIRMATION_OVER
// messages makes the user type the folder name first. Subfolders are never
// touched, and the dialog says so.
import {createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {canEmpty,emptyAbility,FolderSyncingError,prepareEmpty,TYPED_CONFIRMATION_OVER,type EmptyAbility,type EmptyPreview} from '../api/emptyFolder.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {isolate,loadCatalog,t} from '../i18n';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {toast} from './toast/toast.ts';
import {emptyJobRunning,startEmptyJob} from './emptyFolderJob.ts';

// The account's ability, read once per authenticated account from the JMAP session.
const [abilities,setAbilities]=createSignal<Readonly<Record<string,EmptyAbility>>>({});
const loading=new Set<string>();
const abilityKey=(accountId:string,generation:number)=>`${accountId}:${generation}`;

/** This account's empty ability (null until known). Reading it starts the fetch. */
export function useEmptyAbility():()=>EmptyAbility|null {
 const app=useApp();
 return ()=>{
  const owner=preferenceAccount(),accountId=app.accountId();if(!owner||!accountId||owner.id!==accountId)return null;
  const key=abilityKey(accountId,owner.generation),known=abilities()[key];
  if(known)return known;
  if(!loading.has(key)){
   loading.add(key);
   void Promise.resolve().then(()=>createConfiguredClient().session()).then(session=>{if(preferenceAccountIsCurrent(owner))setAbilities(all=>({...all,[key]:emptyAbility(session,accountId)}));}).catch(()=>{}).finally(()=>loading.delete(key));
  }
  return null;
 };
}

/** Whether to offer "Empty folder…" for `box` in this account now. */
export function useCanEmpty():(box:Mailbox)=>boolean {
 const app=useApp(),ability=useEmptyAbility();
 return box=>canEmpty(box,app.mailboxes(),ability());
}

function body(preview:EmptyPreview):string {
 const args={name:isolate(preview.name),count:preview.total};
 if(preview.outcome==='trash')return t('empty-folder-body-trash',args);
 if(preview.role==='trash')return t('empty-folder-body-destroy-trash',args);
 if(preview.role==='junk')return t('empty-folder-body-destroy-junk',args);
 return t('empty-folder-body-destroy-no-trash',args);
}

/** Counts `box` on open; shows the confirmation once the count is in. */
export function EmptyFolderDialog(props:{box:Mailbox;ability:EmptyAbility;onClose:()=>void;onEmptied?:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),accountId=app.accountId(),client=createConfiguredClient(),controller=new AbortController();
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===accountId;
 const [preview,setPreview]=createSignal<EmptyPreview>();
 let closed=false;
 const close=()=>{if(closed)return;closed=true;controller.abort();props.onClose();};
 onCleanup(()=>controller.abort());
 onCleanup(observePreferenceAccount(()=>{if(!current())close();}));
 onMount(()=>{
  if(!accountId||!current()){close();return;}
  if(emptyJobRunning()){toast.info(t('empty-folder-running'));close();return;}
  const name=isolate(props.box.name);
  void Promise.all([loadCatalog('empty-folder'),loadCatalog('admin-shell')])
   .then(()=>prepareEmpty(client,accountId,props.box,app.mailboxes(),props.ability,{current,signal:controller.signal}))
   .then(value=>{
    if(!current()||controller.signal.aborted)return;
    if(value.total===0){toast.info(t('empty-folder-already-empty',{name}));close();return;}
    setPreview(value);
   })
   .catch((error:unknown)=>{if(current()&&!controller.signal.aborted){if(error instanceof FolderSyncingError)toast.info(t('empty-folder-syncing',{name}));else toast.error(t('empty-folder-prepare-failed',{name}));close();}});
 });
 function confirm():void {
  const value=preview();if(!value||!current())return;
  const started=startEmptyJob(client,value,{current,clearSelection:()=>{if(app.selectedMailboxId()===value.mailboxId)app.clearSelection();},refresh:async()=>{await Promise.allSettled([app.refreshFolders(),app.refreshCurrentMailbox()]);props.onEmptied?.();}});
  if(started)close();
 }
 return <Show when={preview()}>{value=>
  <ConfirmDialog open title={t('empty-folder-title',{name:isolate(value().name)})} body={body(value())}
   confirmLabel={t(value().outcome==='trash'?'empty-folder-confirm-trash':'empty-folder-confirm-destroy')} cancelLabel={t('empty-folder-cancel')}
   danger={value().outcome==='destroy'} testId="empty-folder-confirm"
   {...(value().outcome==='destroy'&&value().total>TYPED_CONFIRMATION_OVER?{typedConfirmation:value().name}:{})}
   onConfirm={confirm} onCancel={close}/>
 }</Show>;
}
