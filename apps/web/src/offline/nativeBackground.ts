import {ApiError,NetworkError,type Client,type Me} from '../api/client.ts';
import type {JmapSession,JmapResponse} from '../api/jmap-types.ts';
import {CAP_MAIL} from '../api/jmap-types.ts';
import {apiFetch} from '../api/http.ts';
import {nativeServerBase} from '../api/transport.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {installLinkedMailbox} from '../api/linkedMailboxSession.ts';
import {jmapApiPath} from '../api/jmapDiscovery.ts';
import {platformReady} from '../platform/index.ts';
import {encryptedOutboxStore} from './encryptedOutbox.ts';
import {idbOutboxStore} from './idb.ts';
import {accountOutbox,outboundAccount} from './accountOutbox.ts';
import {drainOutbox,type OutboxStore,type QueuedItem} from './outboxCore.ts';
import {prepareLocalFiles} from './localFiles.ts';
import {mintTrackingToken,bindTrackingToken} from '../api/tracking.ts';

interface NativeBackgroundConfig {native?:boolean;backgroundReplay?:boolean;serverUrl?:string}
const config=()=>(globalThis as {__MW_CONFIG__?:NativeBackgroundConfig}).__MW_CONFIG__;
export const nativeBackgroundStartup=():boolean=>config()?.native===true&&config()?.backgroundReplay===true;
const queue=()=>encryptedOutboxStore(idbOutboxStore());

export async function requestNativeBackgroundReplay():Promise<boolean>{
 const base=nativeServerBase();if(!base)return false;
 const pending=(await queue().all() as QueuedItem[]).some(row=>row.state==='queued'&&row.cacheBase===`native:${base}`);
 const {invokeBackground}=await import('../platform/tauri.ts');
 return (await invokeBackground('mw_background_replay_schedule',{enabled:pending})).supported;
}

/** Replay uses the same encrypted store and drain as tabs, with no cached
 * identity fallback. Unknown legacy rows and other configured servers stay put. */
export async function replayNativeQueue(deps:{
 base:string;store:OutboxStore;
 verify:(linkId:string|null)=>Promise<{me:Me;session:JmapSession}>;
 run:(row:QueuedItem,identity:{me:Me;session:JmapSession},store:OutboxStore)=>Promise<void>;
}):Promise<boolean>{
 const contexts=new Map<string,QueuedItem>();
 for(const row of await deps.store.all() as QueuedItem[]){
  if(row.state==='queued'&&row.cacheBase===`native:${deps.base}`&&row.linkId!==undefined&&row.storageAuthority&&outboundAccount(row))contexts.set(JSON.stringify([row.linkId,row.storageAuthority,outboundAccount(row)]),row);
 }
 for(const row of contexts.values()){
  const identity=await deps.verify(row.linkId??null),account=outboundAccount(row)!;
  if(identity.me.accountId!==account||identity.me.storageAuthority!==row.storageAuthority||identity.me.passwordChangeRequired||identity.session.primaryAccounts[CAP_MAIL]!==account||!Object.hasOwn(identity.session.accounts,account))continue;
  const owned=accountOutbox(deps.store,account,row.storageAuthority!);
  const selected={...owned,all:async()=>(await owned.all()).filter(item=>(item as QueuedItem).cacheBase===`native:${deps.base}`&&(item as QueuedItem).linkId===row.linkId)};
  await deps.run(row,identity,selected);
 }
 return !(await deps.store.all() as QueuedItem[]).some(row=>row.state==='queued'&&row.cacheBase===`native:${deps.base}`);
}

/** Dedicated hidden-shell entry: no App, login screen, dialogs or cached Me.
 * Bearer material is read from the OS keychain, never copied into scheduler jobs. */
export async function startNativeBackgroundReplay():Promise<void>{
 let complete=false;
 try{
  const base=nativeServerBase(),platform=await platformReady();
  if(!base||!await platform.getSessionToken())return;
  complete=await replayNativeQueue({base,store:queue(),verify:async linkId=>{
   suspendAccountPreferences();installLinkedMailbox(null);
   const read=async(path:string)=>{const response=await apiFetch(path,{mailboxContext:linkId,cache:'no-store',redirect:'error',credentials:'omit'});if(!response.ok)throw new ApiError(response.status,'Background identity refused');return response;};
   const me=await (await read('/api/me')).json() as Me,session=await (await read('/jmap/session')).json() as JmapSession;return {me,session};
  },run:async(row,{me,session},store)=>{
   const context=row.linkId?{linkId:row.linkId,accountId:me.accountId,storageAuthority:me.storageAuthority!}:null;
   installLinkedMailbox(context);activateAccountPreferences(me.accountId,me.storageAuthority);
   const api=jmapApiPath(session.apiUrl,base);
   const headers={'X-Mailwoman-Account':me.accountId,'X-Mailwoman-Storage-Authority':me.storageAuthority!};
   const send=async(path:string,init:RequestInit={}):Promise<Response>=>{
    let response:Response;
    try{response=await apiFetch(path,{...init,credentials:'omit',redirect:'error',headers:{...headers,...Object.fromEntries(new Headers(init.headers))}});}catch(cause){throw new NetworkError(cause instanceof Error?cause.message:'Background connection unavailable');}
    if(response.status>=500||response.status===429)throw new NetworkError('Background service unavailable');
    if(!response.ok)throw new ApiError(response.status,'Background replay refused');return response;
   };
   const client={jmap:async(request,options)=>{
    const outgoing=await prepareLocalFiles(request,{accountId:me.accountId,storageAuthority:me.storageAuthority!,base:`native:${base}`},async file=>{
     const upload=new URL(session.uploadUrl.replace('{accountId}',encodeURIComponent(me.accountId)),new URL(base).origin),root=new URL(base);
     const prefix=root.pathname.replace(/\/$/,'');
     if(upload.origin!==root.origin||!upload.pathname.startsWith(`${prefix}/jmap/upload/`)||upload.username||upload.password||upload.hash||upload.search)throw new Error('Attachment upload outside Mailwoman');
     return await (await send(upload.pathname.slice(prefix.length),{method:'POST',headers:{'content-type':file.type},body:file})).json() as {blobId:string;size:number;type:string};
    });
    return await (await send(api,{method:'POST',headers:{'content-type':'application/json',...(typeof options?.idempotencyKey==='string'?{'Idempotency-Key':options.idempotencyKey}:{})},body:JSON.stringify(outgoing),...(options?.signal?{signal:options.signal}:{})})).json() as JmapResponse;
   }} as Client;
   await drainOutbox(store,client,{mint:(subject,to)=>mintTrackingToken(subject,to,send),bind:(token,id)=>bindTrackingToken(token,id,send)});
  }});
 }finally{
  suspendAccountPreferences();installLinkedMailbox(null);
  const {invokeBackground}=await import('../platform/tauri.ts');
  await invokeBackground('mw_background_replay_complete',{complete});
 }
}
