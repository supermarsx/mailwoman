import type {PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {offlineNamespace,offlinePolicy,offlineLimits,reportOffline} from './runtime.ts';
import {offlineServerBase} from './cacheScope.ts';
import {offlineVault} from './vault.ts';
import {RecoveryError} from '../api/recovery.ts';

interface SavedBlob {data:string;type:string;disposition:string|null}
function encode(bytes:Uint8Array):string{let value='';for(let offset=0;offset<bytes.length;offset+=8192)value+=String.fromCharCode(...bytes.subarray(offset,offset+8192));return btoa(value);}
function decode(value:string):Uint8Array{return Uint8Array.from(atob(value),char=>char.charCodeAt(0));}
async function readBounded(response:Response):Promise<Uint8Array|undefined>{
 const reader=response.body?.getReader();if(!reader)return new Uint8Array();
 const chunks:Uint8Array[]=[];let size=0;
 try{for(;;){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>10*1024*1024){void reader.cancel();return;}chunks.push(value);}const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}return bytes;}finally{reader.releaseLock();}
}
/** Plaintext Cache Storage never receives message or attachment bytes. Only a
 * successful policy-checked download may populate this encrypted cache. */
export async function offlineBlobRead(input:string,path:string,init:RequestInit|undefined,owner:PreferenceAccount|null,send:()=>Promise<Response>):Promise<Response>{
 if(!owner?.storageAuthority||!offlinePolicy(owner).enabled||!path.startsWith('/jmap/download/')||(init?.method??'GET')!=='GET')return send();
 const root=offlineNamespace(offlineServerBase(),{accountId:owner.id,storageAuthority:owner.storageAuthority}),namespace=`${root}:blobs`;
 const target=new URL(input,location.href);target.searchParams.sort();const key=target.href;
 try{
  const response=await send();
  if(![502,503,504].includes(response.status)){
   const size=Number(response.headers.get('content-length'));
   if(response.status===200&&(!size||size<=10*1024*1024)){
    try{const bytes=await readBounded(response.clone());
    if(bytes&&preferenceAccountIsCurrent(owner))await (await offlineVault()).put(namespace,key,{data:encode(bytes),type:response.headers.get('content-type')??'application/octet-stream',disposition:response.headers.get('content-disposition')} satisfies SavedBlob,offlineLimits({budgetGroup:root}));}catch{/* Cache failure never breaks a successful download. */}
   }
   return response;
  }
  throw new TypeError('Download service unavailable');
 }catch(error){
  if(init?.signal?.aborted||!preferenceAccountIsCurrent(owner))throw error;
  const unavailable=error instanceof TypeError||error instanceof Error&&error.name==='NetworkError'||error instanceof RecoveryError&&['network','offline','server'].includes(error.reason);
  if(!unavailable)throw error;
  const saved=await (await offlineVault()).get<SavedBlob>(namespace,key).catch(()=>undefined);if(!saved)throw error;
  if(!preferenceAccountIsCurrent(owner))throw new DOMException('Mailbox changed','AbortError');
  const bytes=decode(saved.data),headers=new Headers({'content-type':saved.type,'x-content-type-options':'nosniff','content-security-policy':"default-src 'none'; sandbox",'x-mailwoman-offline':'1','accept-ranges':'bytes'});
  if(saved.disposition)headers.set('content-disposition',saved.disposition);
  reportOffline(true);
  const range=new Headers(init?.headers).get('range');
  if(range){
   const match=/^bytes=(\d*)-(\d*)$/.exec(range);
   const start=match?(match[1]?Number(match[1]):Math.max(0,bytes.length-Number(match[2]))):NaN;
   const end=match&&match[1]&&match[2]?Math.min(bytes.length-1,Number(match[2])):bytes.length-1;
   if(!Number.isSafeInteger(start)||start<0||start>end||start>=bytes.length)return new Response(null,{status:416,headers:{'content-range':`bytes */${bytes.length}`}});
   const part=bytes.slice(start,end+1);headers.set('content-range',`bytes ${start}-${end}/${bytes.length}`);headers.set('content-length',String(part.length));return new Response(part.buffer,{status:206,headers});
  }
  headers.set('content-length',String(bytes.length));return new Response(bytes.slice().buffer,{headers});
 }
}
