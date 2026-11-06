// Properties for a selection of messages: how many, how big, which dates, which
// senders and folders, a ZIP of the .eml files and a permanent delete. Everything
// comes from the loaded rows; the ZIP fetches one message at a time and stops at
// BULK_ZIP_LIMIT, and the delete goes through the same server gates as the
// single-message dialog (legal hold, `properties.message.permanentDelete`).
import {For,Show,createMemo,createSignal,createUniqueId,onMount,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import type {Email} from '../../api/jmap-types.ts';
import {destroyEmails,responseFor} from '../../api/jmap.ts';
import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {formatDateTime,isolate,loadCatalog,t} from '../../i18n';
import {SecurityPopup} from '../SecurityPanel.tsx';
import {ConfirmDialog} from '../../screens/Admin/shell/ConfirmDialog.tsx';
import {Icon} from '../Icon.tsx';
import {toast} from '../toast/toast.ts';
import {buildZip} from '../../viewers/zipStore.ts';
import {formatBytes} from './folderStats.ts';
import {BULK_ZIP_LIMIT,summarizeSelection,zipEntryName} from './bulkProps.ts';
import {messageDownloadEnabled,permanentDeletePolicy} from './policy.ts';
import * as css from './properties.css.ts';

const SENDERS_SHOWN=5;

export function BulkPropertiesDialog(props:{emails:readonly Email[];onClose:()=>void}):JSX.Element {
 const app=useApp(),client=createConfiguredClient(),titleId=`props-bulk-${createUniqueId()}`;
 onMount(()=>{void loadCatalog('properties');});
 const summary=createMemo(()=>summarizeSelection(props.emails));
 const folderName=(id:string)=>{const box=app.mailboxes().find(value=>value.id===id);return box?.mailwomanPath??box?.name??id;};
 const [step,setStep]=createSignal<0|1|2>(0);
 const [zipping,setZipping]=createSignal<{done:number;total:number}|null>(null);
 let cancelled=false;
 const policy=()=>permanentDeletePolicy();
 const describe=(error:unknown)=>error instanceof Error&&error.message?error.message:t('props-delete-failed');

 async function downloadZip():Promise<void> {
  const chosen=props.emails.slice(0,BULK_ZIP_LIMIT);
  cancelled=false;
  setZipping({done:0,total:chosen.length});
  try{
   const entries=[];
   for(const [index,email] of chosen.entries()){
    if(cancelled)return;
    const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(email.id)}?format=eml`));
    if(!response.ok)throw new Error(String(response.status));
    entries.push({name:zipEntryName(index,email.subject),data:new Uint8Array(await response.arrayBuffer())});
    setZipping({done:index+1,total:chosen.length});
   }
   const url=URL.createObjectURL(buildZip(entries)),a=document.createElement('a');
   a.href=url;a.download='messages.zip';document.body.appendChild(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),10_000);
   toast.success(t('props-bulk-zip-done',{count:entries.length}));
  }catch{toast.error(t('props-download-failed'));}
  finally{setZipping(null);}
 }

 async function destroy():Promise<void> {
  const account=app.accountId();if(!account)return;
  const ids=props.emails.map(email=>email.id);
  const response=await client.jmap(destroyEmails(account,ids,true));
  const result=responseFor<{destroyed?:string[];notDestroyed?:Record<string,{type?:string;description?:string}>}>(response,'set');
  const failed=ids.filter(id=>!result.destroyed?.includes(id));
  if(failed.length===ids.length)throw new Error(result.notDestroyed?.[failed[0]!]?.description??result.notDestroyed?.[failed[0]!]?.type??'refused');
  setStep(0);props.onClose();void app.refreshCurrentMailbox();
  toast.success(failed.length?t('props-bulk-delete-partial',{done:ids.length-failed.length,failed:failed.length}):t('props-bulk-delete-done',{count:ids.length}));
 }

 return <SecurityPopup title={t('props-bulk-title',{count:summary().count})} labelledBy={titleId} onClose={()=>{cancelled=true;props.onClose();}}>
  <div class={`${css.body} ${css.phone}`} data-testid="bulk-properties">
   <dl class={css.grid} data-testid="bulk-summary">
    <dt>{t('props-bulk-count')}</dt><dd data-testid="bulk-count">{summary().count}</dd>
    <dt>{t('props-bulk-size')}</dt><dd data-testid="bulk-size">{formatBytes(summary().totalBytes)}</dd>
    <dt>{t('props-bulk-dates')}</dt><dd data-testid="bulk-dates">
     <Show when={summary().oldest&&summary().newest} fallback={t('props-none')}>{formatDateTime(summary().oldest!)} – {formatDateTime(summary().newest!)}</Show>
    </dd>
    <dt>{t('props-bulk-unread')}</dt><dd>{summary().unread}</dd>
    <dt>{t('props-bulk-attachments')}</dt><dd>{summary().withAttachments}</dd>
    <dt>{t('props-bulk-senders')}</dt><dd data-testid="bulk-senders">
     <p class={css.note}>{t('props-bulk-senders-count',{count:summary().senders.length})}</p>
     <ul><For each={summary().senders.slice(0,SENDERS_SHOWN)}>{sender=><li>{isolate(sender.name?`${sender.name} <${sender.email}>`:sender.email)} ({sender.count})</li>}</For></ul>
     <Show when={summary().senders.length>SENDERS_SHOWN}><p class={css.note}>{t('props-bulk-senders-more',{count:summary().senders.length-SENDERS_SHOWN})}</p></Show>
    </dd>
    <dt>{t('props-bulk-folders')}</dt><dd data-testid="bulk-folders">
     <ul><For each={summary().folders}>{folder=><li>{isolate(folderName(folder.id))} ({folder.count})</li>}</For></ul>
    </dd>
   </dl>
   <p class={css.note}>{t('props-bulk-loaded-note')}</p>
   <div class={css.actions}>
    <Show when={messageDownloadEnabled()}>
     <button type="button" class="btn btn--ghost" data-testid="bulk-zip" disabled={zipping()!==null} onClick={()=>void downloadZip()}><Icon name="download"/>{zipping()?t('props-bulk-zip-progress',{done:zipping()!.done,total:zipping()!.total}):t('props-bulk-zip')}</button>
     <Show when={summary().count>BULK_ZIP_LIMIT}><span class={css.note}>{t('props-bulk-zip-limit',{max:BULK_ZIP_LIMIT})}</span></Show>
    </Show>
    <button type="button" class="btn btn--danger" data-testid="bulk-delete" disabled={policy()==='deny'} title={policy()==='deny'?t('props-delete-denied'):undefined} onClick={()=>setStep(1)}><Icon name="delete"/>{t('props-bulk-delete')}</button>
   </div>
   <Show when={policy()==='deny'}><p class={css.note}>{t('props-delete-denied')}</p></Show>
   <ConfirmDialog open={step()===1} title={t('props-bulk-delete-confirm-title',{count:summary().count})} body={t('props-bulk-delete-confirm-body')} confirmLabel={policy()==='confirm-twice'?t('props-delete-continue'):t('props-delete')} onCancel={()=>setStep(0)} onConfirm={async()=>{if(policy()==='confirm-twice'){setStep(2);return;}await destroy();}} testId="bulk-delete-confirm" describeError={describe}/>
   <ConfirmDialog open={step()===2} title={t('props-delete-confirm2-title')} body={t('props-delete-confirm2-body')} confirmLabel={t('props-delete')} onCancel={()=>setStep(0)} onConfirm={destroy} testId="bulk-delete-confirm2" describeError={describe}/>
  </div>
 </SecurityPopup>;
}
