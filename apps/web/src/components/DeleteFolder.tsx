import {createSignal,createUniqueId,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {canDeleteFolder,deleteFolder,type DeleteFolderProblem} from '../api/deleteFolder.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import type {RenameContext} from './RenameFolder.tsx';
import {isolate,loadCatalog,t} from '../i18n';
import {createFocusTrap} from './a11y';
import * as shell from '../screens/Admin/shell/shell.css.ts';

export function DeleteFolderDialog(props:{box:Mailbox;context:RenameContext;onClose:()=>void}):JSX.Element {
 const app=useApp(),id=createUniqueId(),owner=preferenceAccount(),accountId=app.accountId(),client=createConfiguredClient(),controller=new AbortController();
 const [busy,setBusy]=createSignal(false),[consent,setConsent]=createSignal(false),[hasEmails,setHasEmails]=createSignal(props.box.totalEmails>0),[error,setError]=createSignal<DeleteFolderProblem|null>(null),[element,setElement]=createSignal<HTMLElement>();
 let cancelButton:HTMLButtonElement|undefined;
 const current=()=>!controller.signal.aborted&&!!owner&&owner.id===accountId&&preferenceAccountIsCurrent(owner)&&app.accountId()===accountId;
 onCleanup(()=>controller.abort());onMount(()=>{void loadCatalog('rename-folder');});
 const cancel=()=>{if(!busy())props.onClose();};
 createFocusTrap(element,{onEscape:cancel,initialFocus:()=>cancelButton});
 async function submit(event:Event):Promise<void>{
  event.preventDefault();if(busy()||!current()||!accountId)return;
  const target=app.mailboxes().find(box=>box.id===props.box.id);
  if(!target||!canDeleteFolder(target,app.mailboxes(),props.context)){setError('forbidden');return;}
  if(target.totalEmails>0&&!consent()){setHasEmails(true);return;}
  setBusy(true);setError(null);
  try{
   const result=await deleteFolder(client,accountId,target.id,consent(),{current,signal:controller.signal});
   if(!current())return;
   if(!result.ok){setError(result.problem);if(result.problem==='mailboxHasEmail')setHasEmails(true);return;}
   app.showToast('success',t('delete-folder-done',{name:isolate(target.name)}));
   await app.refreshFolders().catch(()=>{});
   if(current())props.onClose();
  }catch{if(current())setError('failed');}finally{if(current())setBusy(false);}
 }
 return <Portal><div class={shell.dialogScrim} onClick={cancel}/><form ref={setElement} class={shell.dialog} role="dialog" aria-modal="true" aria-labelledby={`${id}-title`} aria-describedby={`${id}-description`} tabindex="-1" onSubmit={event=>void submit(event)}>
  <h2 id={`${id}-title`} class={shell.dialogTitle}>{t('delete-folder-title',{name:isolate(props.box.name)})}</h2>
  <p id={`${id}-description`}>{t('delete-folder-description')}</p>
  <Show when={hasEmails()}><label><input type="checkbox" checked={consent()} disabled={busy()} onChange={event=>setConsent(event.currentTarget.checked)}/>{t('delete-folder-consent')}</label></Show>
  <Show when={error()}>{problem=><p role="alert">{t(`delete-folder-error-${problem()}`)}</p>}</Show>
  <div class={shell.dialogActions}><button ref={cancelButton} type="button" class="btn btn--ghost" disabled={busy()} onClick={cancel}>{t('rename-folder-cancel')}</button><button type="submit" class="btn btn--primary" disabled={busy()||(hasEmails()&&!consent())}>{t(busy()?'delete-folder-deleting':'delete-folder-action')}</button></div>
 </form></Portal>;
}
