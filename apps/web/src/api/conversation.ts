import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type Email,type EmailGetResponse} from './jmap-types.ts';
import {responseFor} from './jmap.ts';

export const CONVERSATION_PAGE=50;
export const CONVERSATION_LIMIT=5000;
export interface ConversationSnapshot {accountId:string;threadId:string;ids:string[];total:number}
interface ThreadResult {accountId:string;list:{id:string;emailIds:string[]}[];notFound:string[]}
type Transport=Pick<Client,'jmap'>;
export async function conversationSnapshot(client:Transport,accountId:string,threadId:string,signal:AbortSignal):Promise<ConversationSnapshot>{
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Thread/get',{accountId,ids:[threadId]},'thread']]},{signal});
 const result=responseFor<ThreadResult>(response,'thread');
 if(result.accountId!==accountId||!Array.isArray(result.list)||result.list.length!==1||result.list[0]?.id!==threadId||!Array.isArray(result.list[0].emailIds))throw new Error('Invalid conversation response');
 const raw=result.list[0].emailIds;const ids=raw.slice(0,CONVERSATION_LIMIT);
 if(ids.some(id=>typeof id!=='string'||!id||id.length>512)||new Set(ids).size!==ids.length)throw new Error('Invalid conversation membership');
 return {accountId,threadId,ids,total:raw.length};
}
export async function conversationPage(client:Transport,snapshot:ConversationSnapshot,offset:number,signal:AbortSignal):Promise<{messages:Email[];next:number;missing:number}>{
 if(!Number.isInteger(offset)||offset<0||offset>snapshot.ids.length)throw new Error('Invalid conversation position');
 const ids=snapshot.ids.slice(offset,offset+CONVERSATION_PAGE);if(!ids.length)return {messages:[],next:offset,missing:0};
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/get',{accountId:snapshot.accountId,ids,properties:['id','threadId','subject','from','receivedAt','mailboxIds','keywords']},'members']]},{signal});
 const result=responseFor<EmailGetResponse>(response,'members');
 if(result.accountId!==snapshot.accountId||!Array.isArray(result.list)||result.list.length>ids.length||result.list.some(message=>!ids.includes(message.id)||message.threadId!==snapshot.threadId)||new Set(result.list.map(message=>message.id)).size!==result.list.length)throw new Error('Invalid conversation messages');
 const byId=new Map(result.list.map(message=>[message.id,message]));
 return {messages:ids.flatMap(id=>byId.has(id)?[byId.get(id)!]:[]),next:offset+ids.length,missing:ids.length-result.list.length};
}
