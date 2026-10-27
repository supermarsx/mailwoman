import type {Client} from '../api/client.ts';
import {CAP_CORE,CAP_MAIL,type EmailGetResponse,type EmailSetResponse,type MailboxGetResponse} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
export type FolderMutation={kind:'read';read:boolean}|{kind:'move';target:string};
export interface FolderMutationResult {updated:string[];refused:string[];skipped:string[];uncertain:string[]}
/** Bound to the caller's selected IDs and verified authority. Never retries a write. */
export async function mutateFolderMessages(client:Pick<Client,'jmap'>,input:{accountId:string;mailboxId:string;virtual:boolean;ids:readonly string[];operation:FolderMutation;current:()=>boolean;signal:AbortSignal}):Promise<FolderMutationResult>{
 const {accountId,mailboxId,operation,current,signal}=input;
 const ids=[...new Set(input.ids)];
 if(ids.length===0||ids.length>100||ids.some(id=>!id||id.length>512))throw new Error('Invalid selection');
 const result:FolderMutationResult={updated:[],refused:[],skipped:[],uncertain:[]};
 const check=()=>{if(!current()||signal.aborted)throw new DOMException('Mailbox changed','AbortError');};
 check();
 if(operation.kind==='move'){
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/get',{accountId,ids:[operation.target]},'target']]},{signal});check();
  const target=responseFor<MailboxGetResponse>(response,'target');
  if(target.accountId!==accountId||target.list.length!==1||target.list[0]?.id!==operation.target||target.list[0].mailwomanSearchQuery!==undefined)throw new Error('Destination unavailable');
 }
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId,ids,properties:['id','mailboxIds','keywords']},'selected']]},{signal});check();
 const owned=responseFor<EmailGetResponse>(response,'selected');
 if(owned.accountId!==accountId||typeof owned.state!=='string'||!Array.isArray(owned.list)||owned.list.some(email=>!ids.includes(email.id)))throw new Error('Invalid message state');
 const patches:Record<string,Record<string,unknown>>={};
 for(const id of ids){
  const email=owned.list.find(value=>value.id===id);
  if(!email||(!input.virtual&&!email.mailboxIds?.[mailboxId])){result.skipped.push(id);continue;}
  if(operation.kind==='read'){
   if((email.keywords?.['$seen']===true)===operation.read){result.skipped.push(id);continue;}
   patches[id]={'keywords/$seen':operation.read?true:null};
  }else{
   if(email.mailboxIds?.[operation.target]){result.skipped.push(id);continue;}
   patches[id]={mailboxIds:{[operation.target]:true}};
  }
 }
 const targets=Object.keys(patches);if(!targets.length)return result;
 check();
 try{
  const changed=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId,ifInState:owned.state,update:patches},'update']]},{signal});
  check();
  const tuple=changed.methodResponses.find(value=>value[2]==='update');
  if(tuple?.[0]==='error'){result.refused=targets;return result;}
  const set=responseFor<EmailSetResponse>(changed,'update');if(set.accountId!==accountId)throw new Error('Invalid mutation response');
  for(const id of targets){
   if(Object.hasOwn(set.updated??{},id))result.updated.push(id);
   else if(Object.hasOwn(set.notUpdated??{},id))result.refused.push(id);
   else result.uncertain.push(id);
  }
 }catch(error){if(!current()||signal.aborted)throw error;result.uncertain=targets;}
 return result;
}
