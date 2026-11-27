import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {readWorkspaceJmap,workspaceEffectiveSettings,updateWorkspaceMessages,replayWorkspaceFolders,type UnifiedRow,type WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Mailbox} from '../api/jmap-types.ts';
import {eligibleSmartArchiveFolders,needsSmartArchiveReview,planSmartArchive,smartArchivePolicy,type SmartArchiveGroup,type SmartArchiveStrategy} from './smartArchive.ts';
import {requestSmartArchive} from './smartArchiveRequest.ts';
import {recordUndoable,undoOnly} from './undoHistory.ts';
import {toast} from '../components/toast/toast.ts';
import {t} from '../i18n';
/** All lookups, effective policies, moves and undo retain their explicit mailbox target. */
export async function smartArchiveWorkspace(rows:UnifiedRow[],boxesFor:(row:UnifiedRow)=>Mailbox[],signal:AbortSignal,strategy?:SmartArchiveStrategy,onChanged?:()=>void):Promise<UnifiedRow[]>{
 const owner=preferenceAccount();if(!owner||!smartArchivePolicy().enabled)return [];
 const current=()=>!signal.aborted&&preferenceAccountIsCurrent(owner);
 const plans:{target:WorkspaceMailbox;boxes:Mailbox[];rows:UnifiedRow[];groups:SmartArchiveGroup[]}[]=[];
 let review=false;
 for(const target of new Map(rows.map(row=>[row.target.storageAuthority,row.target])).values()){
  if(!current())return [];
  const subset=rows.filter(row=>row.target.storageAuthority===target.storageAuthority),boxes=boxesFor(subset[0]!);
  const policy=smartArchivePolicy(await workspaceEffectiveSettings(target,signal),true);
  if(!current())return [];
  if(!policy.enabled){toast.info(t('mail-smart-archive-disabled'));continue;}
  const groups=await planSmartArchive(subset.map(row=>row.email),boxes,policy,strategy??policy.strategy,target.accountId,request=>readWorkspaceJmap(target,request,signal));
  if(!current())return [];
  review=review||needsSmartArchiveReview(groups,policy);
  plans.push({target,boxes,rows:subset,groups:groups.map(group=>({...group,key:JSON.stringify([target.storageAuthority,group.key]),label:`${target.label} · ${group.label}`}))});
 }
 let groups=plans.flatMap(plan=>plan.groups);
 if(!groups.length){if(current())toast.info(t('mail-smart-archive-no-destination'));return [];}
 if(review){const chosen=await requestSmartArchive(owner,groups);if(!chosen||!current())return [];groups=chosen;}
 // Recheck every source policy before the first write.
 for(const plan of plans){
  const policy=smartArchivePolicy(await workspaceEffectiveSettings(plan.target,signal),true),allowed=new Set(eligibleSmartArchiveFolders(plan.boxes,policy,policy.fallback==='archive').map(box=>box.id));
  if(!current()||!policy.enabled)return [];
  plan.groups=plan.groups.map(group=>groups.find(value=>value.key===group.key)!);
  if(plan.groups.some(group=>!group||!allowed.has(group.target)||!group.candidates.some(candidate=>candidate.id===group.target)))throw new Error('Smart Archive destination changed');
 }
 const moved:UnifiedRow[]=[],replays:{target:WorkspaceMailbox;changes:{id:string;expected:Record<string,boolean>;folders:Record<string,boolean>}[]}[]=[];
 for(const plan of plans)for(const destination of new Set(plan.groups.map(group=>group.target))){
  if(!current())return moved;
  const ids=new Set(plan.groups.filter(group=>group.target===destination).flatMap(group=>group.ids)),subset=plan.rows.filter(row=>ids.has(row.email.id));
  let written:string[];
  try{written=await updateWorkspaceMessages(plan.target,plan.boxes,subset.map(row=>row.email),'move',signal,destination);}
  catch{if(current())toast.error(t('mail-context-failed'));continue;}
  const successful=subset.filter(row=>written.includes(row.email.id));moved.push(...successful);
  if(successful.length)replays.push({target:plan.target,changes:successful.map(row=>({id:row.email.id,expected:{[destination]:true},folders:{...row.email.mailboxIds}}))});
 }
 if(!current())return moved;
 if(moved.length){
  const label=t('mail-smart-archive-count',{count:moved.length});
  const replay=async(reverse:boolean)=>{if(!preferenceAccountIsCurrent(owner))throw new Error('Smart Archive account changed');for(const batch of replays){if(!preferenceAccountIsCurrent(owner))throw new Error('Smart Archive account changed');await replayWorkspaceFolders(batch.target,batch.changes.map(change=>reverse?change:{id:change.id,expected:change.folders,folders:change.expected}),new AbortController().signal);}onChanged?.();};
  const entry=recordUndoable({label,accountId:owner.id,undo:()=>replay(true),redo:()=>replay(false)});
  toast.success(label,entry?{action:{label:t('mail-undo-action'),run:async()=>{await undoOnly(entry.id);}}}:{});
 }
 if(moved.length<rows.length)toast.info(t('mail-smart-archive-skipped',{count:rows.length-moved.length}));
 return moved;
}
