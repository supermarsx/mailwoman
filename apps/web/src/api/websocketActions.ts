import {actionConnectionStability} from '../realtime/connectionStability.ts';
import {ApiError} from './errors.ts';
/** A separate request socket keeps long JSON actions from blocking push events.
 * It uses the same authenticated HTTP routes on the server. Files stay HTTP. */
export function createWebSocketActions(options:{endpoint:string;WebSocketImpl?:typeof WebSocket;timeoutMs?:number;onConnection?:(connected:boolean)=>void}){
 let socket:WebSocket|undefined,ready=false,sequence=0,opening:Promise<boolean>|undefined,disabledUntil=0,retired=false;
 let finishOpening:((ready:boolean)=>void)|undefined,onRetired:(()=>void)|undefined;
 const pending=new Map<string,{resolve:(value:Response|undefined)=>void;reject:(error:unknown)=>void;stop:()=>void}>();
 function close():void{
  finishOpening?.(false);finishOpening=undefined;
  ready=false;const previous=socket;socket=undefined;opening=undefined;if(previous){previous.onmessage=null;previous.onerror=null;previous.onclose=null;previous.close();}
  for(const entry of pending.values()){entry.stop();entry.reject(new ApiError(503,'The action may have completed. Check the mailbox or Outbox before retrying.',{code:'outcomeUnknown'}));}pending.clear();onRetired?.();onRetired=undefined;
 }
 function drain():void{if(retired&&pending.size===0)close();}
 function retire(done:()=>void):void{retired=true;onRetired=done;drain();}
 function connect():Promise<boolean>{
  if(retired)return Promise.resolve(false);if(ready)return Promise.resolve(true);if(opening)return opening;if(Date.now()<disabledUntil)return Promise.resolve(false);
  const Constructor=options.WebSocketImpl??globalThis.WebSocket;if(typeof Constructor!=='function')return Promise.resolve(false);
  opening=new Promise(resolve=>{
   let settled=false;const finish=(value:boolean)=>{if(settled)return;settled=true;clearTimeout(timer);finishOpening=undefined;if(!value)disabledUntil=Date.now()+30000;resolve(value);};
   finishOpening=finish;
   const timer=setTimeout(()=>{finish(false);close();},5000);
   try{socket=new Constructor(options.endpoint);}catch{finish(false);opening=undefined;return;}
   socket.onmessage=event=>{
    let value:{type?:string;protocol?:string;id?:string;status?:number;headers?:Record<string,string>;body?:string;notDispatched?:boolean};try{value=JSON.parse(String(event.data));}catch{return;}
    if(value.type==='ready'&&value.protocol==='mailwoman-actions-v1'){ready=true;options.onConnection?.(true);finish(true);return;}
    if(typeof value.id!=='string')return;const entry=pending.get(value.id);if(!entry)return;
    pending.delete(value.id);entry.stop();
    if(typeof value.status!=='number'||value.status<200||value.status>599||typeof value.body!=='string'){entry.reject(new ApiError(502,'Invalid action acknowledgement',{code:'outcomeUnknown'}));drain();return;}
    if(value.status===403&&value.notDispatched===true){entry.resolve(undefined);close();return;}
    entry.resolve(new Response([204,205,304].includes(value.status)?null:value.body,{status:value.status,...(value.headers?{headers:value.headers}:{})}));drain();
   };
   socket.onerror=()=>{if(!retired)options.onConnection?.(false);finish(false);close();};socket.onclose=()=>{if(!retired)options.onConnection?.(false);finish(false);close();};
  });
  const attempt=opening;return attempt.then(value=>{if(!value&&opening===attempt)opening=undefined;return value;});
 }
 async function send(path:string,init:RequestInit):Promise<Response|undefined>{
  if(retired||pending.size>=8||!await connect())return undefined;
  if(init.signal?.aborted)throw init.signal.reason;
  const held=socket;if(retired||!ready||!held||pending.size>=8)return undefined;
  const id=String(++sequence),headers=Object.fromEntries(new Headers(init.headers).entries());
  const frame=JSON.stringify({id,path,method:(init.method??'GET').toUpperCase(),headers,...(typeof init.body==='string'?{body:init.body}:{})});
  if(new TextEncoder().encode(frame).length>2*1024*1024)return undefined;
  return new Promise<Response|undefined>((resolve,reject)=>{
   const abort=()=>{pending.delete(id);stop();reject(new ApiError(503,'The action may have completed',{code:'outcomeUnknown'}));drain();};
   const timer=setTimeout(abort,options.timeoutMs??120000);
   const stop=()=>{clearTimeout(timer);init.signal?.removeEventListener('abort',abort);};
   pending.set(id,{resolve,reject,stop});init.signal?.addEventListener('abort',abort,{once:true});
   // Once send is attempted, never fall back: a lost acknowledgement does not
   // prove that the mutation did not run. Existing idempotency keys survive.
   try{held.send(frame);}catch{pending.delete(id);stop();reject(new ApiError(503,'The action may have completed. Check the mailbox or Outbox before retrying.',{code:'outcomeUnknown'}));drain();}
  });
 }
 return {send,close,retire};
}
let client:ReturnType<typeof createWebSocketActions>|undefined,endpoint='',proof='';
const retiring=new Set<ReturnType<typeof createWebSocketActions>>();
export function clearWebSocketActions():void{actionConnectionStability.reset();client?.close();for(const old of retiring)old.close();retiring.clear();client=undefined;endpoint='';proof='';}
export async function websocketAction(input:string,init:RequestInit|undefined,base:string,enabled=true):Promise<Response|undefined>{
 if(!enabled||init?.keepalive||init?.credentials==='omit'||init?.body!=null&&typeof init.body!=='string')return undefined;
 const root=new URL(base,globalThis.location?.href),target=new URL(input,globalThis.location?.href);
 if(root.origin!==globalThis.location?.origin||target.origin!==root.origin)return undefined;
 const prefix=root.pathname.replace(/\/$/,''),path=target.pathname.slice(prefix.length)+target.search;
 if(path.includes('%')||path.includes('..')||!target.pathname.startsWith(`${prefix}/`)||!(path==='/jmap/api'||path==='/api/version'||path.startsWith('/api/mail/')||path==='/api/settings'||path.startsWith('/api/settings/')||path.startsWith('/api/account/')||['/api/tasks/','/api/notes/','/api/contacts/','/api/calendar/','/api/events/'].some(prefix=>path.startsWith(prefix))))return undefined;
 const method=(init?.method??'GET').toUpperCase();
 if(!['GET','POST','PUT','PATCH','DELETE'].includes(method))return undefined;
 // Authentication, security proofs, credentials, files and redirects stay on HTTP.
 if(/\/(?:login|logout|password|passkey|security|session|token|push|export|import|upload|download|avatar)(?:\/|$)/.test(path))return undefined;
 const headers=new Headers(init?.headers);
 if([...headers.keys()].some(name=>!['content-type','x-csrf-token','x-mailwoman-link','x-mailwoman-account','x-mailwoman-storage-authority','x-mailwoman-client','idempotency-key','if-match','if-none-match'].includes(name)))return undefined;
 if(init?.body&&!headers.get('content-type')?.startsWith('application/json'))return undefined;
 const url=new URL(`${prefix}/jmap/actions`,root.origin);url.protocol=root.protocol==='https:'?'wss:':'ws:';
 const cookieProof=typeof document==='undefined'?null:document.cookie.split(';').map(part=>part.trim()).find(part=>part.startsWith('mw_csrf='))?.slice('mw_csrf='.length);
 const nextProof=headers.get('x-csrf-token')??cookieProof??'';
 if(endpoint!==url.href||proof!==nextProof){
  if(client&&endpoint===url.href){const old=client;retiring.add(old);old.retire(()=>retiring.delete(old));}
  else clearWebSocketActions();
  endpoint=url.href;proof=nextProof;client=createWebSocketActions({endpoint,onConnection:connected=>actionConnectionStability.observe(connected)});
 }
 return client!.send(path,init??{});
}
