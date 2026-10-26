import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type EmailGetResponse,type EmailQueryResponse,type EmailSetResponse,type MailboxGetResponse} from './jmap-types.ts';
import {responseFor} from './jmap.ts';
export const TRASH_LIMIT=5000;
export interface TrashSnapshot {accountId:string;mailboxId:string;ids:string[];more:boolean}
export interface TrashResult {deleted:number;skipped:number;remaining:number;stopped:boolean}
type Transport=Pick<Client,'jmap'>;
interface Authority {current:()=>boolean;signal:AbortSignal}
function admitted(authority:Authority):void {if(!authority.current()||authority.signal.aborted)throw new Error('Trash operation is no longer active');}
async function trashFolder(client:Transport,accountId:string,mailboxId:string,authority:Authority):Promise<void> {
 admitted(authority);const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/get',{accountId,ids:[mailboxId],properties:['id','role']},'box']]},{signal:authority.signal});admitted(authority);
 const result=responseFor<MailboxGetResponse>(response,'box');if(result.accountId!==accountId||result.list.length!==1||result.list[0]?.id!==mailboxId||result.list[0]?.role!=='trash')throw new Error('This folder is no longer the trash folder');
}
/** Preview a stable, bounded snapshot. New arrivals after confirmation are excluded. */
export async function prepareTrash(client:Transport,accountId:string,mailboxId:string,authority:Authority):Promise<TrashSnapshot> {
 await trashFolder(client,accountId,mailboxId,authority);const ids:string[]=[];let state:string|undefined;
 for(;;){
  admitted(authority);const size=ids.length===TRASH_LIMIT?1:Math.min(1000,TRASH_LIMIT-ids.length);
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{inMailbox:mailboxId},position:ids.length,limit:size},'query']]},{signal:authority.signal});admitted(authority);
  const result=responseFor<EmailQueryResponse>(response,'query');if(result.accountId!==accountId||result.position!==ids.length||typeof result.queryState!=='string'||(state!==undefined&&state!==result.queryState)||!Array.isArray(result.ids)||result.ids.length>size||result.ids.some(id=>typeof id!=='string'||!id||id.length>512))throw new Error('Trash changed while preparing the preview');
  state=result.queryState;if(ids.length===TRASH_LIMIT)return {accountId,mailboxId,ids,more:result.ids.length>0};
  ids.push(...result.ids);if(new Set(ids).size!==ids.length)throw new Error('Trash query repeated messages');
  if(result.ids.length<size)return {accountId,mailboxId,ids,more:false};
 }
}
/** No retries after an uncertain mutation. Recheck folder membership and state per batch. */
export async function emptyTrash(client:Transport,snapshot:TrashSnapshot,authority:Authority,progress:(result:TrashResult)=>void=()=>{}):Promise<TrashResult> {
 if(snapshot.ids.length>TRASH_LIMIT||new Set(snapshot.ids).size!==snapshot.ids.length)throw new Error('Invalid trash preview');
 const result:TrashResult={deleted:0,skipped:0,remaining:snapshot.ids.length,stopped:false};
 try {
  await trashFolder(client,snapshot.accountId,snapshot.mailboxId,authority);
  for(let offset=0;offset<snapshot.ids.length;offset+=100){
   admitted(authority);const ids=snapshot.ids.slice(offset,offset+100);const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId:snapshot.accountId,ids,properties:['id','mailboxIds']},'messages']]},{signal:authority.signal});admitted(authority);
   const messages=responseFor<EmailGetResponse>(response,'messages');if(messages.accountId!==snapshot.accountId||typeof messages.state!=='string'||!Array.isArray(messages.list)||messages.list.some(message=>!ids.includes(message.id)))throw new Error('Invalid message state');
   const targets=[...new Set(messages.list.filter(message=>message.mailboxIds?.[snapshot.mailboxId]).map(message=>message.id))];result.skipped+=ids.length-targets.length;
   if(targets.length){
    admitted(authority);const mutation=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId:snapshot.accountId,ifInState:messages.state,destroy:targets},'delete']]},{signal:authority.signal});admitted(authority);
    const changed=responseFor<EmailSetResponse>(mutation,'delete');if(changed.accountId!==snapshot.accountId)throw new Error('Invalid deletion response');
    const deleted=new Set((changed.destroyed??[]).filter(id=>targets.includes(id)));result.deleted+=deleted.size;if(deleted.size!==targets.length)result.stopped=true;
   }
   result.remaining=snapshot.ids.length-result.deleted-result.skipped;progress({...result});if(result.stopped)break;
  }
 }catch{result.stopped=true;}
 result.remaining=snapshot.ids.length-result.deleted-result.skipped;return result;
}
