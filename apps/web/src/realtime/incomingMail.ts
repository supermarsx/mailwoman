import {createRoot,createEffect} from 'solid-js';
// Incoming mail is an Email/changes event, not a difference between pages of
// the visible list. Changing folders, marking unread and paging cannot notify.
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
import type {Client} from '../api/client.ts';
import {CAP_MAIL,type ChangesResponse,type Email,type EmailGetResponse,type EmailQueryResponse,type MailboxGetResponse,type JmapResponse,type Mailbox} from '../api/jmap-types.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
export interface IncomingMail {owner:PreferenceAccount;accountId:string;messages:Email[];target?:WorkspaceMailbox;mailboxes?:readonly Mailbox[]}
interface Options {owner?:()=>PreferenceAccount|null;client:Pick<Client,'jmap'>;accountId:()=>string|null;mailboxes:()=>readonly Mailbox[];deliver:(batch:IncomingMail)=>void;now?:()=>number}
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
 let started=false,generation=0,cursor:string|null=null,busy=false,pending=false,recovering=false,failures=0;
 const announced=new Set<string>();
 let readyOff:(()=>void)|undefined;
 let off:(()=>void)|undefined,controller:AbortController|undefined,timer:ReturnType<typeof setTimeout>|undefined;
 function reset():void{generation++;startedAt=now();controller?.abort();controller=undefined;cursor=null;busy=false;pending=false;recovering=false;failures=0;announced.clear();if(timer)clearTimeout(timer);timer=undefined;}
 async function drain():Promise<void>{
  if(!started||busy)return;
  if(timer){clearTimeout(timer);timer=undefined;}
  const owner=options.owner?.()??preferenceAccount(),accountId=options.accountId();if(!owner||!accountId)return;
  const epoch=generation;const current=()=>started&&epoch===generation&&preferenceAccountIsCurrent(owner)&&options.accountId()===accountId;
  const abort=new AbortController();controller=abort;busy=true;
  const request=async<T>(name:string,args:Record<string,unknown>):Promise<T>=>result<T>(await options.client.jmap({using:[CAP_MAIL],methodCalls:[[name,{accountId,...args},'incoming']]},{signal:abort.signal}),name);
  const classify=async(list:Email[]):Promise<{messages:Email[];boxes:readonly Mailbox[]}>=>{
   let boxes=options.mailboxes();
   if(list.some(message=>Object.keys(message.mailboxIds??{}).some(id=>!boxes.some(box=>box.id===id)))){
    const refreshed=await request<MailboxGetResponse>('Mailbox/get',{});
    if(!current()||refreshed.accountId!==accountId||!Array.isArray(refreshed.list))throw new Error('Incoming mailbox metadata unavailable');boxes=refreshed.list;
   }
   return {messages:list.filter(message=>incomingUnread(message,boxes)),boxes};
  };
  try{
   if(cursor===null){
    const seed=await request<EmailGetResponse>('Email/get',{ids:[],properties:['id']});
    if(!current()||seed.accountId!==accountId||typeof seed.state!=='string')return;
    cursor=seed.state; // First contact seeds state; it never announces old unread.
    if(recovering){
     // An expired delta token needs a bounded recent-mail baseline, not a silent
     // reseed that forgets everything delivered while the stream was down.
     for(let position=0;position<800;position+=100){
      const query=await request<EmailQueryResponse>('Email/query',{filter:{after:new Date(startedAt).toISOString()},sort:[{property:'receivedAt',isAscending:false}],position,limit:100});
      if(!current()||query.accountId!==accountId||!Array.isArray(query.ids))return;
      if(query.ids.length){
       const got=await request<EmailGetResponse>('Email/get',{ids:query.ids,properties:['id','threadId','subject','from','mailboxIds','keywords','receivedAt']});
       if(!current()||got.accountId!==accountId||!Array.isArray(got.list))return;
       const classified=await classify(got.list);if(!current())return;
       const ids=new Set(query.ids);const messages=classified.messages.filter(message=>ids.has(message.id)&&!announced.has(message.id)&&Date.parse(message.receivedAt)>=startedAt);
       if(messages.length){for(const message of messages)announced.add(message.id);options.deliver({owner,accountId,messages,mailboxes:classified.boxes});}
      }
      if(query.ids.length<100)break;
     }
     recovering=false;pending=true;
    }
   }
   for(let page=0;pending&&page<8&&current();page++){
    pending=false;
    const change:ChangesResponse=await request<ChangesResponse>('Email/changes',{sinceState:cursor,maxChanges:100});
    if(!current())return;
    if(change.accountId!==accountId||typeof change.newState!=='string'||!Array.isArray(change.created)||change.created.length>1000||(change.hasMoreChanges&&change.newState===cursor))throw new Error('Invalid incoming-mail state');
    let messages:Email[]=[],boxes:readonly Mailbox[]=options.mailboxes();
    if(change.created.length){
     const details=await request<EmailGetResponse>('Email/get',{ids:change.created,properties:['id','threadId','subject','from','mailboxIds','keywords','receivedAt']});
     if(!current()||details.accountId!==accountId||!Array.isArray(details.list))return;
     const classified=await classify(details.list);if(!current())return;boxes=classified.boxes;
     const ids=new Set(change.created);messages=classified.messages.filter(email=>ids.has(email.id)&&(!Number.isFinite(Date.parse(email.receivedAt))||Date.parse(email.receivedAt)>=startedAt-300_000));
    }
    cursor=change.newState;
    const fresh=messages.filter(message=>!announced.has(message.id));
    if(fresh.length){for(const message of fresh)announced.add(message.id);while(announced.size>2000)announced.delete(announced.values().next().value!);options.deliver({owner,accountId,messages:fresh,mailboxes:boxes});}
    pending ||=change.hasMoreChanges;
   }
   failures=0;
  }catch(error){
   if(current()){
    pending=false;
    if((error as {type?:string}|null)?.type==='cannotCalculateChanges'){cursor=null;recovering=true;}
    if(recovering)cursor=null;
    failures++;
    if(timer)clearTimeout(timer);
    timer=setTimeout(()=>{timer=undefined;pending=true;void drain();},Math.min(30000,1000*2**Math.min(failures-1,5)));
   }
  }finally{
   if(epoch===generation){busy=false;controller=undefined;if(pending&&started){timer=setTimeout(()=>{timer=undefined;void drain();},0);}}
  }
 }
 return {
  start(){
   if(started)return;started=true;
   let lastAccount=options.accountId();
   readyOff=createRoot(dispose=>{createEffect(()=>{const account=options.accountId();if(account!==lastAccount){lastAccount=account;reset();void drain();}});return dispose;});
   off=observePreferenceAccount(()=>{reset();void drain();});void drain();
  },
  stop(){started=false;readyOff?.();readyOff=undefined;off?.();off=undefined;reset();},
  changed(){if(!started)return;pending=true;void drain();},
 };
}
