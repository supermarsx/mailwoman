// The user's own sent messages, for the "Replaces an earlier message" picker in
// Message options, and the lookup behind the reader's "Superseded by a newer
// message" badge. Both go through the engine's search (`supersedes:<Message-ID>`
// is an indexed keyword written at ingest); answers are cached per message.
import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL} from './jmap-types.ts';

export interface SentChoice {id:string;messageId:string;subject:string;date:string|null}

/** `<id@host>` for a JMAP `messageId` entry (which comes without brackets). */
export const bracketed=(messageId:string):string=>`<${messageId.replace(/^<|>$/g,'')}>`;

/** Recent messages in the Sent folder whose subject or recipients match `query` (all of them when empty). */
export async function findSentMessages(client:Pick<Client,'jmap'>,accountId:string,sentMailboxId:string,query:string,limit=15):Promise<SentChoice[]> {
 const text=query.trim().replace(/["()]/g,' ').trim();
 const filter:Record<string,unknown>={inMailbox:sentMailboxId,...(text!==''?{text}:{})};
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[
  ['Email/query',{accountId,filter,sort:[{property:'receivedAt',isAscending:false}],limit},'q'],
  ['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','subject','receivedAt','sentAt','messageId']},'g'],
 ]});
 const get=response.methodResponses.find(item=>item[2]==='g');
 if(!get||get[0]==='error')return [];
 const list=(get[1] as {list?:{id:string;subject?:string|null;receivedAt?:string;sentAt?:string|null;messageId?:string[]|null}[]}).list??[];
 return list.flatMap(email=>{
  const messageId=email.messageId?.[0];
  return messageId?[{id:email.id,messageId:bracketed(messageId),subject:email.subject??'',date:email.sentAt??email.receivedAt??null}]:[];
 });
}

const supersededCache=new Map<string,boolean>();
export function forgetSuperseded():void {supersededCache.clear();}

/** Whether a newer message says it replaces `messageId` (the engine's `supersedes:` index). `false` when the search is unavailable. */
export async function isSuperseded(client:Pick<Client,'jmap'>,accountId:string,messageId:string):Promise<boolean> {
 const id=messageId.replace(/^<|>$/g,'').trim().toLowerCase();
 if(!id.includes('@')||/[\s"()]/u.test(id))return false;
 const key=`${accountId}|${id}`,known=supersededCache.get(key);
 if(known!==undefined)return known;
 try{
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/query',{accountId,filter:{text:`supersedes:${id}`},limit:1,calculateTotal:true},'q']]});
  const query=response.methodResponses.find(item=>item[2]==='q');
  const total=query&&query[0]!=='error'?(query[1] as {total?:unknown}).total:undefined;
  if(typeof total!=='number')return false;
  supersededCache.set(key,total>0);
  return total>0;
 }catch{return false;}
}
