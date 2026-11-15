import {importReplay} from './replayImport.ts';
import {queuedPimRequest,pendingResponse,resolvePendingIds} from './pendingObjects.ts';
import {encryptedOutboxStore} from './encryptedOutbox.ts';
import {accountOutbox} from './accountOutbox.ts';
import {idbAvailable,idbOutboxStore} from './idb.ts';
import {enqueueOutbound,type QueuedItem,type OutboxStore} from './outboxCore.ts';
import {basePath} from '../api/basePath.ts';
import {requestBackgroundReplay} from '../sw/register.ts';
import {publishOfflineChange,observeOfflineChanges} from './events.ts';
import {reportAccountHealth,clearAccountHealth} from '../realtime/accountHealth.ts';
import {prepareLocalFiles} from './localFiles.ts';
import {ApiError,NetworkError,type Client,type Me,type JmapCallOptions} from '../api/client.ts';
import type {JmapRequest,JmapSession} from '../api/jmap-types.ts';
import {activeLinkedMailbox,linkedMailboxEpoch} from '../api/linkedMailboxSession.ts';
import {offlineVault,type OfflineVault} from './vault.ts';
import {readJmapOffline,readOnlyJmap,rememberJmap} from './jmapCache.ts';
import {configureOfflinePolicy,forgetOfflineBootstrap,isOffline,observeOffline,observeOfflinePolicy,offlineNamespace,offlinePolicy,registerOfflineVerifier,registerOfflineRevoker,readOfflineBootstrap,reportOffline,saveOfflineBootstrap} from './runtime.ts';

const revoked=(error:unknown):boolean=>error instanceof ApiError&&([401,403].includes(error.status)||error.status===409&&error.code==='accountChanged');
const unavailable=(error:unknown):boolean=>error instanceof NetworkError||error instanceof TypeError||(error instanceof ApiError&&[502,503,504].includes(error.status));
interface OfflineProfile {generation:number;identities:Map<string,Me>;sessions:Map<string,JmapSession>}
const profiles=new Map<string,OfflineProfile>();
function profileFor(base:string,storage:()=>Promise<OfflineVault>):OfflineProfile {
 const existing=profiles.get(base);if(existing)return existing;
 const profile:OfflineProfile={generation:0,identities:new Map(),sessions:new Map()};profiles.set(base,profile);
 observeOfflinePolicy(()=>{
  const primary=profile.identities.get('');
  if(primary)void (offlinePolicy(primary).enabled?saveOfflineBootstrap(base,primary,storage):forgetOfflineBootstrap(base,storage)).catch(()=>undefined);
  for(const [link,me] of profile.identities){
   const session=profile.sessions.get(link);
   if(session&&offlinePolicy(me).enabled)void storage().then(vault=>vault.put(offlineNamespace(base,me),'session',session)).catch(()=>undefined);
  }
 });
 return profile;
}
observeOfflineChanges(kind=>{if(kind==='logout')for(const profile of profiles.values()){profile.generation++;profile.identities.clear();profile.sessions.clear();}});
/** The UI keeps local view state. Authentication and cached account data remain
 * separate: a local snapshot never authorizes replay until /me is verified. */
export function withOfflineClient(client:Client,base:()=>string,storage:()=>Promise<OfflineVault>=offlineVault,queue?:()=>OutboxStore):Client {
 const outbox=queue??(()=>encryptedOutboxStore(idbOutboxStore()));
 let profile=profileFor(base(),storage);let {identities,sessions}=profile;let currentBase=base(),verifying:Promise<Me>|undefined;
 const listeners=new Set<(up:boolean)=>void>();
 const emit=(up:boolean)=>{for(const listener of listeners)listener(up);};
 const context=()=>activeLinkedMailbox()?.linkId??'';
 const syncBase=()=>{if(base()!==currentBase){currentBase=base();profile=profileFor(currentBase,storage);({identities,sessions}=profile);verifying=undefined;reportOffline(false);}};
 const capture=()=>{const origin=base(),epoch=linkedMailboxEpoch(),held=profile,generation=profile.generation;return()=>{if(origin!==base()||epoch!==linkedMailboxEpoch()||held.generation!==generation)throw new DOMException('Offline request ownership changed','AbortError');};};
 const bootKey=(link:string)=>`context:${link}`;
 const bootstrapNamespace=()=>`offline-bootstrap:${base()}`;
 observeOffline(value=>emit(!value));
 client.onNetwork(up=>{if(!up)reportOffline(true);else if(!isOffline())emit(true);});

 async function remember(me:Me,link:string):Promise<void>{
  identities.set(link,me);
  if(!offlinePolicy(me).enabled||!me.storageAuthority)return;
  const vault=await storage();await vault.put(bootstrapNamespace(),bootKey(link),me);
  if(!link)await saveOfflineBootstrap(base(),me,storage);
 }
 async function verify(link:string):Promise<Me>{
  const expected=identities.get(link);
  const live=await client.me(link?{linkId:link}:undefined);
  if(expected&&(expected.accountId!==live.accountId||expected.storageAuthority!==live.storageAuthority)){
   identities.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);
   throw new ApiError(401,'The signed-in account changed; sign in again before replaying offline work');
  }
  await remember(live,link).catch(()=>undefined);
  reportOffline(false);return live;
 }
 async function ensureLive():Promise<void>{
  if(!isOffline())return;
  if(typeof navigator!=='undefined'&&navigator.onLine===false)throw new NetworkError('Working offline');
  const link=context();verifying??=verify(link).finally(()=>{verifying=undefined;});await verifying;
 }
 registerOfflineVerifier(ensureLive);
 registerOfflineRevoker(async()=>{identities.clear();sessions.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');});
 return {
  ...client,offlineReads:true,
  onNetwork(listener){listeners.add(listener);return()=>listeners.delete(listener);},
  async login(input){syncBase();profile.generation++;const me=await client.login(input);identities.clear();sessions.clear();clearAccountHealth();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');await remember(me,'').catch(()=>undefined);reportOffline(false);return me;},
  async logout(){profile.generation++;try{await client.logout();}finally{identities.clear();clearAccountHealth();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');reportOffline(false);}},
  async me(options){
   syncBase();const admitted=capture(),link=options?(options.linkId??''):context();
   try{if(typeof navigator!=='undefined'&&navigator.onLine===false)throw new NetworkError('Working offline');const me=isOffline()?await verify(link):await client.me(options);admitted();const previous=identities.get(link);if(previous&&(previous.accountId!==me.accountId||previous.storageAuthority!==me.storageAuthority))throw new ApiError(401,'The signed-in account changed');await remember(me,link).catch(()=>undefined);reportOffline(false);return me;}
   catch(error){
    if(revoked(error)){identities.clear();sessions.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');throw error;}
    if(!unavailable(error))throw error;
    admitted();const bootstrap=await readOfflineBootstrap(base(),storage).catch(()=>undefined);
    if(!bootstrap)throw error;
    configureOfflinePolicy(bootstrap.policy,bootstrap.me);
    const cached=link?await (await storage()).get<Me>(bootstrapNamespace(),bootKey(link)):bootstrap.me;
    if(!cached?.storageAuthority)throw error;
    identities.set(link,cached);reportOffline(true);return cached;
   }
  },
  async session(options){
   syncBase();const admitted=capture(),link=options?.linkId??context(),owner=identities.get(link);
   try{
    await ensureLive();admitted();const session=await client.session(options);admitted();sessions.set(link,session);
    if(owner?.storageAuthority&&offlinePolicy(owner).enabled)await (await storage()).put(offlineNamespace(base(),owner),'session',session).catch(()=>false);
    admitted();return session;
   }catch(error){
    if(revoked(error)){identities.clear();sessions.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');}
    if(!unavailable(error)||!owner?.storageAuthority||!offlinePolicy(owner).enabled)throw error;
    admitted();if(!await readOfflineBootstrap(base(),storage).catch(()=>undefined))throw error;const cached=await (await storage()).get<JmapSession>(offlineNamespace(base(),owner),'session');
    if(!cached)throw error;reportOffline(true);return cached;
   }
  },
  async jmap(request:JmapRequest,options?:JmapCallOptions){
   syncBase();const admitted=capture(),owner=identities.get(context());
   if(owner?.storageAuthority&&offlinePolicy(owner).enabled&&idbAvailable())await importReplay(await storage(),offlineNamespace(base(),owner)).catch(()=>undefined);
   if(!options?.offlineReplay&&owner?.storageAuthority&&queuedPimRequest(request)&&(isOffline()||typeof navigator!=='undefined'&&navigator.onLine===false)){
    const store=accountOutbox(outbox(),owner.accountId,owner.storageAuthority);
    const item=await enqueueOutbound(store,{type:'pim',payload:{request,callId:request.methodCalls[0]![2],callIds:request.methodCalls.map(call=>call[2])},context:{linkId:context()||null,serverBase:basePath(),cacheBase:base(),...(typeof options?.idempotencyKey==='string'?{idempotencyKey:options.idempotencyKey}:{})}});
    admitted();publishOfflineChange('queue');void requestBackgroundReplay();return pendingResponse(item as QueuedItem);
   }
   try{
    await ensureLive();admitted();
    let outgoing=owner?.storageAuthority?await resolvePendingIds({cacheBase:base(),storageAuthority:owner.storageAuthority} as QueuedItem,request):request;
    if(owner?.storageAuthority&&JSON.stringify(request).includes('mw-local:')){
     const {uploadBlob}=await import('../api/jmap.ts'),session=await client.session();admitted();
     outgoing=await prepareLocalFiles(request,{accountId:owner.accountId,storageAuthority:owner.storageAuthority,base:base()},file=>uploadBlob(session.uploadUrl,owner.accountId,file));
    }
    admitted();const response=await client.jmap(outgoing,options);
    if(owner){const errors=response.methodResponses.filter(row=>row[0]==='error');reportAccountHealth(owner,context()||null,errors.some(row=>['serverFail','serverUnavailable','accountNotFound','accountReadOnly','rateLimit'].includes(String(row[1]['type'])))?'degraded':'online');}
    admitted();if(owner?.storageAuthority&&offlinePolicy(owner).enabled)await rememberJmap(await storage(),offlineNamespace(base(),owner),outgoing,response).catch(()=>undefined);
    return response;
   }catch(error){
    if(revoked(error)){identities.clear();sessions.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);publishOfflineChange('logout');}
    if(owner&&!options?.signal?.aborted)reportAccountHealth(owner,context()||null,error instanceof ApiError&&[401,403].includes(error.status)?'auth-expired':error instanceof ApiError&&[429,502,503,504].includes(error.status)?'degraded':'offline',error instanceof ApiError?error.retryAfterSecs:undefined);
    if(options?.offlineReplay||options?.signal?.aborted||!unavailable(error)||!owner?.storageAuthority||!offlinePolicy(owner).enabled||!readOnlyJmap(request))throw error;
    admitted();const bootstrap=await readOfflineBootstrap(base(),storage).catch(()=>undefined);const primary=identities.get('');if(!bootstrap||primary&&bootstrap.me.storageAuthority!==primary.storageAuthority)throw error;const pending=idbAvailable()||queue?await accountOutbox(outbox(),owner.accountId,owner.storageAuthority).all().catch(()=>[]):[];const mappingOwner={cacheBase:base(),storageAuthority:owner.storageAuthority} as QueuedItem;
    const localRequest=idbAvailable()?await resolvePendingIds(mappingOwner,request,false):request;
    const projected=await Promise.all((pending as QueuedItem[]).map(async item=>item.type==='pim'&&idbAvailable()?{...item,payload:{...(item.payload as {request:JmapRequest}),request:await resolvePendingIds(item,(item.payload as {request:JmapRequest}).request,false)}}:item));
    const cached=await readJmapOffline(await storage(),offlineNamespace(base(),owner),localRequest,owner.username,projected);
    if(!cached)throw error;reportOffline(true);return cached;
   }
  },
 };
}
