import {createMemo,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {deleteFolderTree,type StructureProblem} from '../api/folderStructure.ts';
import {spamFolderHidden} from '../state/mailFeatures.ts';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {descendantsOf,rolledTotal,treeRows} from './folderTree.ts';
import {type RenameContext} from './RenameFolder.tsx';
import {runTransfer} from './folderOps.ts';
import {isolate,loadCatalog,t} from '../i18n';
import {toast} from './toast/toast.ts';
import * as css from '../screens/Settings/styles.css.ts';
/** The indented folder list the pickers share: every folder in tree order, depth shown by indentation and a leading mark. */
export function pickerOptions(boxes:readonly Mailbox[],allowed:(box:Mailbox)=>boolean):{id:string;label:string;depth:number}[] {
 return treeRows(boxes,()=>true).filter(row=>allowed(row.box)).map(row=>({id:row.box.id,depth:row.depth,label:`${'   '.repeat(row.depth)}${row.depth>0?'└ ':''}${row.box.name}`}));
}
function authority(owner:ReturnType<typeof preferenceAccount>,app:ReturnType<typeof useApp>,controller:AbortController){
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id&&!controller.signal.aborted;
 return {current,signal:controller.signal};
}
/** Choose a folder from an indented list. `top` adds a "Top level" entry. */
function FolderList(props:{options:{id:string;label:string}[];top?:boolean;value:string|null;onChange:(id:string|null)=>void;label:string}):JSX.Element {
 return <label class={css.field}>{props.label}<select class={css.select} size={Math.min(8,props.options.length+(props.top?1:0))} value={props.value??''} onChange={event=>props.onChange(event.currentTarget.value||null)}>
  <Show when={props.top}><option value="">{t('folder-top-level')}</option></Show>
  <For each={props.options}>{option=><option value={option.id}>{option.label}</option>}</For>
 </select></label>;
}
/** Copy or move every message of a folder (and optionally of its subfolders) to a chosen folder. */
export function TransferDialog(props:{box:Mailbox;mode:'copy'|'move';onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();
 const [target,setTarget]=createSignal<string|null>(null),[withChildren,setWithChildren]=createSignal(false);
 const auth=authority(owner,app,controller);
 onMount(()=>{void loadCatalog('mail');});
 // The job outlives this dialog, so its abort belongs to the account, not the dialog.
 onCleanup(observePreferenceAccount(()=>{if(!auth.current())props.onClose();}));
 const subs=()=>descendantsOf(app.mailboxes(),props.box.id);
 const sources=()=>[props.box,...(withChildren()?subs():[])];
 const options=createMemo(()=>{const sourceIds=new Set(sources().map(box=>box.id));return pickerOptions(app.mailboxes(),box=>!sourceIds.has(box.id)&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayAddItems!==false&&!spamFolderHidden(box));});
 const total=()=>withChildren()?rolledTotal(app.mailboxes(),props.box.id):props.box.totalEmails;
 async function confirm():Promise<void>{
  const destination=target();
  if(!auth.current()||!owner||!destination||!app.mailboxes().some(box=>box.id===destination))throw new Error(t('folder-ops-choose-folder'));
  const hooks={app:{accountId:app.accountId,clearFolderUnread:app.clearFolderUnread,refreshFolders:app.refreshFolders,refreshCurrentMailbox:app.refreshCurrentMailbox},client,accountId:owner.id,current:()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id};
  const list=sources().filter(box=>box.id!==destination);
  props.onClose();
  for(const source of list){if(!hooks.current())break;await runTransfer(hooks,source.name,source.id,destination,props.mode);}
 }
 const name=()=>isolate(props.box.name);
 return <ConfirmDialog open title={t(props.mode==='move'?'folder-ops-move-all-title':'folder-ops-copy-all-title',{name:name()})} body={t(props.mode==='move'?'folder-ops-move-all-body':'folder-ops-copy-all-body',{count:total()})} confirmLabel={t(props.mode==='move'?'folder-ops-move-all':'folder-ops-copy-all')} danger={false} onConfirm={confirm} onCancel={props.onClose}>
  <FolderList options={options()} value={target()} onChange={setTarget} label={t('folder-ops-destination')}/>
  <Show when={subs().length>0}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={withChildren()} onChange={event=>setWithChildren(event.currentTarget.checked)}/>{t('folder-ops-include-subfolders',{count:subs().length})}</label></Show>
 </ConfirmDialog>;
}
/** Delete a folder with its subfolders; names how many folders and messages go. */
export function DeleteTreeDialog(props:{box:Mailbox;context:RenameContext;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();
 const auth=authority(owner,app,controller);
 onMount(()=>{void loadCatalog('mail');void loadCatalog('rename-folder');});
 onCleanup(()=>controller.abort());onCleanup(observePreferenceAccount(()=>{if(!auth.current())props.onClose();}));
 const subs=()=>descendantsOf(app.mailboxes(),props.box.id);
 const items=()=>rolledTotal(app.mailboxes(),props.box.id);
 async function confirm():Promise<void>{
  const target=app.mailboxes().find(box=>box.id===props.box.id);
  if(!auth.current()||!owner||!target)throw new Error(t('folder-ops-error-failed'));
  const result=await deleteFolderTree(client,owner.id,target,app.mailboxes(),true,auth);
  if(!auth.current())return;
  await app.refreshFolders().catch(()=>{});
  if(!result.ok)throw new Error(t(`folder-ops-error-${result.problem satisfies StructureProblem}`));
  toast.success(t('delete-folder-done',{name:isolate(props.box.name)}));props.onClose();
 }
 return <ConfirmDialog open title={t('delete-folder-title',{name:isolate(props.box.name)})} body={t('folder-ops-delete-tree-body',{folders:subs().length+1,count:items()})} confirmLabel={t('delete-folder-action')} danger onConfirm={confirm} onCancel={props.onClose}/>;
}
