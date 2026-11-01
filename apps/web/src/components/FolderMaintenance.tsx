import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {CAP_CORE,CAP_MAIL,type Mailbox} from '../api/jmap-types.ts';
import {nameProblem} from '../api/renameFolder.ts';
import {cleanupCutoff,cleanupMessages,previewCleanup,type AgeUnit,type CleanupOutcome} from '../api/folderMaintenance.ts';
import {expectedOutcome} from '../api/emptyFolder.ts';
import {useRenameContext} from './RenameFolder.tsx';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {createContextMenu} from './ContextMenu.tsx';
import {loadCatalog,t} from '../i18n';
import * as css from '../screens/Settings/styles.css.ts';
export function CreateFolderDialog(props:{parentId?:string;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),client=createConfiguredClient(),context=useRenameContext(),controller=new AbortController();
 const [name,setName]=createSignal(''),[parent,setParent]=createSignal(props.parentId??'');
 const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id;
 onCleanup(()=>controller.abort());onCleanup(observePreferenceAccount(()=>{if(!current())props.onClose();}));
 async function save():Promise<void>{
  const ability=context();if(!current()||!ability?.manages)throw new Error(t('folder-create-unavailable'));
  const parentBox=app.mailboxes().find(box=>box.id===parent());if(parent()&&(!parentBox||parentBox.myRights?.mayCreateChild!==true))throw new Error(t('folder-create-unavailable'));
  const problem=nameProblem(name(),{id:'',parentId:parent()||null} as Mailbox,app.mailboxes(),ability.delimiter,ability.maxLength);if(problem)throw new Error(t('folder-create-invalid'));
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId:owner!.id,create:{folder:{name:name(),parentId:parent()||null,isSubscribed:true}}},'create']]},{signal:controller.signal});if(!current())return;
  const tuple=response.methodResponses.find(row=>row[2]==='create');const reply=tuple?.[1] as {accountId?:string;created?:Record<string,{id?:string}>}|undefined;
  if(tuple?.[0]!=='Mailbox/set'||reply?.accountId!==owner!.id||!reply.created?.['folder']?.id)throw new Error(t('folder-create-failed'));
  await app.refreshFolders();if(current()){app.showToast('success',t('folder-created'));props.onClose();}
 }
 return <ConfirmDialog open title={t('folder-create')} confirmLabel={t('folder-create')} danger={false} onConfirm={save} onCancel={props.onClose}>
  <label class={css.field}>{t('folder-name')}<input class={css.input} value={name()} onInput={event=>setName(event.currentTarget.value)} required maxlength={context()?.maxLength??255}/></label>
  <label class={css.field}>{t('folder-parent')}<select class={css.select} value={parent()} onChange={event=>setParent(event.currentTarget.value)}><option value="">{t('folder-top-level')}</option><For each={app.mailboxes().filter(box=>box.myRights?.mayCreateChild===true&&box.mailwomanSearchQuery===undefined)}>{box=><option value={box.id}>{box.name}</option>}</For></select></label>
 </ConfirmDialog>;
}
export function CleanupFolderDialog(props:{box:Mailbox;onClose:()=>void}):JSX.Element {
 const app=useApp(),owner=preferenceAccount(),client=createConfiguredClient(),controller=new AbortController();const current=()=>!!owner&&preferenceAccountIsCurrent(owner)&&app.accountId()===owner.id;
 const [amount,setAmount]=createSignal(30),[unit,setUnit]=createSignal<AgeUnit>('days'),[preview,setPreview]=createSignal<{ids:string[];before:string;outcome:CleanupOutcome}>();
 onCleanup(()=>controller.abort());onCleanup(observePreferenceAccount(()=>{if(!current())props.onClose();}));
 const permanent=()=>(preview()?.outcome??expectedOutcome(props.box,app.mailboxes())).outcome==='destroy';
 async function confirm():Promise<void>{if(!current())return;const prior=preview();if(!prior){const before=cleanupCutoff(amount(),unit());const ids=await previewCleanup(client,owner!.id,props.box.id,before,{current,signal:controller.signal});if(current())setPreview({ids,before,outcome:expectedOutcome(props.box,app.mailboxes())});return;}
  try{const count=await cleanupMessages(client,owner!.id,props.box,app.mailboxes(),prior.ids,prior.before,{current,signal:controller.signal},prior.outcome);if(current()){app.showToast('success',t('folder-cleanup-done',{count}));props.onClose();}}
  catch(error){if(current())setPreview(undefined);throw error;}
  finally {if(current())await Promise.allSettled([app.refreshFolders(),app.refreshCurrentMailbox()]);}
 }
 return <ConfirmDialog open title={t('folder-cleanup-title',{name:props.box.name})} body={preview()?t(permanent()?'folder-cleanup-permanent':'folder-cleanup-trash',{count:preview()!.ids.length}):t('folder-cleanup-help')} confirmLabel={t(preview()?'folder-cleanup-confirm':'folder-cleanup-preview')} danger={!!preview()&&permanent()} onConfirm={confirm} onCancel={props.onClose}>
  <Show when={!preview()} fallback={<p>{t('folder-cleanup-before',{date:preview()?.before??''})}</p>}><label class={css.field}>{t('folder-cleanup-age')}<input class={css.input} type="number" min="1" max="10000" value={amount()} onInput={event=>setAmount(event.currentTarget.valueAsNumber)}/><select class={css.select} value={unit()} onChange={event=>setUnit(event.currentTarget.value as AgeUnit)}><For each={['days','weeks','months','years'] as const}>{value=><option value={value}>{t(`folder-age-${value}`)}</option>}</For></select></label></Show>
 </ConfirmDialog>;
}
/** Folder creation stays in the sidebar background context menu. */
export function FolderCreationControl(props:{children?:JSX.Element}):JSX.Element {
 const app=useApp(),ability=useRenameContext();const [open,setOpen]=createSignal(false);void loadCatalog('mail');
 onMount(()=>{createEffect(()=>{ability();});});
 const context=createContextMenu({label:()=>t('mail-nav-mailboxes'),scope:()=>`${app.accountId()}:${preferenceAccount()?.generation}`,actions:()=>ability()?.manages?[{id:'create-folder',label:t('folder-create'),icon:'folder',run:()=>{setOpen(true);}}]:[]});
 onCleanup(observePreferenceAccount(()=>{context.close(false);setOpen(false);}));
 return <div onContextMenu={event=>context.open(event)} onKeyDown={context.keydown}>{props.children}{context.menu()}<Show when={open()}><CreateFolderDialog onClose={()=>setOpen(false)}/></Show></div>;
}
