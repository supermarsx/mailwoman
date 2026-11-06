// How many messages in the account came from one mailing list, by List-Id.
// The engine indexes each message's List-Id (`list:<id>` search), so this is
// one count query rather than a walk over the loaded folder. Answers are cached
// per account and list for a minute; a failure or an engine that predates the
// index answers `null`, and the badge falls back to the sender count.
import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL} from './jmap-types.ts';

export interface ListCount {count:number;last:string|null}
const TTL_MS=60_000;
const cache=new Map<string,{at:number;value:ListCount}>();

export const normalizeListId=(listId:string):string=>listId.trim().replace(/^<|>$/g,'').toLowerCase();
/** The search the engine answers: `list:<id>`. Ids with a space or a quote are not searchable. */
export function listQuery(listId:string):string|null {
 const id=normalizeListId(listId);
 return id!==''&&!/[\s"()]/u.test(id)?`list:${id}`:null;
}
export function forgetListCounts():void {cache.clear();}

export async function listMessageCount(client:Pick<Client,'jmap'>,accountId:string,listId:string,now:number=Date.now()):Promise<ListCount|null> {
 const text=listQuery(listId);
 if(text===null)return null;
 const key=`${accountId}|${normalizeListId(listId)}`,hit=cache.get(key);
 if(hit&&now-hit.at<TTL_MS)return hit.value;
 try{
  const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[
   ['Email/query',{accountId,filter:{text},sort:[{property:'receivedAt',isAscending:false}],limit:1,calculateTotal:true},'q'],
   ['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','receivedAt']},'g'],
  ]});
  const query=response.methodResponses.find(item=>item[2]==='q'),get=response.methodResponses.find(item=>item[2]==='g');
  if(!query||query[0]==='error')return null;
  const total=(query[1] as {total?:unknown}).total;
  if(typeof total!=='number')return null;
  const first=get&&get[0]!=='error'?((get[1] as {list?:{receivedAt?:string}[]}).list?.[0]?.receivedAt??null):null;
  const value={count:total,last:first};
  cache.set(key,{at:now,value});
  return value;
 }catch{return null;}
}
