import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type EmailSetResponse} from './jmap-types.ts';
/** Whole-folder operations in bounded batches: mark all read, copy all, move all, and the newest-message time per folder.
 * Every call is one JMAP request of at most `BATCH` messages; none is retried, and a refusal is counted, never hidden. */
export const BATCH=100;
/** Pages never exceed this many, so a malformed server can not loop the client. */
const MAX_PAGES=2000;
export interface BulkAuthority {current:()=>boolean;signal?:AbortSignal;stopRequested?:()=>boolean}
export interface BulkResult {processed:number;refused:number;total:number;stopped:boolean;/** The messages that changed, so the change can be undone. */changed:string[]}
type Transport=Pick<Client,'jmap'>;
const pointer=(id:string)=>id.replaceAll('~','~0').replaceAll('/','~1');
const check=(authority:BulkAuthority)=>{if(!authority.current()||authority.signal?.aborted)throw new DOMException('Account changed','AbortError');};
const init=(authority:BulkAuthority)=>authority.signal?{signal:authority.signal}:undefined;
/** Every message id in a folder (optionally only the unread), in pages. */
export async function listFolderIds(client:Transport,accountId:string,mailboxId:string,authority:BulkAuthority,options:{unreadOnly?:boolean}={}):Promise<string[]> {
 const ids:string[]=[];
 for(let page=0;page<MAX_PAGES;page++){
  check(authority);
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{inMailbox:mailboxId,...(options.unreadOnly?{notKeyword:'$seen'}:{})},position:ids.length,limit:BATCH},'page']]},init(authority));
  check(authority);
  const found=response.methodResponses.find(row=>row[2]==='page');
  const reply=found?.[1] as {accountId?:string;ids?:string[]}|undefined;
  if(found?.[0]!=='Email/query'||reply?.accountId!==accountId||!Array.isArray(reply.ids))throw new Error('Invalid folder page');
  if(reply.ids.length===0)break;
  ids.push(...reply.ids);
  if(reply.ids.length<BATCH)break;
 }
 return [...new Set(ids)];
}
/** One Email/set of `patch` over `ids`; returns how many were updated and how many refused. */
async function patchAll(client:Transport,accountId:string,ids:readonly string[],patch:Record<string,unknown>,authority:BulkAuthority,progress?:(done:number)=>void):Promise<{updated:number;refused:number;stopped:boolean;changed:string[]}> {
 let updated=0,refused=0;const changed:string[]=[];
 for(let at=0;at<ids.length;at+=BATCH){
  if(authority.stopRequested?.())return {updated,refused,stopped:true,changed};
  check(authority);
  const slice=ids.slice(at,at+BATCH);
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId,update:Object.fromEntries(slice.map(id=>[id,patch]))},'set']]},init(authority));
  check(authority);
  const found=response.methodResponses.find(row=>row[2]==='set');
  if(found?.[0]!=='Email/set'){refused+=slice.length;continue;}
  const reply=found[1] as Partial<EmailSetResponse>;
  if(reply.accountId!==accountId){refused+=slice.length;continue;}
  const ok=Object.keys(reply.updated??{});changed.push(...ok);const done=ok.length;updated+=done;refused+=slice.length-done;
  progress?.(updated);
 }
 return {updated,refused,stopped:false,changed};
}
/** Mark every unread message in `mailboxIds` as read. */
export async function markFolderRead(client:Transport,accountId:string,mailboxIds:readonly string[],authority:BulkAuthority,progress?:(done:number,total:number)=>void):Promise<BulkResult> {
 const per=[];let total=0;
 for(const mailboxId of mailboxIds){const ids=await listFolderIds(client,accountId,mailboxId,authority,{unreadOnly:true});per.push(ids);total+=ids.length;}
 const ids=[...new Set(per.flat())];total=ids.length;
 const result=await patchAll(client,accountId,ids,{'keywords/$seen':true},authority,done=>progress?.(done,total));
 return {processed:result.updated,refused:result.refused,total,stopped:result.stopped,changed:result.changed};
}
/** Copy (add to `targetId`) or move (add to `targetId`, remove from `sourceId`) every message of a folder. */
export async function transferFolder(client:Transport,accountId:string,sourceId:string,targetId:string,mode:'copy'|'move',authority:BulkAuthority,progress?:(done:number,total:number)=>void):Promise<BulkResult> {
 if(sourceId===targetId)throw new Error('Source and destination are the same');
 const ids=await listFolderIds(client,accountId,sourceId,authority);
 const patch:Record<string,unknown>={[`mailboxIds/${pointer(targetId)}`]:true,...(mode==='move'?{[`mailboxIds/${pointer(sourceId)}`]:null}:{})};
 const result=await patchAll(client,accountId,ids,patch,authority,done=>progress?.(done,ids.length));
 return {processed:result.updated,refused:result.refused,total:ids.length,stopped:result.stopped,changed:result.changed};
}
/** The newest message time of each folder (ms since the epoch), for ordering folders by last activity. One request: a one-message query per folder, then one get. */
export async function lastActivity(client:Transport,accountId:string,mailboxIds:readonly string[],authority:BulkAuthority):Promise<Map<string,number>> {
 const out=new Map<string,number>();
 for(let at=0;at<mailboxIds.length;at+=25){
  check(authority);
  const slice=mailboxIds.slice(at,at+25);
  const calls=slice.flatMap((id,index)=>[
   ['Email/query',{accountId,filter:{inMailbox:id},sort:[{property:'receivedAt',isAscending:false}],limit:1},`q${index}`],
   ['Email/get',{accountId,'#ids':{resultOf:`q${index}`,name:'Email/query',path:'/ids'},properties:['receivedAt']},`g${index}`],
  ] as [string,Record<string,unknown>,string][]);
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:calls},init(authority));
  check(authority);
  slice.forEach((id,index)=>{
   const found=response.methodResponses.find(row=>row[2]===`g${index}`);
   const list=(found?.[1] as {list?:{receivedAt?:string}[]}|undefined)?.list;
   const time=Date.parse(list?.[0]?.receivedAt??'');if(Number.isFinite(time))out.set(id,time);
  });
 }
 return out;
}
/** Undo a mark-all-read: the messages it changed become unread again. */
export async function markUnread(client:Transport,accountId:string,ids:readonly string[],authority:BulkAuthority):Promise<BulkResult> {
 const result=await patchAll(client,accountId,ids,{'keywords/$seen':null},authority);
 return {processed:result.updated,refused:result.refused,total:ids.length,stopped:result.stopped,changed:result.changed};
}
/** Undo a move: `ids` go back from `targetId` to `sourceId`. */
export async function undoTransfer(client:Transport,accountId:string,sourceId:string,targetId:string,ids:readonly string[],authority:BulkAuthority):Promise<BulkResult> {
 const result=await patchAll(client,accountId,ids,{[`mailboxIds/${pointer(sourceId)}`]:true,[`mailboxIds/${pointer(targetId)}`]:null},authority);
 return {processed:result.updated,refused:result.refused,total:ids.length,stopped:result.stopped,changed:result.changed};
}
