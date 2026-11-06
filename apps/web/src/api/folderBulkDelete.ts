import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type Mailbox} from './jmap-types.ts';
import {descendantsOf} from '../components/folderTree.ts';
import {canDeleteTree,deleteFolderTree,type StructureProblem} from './folderStructure.ts';
import type {RenameAbility,RenameAuthority} from './renameFolder.ts';
/** Bulk folder actions of Settings > Folders: delete several folders at once, subscribe or unsubscribe several. */
type Transport=Pick<Client,'jmap'>;

export interface DeletePlan {
 /** Top-most folders to delete (a selected folder inside another selected one goes with it). */
 roots:Mailbox[];
 /** Every folder that goes, roots and their subfolders. */
 folders:Mailbox[];
 /** Messages held in those folders (a message filed in two of them counts twice). */
 messages:number;
 /** Selected folders that cannot be deleted: system folders, saved searches, folders the account may not delete. */
 skipped:Mailbox[];
}
/** What deleting `selected` would remove, without removing anything. */
export function deletePlan(selected:readonly Mailbox[],boxes:readonly Mailbox[],ability:RenameAbility|null):DeletePlan {
 const eligible=selected.filter(box=>canDeleteTree(box,boxes,ability));
 const skipped=selected.filter(box=>!eligible.includes(box));
 const eligibleIds=new Set(eligible.map(box=>box.id));
 const roots=eligible.filter(box=>{
  let cursor=boxes.find(value=>value.id===box.parentId),guard=0;
  while(cursor&&guard++<64){if(eligibleIds.has(cursor.id))return false;cursor=boxes.find(value=>value.id===cursor?.parentId);}
  return true;
 });
 const folders=roots.flatMap(root=>[root,...descendantsOf(boxes,root.id)]);
 return {roots,folders,messages:folders.reduce((sum,box)=>sum+(box.totalEmails||0),0),skipped};
}
export interface BulkDeleteResult {deleted:string[];failed:{id:string;problem:StructureProblem}[]}
/** Delete the plan's roots one after another, each with its subfolders. Continues past a refusal and reports it. */
export async function bulkDeleteFolders(client:Transport,accountId:string,plan:DeletePlan,boxes:readonly Mailbox[],authority:RenameAuthority):Promise<BulkDeleteResult> {
 const result:BulkDeleteResult={deleted:[],failed:[]};
 for(const root of plan.roots){
  if(!authority.current()||authority.signal?.aborted){result.failed.push({id:root.id,problem:'failed'});continue;}
  const outcome=await deleteFolderTree(client,accountId,root,boxes,true,authority);
  if(outcome.ok)result.deleted.push(...[root,...descendantsOf(boxes,root.id)].map(box=>box.id));
  else result.failed.push({id:root.id,problem:outcome.problem});
 }
 return result;
}
export interface SubscribeResult {updated:string[];failed:string[]}
/** Subscribe or unsubscribe folders in one request (JMAP `isSubscribed`; IMAP-backed accounts SUBSCRIBE/UNSUBSCRIBE on the engine side). */
export async function setFoldersSubscribed(client:Transport,accountId:string,ids:readonly string[],subscribed:boolean,authority:RenameAuthority):Promise<SubscribeResult> {
 if(!ids.length||!authority.current()||authority.signal?.aborted)return {updated:[],failed:[...ids]};
 const update=Object.fromEntries(ids.map(id=>[id,{isSubscribed:subscribed}]));
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,update},'subscribe-folders']]},authority.signal?{signal:authority.signal}:undefined);
 const found=response.methodResponses.find(item=>item[2]==='subscribe-folders');
 if(found?.[0]!=='Mailbox/set'||(found[1] as {accountId?:string}).accountId!==accountId)return {updated:[],failed:[...ids]};
 const reply=found[1] as {updated?:Record<string,unknown>|null};
 const updated=ids.filter(id=>!!reply.updated&&id in reply.updated);
 return {updated,failed:ids.filter(id=>!updated.includes(id))};
}
