import {actionConnectionStability} from '../realtime/connectionStability.ts';
import {consolePath} from './adminPath.ts';
import {adminSessionGeneration} from './adminSession.ts';
import {ApiError,ActionCompletionError} from './errors.ts';
import {sendActionCookieDelivery} from './workerHttp.ts';
import {actionContract,actionHeaders,actionRoute,validActionPath,type ActionActor} from './websocketActionRoute.ts';

interface CookieDelivery {token:string;expiresInMs:number}
interface Ack {type?:string;protocol?:string;id?:string;status?:number;headers?:unknown;body?:string;actionOutcome?:string;httpRequired?:boolean;cookieDelivery?:unknown}
interface Job {frame:string;init:RequestInit;bytes:number;resolve:(response:Response|undefined)=>void;reject:(error:unknown)=>void;stop:()=>void;sent:boolean;done:boolean;assertAuthority:()=>void;completed?:Response}
interface EncodedFrame {text:string;bytes:number}
// Only module-owned request snapshots enter this map. The public send API never
// accepts caller-supplied serialized frames that could override current CSRF.
const preparedFrames=new WeakMap<RequestInit,EncodedFrame>();
function encodeFrame(path:string,init:RequestInit,headers:Record<string,string>):EncodedFrame{
 const text=JSON.stringify({id:'',path,method:(init.method??'GET').toUpperCase(),headers,...(typeof init.body==='string'?{body:init.body}:{})});
 return {text,bytes:new TextEncoder().encode(text).length};
}
function frameFits(frame:EncodedFrame):boolean{return frame.bytes+32<=actionContract.limits.frameBytes;}
const MAX_QUEUED=16,MAX_QUEUED_BYTES=8*1024*1024;
function unknown():ApiError{return new ApiError(409,'The action outcome is unknown. Check its result before retrying.',{code:'outcomeUnknown'});}
function busy():ApiError{return new ApiError(429,'The action channel is busy; no action was sent.',{code:'actionNotDispatchedBusy'});}
function replyHeaders(value:unknown):Headers|undefined{
 if(value!==undefined&&(value===null||typeof value!=='object'||Array.isArray(value)))return;
 const headers=new Headers();let length=0,count=0;const names=new Set<string>();
 for(const [key,entry] of Object.entries(value??{})){
  const name=key.toLowerCase();
  if(typeof entry!=='string'||names.has(name)||name==='set-cookie'||name==='set-cookie2'||/[\r\n\0]/.test(entry)||++count>actionContract.limits.headers)return;
  names.add(name);length+=new TextEncoder().encode(key+entry).length;if(length>actionContract.limits.headerBytes)return;
  try{headers.set(name,entry);}catch{return;}
 }
 return headers;
}
/** A lost ACK is terminal, including for a read. Only an explicit pre-dispatch
 * HTTP control refusal can return undefined after send. Queued jobs never spill. */
export function createWebSocketActions(options:{endpoint:string;WebSocketImpl?:typeof WebSocket;timeoutMs?:number;maxInFlight?:number;onConnection?:(connected:boolean)=>void;onCookieDelivery?:(receipt:CookieDelivery,response:Response,init:RequestInit,assertAuthority:()=>void)=>Promise<void>}){
 let socket:WebSocket|undefined,ready=false,sequence=0,opening:Promise<boolean>|undefined,disabledUntil=0,retired=false,completing=false;
 let finishOpening:((ready:boolean)=>void)|undefined,onRetired:(()=>void)|undefined;
 const pending=new Map<string,Job>(),queue:Job[]=[];let queueBytes=0;
 const limit=Math.min(8,Math.max(1,options.maxInFlight??8));
 function settle(job:Job,error?:unknown,response?:Response):void{if(job.done)return;job.done=true;job.stop();if(error!==undefined)job.reject(error);else job.resolve(response);}
 function close(fallbackUnsent=false):void{
  finishOpening?.(false);finishOpening=undefined;ready=false;const old=socket;socket=undefined;opening=undefined;
  if(old){old.onmessage=null;old.onerror=null;old.onclose=null;old.close();}
  for(const job of pending.values())if(!job.completed)settle(job,unknown());pending.clear();
  for(const job of queue.splice(0))settle(job,fallbackUnsent?undefined:new ApiError(409,'The action channel changed before dispatch.',{code:'actionNotDispatched'}));queueBytes=0;
  onRetired?.();onRetired=undefined;
 }
 function drain():void{if(retired&&pending.size===0&&!completing)close();else pump();}
 function retire(done:()=>void):void{retired=true;onRetired=done;for(const job of queue.splice(0))settle(job,new ApiError(409,'Authentication changed before dispatch.',{code:'actionNotDispatched'}));queueBytes=0;drain();}
 function pump():void{
  if(!ready||retired||completing||!socket)return;
  while(queue.length&&pending.size<limit){const job=queue.shift()!;queueBytes-=job.bytes;if(job.done)continue;
   try{job.assertAuthority();}catch(error){settle(job,error);continue;}
   const id=String(++sequence);const frame=job.frame.replace('"id":""',`"id":"${id}"`);job.sent=true;pending.set(id,job);
   try{socket.send(frame);}catch{pending.delete(id);settle(job,unknown());close();return;}
  }
 }
 async function receive(value:Ack):Promise<void>{
  if(value.type==='ready'&&value.protocol===actionContract.protocol){ready=true;options.onConnection?.(true);finishOpening?.(true);pump();return;}
  if(typeof value.id!=='string')return;const job=pending.get(value.id);if(!job||job.completed)return;
  if(!Number.isInteger(value.status)||value.status!<200||value.status!>599||typeof value.body!=='string'||new TextEncoder().encode(value.body).length>actionContract.limits.responseBytes){settle(job,unknown());pending.delete(value.id);close();return;}
  if(value.actionOutcome==='notDispatched'){pending.delete(value.id);if(value.httpRequired===true)settle(job);else settle(job,new ApiError(value.status!,'The server refused the action before dispatch.',{code:'actionNotDispatched'}));close();return;}
  // Old servers omit the additive outcome field. Their supported normal ACK
  // remains usable, but never grants post-send HTTP fallback.
  if(value.actionOutcome!==undefined&&value.actionOutcome!=='completed'){pending.delete(value.id);settle(job,unknown());close();return;}
  const headers=replyHeaders(value.headers);if(!headers){pending.delete(value.id);settle(job,unknown());close();return;}
  let response:Response;try{response=new Response([204,205,304].includes(value.status!)?null:value.body,{status:value.status!,headers});}catch{pending.delete(value.id);settle(job,unknown());close();return;}
  let completedTransportFailure=false;
  if(value.body.includes('websocketActionCompletedTransportFailure')){
   try{completedTransportFailure=(JSON.parse(value.body) as {code?:unknown}).code==='websocketActionCompletedTransportFailure';}catch{/* Normal non-JSON responses remain valid. */}
  }
  if(value.cookieDelivery!==undefined){
   const receipt=value.cookieDelivery as Partial<CookieDelivery>|null;
   if(!receipt||typeof receipt.token!=='string'||(!/^[a-f0-9]+$/.test(receipt.token)||receipt.token.length!==actionContract.cookieHandoff.tokenCharacters)||!Number.isInteger(receipt.expiresInMs)||receipt.expiresInMs!<=0||receipt.expiresInMs!>actionContract.cookieHandoff.receiptTtlMs||!options.onCookieDelivery){pending.delete(value.id);settle(job,new ActionCompletionError(response));close();return;}
   completing=true;retired=true;job.completed=response;
   // Own the completed response independently of caller cancellation. Its abort
   // cannot erase the result or cause the application frame to be replayed.
   job.stop();
   try{await options.onCookieDelivery(receipt as CookieDelivery,response,job.init,job.assertAuthority);settle(job,completedTransportFailure?new ActionCompletionError(response):undefined,response);}
   catch{settle(job,new ActionCompletionError(response));}
   finally{pending.delete(value.id);completing=false;close();}
   return;
  }
  pending.delete(value.id);if(completedTransportFailure){retired=true;settle(job,new ActionCompletionError(response));close();return;}settle(job,undefined,response);drain();
 }
 function connect():Promise<boolean>{
  if(retired)return Promise.resolve(false);if(ready)return Promise.resolve(true);if(opening)return opening;if(Date.now()<disabledUntil)return Promise.resolve(false);
  const Constructor=options.WebSocketImpl??globalThis.WebSocket;if(typeof Constructor!=='function')return Promise.resolve(false);
  opening=new Promise(resolve=>{
   let settled=false;const finish=(value:boolean)=>{if(settled)return;settled=true;clearTimeout(timer);finishOpening=undefined;if(!value)disabledUntil=Date.now()+30000;resolve(value);};
   finishOpening=finish;const timer=setTimeout(()=>{finish(false);close(true);},5000);
   try{socket=new Constructor(options.endpoint);}catch{finish(false);return;}
   socket.onmessage=event=>{
    if(typeof event.data!=='string'||new TextEncoder().encode(event.data).length>actionContract.limits.replyFrameBytes){close();return;}
    let value:Ack;try{value=JSON.parse(event.data) as Ack;if(!value||typeof value!=='object'){close();return;}}catch{close();return;}
    void receive(value).catch(()=>close());
   };
   socket.onerror=()=>{if(!retired)options.onConnection?.(false);const fallback=!ready;finish(false);close(fallback);};socket.onclose=()=>{if(!retired)options.onConnection?.(false);const fallback=!ready;finish(false);close(fallback);};
  });
  const attempt=opening;return attempt.then(value=>{if(!value&&opening===attempt)opening=undefined;return value;});
 }
 async function send(path:string,init:RequestInit,assertAuthority:()=>void=()=>{}):Promise<Response|undefined>{
  const headers=actionHeaders(init.headers),method=(init.method??'GET').toUpperCase();
  if(!validActionPath(path)||!headers||!actionContract.methods.includes(method)||init.body!=null&&typeof init.body!=='string')throw new ApiError(400,'Invalid action request',{code:'actionNotDispatched'});
  const encoded=preparedFrames.get(init)??encodeFrame(path,init,headers);preparedFrames.delete(init);
  // Body acceptance does not imply that the JSON-string envelope fits. This is
  // known unsupported BEFORE queue/socket admission, so ordinary HTTP is safe.
  if(!frameFits(encoded))return undefined;
  const frame=encoded.text,size=encoded.bytes;
  if(init.signal?.aborted)throw init.signal.reason;
  if(retired)throw new ApiError(409,'Action authority retired',{code:'actionNotDispatched'});
  if(queue.length>=MAX_QUEUED||queueBytes+size>MAX_QUEUED_BYTES)throw busy();
  // Reserve bounded queue space BEFORE waiting for the shared handshake.
  let job!:Job;
  const result=new Promise<Response|undefined>((resolve,reject)=>{
   const abort=()=>{if(job.done)return;const index=queue.indexOf(job);if(index>=0){queue.splice(index,1);queueBytes-=job.bytes;}if(job.sent){for(const [id,row]of pending)if(row===job)pending.delete(id);settle(job,unknown());close();}else{settle(job,init.signal?.reason??new DOMException('Request aborted','AbortError'));drain();}};
   const timer=setTimeout(abort,Math.min(120000,Math.max(1,options.timeoutMs??120000)));
   const stop=()=>{clearTimeout(timer);init.signal?.removeEventListener('abort',abort);};
   job={frame,init,bytes:size,resolve,reject,stop,sent:false,done:false,assertAuthority};queue.push(job);queueBytes+=size;init.signal?.addEventListener('abort',abort,{once:true});
  });
  // The reservation promise owns its errors immediately, even during handshake.
  void connect().then(connected=>{if(!connected&&!job.done&&!job.sent){const index=queue.indexOf(job);if(index>=0){queue.splice(index,1);queueBytes-=job.bytes;}settle(job);}else pump();},()=>{settle(job);});
  pump();return result;
 }
 return {send,close,retire};
}

type Channel=ReturnType<typeof createWebSocketActions>;
interface Actor {client:Channel;endpoint:string;proof:string;epoch:number;generation:number}
const actors=new Map<ActionActor,Actor>(),retiring=new Set<Channel>(),adminUnsent=new Set<()=>void>();
let generation=0,adminPending=0,adminBytes=0,adminTail:Promise<void>=Promise.resolve();
let blockedAdmin:{generation:number;epoch:number;proof:string}|undefined;
function cookieProof():string{return typeof document==='undefined'?'':document.cookie.split(';').map(p=>p.trim()).find(p=>p.startsWith('mw_csrf='))?.slice(8)??'';}
export function clearWebSocketActions():void{generation++;for(const cancel of adminUnsent)cancel();actionConnectionStability.reset();for(const actor of actors.values())actor.client.close();for(const old of retiring)old.close();retiring.clear();actors.clear();blockedAdmin=undefined;}
async function delivered(response:Response):Promise<boolean>{
 const reader=response.body?.getReader();if(!reader)return false;let text='',bytes=0;const decoder=new TextDecoder('utf-8',{fatal:true});
 try{for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.byteLength;if(bytes>4096){await reader.cancel();return false;}text+=decoder.decode(part.value,{stream:true});}text+=decoder.decode();return (JSON.parse(text) as {delivered?:unknown}).delivered===true;}catch{return false;}finally{reader.releaseLock();}
}
async function deliverCookie(endpoint:string,receipt:CookieDelivery,init:RequestInit,assertAuthority:()=>void):Promise<void>{
 const deadline=Date.now()+Math.min(receipt.expiresInMs,10000);const old=new Headers(init.headers).get('x-csrf-token')??cookieProof();
 for(let attempt=0;attempt<2;attempt++){
  assertAuthority();const remaining=deadline-Date.now();if(remaining<=0)break;const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),remaining);
  try{
   // Only this control is idempotently retried, never the application frame.
   const response=await sendActionCookieDelivery(endpoint,{method:'POST',headers:{'content-type':'application/json','x-csrf-token':attempt===0?old:cookieProof()||old},body:JSON.stringify({token:receipt.token}),signal:controller.signal});
   if(response.status===200&&response.headers.get('content-type')?.startsWith('application/json')&&await delivered(response))return;
   if(response.status<500)break;
  }catch{/* A lost control ACK may be retried with the same receipt and new cookie. */}
  finally{clearTimeout(timer);}
 }
 throw new Error('Cookie completion unavailable');
}
export async function websocketAction(input:string,init:RequestInit|undefined,base:string,enabled=true,assertAuthority:()=>void=()=>{}):Promise<Response|undefined>{
 if(!enabled)return;
 const route=actionRoute(input,init,base,consolePath());if(!route)return;
 const snapshot:RequestInit={...init,headers:actionHeaders(init?.headers)!};
 let encoded:EncodedFrame|undefined=encodeFrame(route.path,snapshot,snapshot.headers as Record<string,string>);
 // Also precedes the administrator's serialized turn admission. Do not turn a
 // local transport-size mismatch into either queue debt or a server refusal.
 if(!frameFits(encoded))return undefined;
 // Admin turns refresh CSRF after prior cookie completion. Regenerate from that
 // actual request in send; don't retain an extra serialized body in its queue.
 if(route.actor==='admin')encoded=undefined;
 const admitted=generation,adminEpoch=adminSessionGeneration();
 const check=()=>{assertAuthority();if(admitted!==generation||route.actor==='admin'&&adminEpoch!==adminSessionGeneration())throw new ApiError(409,'Authentication changed before dispatch',{code:'actionNotDispatched'});};
 const run=async()=>{
  check();if(init?.signal?.aborted)throw init.signal.reason;
  const currentProof=cookieProof(),proof=route.actor==='admin'?(currentProof||new Headers(snapshot.headers).get('x-csrf-token')||''):(new Headers(snapshot.headers).get('x-csrf-token')??currentProof);let actor=actors.get(route.actor);
  if(route.actor==='admin'&&blockedAdmin&&blockedAdmin.generation===generation&&blockedAdmin.epoch===adminEpoch&&blockedAdmin.proof===currentProof)throw new ApiError(409,'Sign in again before sending another administrator action.',{code:'actionNotDispatched'});
  if(!actor||actor.endpoint!==route.endpoint||actor.proof!==proof||actor.generation!==admitted||route.actor==='admin'&&actor.epoch!==adminEpoch){
   if(actor){if(retiring.size>=8)throw busy();const old=actor.client;retiring.add(old);old.retire(()=>retiring.delete(old));}
   const client:Channel=createWebSocketActions({endpoint:route.endpoint,maxInFlight:route.actor==='admin'?1:actionContract.webmail.maxInFlight,onConnection:connected=>actionConnectionStability.observe(connected),...(route.cookieEndpoint?{onCookieDelivery:async(receipt:CookieDelivery,_response:Response,request:RequestInit,requestCheck:()=>void)=>{const deliveryGeneration=generation,deliveryEpoch=adminSessionGeneration();try{await deliverCookie(route.cookieEndpoint!,receipt,request,requestCheck);if(deliveryGeneration===generation&&deliveryEpoch===adminSessionGeneration())blockedAdmin=undefined;}catch(error){if(deliveryGeneration===generation&&deliveryEpoch===adminSessionGeneration())blockedAdmin={generation,epoch:deliveryEpoch,proof:cookieProof()};throw error;}finally{if(actors.get('admin')?.client===client)actors.delete('admin');}}}:{})});
   actor={client,endpoint:route.endpoint,proof,epoch:adminEpoch,generation:admitted};actors.set(route.actor,actor);
  }
  // Each admin turn reads browser CSRF after the prior cookie handoff. Caller
  // supplied stale proof is not reused for a queued, previously unsent action.
  const request={...snapshot};if(route.actor==='admin'){const headers=new Headers(snapshot.headers),current=cookieProof();if(current)headers.set('x-csrf-token',current);request.headers=headers;}
  else if(encoded)preparedFrames.set(request,encoded);
  return actor.client.send(route.path,request,check);
 };
 if(route.actor!=='admin')return run();
 const bytes=new TextEncoder().encode(JSON.stringify({path:route.path,headers:actionHeaders(snapshot.headers),body:snapshot.body??''})).length;
 if(adminPending>=MAX_QUEUED||adminBytes+bytes>MAX_QUEUED_BYTES)throw busy();adminPending++;adminBytes+=bytes;
 let cancelled=false;let stop=()=>{};
 const waiting=new Promise<Response|undefined>((resolve,reject)=>{
  const cancel=()=>{if(cancelled)return;cancelled=true;reject(init?.signal?.reason??new ApiError(409,'Administrator action cancelled or expired before dispatch.',{code:'actionNotDispatched'}));};adminUnsent.add(cancel);
  const timer=setTimeout(cancel,30000);init?.signal?.addEventListener('abort',cancel,{once:true});stop=()=>{clearTimeout(timer);init?.signal?.removeEventListener('abort',cancel);};
  const turn=adminTail.then(async()=>{adminUnsent.delete(cancel);stop();if(cancelled)return;try{resolve(await run());}catch(error){reject(error);}});
  adminTail=turn.then(()=>{adminPending--;adminBytes-=bytes;},()=>{adminPending--;adminBytes-=bytes;});
 });
 try{return await waiting;}finally{stop();}
}
