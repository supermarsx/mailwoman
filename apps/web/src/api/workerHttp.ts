import {websocketAction} from './websocketActions.ts';
import {ApiError,NetworkError} from './errors.ts';
import {consolePath} from './adminPath.ts';
import {basePath} from './basePath.ts';
/** Browser-managed cookie completion only: never the application action, no
 * generic recovery/CSRF refresh, and no response-supplied target URL. */
export function sendActionCookieDelivery(input:string,init:RequestInit):Promise<Response>{
 const target=new URL(input,globalThis.location?.href),expected=new URL(`${basePath()}${consolePath()}/actions/cookies`,globalThis.location?.href);
 if(target.href!==expected.href||target.origin!==globalThis.location?.origin||init.method!=='POST')throw new ApiError(400,'Invalid cookie delivery control');
 return fetch(target.href,{...init,credentials:'same-origin',redirect:'error',cache:'no-store'});
}
/** Worker transport cannot read document.cookie. The caller acquires its CSRF
 * token from live /api/me, and supplies verified account-binding headers. Every
 * URL remains inside this Mailwoman origin and deployment prefix. */
export async function workerHttp(base:string,path:string,init:RequestInit&{allowWebSocket?:boolean;assertAuthority?:()=>void}={}):Promise<Response>{
 const root=new URL(base,globalThis.location?.href),target=new URL(path,root.origin);
 const prefix=root.pathname.replace(/\/$/,'');
 if(root.origin!==globalThis.location?.origin||target.origin!==root.origin||target.username||target.password||!target.pathname.startsWith(`${prefix}/api/`)&&!target.pathname.startsWith(`${prefix}/jmap/`))throw new ApiError(400,'Worker request outside Mailwoman');
 let response:Response;
 try{
  const {allowWebSocket,assertAuthority,...request}=init;
  let action:Response|undefined;
  if(allowWebSocket===true){
   assertAuthority?.();
   action=await websocketAction(target.href,request,root.href,true,assertAuthority);
  }
  if(action!==undefined)response=action;
  else{
   // A known-unsent handshake/transport fallback can await for several seconds.
   // Its caller's account generation must still be current at HTTP dispatch.
   assertAuthority?.();
   response=await fetch(target.href,{...request,credentials:'same-origin',redirect:'error',cache:'no-store'});
  }
 }catch(error){if(error instanceof ApiError)throw error;throw new NetworkError('Worker connection unavailable');}
 // A 304 answers a conditional request the caller made (t52w: the worker's
 // session revalidation); without If-None-Match it stays a refusal.
 if(!response.ok&&!(response.status===304&&new Headers(init.headers).has('if-none-match'))){const body=await response.clone().json().catch(()=>({})) as {error?:string;code?:string};throw new ApiError(response.status,body.error??'Worker request refused',{...(body.code?{code:body.code}:{}),...(/^\d+$/.test(response.headers.get('retry-after')??'')?{retryAfterSecs:Number(response.headers.get('retry-after'))}:{})});}
 return response;
}
