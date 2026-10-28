// Empty a whole folder (t45). An ordinary folder's messages move to Trash; Trash
// and Junk, and any folder of an account that has no Trash, are emptied for good.
// Child folders are never touched: only messages whose mailbox is this one.
//
// On the native engine the server does the work, one bounded batch per call
// (`Mailbox/empty`, see crates/mw-engine/src/empty_folder.rs): it decides the
// outcome, refuses a request whose outcome or count no longer matches what the
// user confirmed, and checks the folder belongs to this account. Against an
// upstream JMAP server (proxy mode) the same loop runs on standard methods:
// `Email/query` pages of the folder, then `Email/set` moves or destroys them.
import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type EmailQueryResponse,type EmailSetResponse,type JmapSession,type Mailbox} from './jmap-types.ts';
import {responseFor} from './jmap.ts';

export type EmptyOutcome='trash'|'destroy';
/** What this account's backend can do. `native` means the server speaks `Mailbox/empty`. */
export interface EmptyAbility {native:boolean;moveToTrash:boolean;destroy:boolean}
export interface EmptyPreview {accountId:string;mailboxId:string;name:string;role:string|null;outcome:EmptyOutcome;trashId:string|null;total:number;native:boolean}
export interface EmptyResult {processed:number;remaining:number;refused:number;uncertain:boolean;stopped:boolean}
export interface EmptyAuthority {current:()=>boolean;signal:AbortSignal;stopRequested?:()=>boolean}
type Transport=Pick<Client,'jmap'>;

/** A permanent empty of more than this many messages asks the user to type the folder name. */
export const TYPED_CONFIRMATION_OVER=100;
export const EMPTY_BATCH=100;
const PERMANENT_ROLES=new Set(['trash','junk']);

/** No extension in the session (an upstream JMAP server): standard methods, both outcomes. */
export function emptyAbility(session:Pick<JmapSession,'accounts'>|null|undefined,accountId:string):EmptyAbility {
 const mail=session?.accounts?.[accountId]?.accountCapabilities?.[CAP_MAIL] as {mailwomanEmptyMailbox?:{moveToTrash?:unknown;destroy?:unknown}}|undefined;
 const ext=mail?.mailwomanEmptyMailbox;
 if(!ext)return {native:false,moveToTrash:true,destroy:true};
 return {native:true,moveToTrash:ext.moveToTrash===true,destroy:ext.destroy===true};
}

function trashOf(mailboxes:readonly Mailbox[]):Mailbox[] {return mailboxes.filter(box=>box.role==='trash'&&box.mailwomanSearchQuery===undefined);}

/** The outcome the client expects; the native server has the final say in the preview. */
export function expectedOutcome(box:Mailbox,mailboxes:readonly Mailbox[]):{outcome:EmptyOutcome;trashId:string|null} {
 if(box.role!==null&&PERMANENT_ROLES.has(box.role))return {outcome:'destroy',trashId:null};
 const trash=trashOf(mailboxes).filter(item=>item.id!==box.id);
 return trash.length===1?{outcome:'trash',trashId:trash[0]!.id}:{outcome:'destroy',trashId:null};
}

/** Whether "Empty folder…" is offered for `box` at all. */
export function canEmpty(box:Mailbox,mailboxes:readonly Mailbox[],ability:EmptyAbility|null):boolean {
 if(!ability||box.mailwomanSearchQuery!==undefined)return false;
 const rights=(box as Mailbox&{myRights?:{mayRemoveItems?:boolean}}).myRights;
 if(rights?.mayRemoveItems===false)return false;
 return expectedOutcome(box,mailboxes).outcome==='trash'?ability.moveToTrash:ability.destroy;
}

function admitted(authority:EmptyAuthority):void {if(!authority.current()||authority.signal.aborted)throw new Error('Folder operation is no longer active');}

interface NativeReply {accountId:string;mailboxId:string;outcome:EmptyOutcome;trashId:string|null;total:number;processed:number;remaining:number;notProcessed?:Record<string,unknown>}
interface NativeRefusal {type:string;description?:string;mailwomanTotal?:number;mailwomanOutcome?:EmptyOutcome}
type NativeAnswer={ok:true;reply:NativeReply}|{ok:false;refusal:NativeRefusal};

async function nativeCall(client:Transport,args:Record<string,unknown>,authority:EmptyAuthority):Promise<NativeAnswer> {
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/empty',args,'empty']]},{signal:authority.signal});
 const found=response.methodResponses.find(item=>item[2]==='empty');if(!found)throw new Error('No Mailbox/empty response');
 if(found[0]==='error')return {ok:false,refusal:found[1] as unknown as NativeRefusal};
 const reply=found[1] as unknown as NativeReply;
 if(reply.accountId!==args['accountId']||reply.mailboxId!==args['mailboxId']||typeof reply.total!=='number'||typeof reply.processed!=='number'||typeof reply.remaining!=='number')throw new Error('Invalid Mailbox/empty response');
 return {ok:true,reply};
}

/** The count and outcome the dialog shows. On the native engine this asks the
 *  server (and takes its count and outcome when they differ from the cached
 *  folder list); it changes nothing. */
export async function prepareEmpty(client:Transport,accountId:string,box:Mailbox,mailboxes:readonly Mailbox[],ability:EmptyAbility,authority:EmptyAuthority):Promise<EmptyPreview> {
 admitted(authority);
 let {outcome,trashId}=expectedOutcome(box,mailboxes);
 const base={accountId,mailboxId:box.id,name:box.name,role:box.role,native:ability.native};
 if(ability.native){
  let total=box.totalEmails;
  for(let attempt=0;attempt<3;attempt++){
   const answer=await nativeCall(client,{accountId,mailboxId:box.id,outcome,expectedTotal:total,limit:0},authority);admitted(authority);
   if(answer.ok)return {...base,outcome:answer.reply.outcome,trashId:answer.reply.trashId,total:answer.reply.total};
   const refusal=answer.refusal;
   if(refusal.type!=='stateMismatch')throw new Error(refusal.description??refusal.type);
   if(typeof refusal.mailwomanTotal==='number')total=refusal.mailwomanTotal;
   if(refusal.mailwomanOutcome==='trash'||refusal.mailwomanOutcome==='destroy'){outcome=refusal.mailwomanOutcome;trashId=null;}
  }
  throw new Error('The folder keeps changing');
 }
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{inMailbox:box.id},calculateTotal:true,limit:0},'count']]},{signal:authority.signal});admitted(authority);
 const counted=responseFor<EmailQueryResponse>(response,'count');
 if(counted.accountId!==accountId)throw new Error('Invalid count');
 return {...base,outcome,trashId,total:typeof counted.total==='number'?counted.total:box.totalEmails};
}

/** Run the empty batch by batch. Stops when the folder is empty, a batch makes
 *  no progress or is refused, the previewed count has been reached (mail that
 *  keeps arriving is not chased), the user asks to stop, or the account changes.
 *  An error during a mutation leaves that batch's outcome unknown: no retry. */
export async function runEmpty(client:Transport,preview:EmptyPreview,authority:EmptyAuthority,progress:(result:EmptyResult)=>void=()=>{}):Promise<EmptyResult> {
 const result:EmptyResult={processed:0,remaining:preview.total,refused:0,uncertain:false,stopped:false};
 const ceiling=preview.total+Math.max(2,Math.floor(preview.total/20));
 let mutating=false;
 try{
  for(;;){
   if(result.remaining===0)break;
   if(authority.stopRequested?.()||result.processed>=ceiling){result.stopped=result.remaining>0;break;}
   admitted(authority);
   const limit=Math.min(EMPTY_BATCH,Math.max(1,ceiling-result.processed));
   let processed=0,refused=0,remaining=0;
   mutating=true;
   if(preview.native){
    const answer=await nativeCall(client,{accountId:preview.accountId,mailboxId:preview.mailboxId,outcome:preview.outcome,expectedTotal:result.remaining,limit},authority);
    mutating=false;
    if(!answer.ok){result.stopped=true;break;}
    processed=answer.reply.processed;refused=Object.keys(answer.reply.notProcessed??{}).length;remaining=answer.reply.remaining;
   }else{
    ({processed,refused,remaining}=await standardBatch(client,preview,limit,authority));
    mutating=false;
   }
   if(!authority.current())throw new Error('Account changed');
   result.processed+=processed;result.refused+=refused;result.remaining=remaining;
   progress({...result});
   if(refused>0||processed===0){result.stopped=result.remaining>0;break;}
  }
 }catch{result.uncertain=mutating;result.stopped=true;}
 return result;
}

/** One batch on standard JMAP (proxy mode): the folder's first page, then one
 *  `Email/set` that moves (patching `mailboxIds`) or destroys exactly those ids. */
async function standardBatch(client:Transport,preview:EmptyPreview,limit:number,authority:EmptyAuthority):Promise<{processed:number;refused:number;remaining:number}> {
 const {accountId,mailboxId}=preview;
 const page=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{inMailbox:mailboxId},position:0,limit},'page']]},{signal:authority.signal});admitted(authority);
 const query=responseFor<EmailQueryResponse>(page,'page');
 if(query.accountId!==accountId||!Array.isArray(query.ids)||query.ids.length>limit||new Set(query.ids).size!==query.ids.length)throw new Error('Invalid folder page');
 const ids=query.ids;if(!ids.length)return {processed:0,refused:0,remaining:0};
 const set=preview.outcome==='trash'&&preview.trashId
  ?{update:Object.fromEntries(ids.map(id=>[id,{[`mailboxIds/${mailboxId}`]:null,[`mailboxIds/${preview.trashId}`]:true}]))}
  :{destroy:ids};
 if(preview.outcome==='trash'&&!preview.trashId)throw new Error('No Trash folder');
 const mutation=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId,...set},'set'],['Email/query',{accountId,filter:{inMailbox:mailboxId},calculateTotal:true,limit:0},'left']]},{signal:authority.signal});
 const changed=responseFor<EmailSetResponse>(mutation,'set');
 if(changed.accountId!==accountId)throw new Error('Invalid response');
 const done=preview.outcome==='trash'?Object.keys(changed.updated??{}).filter(id=>ids.includes(id)).length:(changed.destroyed??[]).filter(id=>ids.includes(id)).length;
 const refused=Object.keys((preview.outcome==='trash'?changed.notUpdated:changed.notDestroyed)??{}).length;
 if(done+refused!==ids.length)throw new Error('The response omitted results');
 let remaining:number;
 try{const left=responseFor<EmailQueryResponse>(mutation,'left');remaining=typeof left.total==='number'?left.total:Math.max(0,preview.total-done);}catch{remaining=Math.max(0,preview.total-done);}
 return {processed:done,refused,remaining};
}
