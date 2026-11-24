import {nativeReadIntentHeaders} from '../api/nativeReadIntent.ts';
import {clearWebSocketActions} from '../api/websocketActions.ts';
import {jmapReadsOnly} from '../api/jmapReadOnly.ts';
import {workerHttp} from '../api/workerHttp.ts';
import {ApiError,terminalActionError} from '../api/errors.ts';
import type {Me} from '../api/client.ts';
import type {JmapRequest,JmapResponse,JmapSession} from '../api/jmap-types.ts';
import {jmapApiPath} from '../api/jmapDiscovery.ts';
import {createPushClient,type PushClientImpl,type PushStatus,type PushStatusDetail} from '../realtime/pushClientCore.ts';
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
 let status:{status:PushStatus;transport:PushTransport;detail?:PushStatusDetail}={status:'closed',transport:'offline'};
 function clear(){dataEpoch++;snapshots.clear();pending.clear();bytes=0;}
 function broadcast(event:string,value:unknown){for(const peer of peers.values())if(peer.push&&peer.authority===primary)peer.send({event,value});}
 function stopUnused(){if(![...peers.values()].some(peer=>peer.push&&peer.authority===primary)){push?.close();push=undefined;}}
 const timer=setInterval(()=>{for(const peer of peers.values())if(Date.now()-peer.lastSeen>45000)peer.push=false;stopUnused();},15000);
 // t52w: one GET /api/me per (base, link) answers every message for IDENTITY_REUSE_MS.
 // What this client-side check protects: that the account the window expects
 // (`context.owner`) is the one the cookie now names, the CSRF token a write
 // carries, and that a 500 ms shared snapshot is served only to a principal
 // validated moments ago. It is not authorization: every request after it
 // still carries the cookie and the server decides each one. Reusing it for
 // 5 s is safe because every event that changes who the cookie names drops
 // it at once: a sign-in or sign-out in any tab (`reset`, the worker is
 // shared), a push account change, an owner the cached answer does not match
 // (asked again before refusing), and any 401/403/409 from any request, so a
 // revoked cookie is noticed on its next request rather than after 5 s.
 // Writes, the window's own 'me' and the push session probe always ask the server.
 const IDENTITY_REUSE_MS=5000;
 const identities=new Map<string,{generation:number;at:number;value:Promise<Me&{csrfToken?:string}>}>();
 // t52w: the JMAP session document per (authority, link, data epoch) for
 // SESSION_REUSE_MS (the page's session cache keeps it as long). A pushed
 // change bumps the epoch, an identity drop clears it. After either, the
 // document is asked again with If-None-Match (the route's ETag) and a 304
 // keeps the last one; `validated` holds it across epochs per (base,
 // authority, link, generation).
 const SESSION_REUSE_MS=60_000;
 const sessions=new Map<string,{at:number;value:Promise<JmapSession>}>();
 const validated=new Map<string,{etag:string;value:JmapSession}>();
 function forgetIdentity(){identities.clear();sessions.clear();validated.clear();}
 async function liveIdentity(context:NetworkContext):Promise<Me&{csrfToken?:string}>{
  const headers=context.linkId?{'X-Mailwoman-Link':context.linkId}:{};
  return await (await workerHttp(context.base,`${new URL(context.base).pathname.replace(/\/$/,'')}/api/me`,{headers})).json() as Me&{csrfToken?:string};
 }
 async function identity(context:NetworkContext,fresh=false):Promise<Me&{csrfToken?:string}>{
  const key=JSON.stringify([context.base,context.linkId]);
  let reused=false,live:Me&{csrfToken?:string};
  const kept=identities.get(key);
  if(!fresh&&kept&&kept.generation===generation&&Date.now()-kept.at<IDENTITY_REUSE_MS){reused=true;live=await kept.value;}
  else{
   const value=liveIdentity(context),entry={generation,at:Date.now(),value};
   identities.set(key,entry);
   value.then(()=>{entry.at=Date.now();},()=>{if(identities.get(key)===entry)identities.delete(key);});
   live=await value;
  }
  const mismatch=(me:Me)=>!!context.owner&&(me.accountId!==context.owner.accountId||me.storageAuthority!==context.owner.storageAuthority);
  // A kept answer that names another account than the window expects may be the
  // stale one: ask the server before refusing.
  if(reused&&mismatch(live)){forgetIdentity();return identity(context,true);}
  if(mismatch(live))throw new ApiError(409,'The signed-in account changed',{code:'accountChanged'});
  if(live.passwordChangeRequired)throw new ApiError(403,'Password change required',{code:'passwordChangeRequired'});
  return live;
 }
 async function execute(peer:Peer,message:NetworkRequest):Promise<unknown>{
  const context=message.context,base=new URL(context.base),prefix=base.pathname.replace(/\/$/,'');
  if(base.origin!==globalThis.location?.origin)throw new ApiError(400,'Shared worker requires this Mailwoman origin');
  if(message.method==='lease')return true;
  if(message.method==='reset'){clearWebSocketActions();generation++;forgetIdentity();views.clear();clear();push?.close();push=undefined;for(const value of peers.values()){value.push=false;value.send({event:'reset',value:null});}return true;}
  if(message.method==='push.stop'){peer.push=false;stopUnused();return true;}
  if(message.method.startsWith('push.')){
   // A push subscription is a read of who the cookie names: a kept answer (<= 5 s) serves.
   const live=await identity({base:context.base,linkId:null});
   if(!live.storageAuthority||message.primaryAuthority&&message.primaryAuthority!==live.storageAuthority)throw new ApiError(409,'Push account changed');
   if(primary!==live.storageAuthority||root!==context.base){
    if(primary)broadcast('status',{status:'unauthorized',transport:'offline'});
    for(const value of peers.values())value.push=false;
    if(primary)generation++;forgetIdentity();views.clear();push?.close();push=undefined;clear();primary=live.storageAuthority;root=context.base;
   }
   peer.authority=primary;peer.push=true;
   if(!push){
    const ws=new URL(`${prefix}/jmap/ws`,base.origin);ws.protocol=base.protocol==='https:'?'wss:':'ws:';
    push=createPushClient({wsUrl:ws.href,sseUrl:`${prefix}/jmap/eventsource`,checkSession:async()=>{try{await identity({base:context.base,linkId:null},true);return 'ok';}catch(error){return error instanceof ApiError&&[401,403,409].includes(error.status)?'unauthorized':'unknown';}},onStatus:(next,transport,detail)=>{status={status:next,transport,...(detail?{detail}:{})};broadcast('status',status);}});
    push.onStateChange(change=>{clear();broadcast('change',change);});
    push.onMailboxChange?.(change=>{clear();broadcast('mailbox',change);});
    push.connect();
   }else if(message.method==='push.reconnect')push.reconnect();
   peer.send({event:'status',value:status});return true;
  }
  const key=JSON.stringify([context,message.method,message.body,message.idempotencyKey]);
  const read=message.method!=='jmap'||message.body!==undefined&&jmapReadsOnly(message.body);
  // Sharing only responses from a principal validated moments ago (see
  // `identity`) keeps the cache from extending a revoked cookie's lifetime.
  // A write and the window's own `me` always ask the server.
  const admitted=generation;const live=await identity(context,!read||message.method==='me');
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
   const readSession=():Promise<JmapSession>=>{
    const key=JSON.stringify([context.base,live.storageAuthority??live.accountId,context.linkId,epoch,generation]);
    const kept=sessions.get(key);
    if(kept&&Date.now()-kept.at<SESSION_REUSE_MS)return kept.value;
    const known=JSON.stringify([context.base,live.storageAuthority??live.accountId,context.linkId,generation]);
    const value=(async()=>{
     const previous=validated.get(known),request=new Headers(headers);
     if(previous)request.set('If-None-Match',previous.etag);
     const response=await workerHttp(context.base,`${prefix}/jmap/session`,{headers:request,allowWebSocket:context.preferWebSocket===true,assertAuthority:()=>{if(admitted!==generation)throw new ApiError(409,'Shared session changed',{code:'actionNotDispatched'});}});
     if(response.status===304&&previous)return previous.value;
     const document=await response.json() as JmapSession,etag=response.headers.get('etag');
     if(admitted===generation){if(etag)validated.set(known,{etag,value:document});else validated.delete(known);while(validated.size>16)validated.delete(validated.keys().next().value!);}
     return document;
    })();
    const entry={at:Date.now(),value};sessions.set(key,entry);
    value.then(()=>{entry.at=Date.now();},()=>{if(sessions.get(key)===entry)sessions.delete(key);});
    while(sessions.size>16)sessions.delete(sessions.keys().next().value!);
    return value;
   };
   if(message.method==='session')value=await readSession();
   else{
    if(!message.body||message.body.methodCalls.length>128)throw new ApiError(400,'Invalid shared JMAP request');
    const session=await readSession();
    const path=jmapApiPath(session.apiUrl,context.base);
    if(message.idempotencyKey)headers.set('Idempotency-Key',message.idempotencyKey);
    if(!context.linkId)for(const [key,value]of Object.entries(nativeReadIntentHeaders(message.body)))headers.set(key,value);
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
  async receive(id:string,message:NetworkRequest){const peer=peers.get(id);if(!peer||typeof message?.id!=='string')return;peer.lastSeen=Date.now();try{peer.send({id:message.id,result:await execute(peer,message)});}catch(error){if(error instanceof ApiError&&[401,403,409].includes(error.status))forgetIdentity();const value=error instanceof Error?error:new Error('Shared request failed');peer.send({id:message.id,error:{name:value.name,message:value.message,...(value instanceof ApiError?{status:value.status,...(value.retryAfterSecs!==undefined?{retryAfterSecs:value.retryAfterSecs}:{}),...(value.code?{code:value.code}:{})}:{})}});}},
  close(){clearInterval(timer);push?.close();peers.clear();clear();},
 };
}
