import {createEffect,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {emptyTrash,prepareTrash,TRASH_LIMIT,type TrashResult,type TrashSnapshot} from '../api/emptyTrash.ts';
import {wireDialogFocus} from '../modules/contacts/dialogFocus.ts';
import {t} from '../i18n';
import {Icon} from './Icon.tsx';
import './sweepDialog.css.ts';

function EmptyTrashDialog(props:{mailboxId:string;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),accountId=app.accountId();const controller=new AbortController();const client=createConfiguredClient();
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===accountId;
 const [preview,setPreview]=createSignal<TrashSnapshot>();const [error,setError]=createSignal(false);const [busy,setBusy]=createSignal(false);const [result,setResult]=createSignal<TrashResult>();const [finished,setFinished]=createSignal(false);
 let dialog!:HTMLDivElement;onCleanup(()=>controller.abort());
 createEffect(()=>{if(app.accountId()!==accountId||app.selectedMailboxId()!==props.mailboxId)props.onClose();});
 onMount(()=>{const dispose=wireDialogFocus(dialog,props.onClose);onCleanup(dispose);if(accountId)void prepareTrash(client,accountId,props.mailboxId,{current,signal:controller.signal}).then(value=>{if(current()&&!controller.signal.aborted)setPreview(value);}).catch(()=>{if(current())setError(true);});});
 async function run():Promise<void>{const snapshot=preview();if(!snapshot||busy()||finished())return;setBusy(true);const value=await emptyTrash(client,snapshot,{current,signal:controller.signal},setResult);if(current()&&!controller.signal.aborted){setResult(value);setFinished(true);setBusy(false);app.clearSelection();await Promise.allSettled([app.refreshFolders(),app.refreshCurrentMailbox()]);}}
 return <Portal><div class="sweep__backdrop" role="alertdialog" aria-modal="true" aria-label={t('mail-empty-trash')} ref={dialog} tabindex="-1"><div class="sweep"><h2>{t('mail-empty-trash')}</h2>
  <Show when={!preview()&&!error()}><p role="status">{t('mail-trash-preparing')}</p></Show><Show when={error()}><p role="alert">{t('mail-trash-prepare-failed')}</p></Show>
  <Show when={preview()}>{value=><><p>{t('mail-trash-confirm',{count:value().ids.length})}</p><Show when={value().more}><p>{t('mail-trash-limit',{count:TRASH_LIMIT})}</p></Show></>}</Show>
  <Show when={result()}>{value=><p role="status" aria-live="polite">{t(value().stopped?'mail-trash-stopped':'mail-trash-progress',{deleted:value().deleted,skipped:value().skipped,remaining:value().remaining})}</p>}</Show>
  <footer class="sweep__footer"><button type="button" class="btn btn--ghost" onClick={props.onClose}>{t(busy()?'mail-trash-stop':'mail-compose-close')}</button><button type="button" class="btn btn--primary" disabled={!preview()?.ids.length||busy()||finished()} onClick={()=>void run()}><Icon name="delete"/>{t('mail-trash-permanent')}</button></footer>
 </div></div></Portal>;
}
export function EmptyTrash():JSX.Element {
 const app=useApp();const [open,setOpen]=createSignal(false);const folder=()=>app.mailboxes().find(box=>box.id===app.selectedMailboxId()&&box.role==='trash');
 return <Show when={folder()}>{box=><><button type="button" class="btn btn--ghost" aria-label={t('mail-empty-trash')} title={t('mail-trash-help')} onClick={()=>setOpen(true)}><Icon name="delete"/></button><Show when={open()}><EmptyTrashDialog mailboxId={box().id} onClose={()=>setOpen(false)}/></Show></>}</Show>;
}
