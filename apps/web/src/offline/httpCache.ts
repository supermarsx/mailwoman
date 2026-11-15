import {RecoveryError} from '../api/recovery.ts';
import type {PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {offlineVault} from './vault.ts';
import {offlineLimits,offlinePolicy,reportOffline} from './runtime.ts';

const paths=new Set(['/api/settings','/api/account/preferences','/api/client-policy','/api/accounts','/api/features']);
interface SavedResponse {status:number;contentType:string;body:string}
const namespace=(owner:PreferenceAccount)=>JSON.stringify(['offline-http',owner.storageAuthority,owner.id]);
export async function seedOfflineHttp(owner:PreferenceAccount,path:string,body:unknown):Promise<void>{
 if(!offlinePolicy().enabled||!owner.storageAuthority||!paths.has(path))return;
 await (await offlineVault()).put(namespace(owner),path,{status:200,contentType:'application/json',body:JSON.stringify(body)} satisfies SavedResponse,offlineLimits());
}
/** Only explicitly listed, authenticated user GETs can use local snapshots.
 * Admin requests, writes, remote URLs and explicit refusals never fall back. */
export async function offlineHttpRead(path:string,init:RequestInit|undefined,owner:PreferenceAccount|null,send:()=>Promise<Response>):Promise<Response>{
 if(!offlinePolicy().enabled||!owner?.storageAuthority||(init?.method??'GET').toUpperCase()!=='GET'||!paths.has(path))return send();
 try{
  const response=await send();
  if(![502,503,504].includes(response.status)){
   if(response.ok&&response.headers.get('content-type')?.includes('json')){
    const body=await response.clone().text();
    await (await offlineVault()).put(namespace(owner),path,{status:response.status,contentType:response.headers.get('content-type')!,body} satisfies SavedResponse,offlineLimits()).catch(()=>false);
   }
   return response;
  }
  throw new TypeError('Server is unavailable');
 }catch(error){
  if(init?.signal?.aborted||(error instanceof DOMException&&error.name==='AbortError'))throw error;
  if(!(error instanceof Error&&error.name==='NetworkError')&&!(error instanceof TypeError)&&!(error instanceof RecoveryError&&['offline','network','server'].includes(error.reason))&&!(error instanceof DOMException&&['NetworkError','TimeoutError'].includes(error.name)))throw error;
  const cached=await (await offlineVault()).get<SavedResponse>(namespace(owner),path).catch(()=>undefined);
  if(!cached)throw error;
  reportOffline(true);
  return new Response(cached.body,{status:cached.status,headers:{'content-type':cached.contentType,'x-mailwoman-offline':'1'}});
 }
}
