import {createResource,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {planEmpty,runBulkEmpty,type BulkEmptyProgress,type BulkEmptyReport} from '../api/folderBulkEmpty.ts';
import type {EmptyAbility} from '../api/emptyFolder.ts';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {isolate,loadCatalog,t} from '../i18n';
import {toast} from './toast/toast.ts';
import * as css from '../screens/Settings/styles.css.ts';
/**
 * Empty several folders. The dialog counts each folder first and says, per folder, whether its mail goes to Trash or is
 * deleted for good (Trash, Junk and accounts without Trash are always permanent, so a permanent line is only ever shown
 * where the backend and the administrator allow it: a folder the account cannot empty is skipped, not offered).
 * Subfolders are never touched. The run reports progress and lists the folders that failed.
 */
export function BulkEmptyFoldersDialog(props:{ids:readonly string[];ability:EmptyAbility;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),accountId=app.accountId(),client=createConfiguredClient(),controller=new AbortController();
 const authority={current:()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===accountId&&!controller.signal.aborted,signal:controller.signal};
 onMount(()=>{void loadCatalog('folder-dnd');void loadCatalog('mail');});
 onCleanup(()=>controller.abort());onCleanup(observePreferenceAccount(()=>{if(!authority.current())props.onClose();}));
 const [plan]=createResource(async()=>{
  if(!accountId)return null;
  return planEmpty(client,accountId,app.mailboxes().filter(box=>props.ids.includes(box.id)),app.mailboxes(),props.ability,authority);
 });
 const [progress,setProgress]=createSignal<BulkEmptyProgress|null>(null);
 const [report,setReport]=createSignal<BulkEmptyReport|null>(null);
 const runnable=()=>(plan()?.rows.filter(row=>row.skip===null).length??0)>0;
 async function confirm():Promise<void>{
  const current=plan();if(!current||!authority.current()||!runnable())throw new Error(t('folder-ops-error-failed'));
  const result=await runBulkEmpty(client,current,authority,setProgress);
  await app.refreshFolders().catch(()=>undefined);
  setReport(result);
  if(!result.failed.length){toast.success(t('settings-folders-bulk-empty-done',{count:result.emptied.length,messages:result.processed}));props.onClose();}
  else throw new Error(t('settings-folders-bulk-empty-partial',{done:result.emptied.length,failed:result.failed.length}));
 }
 const skipText=(skip:string):string=>t(`settings-folders-bulk-empty-skip-${skip}`);
 return <ConfirmDialog open title={t('settings-folders-bulk-empty-title',{count:props.ids.length})}
  body={plan.loading?t('settings-folders-bulk-empty-counting'):t('settings-folders-bulk-empty-body',{total:plan()?.total??0,permanent:plan()?.permanent??0})}
  confirmLabel={t('settings-folders-bulk-empty-confirm')} danger={(plan()?.permanent??0)>0} onConfirm={confirm} onCancel={props.onClose}>
  <ul class={css.meta} aria-label={t('settings-folders-bulk-empty-list')} data-testid="bulk-empty-list">
   <For each={plan()?.rows??[]}>{row=><li data-testid={`bulk-empty-${row.box.id}`}>{isolate(row.box.name)}{': '}{row.skip?skipText(row.skip):row.preview!.outcome==='trash'?t('settings-folders-bulk-empty-to-trash',{count:row.preview!.total}):t('settings-folders-bulk-empty-permanent',{count:row.preview!.total})}</li>}</For>
  </ul>
  <Show when={progress()}>{p=><p role="status" data-testid="bulk-empty-progress">{t('settings-folders-bulk-empty-progress',{done:p().done,of:p().of,messages:p().processed,name:isolate(p().current??'')})}</p>}</Show>
  <Show when={report()?.failed.length}><ul role="alert" data-testid="bulk-empty-failed"><For each={report()!.failed}>{f=><li>{isolate(f.name)}{': '}{t(`settings-folders-bulk-empty-fail-${f.reason}`,{remaining:f.remaining})}</li>}</For></ul></Show>
 </ConfirmDialog>;
}
