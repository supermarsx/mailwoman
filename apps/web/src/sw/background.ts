import {prepareLocalFiles} from '../offline/localFiles.ts';
import type {Client} from '../api/client.ts';
import {ApiError,NetworkError} from '../api/errors.ts';
import {encryptedOutboxStore} from '../offline/encryptedOutbox.ts';
import {idbOutboxStore} from '../offline/idb.ts';
import {accountOutbox,outboundAccount} from '../offline/accountOutbox.ts';
import {drainOutbox,type QueuedItem} from '../offline/outboxCore.ts';

import {jmapApiPath} from '../api/jmapDiscovery.ts';
import type {JmapResponse,JmapSession} from '../api/jmap-types.ts';

interface SyncEvent {tag:string;waitUntil(work:Promise<unknown>):void}
interface WorkerScope {registration:{scope:string};addEventListener(name:'sync',callback:(event:SyncEvent)=>void):void;clients:{matchAll():Promise<{postMessage(value:unknown):void}[]>}}
const scope=globalThis as unknown as WorkerScope;
const TAG='mw-outbox';

/** Same encrypted queue and drain lock as the tabs. Authorization is acquired
 * afresh from the server; a cached identity never authorizes background work. */
export async function replayBackground():Promise<void>{
 const root=new URL(scope.registration.scope),base=root.pathname.replace(/\/$/,'');
 const store=encryptedOutboxStore(idbOutboxStore());
 const pending=(await store.all()) as QueuedItem[];
 const contexts=new Map<string,QueuedItem>();
 for(const row of pending)if(row.state==='queued'&&row.storageAuthority&&row.serverBase===base&&row.linkId!==undefined)contexts.set(JSON.stringify([row.storageAuthority,outboundAccount(row),row.linkId]),row);
 for(const row of contexts.values()){
  const headers=new Headers();if(row.linkId)headers.set('X-Mailwoman-Link',row.linkId);
  let csrf='';
  const send=async(path:string,init:RequestInit={}):Promise<Response>=>{
   const target=new URL(path,root);
   if(target.origin!==root.origin||!target.pathname.startsWith(`${base}/api/`)&&!target.pathname.startsWith(`${base}/jmap/`))throw new Error('Background request outside Mailwoman');
   const given=new Headers(init.headers);for(const [key,value] of headers)given.set(key,value);
   if(csrf)given.set('X-CSRF-Token',csrf);
   let response:Response;
   try{response=await fetch(target.href,{...init,headers:given,credentials:'same-origin',redirect:'error',cache:'no-store'});}catch{throw new NetworkError('Background connection unavailable');}
   if(response.status===401||response.status===403)throw new ApiError(response.status,'Background authorization expired');
   if(response.status>=500||response.status===429)throw new NetworkError('Background service unavailable');
   if(!response.ok)throw new ApiError(response.status,'Background request refused');
   return response;
  };
  const live=await (await send(`${base}/api/me`)).json() as {accountId?:string;storageAuthority?:string;csrfToken?:string;passwordChangeRequired?:boolean};
  if(live.accountId!==outboundAccount(row)||live.storageAuthority!==row.storageAuthority||live.passwordChangeRequired||!live.csrfToken)continue;
  csrf=live.csrfToken;
  headers.set('X-Mailwoman-Account',live.accountId!);headers.set('X-Mailwoman-Storage-Authority',live.storageAuthority!);
  const session=await (await send(`${base}/jmap/session`)).json() as JmapSession;
  const api=`${base}${jmapApiPath(session.apiUrl,root.href)}`;
  const client={jmap:async(request,options)=>{
   const given=new Headers({'content-type':'application/json'});
   if(options?.idempotencyKey)given.set('Idempotency-Key',options.idempotencyKey);
   const outgoing=await prepareLocalFiles(request,{accountId:live.accountId!,storageAuthority:live.storageAuthority!,base:root.origin+base},async file=>{const target=session.uploadUrl.replace('{accountId}',encodeURIComponent(live.accountId!));return await (await send(target,{method:'POST',headers:{'content-type':file.type},body:file})).json() as {blobId:string;size:number;type:string};});
   return await (await send(api,{method:'POST',headers:given,body:JSON.stringify(outgoing),...(options?.signal?{signal:options.signal}:{})})).json() as JmapResponse;
  }} as Client;
  const owned=accountOutbox(store,live.accountId!,live.storageAuthority!);
  const selected={...owned,all:async()=>(await owned.all()).filter(item=>(item as QueuedItem).serverBase===base&&(item as QueuedItem).linkId===row.linkId)};
  await drainOutbox(selected,client,{
   mint:async(subject,to)=>{const result=await (await send(`${base}/api/track/open`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({subject,to})})).json() as {token:string;url:string};if(typeof result.token!=='string'||typeof result.url!=='string'||!/^https?:\/\//i.test(result.url))throw new Error('Invalid tracking response');return result;},
   bind:async(token,emailId)=>(await send(`${base}/api/track/open/${encodeURIComponent(token)}`,{method:'PUT',headers:{'content-type':'application/json'},body:JSON.stringify({emailId})})).ok,
  });
  if((await selected.all()).some(item=>item.state==='queued'))throw new NetworkError('Background replay is incomplete');
 }
 for(const client of await scope.clients.matchAll())client.postMessage({type:'mw-outbox-updated'});
}
scope.addEventListener('sync',event=>{if(event.tag===TAG)event.waitUntil(replayBackground());});
