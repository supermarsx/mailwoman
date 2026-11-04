import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type Mailbox} from './jmap-types.ts';
import {descendantsOf,isSystemFolder} from '../components/folderTree.ts';
import type {RenameAbility,RenameAuthority} from './renameFolder.ts';
/** Folder structure changes through Mailbox/set: moving a folder under another parent, and deleting a folder with its subfolders. */
export type StructureProblem='forbidden'|'exists'|'mailboxHasEmail'|'notFound'|'failed';
export type StructureOutcome={ok:true}|{ok:false;problem:StructureProblem};
type Transport=Pick<Client,'jmap'>;
const known=(type:string|undefined):StructureProblem=>type==='forbidden'||type==='notFound'||type==='mailboxHasEmail'?type:type==='invalidProperties'||type==='alreadyExists'?'exists':'failed';
/** Whether `box` may be moved under `parentId` (null: the top level): an ordinary folder you may rename, never into itself or below. */
export function canMoveFolder(box:Mailbox,boxes:readonly Mailbox[],ability:RenameAbility|null,parentId:string|null):boolean {
 if(!ability?.manages||isSystemFolder(box)||box.mailwomanSearchQuery!==undefined||box.myRights?.mayRename!==true)return false;
 if((box.parentId??null)===parentId)return false;
 if(parentId===null)return true;
 const parent=boxes.find(value=>value.id===parentId);
 return !!parent&&parent.id!==box.id&&parent.mailwomanSearchQuery===undefined&&parent.myRights?.mayCreateChild!==false&&!descendantsOf(boxes,box.id).some(value=>value.id===parentId);
}
/** Every folder of the subtree must be an ordinary folder the user may delete. */
export function canDeleteTree(box:Mailbox,boxes:readonly Mailbox[],ability:RenameAbility|null):boolean {
 if(!ability?.manages)return false;
 return [box,...descendantsOf(boxes,box.id)].every(value=>value.role===null&&value.mailwomanSearchQuery===undefined&&value.myRights?.mayDelete===true);
}
export async function moveFolderTo(client:Transport,accountId:string,mailboxId:string,parentId:string|null,authority:RenameAuthority):Promise<StructureOutcome> {
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,update:{[mailboxId]:{parentId}}},'move-folder']]},authority.signal?{signal:authority.signal}:undefined);
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 const found=response.methodResponses.find(item=>item[2]==='move-folder');
 if(found?.[0]!=='Mailbox/set')return {ok:false,problem:'failed'};
 const reply=found[1] as {accountId?:string;updated?:Record<string,unknown>|null;notUpdated?:Record<string,{type?:string}>|null};
 if(reply.accountId!==accountId)return {ok:false,problem:'failed'};
 if(reply.updated&&mailboxId in reply.updated)return {ok:true};
 return {ok:false,problem:known(reply.notUpdated?.[mailboxId]?.type)};
}
/** Delete `box` and all its subfolders, deepest first (a folder with children is refused by the server). `removeEmails` is the explicit policy for any message still inside. Stops at the first refusal and says what was left. */
export async function deleteFolderTree(client:Transport,accountId:string,box:Mailbox,boxes:readonly Mailbox[],removeEmails:boolean,authority:RenameAuthority):Promise<StructureOutcome&{deleted:number}> {
 const nested=descendantsOf(boxes,box.id).reverse();
 const order=[...nested,box];let deleted=0;
 for(const folder of order){
  if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed',deleted};
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,destroy:[folder.id],onDestroyRemoveEmails:removeEmails===true},'delete-folder']]},authority.signal?{signal:authority.signal}:undefined);
  if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed',deleted};
  const found=response.methodResponses.find(item=>item[2]==='delete-folder');
  const reply=found?.[1] as {accountId?:string;destroyed?:string[]|null;notDestroyed?:Record<string,{type?:string}>|null}|undefined;
  if(found?.[0]!=='Mailbox/set'||reply?.accountId!==accountId)return {ok:false,problem:'failed',deleted};
  if(!reply.destroyed?.includes(folder.id))return {ok:false,problem:known(reply.notDestroyed?.[folder.id]?.type),deleted};
  deleted++;
 }
 return {ok:true,deleted};
}
