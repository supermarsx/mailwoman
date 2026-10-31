import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type EmailGetResponse,type EmailQueryResponse,type EmailSetResponse,type Mailbox} from './jmap-types.ts';
import {responseFor} from './jmap.ts';
import {expectedOutcome} from './emptyFolder.ts';
export type CleanupOutcome={outcome:'trash'|'destroy';trashId:string|null};
const pointer=(id:string)=>id.replaceAll('~','~0').replaceAll('/','~1');
export type AgeUnit='days'|'weeks'|'months'|'years';
export function cleanupCutoff(amount:number,unit:AgeUnit,now=new Date()):string {
 if(!Number.isSafeInteger(amount)||amount<1||amount>10000)throw new Error('Choose an age between 1 and 10000.');
 const date=new Date(now);
 if(unit==='days'||unit==='weeks')date.setUTCDate(date.getUTCDate()-amount*(unit==='weeks'?7:1));
 else {const day=date.getUTCDate();date.setUTCDate(1);if(unit==='months')date.setUTCMonth(date.getUTCMonth()-amount);else date.setUTCFullYear(date.getUTCFullYear()-amount);const last=new Date(date);last.setUTCMonth(last.getUTCMonth()+1,0);date.setUTCDate(Math.min(day,last.getUTCDate()));}
 return date.toISOString();
}
type Authority={current:()=>boolean;signal:AbortSignal};
function check(authority:Authority):void {if(!authority.current()||authority.signal.aborted)throw new Error('The active account changed.');}
export async function previewCleanup(client:Pick<Client,'jmap'>,accountId:string,mailboxId:string,before:string,authority:Authority):Promise<string[]> {
 const ids:string[]=[];let position=0;
 for(;;){check(authority);const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{inMailbox:mailboxId,before},sort:[{property:'receivedAt',isAscending:true}],position,limit:100},'old']]},{signal:authority.signal});check(authority);const page=responseFor<EmailQueryResponse>(response,'old');if(page.accountId!==accountId||!Array.isArray(page.ids)||page.ids.length>100)throw new Error('Invalid folder response.');ids.push(...page.ids);position+=page.ids.length;if(page.ids.length<100)break;if(position>=10000)throw new Error('Clean up a smaller age range first (maximum 10000 messages).');}
 return [...new Set(ids)];
}
export async function cleanupMessages(client:Pick<Client,'jmap'>,accountId:string,box:Mailbox,boxes:readonly Mailbox[],ids:readonly string[],before:string,authority:Authority,confirmed?:CleanupOutcome):Promise<number> {
 const outcome=expectedOutcome(box,boxes);if(confirmed&&(confirmed.outcome!==outcome.outcome||confirmed.trashId!==outcome.trashId))throw new Error('The cleanup destination changed. Preview the cleanup again.');let processed=0;
 for(let offset=0;offset<ids.length;offset+=100){check(authority);const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId,ids:ids.slice(offset,offset+100),properties:['id','mailboxIds','receivedAt']},'verify']]},{signal:authority.signal});check(authority);const emails=responseFor<EmailGetResponse>(response,'verify');if(emails.accountId!==accountId||!emails.state)throw new Error('Invalid message response.');const targets=emails.list.filter(email=>ids.slice(offset,offset+100).includes(email.id)&&email.mailboxIds[box.id]&&!!email.receivedAt&&Date.parse(email.receivedAt)<Date.parse(before)).map(email=>email.id);if(!targets.length)continue;
 const mutation=outcome.outcome==='trash'?{update:Object.fromEntries(targets.map(id=>[id,{[`mailboxIds/${pointer(box.id)}`]:null,[`mailboxIds/${pointer(outcome.trashId!)}`]:true}]))}:{destroy:targets};
 const changed=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId,ifInState:emails.state,...mutation},'clean']]},{signal:authority.signal});check(authority);const set=responseFor<EmailSetResponse>(changed,'clean');if(set.accountId!==accountId)throw new Error('Invalid cleanup response.');const done=outcome.outcome==='trash'?Object.keys(set.updated??{}):set.destroyed??[];processed+=targets.filter(id=>done.includes(id)).length;if(targets.some(id=>!done.includes(id)))throw new Error(`Cleanup stopped after ${processed} messages. Some messages could not be updated; refresh before trying again.`);
 }
 return processed;
}
