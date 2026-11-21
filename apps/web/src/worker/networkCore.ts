import {clearWebSocketActions} from '../api/websocketActions.ts';
import {jmapReadsOnly} from '../api/jmapReadOnly.ts';
import {workerHttp} from '../api/workerHttp.ts';
import {ApiError,terminalActionError} from '../api/errors.ts';
import type {Me} from '../api/client.ts';
import type {JmapRequest,JmapResponse,JmapSession} from '../api/jmap-types.ts';
import {jmapApiPath} from '../api/jmapDiscovery.ts';
import {createPushClient,type PushClientImpl,type PushStatus} from '../realtime/pushClientCore.ts';
import type {PushTransport} from '../contracts/push.ts';

export interface NetworkContext {preferWebSocket?:boolean;base:string;linkId:string|null;owner?:{accountId:string;storageAuthority:string}}
export interface NetworkRequest {id:string;method:'me'|'session'|'jmap'|'push.start'|'push.stop'|'push.reconnect'|'reset'|'lease'|'view.put'|'view.get';context:NetworkContext;body?:JmapRequest;idempotencyKey?:string|false;primaryAuthority?:string;view?:{topic:string;value?:unknown;sender:string}}
export type NetworkReply={id:string;result?:unknown;error?:{name:string;message:string;status?:number;code?:string;retryAfterSecs?:number}}|{event:string;value:unknown};
interface Peer {send(reply:NetworkReply):void;lastSeen:number;authority?:string;push:boolean}
/** One actor owns upstream network state, request coalescing, bounded snapshots
 * and the push ladder. Shared selection and notification snapshots are scoped to the verified account; DOM rendering stays in each window. */
export function createNetworkCore(){
 const views=new Map<string,unknown>();
 const peers=new Map<string,Peer>(),pending=new Map<string,Promise<unknown>>();
 const snapshots=new Map<string,{value:unknown;at:number;bytes:number}>();let bytes=0,generation=0,dataEpoch=0;
 let push:PushClientImpl|undefined,primary:string|undefined,root:string|undefined;
 let status:{status:PushStatus;transport:PushTransport}={status:'closed',transport:'offline'};
 function clear(){dataEpoch++;snapshots.clear();pending.clear();bytes=0;}
 function broadcast(event:string,value:unknown){for(const peer of peers.values())if(peer.push&&peer.authority===primary)peer.send({event,value});}
 function stopUnused(){if(![...peers.values()].some(peer=>peer.push&&peer.authority===primary)){push?.close();push=undefined;}}
 const timer=setInterval(()=>{for(const peer of peers.values())if(Date.now()-peer.lastSeen>45000)peer.push=false;stopUnused();},15000);
 async function identity(context:NetworkContext):Promise<Me&{csrfToken?:string}>{
  const headers=context.linkId?{'X-Mailwoman-Link':context.linkId}:{};
  const live=await (await workerHttp(context.base,`${new URL(context.base).pathname.replace(/\/$/,'')}/api/me`,{headers})).json() as Me&{csrfToken?:string};
  if(context.owner&&(live.accountId!==context.owner.accountId||live.storageAuthority!==context.owner.storageAuthority))throw new ApiError(409,'The signed-in account changed',{code:'accountChanged'});
  if(live.passwordChangeRequired)throw new ApiError(403,'Password change required',{code:'passwordChangeRequired'});
  return live;
 }
 async function execute(peer:Peer,message:NetworkRequest):Promise<unknown>{
  const context=message.context,base=new URL(context.base),prefix=base.pathname.replace(/\/$/,'');
  if(base.origin!==globalThis.location?.origin)throw new ApiError(400,'Shared worker requires this Mailwoman origin');
  if(message.method==='lease')return true;
  if(message.method==='reset'){clearWebSocketActions();generation++;views.clear();clear();push?.close();push=undefined;for(const value of peers.values()){value.push=false;value.send({event:'reset',value:null});}return true;}
  if(message.method==='push.stop'){peer.push=false;stopUnused();return true;}
  if(message.method.startsWith('push.')){
   const live=await identity({base:context.base,linkId:null});
   if(!live.storageAuthority||message.primaryAuthority&&message.primaryAuthority!==live.storageAuthority)throw new ApiError(409,'Push account changed');
   if(primary!==live.storageAuthority||root!==context.base){
    if(primary)broadcast('status',{status:'unauthorized',transport:'offline'});
    for(const value of peers.values())value.push=false;
    if(primary)generation++;views.clear();push?.close();push=undefined;clear();primary=live.storageAuthority;root=context.base;
   }
   peer.authority=primary;peer.push=true;
   if(!push){
    const ws=new URL(`${prefix}/jmap/ws`,base.origin);ws.protocol=base.protocol==='https:'?'wss:':'ws:';
    push=createPushClient({wsUrl:ws.href,sseUrl:`${prefix}/jmap/eventsource`,checkSession:async()=>{try{await identity({base:context.base,linkId:null});return 'ok';}catch(error){return error instanceof ApiError&&[401,403,409].includes(error.status)?'unauthorized':'unknown';}},onStatus:(next,transport)=>{status={status:next,transport};broadcast('status',status);}});
    push.onStateChange(change=>{clear();broadcast('change',change);});
    push.onMailboxChange?.(change=>{clear();broadcast('mailbox',change);});
    push.connect();
   }else if(message.method==='push.reconnect')push.reconnect();
   peer.send({event:'status',value:status});return true;
  }
  const key=JSON.stringify([context,message.method,message.body,message.idempotencyKey]);
  const read=message.method!=='jmap'||message.body!==undefined&&jmapReadsOnly(message.body);
  // Sharing only responses from a just-validated principal prevents the cache
  // from extending a revoked cookie's lifetime.
  const admitted=generation;const live=await identity(context);
  if(admitted!==generation)throw new ApiError(401,'Shared session changed');
  if(message.method==='me')return live;
  if(message.method.startsWith('view.')){
   const view=message.view;
   if(!live.storageAuthority||!view||typeof view.topic!=='string'||view.topic.length>8192||typeof view.sender!=='string'||view.sender.length>128)throw new ApiError(400,'Invalid shared view');
   const key=JSON.stringify([live.storageAuthority,live.accountId,view.topic]);
   if(message.method==='view.get')return views.get(key)??null;
   if(JSON.stringify(view.value??null).length>65536)throw new ApiError(400,'Shared view exceeds its limit');
   views.delete(key);views.set(key,view.value??null);while(views.size>128)views.delete(views.keys().next().value!);
   broadcast('view',{...view,accountId:live.accountId,authority:live.storageAuthority});return true;
  }

  if(read){const hit=snapshots.get(key);if(hit&&Date.now()-hit.at<500)return hit.value;const held=pending.get(key);if(held)return held;}
  const epoch=dataEpoch;
  const run=(async()=>{
   const headers=new Headers({'content-type':'application/json'});
   if(context.linkId)headers.set('X-Mailwoman-Link',context.linkId);
   if(live.csrfToken)headers.set('X-CSRF-Token',live.csrfToken);
   if(live.storageAuthority){headers.set('X-Mailwoman-Account',live.accountId);headers.set('X-Mailwoman-Storage-Authority',live.storageAuthority);}
   let value:unknown;
   if(message.method==='session')value=await (await workerHttp(context.base,`${prefix}/jmap/session`,{headers,allowWebSocket:context.preferWebSocket===true,assertAuthority:()=>{if(admitted!==generation)throw new ApiError(409,'Shared session changed',{code:'actionNotDispatched'});}})).json() as JmapSession;
   else{
    if(!message.body||message.body.methodCalls.length>128)throw new ApiError(400,'Invalid shared JMAP request');
    const session=await (await workerHttp(context.base,`${prefix}/jmap/session`,{headers,allowWebSocket:context.preferWebSocket===true,assertAuthority:()=>{if(admitted!==generation)throw new ApiError(409,'Shared session changed',{code:'actionNotDispatched'});}})).json() as JmapSession;
    const path=jmapApiPath(session.apiUrl,context.base);
    if(message.idempotencyKey)headers.set('Idempotency-Key',message.idempotencyKey);
    value=await (await workerHttp(context.base,`${prefix}${path}`,{method:'POST',headers,body:JSON.stringify(message.body),allowWebSocket:context.preferWebSocket===true,assertAuthority:()=>{if(admitted!==generation)throw new ApiError(409,'Shared session changed',{code:'actionNotDispatched'});}})).json() as JmapResponse;
   }
   if(admitted!==generation)throw new ApiError(401,'Shared session changed');
   if(read&&epoch===dataEpoch){const size=new TextEncoder().encode(JSON.stringify(value)).length;if(size<=2*1024*1024){const old=snapshots.get(key);if(old)bytes-=old.bytes;snapshots.delete(key);snapshots.set(key,{value,at:Date.now(),bytes:size});bytes+=size;while(bytes>20*1024*1024||snapshots.size>256){const first=snapshots.keys().next().value!;bytes-=snapshots.get(first)!.bytes;snapshots.delete(first);}}}
   else if(!read){clear();broadcast('invalidate',null);}
   return value;
  })();
  if(read)pending.set(key,run);
  try{return await run;}catch(error){if(!read&&terminalActionError(error)){clear();broadcast('invalidate',null);}throw error;}finally{if(pending.get(key)===run)pending.delete(key);}
 }
 return {
  connect(id:string,send:Peer['send']){peers.set(id,{send,lastSeen:Date.now(),push:false});},
  disconnect(id:string){peers.delete(id);stopUnused();},
  async receive(id:string,message:NetworkRequest){const peer=peers.get(id);if(!peer||typeof message?.id!=='string')return;peer.lastSeen=Date.now();try{peer.send({id:message.id,result:await execute(peer,message)});}catch(error){const value=error instanceof Error?error:new Error('Shared request failed');peer.send({id:message.id,error:{name:value.name,message:value.message,...(value instanceof ApiError?{status:value.status,...(value.retryAfterSecs!==undefined?{retryAfterSecs:value.retryAfterSecs}:{}),...(value.code?{code:value.code}:{})}:{})}});}},
  close(){clearInterval(timer);push?.close();peers.clear();clear();},
 };
}
