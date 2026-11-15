import type {Client} from '../api/client.ts';
import type {JmapRequest} from '../api/jmap-types.ts';

export interface CollectionDelta {state:string;changed:string[];destroyed:string[]}
/** A cursor is committed only after every page has been validated. Callers
 * reload a baseline on unsupported, expired, nonadvancing or oversized deltas. */
export async function collectionDelta(type:string,accountId:string,state:string,using:string[],jmap:Client['jmap'],signal?:AbortSignal):Promise<CollectionDelta|undefined>{
 const changed=new Set<string>(),destroyed=new Set<string>();let cursor=state;
 const valid=(value:unknown):value is string[]=>Array.isArray(value)&&value.length<=1000&&value.every(id=>typeof id==='string'&&id.length>0);
 for(let page=0;page<64;page++){
  const request:JmapRequest={using,methodCalls:[[`${type}/changes`,{accountId,sinceState:cursor,maxChanges:1000},'changes']]};
  const response=await jmap(request,{...(signal?{signal}:{})});
  const item=response.methodResponses.find(row=>row[2]==='changes');
  if(item?.[0]!==`${type}/changes`)return;
  const value=item[1];
  if(value['accountId']!==accountId||value['oldState']!==cursor||typeof value['newState']!=='string'||!valid(value['created'])||!valid(value['updated'])||!valid(value['destroyed']))return;
  for(const id of [...value['created'],...value['updated']]){changed.add(id);destroyed.delete(id);}
  for(const id of value['destroyed']){changed.delete(id);destroyed.add(id);}
  if(changed.size+destroyed.size>10000)return;
  const next=value['newState'];
  if(value['hasMoreChanges']!==true)return {state:next,changed:[...changed],destroyed:[...destroyed]};
  if(next===cursor)return;
  cursor=next;
 }
}
