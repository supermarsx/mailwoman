import {ApiError,type Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type Mailbox} from './jmap-types.ts';
import type {RenameAbility,RenameAuthority} from './renameFolder.ts';

export function canDeleteFolder(box:Mailbox,boxes:readonly Mailbox[],ability:RenameAbility|null):boolean {
 return !!ability?.manages&&box.role===null&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayDelete===true&&!boxes.some(child=>child.parentId===box.id);
}
/** A refusal that holds for the whole account (a legal hold or a processing restriction): the server answers 403 with this code and `scope: "folder"`. */
export type HoldProblem='legalHold'|'processingRestricted';
/** The hold or restriction behind a refused folder delete, or null for any other failure. */
export function holdProblem(error:unknown):HoldProblem|null {
 return error instanceof ApiError&&error.status===403&&(error.code==='legalHold'||error.code==='processingRestricted')?error.code:null;
}
export type DeleteFolderProblem='mailboxHasEmail'|'mailboxHasChild'|'forbidden'|'notFound'|'failed'|HoldProblem;
export type DeleteFolderOutcome={ok:true}|{ok:false;problem:DeleteFolderProblem};
export async function deleteFolder(client:Pick<Client,'jmap'>,accountId:string,mailboxId:string,removeEmails:boolean,authority:RenameAuthority):Promise<DeleteFolderOutcome>{
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 let response:Awaited<ReturnType<Client['jmap']>>;
 try{response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,destroy:[mailboxId],onDestroyRemoveEmails:removeEmails===true},'delete-folder']]},authority.signal?{signal:authority.signal}:undefined);}
 catch(error){const hold=holdProblem(error);if(hold)return {ok:false,problem:hold};throw error;}
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 const found=response.methodResponses.find(item=>item[2]==='delete-folder');
 if(found?.[0]!=='Mailbox/set')return {ok:false,problem:'failed'};
 const reply=found[1] as {accountId?:string;destroyed?:string[];notDestroyed?:Record<string,{type?:string}>};
 if(reply.accountId!==accountId)return {ok:false,problem:'failed'};
 if(reply.destroyed?.includes(mailboxId))return {ok:true};
 const type=reply.notDestroyed?.[mailboxId]?.type;
 return {ok:false,problem:type==='mailboxHasEmail'||type==='mailboxHasChild'||type==='forbidden'||type==='notFound'?type:'failed'};
}
