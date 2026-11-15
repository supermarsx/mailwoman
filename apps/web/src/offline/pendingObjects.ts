import type {JmapRequest,JmapResponse} from '../api/jmap-types.ts';
import type {QueuedItem,PimPayload} from './outboxCore.ts';
import {outboundToRequest} from './outboxCore.ts';
import {offlineVault} from './vault.ts';
type Row=Record<string,unknown>&{id:string};
export const pendingId=(item:string,creation:string,type:string)=>`mw-pending:${item}:${type}:${encodeURIComponent(creation)}`;
const mappingKey=(item:QueuedItem)=>JSON.stringify([item.cacheBase??item.serverBase,item.storageAuthority]);
/** Explicit local rows, distinct from a server acknowledgement. */
export function pendingObject(type:string,id:string,value:Record<string,unknown>):Row{
 const now=new Date().toISOString();
 const defaults:Record<string,unknown>=type==='Note'?{notebookId:'default',title:'',tags:[],color:'#808080',pinned:false,bodyHtml:'',bodyText:'',links:[],createdAt:now,updatedAt:now}:type==='Calendar'?{name:'',color:'#808080',order:0,isVisible:true,isSubscribed:true,role:null,shareWith:[],caldavUrl:null,syncToken:null,isReadOnlyOverlay:false}:type==='CalendarEvent'?{title:'',description:'',locations:[],participants:{},alerts:{},recurrenceRules:[],recurrenceOverrides:{},excludedRecurrenceDates:[],sequence:0,created:now,updated:now}:type==='Task'?{title:'',description:'',start:null,due:null,timeZone:null,priority:0,percentComplete:0,status:'needs-action',links:[],created:now,updated:now}:{name:{full:''},emails:[],phones:[],addresses:[],organizations:[],titles:[],nicknames:[],categories:[],notes:'',photos:[],links:[]};
 return {...defaults,...value,id,mailwomanQueued:true};
}
export function patchObject(row:Row,patch:Record<string,unknown>):Row{
 const value=structuredClone(row);
 for(const [path,item] of Object.entries(patch)){
  const parts=path.split('/').map(part=>part.replace(/~1/g,'/').replace(/~0/g,'~'));let target:Record<string,unknown>=value;
  for(const part of parts.slice(0,-1)){if(!target[part]||typeof target[part]!=='object')target[part]={};target=target[part] as Record<string,unknown>;}
  const key=parts.at(-1)!;if(item===null)delete target[key];else target[key]=item;
 }
 return value;
}
/** Replay projection is read-only: the last server snapshot remains available
 * for conflict checks and failed changes disappear from this projection. */
export function projectPending(rows:Row[],type:string,accountId:string,items:QueuedItem[]):Row[]{
 const result=new Map(rows.map(row=>[row.id,row]));
 for(const item of items){
  if(item.state!=='queued')continue;
  let request:JmapRequest;try{request=outboundToRequest(item);}catch{continue;}
  for(const [method,args] of request.methodCalls){
   if(method!==`${type}/set`||args['accountId']!==accountId)continue;
   for(const [key,create] of Object.entries((args['create']??{}) as Record<string,Record<string,unknown>>)){
    // Offline sends belong to the local queue, not the stored Drafts folder.
    if(item.type==='send'||item.type==='draft')continue;
    const id=pendingId(item.id,key,type);result.set(id,pendingObject(type,id,create));
   }
   for(const [id,patch] of Object.entries((args['update']??{}) as Record<string,Record<string,unknown>>)){const old=result.get(id);if(old)result.set(id,{...patchObject(old,patch),mailwomanQueued:true});}
   for(const id of (args['destroy']??[]) as string[])result.delete(id);
  }
 }
 return [...result.values()];
}
/** Persist creation-ID translations before removing the acknowledged queue
 * item, so later offline edits survive a restart between creation and replay. */
export async function recordPendingIds(item:QueuedItem,response:JmapResponse):Promise<void>{
 if(item.type!=='pim')return;
 const mappings:Record<string,string>={};
 for(const [method,args] of response.methodResponses){for(const [key,raw] of Object.entries((args['created']??{}) as Record<string,{id?:string}>))if(typeof raw.id==='string')mappings[pendingId(item.id,key,method.split('/')[0]!)]=raw.id;}
 if(!Object.keys(mappings).length)return;
 const vault=await offlineVault('indexeddb'),namespace=`pending-identities:${mappingKey(item)}`;
 for(const [id,serverId] of Object.entries(mappings))await vault.put(namespace,id,serverId);
}
export async function resolvePendingIds(item:QueuedItem,request:JmapRequest,strict=true):Promise<JmapRequest>{
 if(!JSON.stringify(request).includes('mw-pending:'))return request;
 const vault=await offlineVault('indexeddb'),namespace=`pending-identities:${mappingKey(item)}`;
 async function resolve(value:unknown):Promise<unknown>{
  if(typeof value==='string'&&value.startsWith('mw-pending:')){const id=await vault.get<string>(namespace,value);if(!id&&strict)throw new Error('An earlier queued item has not synchronized');return id??value;}
  if(Array.isArray(value))return Promise.all(value.map(resolve));
  if(value&&typeof value==='object'){const result:Record<string,unknown>={};for(const [key,item] of Object.entries(value)){const mapped=key.startsWith('mw-pending:')?await resolve(key) as string:key;result[mapped]=await resolve(item);}return result;}
  return value;
 }
 return await resolve(request) as JmapRequest;
}
const PIM_TYPES=new Set(['Calendar','CalendarEvent','Task','Note','AddressBook','ContactCard','ContactGroup']);
export function queuedPimRequest(request:JmapRequest):boolean{return request.methodCalls.length>0&&request.methodCalls.every(([method])=>{const [type,operation]=method.split('/');return PIM_TYPES.has(type!)&&(operation==='set'||method==='CalendarEvent/respond');});}
export function pendingResponse(item:QueuedItem):JmapResponse{
 const request=(item.payload as PimPayload).request;
 return {sessionState:'offline-pending',methodResponses:request.methodCalls.map(([method,args,id])=>{
  const type=method.split('/')[0]!;
  if(method==='CalendarEvent/respond')return [method,{accountId:args['accountId'],action:args['action'],eventId:args['eventId']??null,emailId:args['emailId']??null,sendResponse:args['sendResponse']??true,sent:false,mailwomanQueued:true},id];
  return [method,{accountId:args['accountId'],oldState:args['ifInState']??null,newState:'offline-pending',created:Object.fromEntries(Object.entries((args['create']??{}) as Record<string,Record<string,unknown>>).map(([key,value])=>[key,pendingObject(type,pendingId(item.id,key,type),value)])),updated:Object.fromEntries(Object.keys((args['update']??{}) as object).map(key=>[key,null])),destroyed:args['destroy']??[],notCreated:null,notUpdated:null,notDestroyed:null,mailwomanQueued:true},id];
 })};
}
