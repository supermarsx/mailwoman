import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type Mailbox} from './jmap-types.ts';
import type {RenameAbility,RenameAuthority} from './renameFolder.ts';

export function canDeleteFolder(box:Mailbox,boxes:readonly Mailbox[],ability:RenameAbility|null):boolean {
 return !!ability?.manages&&box.role===null&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayDelete===true&&!boxes.some(child=>child.parentId===box.id);
}
export type DeleteFolderProblem='mailboxHasEmail'|'mailboxHasChild'|'forbidden'|'notFound'|'failed';
export type DeleteFolderOutcome={ok:true}|{ok:false;problem:DeleteFolderProblem};
export async function deleteFolder(client:Pick<Client,'jmap'>,accountId:string,mailboxId:string,removeEmails:boolean,authority:RenameAuthority):Promise<DeleteFolderOutcome>{
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,destroy:[mailboxId],onDestroyRemoveEmails:removeEmails===true},'delete-folder']]},authority.signal?{signal:authority.signal}:undefined);
 if(!authority.current()||authority.signal?.aborted)return {ok:false,problem:'failed'};
 const found=response.methodResponses.find(item=>item[2]==='delete-folder');
 if(found?.[0]!=='Mailbox/set')return {ok:false,problem:'failed'};
 const reply=found[1] as {accountId?:string;destroyed?:string[];notDestroyed?:Record<string,{type?:string}>};
 if(reply.accountId!==accountId)return {ok:false,problem:'failed'};
 if(reply.destroyed?.includes(mailboxId))return {ok:true};
 const type=reply.notDestroyed?.[mailboxId]?.type;
 return {ok:false,problem:type==='mailboxHasEmail'||type==='mailboxHasChild'||type==='forbidden'||type==='notFound'?type:'failed'};
}
