import type {Client} from '../api/client.ts';
import type {Email,JmapRequest,JmapResponse} from '../api/jmap-types.ts';

export interface QuerySnapshot {
 using:string[];accountId:string;query:Record<string,unknown>;properties:unknown;
 queryState:string;emailState:string;ids:string[];rows:Email[];total:number|null;
}
function answer(response:JmapResponse,id:string,name:string):Record<string,unknown>|undefined {
 const item=response.methodResponses.find(row=>row[2]===id);
 return item?.[0]===name?item[1]:undefined;
}
function ids(value:unknown):value is string[]{return Array.isArray(value)&&value.length<=100000&&value.every(id=>typeof id==='string');}
export function queryIdentity(query:Record<string,unknown>):string {
 const {position:_position,limit:_limit,calculateTotal:_total,...identity}=query;return JSON.stringify(identity);
}
/** A continuation is a valid baseline only if it is contiguous and still uses
 * exactly the same query state. Mixed-state pages trigger a bounded resync. */
export function captureQuery(request:JmapRequest,response:JmapResponse,previous?:QuerySnapshot):QuerySnapshot|undefined {
 const query=request.methodCalls.find(call=>call[0]==='Email/query'),get=request.methodCalls.find(call=>call[0]==='Email/get');
 if(!query||!get)return;
 const q=answer(response,query[2],'Email/query'),g=answer(response,get[2],'Email/get');
 if(!q||!g||q['canCalculateChanges']!==true||typeof q['queryState']!=='string'||typeof g['state']!=='string'||!ids(q['ids'])||!Array.isArray(g['list'])||typeof query[1]['accountId']!=='string')return;
 const position=Number(query[1]['position']??0);
 const current:QuerySnapshot={using:request.using,accountId:query[1]['accountId'],query:query[1],properties:get[1]['properties'],queryState:q['queryState'],emailState:g['state'],ids:q['ids'],rows:g['list'] as Email[],total:typeof q['total']==='number'?q['total']:null};
 if(position===0)return current;
 if(!previous||position!==previous.ids.length||current.queryState!==previous.queryState||queryIdentity(current.query)!==queryIdentity(previous.query))return;
 return {...current,ids:[...previous.ids,...current.ids],rows:[...previous.rows,...current.rows],total:current.total??previous.total};
}

/** Apply query removals first, then insertions at their indices in the NEW
 * result. Never mutate the old baseline before every response is validated. */
export async function reconcileQuery(snapshot:QuerySnapshot,jmap:Client['jmap'],signal?:AbortSignal):Promise<QuerySnapshot|undefined>{
 const {position:_position,limit:_limit,calculateTotal:_total,...query}=snapshot.query;
 const call=(calls:JmapRequest['methodCalls'])=>jmap({using:snapshot.using,methodCalls:calls},{...(signal?{signal}:{})});
 const changed=await call([
  ['Email/queryChanges',{...query,sinceQueryState:snapshot.queryState,maxChanges:1000,calculateTotal:true,...(snapshot.ids.at(-1)?{upToId:snapshot.ids.at(-1)}:{})},'delta-query'],
  ['Email/changes',{accountId:snapshot.accountId,sinceState:snapshot.emailState,maxChanges:1000},'delta-emails'],
 ]);
 const q=answer(changed,'delta-query','Email/queryChanges');
 let changes=answer(changed,'delta-emails','Email/changes');
 if(!q||!changes||q['accountId']!==snapshot.accountId||changes['accountId']!==snapshot.accountId||q['oldQueryState']!==snapshot.queryState||typeof q['newQueryState']!=='string'||!ids(q['removed'])||!Array.isArray(q['added'])||q['added'].length>1000)return;
 const updated=new Set<string>();let emailState=snapshot.emailState;
 for(let page=0;page<16;page++){
  if(!changes||changes['oldState']!==emailState||typeof changes['newState']!=='string'||!ids(changes['updated'])||!ids(changes['created'])||!ids(changes['destroyed']))return;
  for(const id of [...changes['updated'],...changes['created']])updated.add(id);
  const next=changes['newState'];if(changes['hasMoreChanges']===true&&next===emailState)return;
  emailState=next;
  if(changes['hasMoreChanges']!==true)break;
  if(page===15)return;
  changes=answer(await call([['Email/changes',{accountId:snapshot.accountId,sinceState:emailState,maxChanges:1000},'delta-emails']]),'delta-emails','Email/changes');
 }
 const removed=new Set(q['removed']);
 const resultIds=snapshot.ids.filter(id=>!removed.has(id));
 const limit=Math.max(snapshot.ids.length,Number(snapshot.query['limit'])||0);
 const total=typeof q['total']==='number'?q['total']:snapshot.total;
 let lastIndex=-1;
 for(const raw of q['added']){
  if(!raw||typeof raw!=='object')return;
  const {id,index}=raw as {id?:unknown;index?:unknown};
  if(typeof id!=='string'||!Number.isSafeInteger(index)||Number(index)<0||Number(index)<lastIndex)return;
  lastIndex=Number(index);
  if(Number(index)>=limit)continue;
  if(Number(index)>resultIds.length||resultIds.includes(id))return;
  resultIds.splice(Number(index),0,id);
 }
 resultIds.length=Math.min(resultIds.length,limit,total??Infinity);
 const target=Math.min(limit,total??limit);
 if(resultIds.length<target){
  const tail=answer(await call([['Email/query',{...query,position:resultIds.length,limit:target-resultIds.length},'delta-tail']]),'delta-tail','Email/query');
  if(!tail||tail['queryState']!==q['newQueryState']||!ids(tail['ids'])||tail['position']!==resultIds.length)return;
  for(const id of tail['ids'])if(!resultIds.includes(id))resultIds.push(id);
  if(resultIds.length!==target)return;
 }
 const byId=new Map(snapshot.rows.map(row=>[row.id,row]));
 const fetchIds=resultIds.filter(id=>updated.has(id)||!byId.has(id));
 if(fetchIds.length){
  const got=answer(await call([['Email/get',{accountId:snapshot.accountId,ids:fetchIds,properties:snapshot.properties},'delta-get']]),'delta-get','Email/get');
  if(!got||got['accountId']!==snapshot.accountId||!Array.isArray(got['list']))return;
  for(const row of got['list'] as Email[])byId.set(row.id,row);
 }
 const rows=resultIds.map(id=>byId.get(id));if(rows.some(row=>!row))return;
 return {...snapshot,queryState:q['newQueryState'],emailState,ids:resultIds,rows:rows as Email[],total};
}

export function queryResponse(snapshot:QuerySnapshot):JmapResponse {
 return {sessionState:snapshot.emailState,methodResponses:[
  ['Email/query',{accountId:snapshot.accountId,queryState:snapshot.queryState,canCalculateChanges:true,position:0,ids:snapshot.ids,...(snapshot.total===null?{}:{total:snapshot.total})},'q'],
  ['Email/get',{accountId:snapshot.accountId,state:snapshot.emailState,list:snapshot.rows,notFound:[]},'g'],
 ]};
}
