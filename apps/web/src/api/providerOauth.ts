import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {primaryMailboxAuthority} from './linkedMailboxSession.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
export type ProviderBridge='bridge-graph'|'bridge-gmail';
export interface ProviderStatus {bridgeId?:ProviderBridge;configured:boolean;connected:boolean;clientId?:string;tenant?:string;flows:('authorizationCode'|'deviceCode')[]}
export interface ProviderFlow {flowId:string;authorizationUrl?:string;redirectUri?:string;userCode?:string;verificationUri?:string;expiresIn:number;interval?:number}
export class ProviderOauthError extends Error {constructor(readonly code:string){super(code);}}
export async function providerRequest<T>(path='',method='GET',body?:unknown,signal?:AbortSignal):Promise<T>{
 const owner=preferenceAccount();if(!owner)throw new ProviderOauthError('signedOut');
 const response=await apiFetch(withBase(`/api/provider/oauth${path}`),{method,credentials:'same-origin',redirect:'error',...(signal?{signal}:{}),headers:{'content-type':'application/json','x-mailwoman-storage-authority':primaryMailboxAuthority()??owner.storageAuthority??''},...(body===undefined?{}:{body:JSON.stringify(body)})});
 const assertCurrent=()=>{if(signal?.aborted||!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');};
 assertCurrent();
 let value:unknown;
 try{value=response.status===204?undefined:await response.json();}catch(cause){assertCurrent();if(response.ok)throw cause;}
 // Fetch resolves before its body. A mailbox switch during JSON parsing must
 // fence both a successful result and an error just like a delayed response.
 assertCurrent();
 if(!response.ok){const code=value&&typeof value==='object'&&'code' in value&&typeof value.code==='string'?value.code:'failed';throw new ProviderOauthError(code);}
 return value as T;
}
const FLOW_KEY='mw.provider-oauth.pending';
export function rememberProviderFlow(flow:ProviderFlow):void{
 if(!preferenceAccount())throw new ProviderOauthError('signedOut');
 const authority=primaryMailboxAuthority()??preferenceAccount()?.storageAuthority;
 if(!authority)throw new ProviderOauthError('signedOut');
 if(!flow.flowId||flow.flowId.length>256||!Number.isFinite(flow.expiresIn)||flow.expiresIn<=0)throw new ProviderOauthError('invalidFlow');
 sessionStorage.setItem(FLOW_KEY,JSON.stringify({id:flow.flowId,authority,expires:Date.now()+Math.min(flow.expiresIn,1800)*1000}));
}
export function pendingProviderFlow():string|undefined{
 if(!preferenceAccount())return undefined;
 try{const value=JSON.parse(sessionStorage.getItem(FLOW_KEY)??'null') as {id?:unknown;authority?:unknown;expires?:unknown}|null;
 const authority=primaryMailboxAuthority()??preferenceAccount()?.storageAuthority;
 return value&&typeof value.id==='string'&&value.id.length<=256&&value.authority===authority&&typeof value.expires==='number'&&value.expires>Date.now()?value.id:undefined;
 }catch{return undefined;}
}
export function forgetProviderFlow():void{try{sessionStorage.removeItem(FLOW_KEY);}catch{/* Correlation is unavailable when browser storage is disabled. */}}
export function providerHttpsUrl(value:string|undefined):string|undefined{try{const url=new URL(value??'');return url.protocol==='https:'&&!url.username&&!url.password?url.href:undefined;}catch{return undefined;}}
