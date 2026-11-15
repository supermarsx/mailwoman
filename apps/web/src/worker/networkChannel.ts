import {jmapReadsOnly} from '../api/jmapReadOnly.ts';
import {ApiError,NetworkError} from '../api/errors.ts';
import type {Connection} from './networkProxy.ts';
import type {NetworkContext,NetworkReply,NetworkRequest,createNetworkCore} from './networkCore.ts';
/** Where SharedWorker is unavailable, a Web Lock elects one window to own the
 * same network actor. BroadcastChannel carries requests and opaque events. */
export function createChannelNetworkConnection(initial:NetworkContext):Connection|undefined{
 if(typeof BroadcastChannel==='undefined'||typeof navigator==='undefined'||!navigator.locks||typeof window==='undefined')return;
 let channel:BroadcastChannel;try{channel=new BroadcastChannel(`mw-network:${initial.base}`);}catch{return;}
 const id=crypto.randomUUID(),events=new Set<(event:string,value:unknown)=>void>();
 const pending=new Map<string,{resolve(value:unknown):void;reject(reason:unknown):void;cleanup():void;request:NetworkRequest}>();
 let core:ReturnType<typeof createNetworkCore>|undefined,sequence=0,leader:string|undefined,stopped=false,campaigning=false,release:(()=>void)|undefined;
 let markReady:(value:boolean)=>void=()=>undefined;
 const ready=new Promise<boolean>(resolve=>{markReady=resolve;});
 const readyTimeout=setTimeout(()=>markReady(false),3000);
 function answer(reply:NetworkReply){
  if('event' in reply){for(const listener of events)listener(reply.event,reply.value);return;}
  const row=pending.get(reply.id);if(!row)return;row.cleanup();pending.delete(reply.id);
  if(reply.error){const error=reply.error;row.reject(error.status?new ApiError(error.status,error.message,{...(error.code?{code:error.code}:{}),...(error.retryAfterSecs!==undefined?{retryAfterSecs:error.retryAfterSecs}:{})}):new NetworkError(error.message));}else row.resolve(reply.result);
 }
 function elect(){
  if(stopped||campaigning)return;campaigning=true;
  void navigator.locks.request(`mw-network:${initial.base}`,{ifAvailable:true},async lock=>{
   if(!lock||stopped)return;
   const {createNetworkCore}=await import('./networkCore.ts');if(stopped)return;
   core=createNetworkCore();leader=id;markReady(true);clearTimeout(readyTimeout);channel.postMessage({kind:'leader',sender:id});
   for(const listener of events)listener('coordinator-reconnected',null);
   resubmit();
   await new Promise<void>(resolve=>{release=resolve;});core.close();core=undefined;
  }).finally(()=>{campaigning=false;});
 }
 const dispatch=(sender:string,message:NetworkRequest)=>{
  if(!core)return;
  // A peer's first request creates its port-equivalent subscription.
  if(!known.has(sender)){known.add(sender);core.connect(sender,reply=>{if(sender===id)answer(reply);else channel.postMessage({kind:'answer',target:sender,reply});});}
  void core.receive(sender,message);
 };
 const known=new Set<string>();
 channel.onmessage=event=>{
  const message=event.data as {kind:string;sender?:string;target?:string;reply?:NetworkReply;request?:NetworkRequest};
  if(message.kind==='leader'&&message.sender){const changed=leader!==message.sender;leader=message.sender;markReady(true);clearTimeout(readyTimeout);if(changed){for(const listener of events)listener('coordinator-reconnected',null);resubmit();}}
  else if(message.kind==='ask'&&message.sender&&message.request)dispatch(message.sender,message.request);
  else if(message.kind==='answer'&&message.target===id&&message.reply)answer(message.reply);
  else if(message.kind==='discover'&&core)channel.postMessage({kind:'leader',sender:id});
  else if(message.kind==='disconnect'&&message.sender){core?.disconnect(message.sender);known.delete(message.sender);}
 };
 function resubmit(){
  for(const [requestId,row] of pending){
   const request=row.request;
   const safe=request.method!=='jmap'||request.idempotencyKey||request.body!==undefined&&jmapReadsOnly(request.body);
   if(safe){if(core)dispatch(id,request);else channel.postMessage({kind:'ask',sender:id,request});}
   else{row.cleanup();pending.delete(requestId);row.reject(new NetworkError('Coordinator changed before the mutation was acknowledged'));}
  }
 }
 const call:Connection['call']=(method,context,extra={},signal)=>new Promise((resolve,reject)=>{
  if(stopped||signal?.aborted){reject(new NetworkError('Coordinator unavailable'));return;}
  const requestId=`${id}:${++sequence}`,timer=setTimeout(()=>{pending.delete(requestId);reject(new NetworkError('Coordinator request timed out'));},30000);
  const abort=()=>{pending.delete(requestId);clearTimeout(timer);reject(new DOMException('Request aborted','AbortError'));};
  const cleanup=()=>{clearTimeout(timer);signal?.removeEventListener('abort',abort);};
  const request={id:requestId,method,context,...extra} satisfies NetworkRequest;
  pending.set(requestId,{resolve,reject,cleanup,request});signal?.addEventListener('abort',abort,{once:true});
  if(core)dispatch(id,request);else channel.postMessage({kind:'ask',sender:id,request});
 });
 const timer=setInterval(()=>{elect();if(core)channel.postMessage({kind:'leader',sender:id});if(leader)void call('lease',initial).catch(()=>undefined);},1500);
 window.addEventListener('pagehide',event=>{if(event.persisted)return;stopped=true;clearInterval(timer);clearTimeout(readyTimeout);channel.postMessage({kind:'disconnect',sender:id});release?.();core?.close();channel.close();for(const row of pending.values()){row.cleanup();row.reject(new NetworkError('Window closed'));}pending.clear();},{once:true});
 channel.postMessage({kind:'discover',sender:id});elect();
 return {mode:'leader',ready,call,subscribe(listener){events.add(listener);return()=>events.delete(listener);}};
}
