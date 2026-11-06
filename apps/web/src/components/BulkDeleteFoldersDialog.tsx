import {createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {bulkDeleteFolders,deletePlan} from '../api/folderBulkDelete.ts';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import type {RenameContext} from './RenameFolder.tsx';
import {isolate,loadCatalog,t} from '../i18n';
import {toast} from './toast/toast.ts';
import * as css from '../screens/Settings/styles.css.ts';
/** Delete several folders (each with its subfolders) after naming how many folders and messages go. Nothing is removed until confirmed. */
export function BulkDeleteFoldersDialog(props:{ids:readonly string[];context:()=>RenameContext|null;onDone:(deleted:string[])=>void;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();
 const auth={current:()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id&&!controller.signal.aborted,signal:controller.signal};
 onMount(()=>{void loadCatalog('folder-dnd');void loadCatalog('mail');});
 onCleanup(()=>controller.abort());onCleanup(observePreferenceAccount(()=>{if(!auth.current())props.onClose();}));
 const [withSubfolders,setWithSubfolders]=createSignal(true);
 const plan=createMemo(()=>deletePlan(app.mailboxes().filter(box=>props.ids.includes(box.id)),app.mailboxes(),props.context(),withSubfolders()));
 async function confirm():Promise<void>{
  const current=plan();
  if(!auth.current()||!owner||!current.roots.length)throw new Error(t('folder-ops-error-failed'));
  const result=await bulkDeleteFolders(client,owner.id,current,app.mailboxes(),auth);
  if(!auth.current())return;
  await app.refreshFolders().catch(()=>{});
  if(result.failed.length){
   const names=result.failed.map(item=>app.mailboxes().find(box=>box.id===item.id)?.name??item.id).map(isolate).join(', ');
   if(!result.deleted.length)throw new Error(t('settings-folders-bulk-delete-failed',{names}));
   toast.error(t('settings-folders-bulk-delete-partial',{done:result.deleted.length,names}));
  }else toast.success(t('settings-folders-bulk-delete-done',{count:result.deleted.length}));
  props.onDone(result.deleted);props.onClose();
 }
 return <ConfirmDialog open title={t('settings-folders-bulk-delete-title',{count:plan().roots.length})} body={t('settings-folders-bulk-delete-body',{folders:plan().folders.length,messages:plan().messages})} confirmLabel={t('settings-folders-bulk-delete-confirm',{count:plan().roots.length})} danger onConfirm={confirm} onCancel={props.onClose}>
  <label class={css.check}><input class={css.checkbox} type="checkbox" data-testid="bulk-delete-subfolders" checked={withSubfolders()} onChange={event=>setWithSubfolders(event.currentTarget.checked)}/>{t('settings-folders-bulk-delete-subfolders')}</label>
  <Show when={!withSubfolders()}><p class={css.meta}>{t('settings-folders-bulk-delete-subfolders-off')}</p></Show>
  <Show when={plan().skipped.length>0}><p class={css.meta} role="note" data-testid="bulk-delete-skipped">{t('settings-folders-bulk-delete-skipped',{count:plan().skipped.length,names:plan().skipped.map(box=>isolate(box.name)).join(', ')})}</p></Show>
  <ul class={css.meta} aria-label={t('settings-folders-bulk-delete-list')}><For each={plan().roots}>{box=><li>{isolate(box.name)}</li>}</For></ul>
 </ConfirmDialog>;
}
