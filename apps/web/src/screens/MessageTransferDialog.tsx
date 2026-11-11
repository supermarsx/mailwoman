import {createResource,createSignal,For,onCleanup,Show,type JSX} from 'solid-js';
import {preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {transferFormats,transferWorkspaceMessage,workspaceFolders,workspaceMailboxes,type TransferOutcome} from '../api/workspaceMail.ts';
import {loadCatalog,t} from '../i18n';
import {transfersEnabled,type MessageTransferRequest} from '../state/messageTransferRequest.ts';
import {InsertDialog} from '../components/compose/insert/InsertDialog.tsx';
import {InlineLoading} from '../components/InlineLoading.tsx';

void loadCatalog('workspace-mail');

export function MessageTransferDialog(props:{request:MessageTransferRequest;activeLink:string|null;onClose:()=>void;onSuccess:(result:TransferOutcome)=>void}):JSX.Element{
 const controller=new AbortController();
 onCleanup(()=>controller.abort());
 const [targets]=createResource(()=>workspaceMailboxes(props.activeLink));
 const source=()=>targets.state==='ready'?targets.latest?.find(target=>target.linkId===props.request.sourceLink):undefined;
 const destinations=()=>source()?targets.latest?.filter(target=>target.storageAuthority!==source()!.storageAuthority)??[]:[];
 const [chosen,setChosen]=createSignal('');
 const target=()=>destinations().find(item=>(item.linkId??'primary')===chosen());
 const [destination]=createResource(target,async account=>{
  const [formats,folders]=await Promise.all([transferFormats(account,controller.signal),workspaceFolders(account,controller.signal)]);
  return {account,available:formats.engineMode&&formats.available,folders:folders.filter(box=>box.mailwomanSearchQuery===undefined&&box.myRights?.mayAddItems===true)};
 });
 const folderList=()=>destination.state==='ready'&&destination.latest?.account===target()?destination.latest?.folders??[]:[];
 const [folder,setFolder]=createSignal('');
 const [move,setMove]=createSignal(false);
 const [busy,setBusy]=createSignal(false);
 const [error,setError]=createSignal(false);
 const [outcome,setOutcome]=createSignal<TransferOutcome>();
 const allowed=()=>transfersEnabled()&&preferenceAccountIsCurrent(props.request.owner);
 const ready=()=>Boolean(allowed()&&source()&&target()&&destination.state==='ready'&&destination.latest?.available&&folderList().some(box=>box.id===folder())&&!busy()&&!outcome());
 async function submit():Promise<void>{
  const from=source(),to=target();
  if(!ready()||!from||!to)return;
  setBusy(true);setError(false);
  try{
   const result=await transferWorkspaceMessage({source:from,target:to,emailId:props.request.email.id,folderId:folder(),move:move(),signal:controller.signal,allowed});
   if(controller.signal.aborted)return;
   if(result==='copied'||result==='moved'){props.onSuccess(result);props.onClose();}
   else setOutcome(result);
  }catch{if(!controller.signal.aborted)setError(true);}
  finally{setBusy(false);}
 }
 return <InsertDialog title={t('workspace-transfer-title')} testId="workspace-transfer" busy={busy()}
  onClose={()=>{if(!busy())props.onClose();}} onSubmit={()=>void submit()} submitLabel={t(move()?'workspace-transfer-move':'workspace-transfer-copy')} submitDisabled={!ready()}>
  <p>{props.request.email.subject||t('mail-no-subject')}</p>
  <Show when={targets.loading||destination.loading}><InlineLoading message={t('common-loading')}/></Show>
  <Show when={targets.error||destination.error||error()}><p role="alert">{t('workspace-transfer-failed')}</p></Show>
  <Show when={!targets.loading&&!targets.error&&!destinations().length}><p>{t('workspace-transfer-no-accounts')}</p></Show>
  <label>{t('workspace-transfer-target')} <select disabled={busy()||!!outcome()} value={chosen()} onChange={event=>{setChosen(event.currentTarget.value);setFolder('');}}>
   <option value="">{t('workspace-transfer-select-account')}</option><For each={destinations()}>{item=><option value={item.linkId??'primary'}>{item.label}</option>}</For>
  </select></label>
  <Show when={target()&&destination.state==='ready'&&destination.latest?.account===target()}>
   <Show when={destination.latest?.available&&folderList().length} fallback={<p>{t('workspace-transfer-unavailable')}</p>}>
    <label>{t('workspace-transfer-folder')} <select disabled={busy()||!!outcome()} value={folder()} onChange={event=>setFolder(event.currentTarget.value)}>
     <option value="">{t('workspace-transfer-select-folder')}</option><For each={folderList()}>{box=><option value={box.id}>{box.name}</option>}</For>
    </select></label>
   </Show>
  </Show>
  <label><input type="checkbox" checked={move()} disabled={busy()||!!outcome()||props.request.email.keywords?.['$draft']===true} onChange={event=>setMove(event.currentTarget.checked)}/> {t('workspace-transfer-move-choice')}</label>
  <Show when={outcome()}>{result=><p role="alert">{t(`workspace-transfer-${result()}`)}</p>}</Show>
 </InsertDialog>;
}
