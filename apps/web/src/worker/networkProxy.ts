import {keyedAttempt} from '../api/recovery.ts';
import {jmapReadsOnly} from '../api/jmapReadOnly.ts';
import {settings as actionSettings} from '../api/settings.ts';
import {createChannelNetworkConnection} from './networkChannel.ts';
import {ApiError,NetworkError,isKeyedCreate,type Client,type JmapCallOptions,type Me,PasswordChangeRequired} from '../api/client.ts';
import type {JmapRequest,JmapResponse,JmapSession} from '../api/jmap-types.ts';
import {activeLinkedMailbox,primaryMailboxAuthority} from '../api/linkedMailboxSession.ts';
import {preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {basePath} from '../api/basePath.ts';
import {createPushClient,type PushClientOptions,type PushClientImpl,type PushStatus,type PushStatusDetail} from '../realtime/pushClient.ts';
import type {StateChange,MailboxChange,PushTransport} from '../contracts/push.ts';
import type {NetworkContext,NetworkRequest,NetworkReply} from './networkCore.ts';

export interface Connection {mode:'shared-worker'|'leader';ready:Promise<boolean>;call(method:NetworkRequest['method'],context:NetworkContext,extra?:Partial<NetworkRequest>,signal?:AbortSignal):Promise<unknown>;subscribe(listener:(event:string,value:unknown)=>void):()=>void}
let shared:Connection|undefined;
function context(linkId=activeLinkedMailbox()?.linkId??null):NetworkContext{
 const account=preferenceAccount(),active=activeLinkedMailbox()?.linkId??null;
 return {preferWebSocket:actionSettings()['realtime.actions.preferWebSocket']?.value===true&&actionSettings()['realtime.actions.enabled']?.value!==false,base:`${location.origin}${basePath()}`,linkId,...(account?.storageAuthority&&linkId===active?{owner:{accountId:account.id,storageAuthority:account.storageAuthority}}:{})};
}
function openConnection():Connection|undefined{
 if(shared)return shared;
 if(typeof SharedWorker==='undefined'){shared=createChannelNetworkConnection(context());return shared;}
 let worker:SharedWorker;
 try{worker=new SharedWorker(new URL('./networkWorker.ts',import.meta.url),{type:'module',name:'mw-jmap-network'});}catch{shared=createChannelNetworkConnection(context());return shared;}
 let sequence=0,healthy=true;
 const events=new Set<(event:string,value:unknown)=>void>();
 const pending=new Map<string,{resolve(value:unknown):void;reject(reason:unknown):void;cleanup():void;mutation:boolean}>();
 function fail(){healthy=false;for(const row of [...pending.values()])row.reject(row.mutation?new ApiError(409,'The action outcome is unknown. Check before retrying.',{code:'outcomeUnknown'}):new NetworkError('Shared worker connection unavailable'));pending.clear();}
 worker.onerror=()=>fail();
 worker.port.onmessage=event=>{
  const reply=event.data as NetworkReply;
  if('event' in reply){for(const listener of events)listener(reply.event,reply.value);return;}
  const row=pending.get(reply.id);if(!row)return;row.cleanup();pending.delete(reply.id);
  if(reply.error){const error=reply.error;row.reject(error.status?new ApiError(error.status,error.message,{...(error.code?{code:error.code}:{}),...(error.retryAfterSecs!==undefined?{retryAfterSecs:error.retryAfterSecs}:{})}):new NetworkError(error.message));}else row.resolve(reply.result);
 };
 worker.port.start();
 const call:Connection['call']=(method,scope,extra={},signal)=>new Promise((resolve,reject)=>{
  if(!healthy){reject(new NetworkError('Shared worker disconnected'));return;}
  if(signal?.aborted){reject(new DOMException('Request aborted','AbortError'));return;}
  const id=String(++sequence),timer=setTimeout(()=>{pending.delete(id);signal?.removeEventListener('abort',abort);reject(method==='jmap'&&extra.body&&!jmapReadsOnly(extra.body)?new ApiError(503,'The action may have completed. Check before retrying.',{code:'outcomeUnknown'}):new NetworkError('Shared request timed out'));},method==='jmap'?130000:30000);
  const abort=()=>{clearTimeout(timer);pending.delete(id);reject(method==='jmap'&&extra.body&&!jmapReadsOnly(extra.body)?new ApiError(409,'The action outcome is unknown. Check before retrying.',{code:'outcomeUnknown'}):new DOMException('Request aborted','AbortError'));};
  const cleanup=()=>{clearTimeout(timer);signal?.removeEventListener('abort',abort);};
  pending.set(id,{resolve,reject:reason=>{cleanup();reject(reason);},cleanup,mutation:method==='jmap'&&extra.body!==undefined&&!jmapReadsOnly(extra.body)});signal?.addEventListener('abort',abort,{once:true});
  worker.port.postMessage({id,method,context:scope,...extra} satisfies NetworkRequest);
 });
 let handshakeTimer:ReturnType<typeof setTimeout>;
 const ready=Promise.race([call('lease',context()).then(()=>true),new Promise<boolean>(resolve=>{handshakeTimer=setTimeout(()=>resolve(false),3000);})]).catch(()=>false).finally(()=>clearTimeout(handshakeTimer));
 const heartbeat=setInterval(()=>{if(healthy)void call('lease',context()).catch(()=>undefined);},15000);
 window.addEventListener('pagehide',event=>{if(event.persisted)return;clearInterval(heartbeat);worker.port.postMessage({method:'disconnect'});worker.port.close();fail();},{once:true});
 shared={mode:'shared-worker',ready,call,subscribe(listener){events.add(listener);return()=>events.delete(listener);}};
 return shared;
}

/**
 * Development and test builds only (`import.meta.env` is replaced at build time, so this is dead code in production):
 * every call crosses postMessage (shared worker) or a BroadcastChannel, and a Solid store proxy, a function or a DOM node
 * in the payload fails there with a message that names no caller. Clone the payload first and name the method instead.
 * The clone is discarded; no call here passes a transfer list, so nothing is transferred or detached.
 */
export function assertCloneable(method:string,payload:unknown):void{
 try{structuredClone(payload);}catch(error){throw new Error(`Network call "${method}" has a payload that cannot be structured-cloned (a Solid store proxy, function or DOM node?): ${error instanceof Error?error.message:String(error)}`);}
}
let guarded:Connection|undefined;
function connection():Connection|undefined{
 const inner=openConnection();if(!inner||!(import.meta.env.DEV||import.meta.env.MODE==='test'))return inner;
 if(guarded?.subscribe===inner.subscribe)return guarded;
 return guarded={...inner,call:(method,scope,extra,signal)=>{try{assertCloneable(method,{method,context:scope,...extra});}catch(error){return Promise.reject(error);}return inner.call(method,scope,extra,signal);}};
}
export function withSharedNetwork(client:Client):Client{
 const channel=connection();if(!channel)return client;
 const network=new Set<(up:boolean)=>void>(),unauthorized=new Set<()=>void>();
 async function run<T>(method:'me'|'session'|'jmap',fallback:()=>Promise<T>,linkId:string|null,body?:JmapRequest,options?:JmapCallOptions):Promise<T>{
  if(!await channel!.ready){
   const expected=context(linkId).owner;
   if(expected){const live=await client.me({linkId});if(live.accountId!==expected.accountId||live.storageAuthority!==expected.storageAuthority)throw new ApiError(409,'The signed-in account changed',{code:'accountChanged'});}
   return fallback();
  }
  try{
   const result=await channel!.call(method,context(linkId),{...(body?{body}:{}),...(options?.idempotencyKey!==undefined?{idempotencyKey:options.idempotencyKey}:{})},options?.signal);
   for(const listener of network)listener(true);return result as T;
  }catch(error){
   if(error instanceof ApiError&&error.status===401)for(const listener of unauthorized)listener();
   if(error instanceof ApiError&&error.code==='passwordChangeRequired')throw new PasswordChangeRequired();
   if(error instanceof NetworkError)for(const listener of network)listener(false);
   throw error;
  }
 }
 return {...client,
  onNetwork(listener){network.add(listener);const stop=client.onNetwork(listener);return()=>{network.delete(listener);stop();};},
  onUnauthenticated(listener){unauthorized.add(listener);const stop=client.onUnauthenticated?.(listener);return()=>{unauthorized.delete(listener);stop?.();};},
  async login(input){const me=await client.login(input);if(await channel.ready)await channel.call('reset',context(null));return me;},
  async logout(){try{await client.logout();}finally{if(await channel.ready)await channel.call('reset',context(null));}},
  me:options=>run<Me>('me',()=>client.me(options),options?options.linkId:activeLinkedMailbox()?.linkId??null),
  session:options=>run<JmapSession>('session',()=>client.session(options),options?options.linkId:activeLinkedMailbox()?.linkId??null),
  jmap:(body,options)=>{
   const scope=context(),linkId=scope.linkId;
   const send=(key?:string)=>{const opts=key?{...options,idempotencyKey:key}:options;return run<JmapResponse>('jmap',()=>client.jmap(body,opts),linkId,body,opts);};
   // The shared path bypasses createClient.jmap's automatic keys. Keep one key
   // for an ambiguous create so a manual retry or leader failover cannot
   // create the same task, contact or folder twice.
   return options?.idempotencyKey===undefined&&isKeyedCreate(body)
    ?keyedAttempt(JSON.stringify(['shared-jmap',scope.base,scope.linkId,scope.owner,body]),key=>send(key)):send();
  },
 };
}

/** Every window retains its connection UI; the worker owns one actual socket. */
export function createSharedPushClient(options:PushClientOptions):PushClientImpl{
 const channel=connection();if(!channel||options.bearer||options.WebSocketImpl||options.EventSourceImpl)return createPushClient(options);
 let active=false,transport:PushTransport='offline',fallback:PushClientImpl|undefined;
 const states=new Set<(change:StateChange)=>void>(),mailboxes=new Set<(change:MailboxChange)=>void>();
 channel.subscribe((event,value)=>{
  if(!active)return;
  if(event==='coordinator-reconnected'){options.onStatus?.('reconnecting','offline');void start('push.start');}
  else if(event==='status'){const status=value as {status:PushStatus;transport:PushTransport;detail?:PushStatusDetail};transport=status.transport;if(status.detail)options.onStatus?.(status.status,transport,status.detail);else options.onStatus?.(status.status,transport);}
  else if(event==='invalidate'){const owner=preferenceAccount();if(owner){const state=`local:${Date.now()}`;for(const listener of states)listener({'@type':'StateChange',changed:{[owner.id]:{Email:state,Mailbox:state,Thread:state,EmailSubmission:state,Calendar:state,CalendarEvent:state,Task:state,Note:state,AddressBook:state,ContactCard:state,ContactGroup:state}}});}}
  else if(event==='change')for(const listener of states)listener(value as StateChange);
  else if(event==='mailbox')for(const listener of mailboxes)listener(value as MailboxChange);
  else if(event==='reset')options.onStatus?.('unauthorized','offline');
 });
 async function start(method:'push.start'|'push.reconnect'){
  active=true;
  if(!await channel!.ready){
   if(!fallback){fallback=createPushClient(options);fallback.onStateChange(value=>{for(const listener of states)listener(value);});fallback.onMailboxChange?.(value=>{for(const listener of mailboxes)listener(value);});}
   if(active)fallback.connect();return;
  }
  if(!active)return;
  try{const authority=primaryMailboxAuthority();await channel!.call(method,context(null),{...(authority?{primaryAuthority:authority}:{})});}
  catch(error){options.onStatus?.(error instanceof ApiError&&[401,403,409].includes(error.status)?'unauthorized':'reconnecting','offline');}
 }
 return {connect(){void start('push.start');},reconnect(){void start('push.reconnect');},close(){active=false;fallback?.close();void channel.call('push.stop',context(null)).catch(()=>undefined);},onStateChange(listener){states.add(listener);return()=>states.delete(listener);},onMailboxChange(listener){mailboxes.add(listener);return()=>mailboxes.delete(listener);},transport:()=>fallback?.transport()??transport};
}

export function networkCoordinationMode():'shared-worker'|'leader'|'local'{return shared?.mode??'local';}

export interface SharedView {topic:string;value:unknown;sender:string;accountId:string;authority:string}
export const sharedWindowId=typeof crypto.randomUUID==='function'?crypto.randomUUID():`${Date.now()}:${Math.random()}`;
export async function publishSharedView(topic:string,value:unknown):Promise<void>{
 const channel=connection(),owner=preferenceAccount(),scope=context();if(!channel||!owner?.storageAuthority||!await channel.ready||preferenceAccount()!==owner)return;
 await channel.call('view.put',scope,{view:{topic,value,sender:sharedWindowId}}).catch(()=>undefined);
}
export async function readSharedView(topic:string):Promise<unknown>{
 const channel=connection(),owner=preferenceAccount(),scope=context();if(!channel||!owner?.storageAuthority||!await channel.ready||preferenceAccount()!==owner)return;
 return channel.call('view.get',scope,{view:{topic,sender:sharedWindowId}}).catch(()=>undefined);
}
export function observeSharedViews(listener:(view:SharedView)=>void):()=>void{
 const channel=connection();return channel?.subscribe((event,value)=>{
  if(event!=='view')return;const view=value as SharedView,owner=preferenceAccount();
  if(view.sender!==sharedWindowId&&owner?.storageAuthority===view.authority&&owner.id===view.accountId)listener(view);
 })??(()=>undefined);
}
