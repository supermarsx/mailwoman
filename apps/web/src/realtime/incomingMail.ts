// Incoming mail is an Email/changes event, not a difference between pages of
// the visible list. Changing folders, marking unread and paging cannot notify.
import type {Client} from '../api/client.ts';
import {CAP_MAIL,type ChangesResponse,type Email,type EmailGetResponse,type JmapResponse,type Mailbox} from '../api/jmap-types.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
export interface IncomingMail {owner:PreferenceAccount;accountId:string;messages:Email[]}
interface Options {client:Pick<Client,'jmap'>;accountId:()=>string|null;mailboxes:()=>readonly Mailbox[];deliver:(batch:IncomingMail)=>void;now?:()=>number}
function result<T>(response:JmapResponse,name:string):T{
 const responsePart=response.methodResponses.find(part=>part[0]===name);
 if(responsePart)return responsePart[1] as T;
 const error=response.methodResponses.find(part=>part[0]==='error')?.[1];
 throw Object.assign(new Error('Incoming-mail state unavailable'),{type:error?.['type']});
}
export function incomingUnread(message:Email,mailboxes:readonly Mailbox[]):boolean{
 if(message.keywords?.['$seen']===true||message.keywords?.['$draft']===true)return false;
 const boxes=mailboxes.filter(box=>message.mailboxIds?.[box.id]);
 if(boxes.some(box=>box.role==='inbox'))return true;
 return boxes.length>0&&!boxes.every(box=>['junk','trash','drafts','sent'].includes(box.role??''));
}
export function createIncomingMail(options:Options):{start:()=>void;stop:()=>void;changed:()=>void}{
 let startedAt=0;const now=options.now??Date.now;
 let started=false,generation=0,cursor:string|null=null,busy=false,pending=false;
 let off:(()=>void)|undefined,controller:AbortController|undefined,timer:ReturnType<typeof setTimeout>|undefined;
 function reset():void{generation++;startedAt=now();controller?.abort();controller=undefined;cursor=null;busy=false;pending=false;if(timer)clearTimeout(timer);timer=undefined;}
 async function drain():Promise<void>{
  if(!started||busy)return;
  const owner=preferenceAccount(),accountId=options.accountId();if(!owner||!accountId)return;
  const epoch=generation;const current=()=>started&&epoch===generation&&preferenceAccountIsCurrent(owner)&&options.accountId()===accountId;
  const abort=new AbortController();controller=abort;busy=true;
  const request=async<T>(name:string,args:Record<string,unknown>):Promise<T>=>result<T>(await options.client.jmap({using:[CAP_MAIL],methodCalls:[[name,{accountId,...args},'incoming']]},{signal:abort.signal}),name);
  try{
   if(cursor===null){
    const seed=await request<EmailGetResponse>('Email/get',{ids:[],properties:['id']});
    if(!current()||seed.accountId!==accountId||typeof seed.state!=='string')return;
    cursor=seed.state; // First contact seeds state; it never announces old unread.
   }
   for(let page=0;pending&&page<8&&current();page++){
    pending=false;
    const change:ChangesResponse=await request<ChangesResponse>('Email/changes',{sinceState:cursor,maxChanges:100});
    if(!current())return;
    if(change.accountId!==accountId||typeof change.newState!=='string'||!Array.isArray(change.created)||change.created.length>1000||(change.hasMoreChanges&&change.newState===cursor))throw new Error('Invalid incoming-mail state');
    let messages:Email[]=[];
    if(change.created.length){
     const details=await request<EmailGetResponse>('Email/get',{ids:change.created,properties:['id','threadId','subject','from','mailboxIds','keywords','receivedAt']});
     if(!current()||details.accountId!==accountId||!Array.isArray(details.list))return;
     const ids=new Set(change.created);messages=details.list.filter(email=>ids.has(email.id)&&incomingUnread(email,options.mailboxes())&&(!Number.isFinite(Date.parse(email.receivedAt))||Date.parse(email.receivedAt)>=startedAt-300_000));
    }
    cursor=change.newState;
    if(messages.length)options.deliver({owner,accountId,messages});
    pending ||=change.hasMoreChanges;
   }
  }catch(error){
   if(current()){pending=false;if((error as {type?:string}|null)?.type==='cannotCalculateChanges')cursor=null;}
   // Keep the last cursor after a transient error. Reconnect or the next state
   // change retries the missing delta, without treating the visible list as new.
  }finally{
   if(epoch===generation){busy=false;controller=undefined;if(pending&&started){timer=setTimeout(()=>{timer=undefined;void drain();},0);}}
  }
 }
 return {
  start(){if(started)return;started=true;off=observePreferenceAccount(()=>{reset();void drain();});void drain();},
  stop(){started=false;off?.();off=undefined;reset();},
  changed(){if(!started)return;pending=true;void drain();},
 };
}
