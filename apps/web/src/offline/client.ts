import {ApiError,NetworkError,type Client,type Me,type JmapCallOptions} from '../api/client.ts';
import type {JmapRequest,JmapSession} from '../api/jmap-types.ts';
import {activeLinkedMailbox,linkedMailboxEpoch} from '../api/linkedMailboxSession.ts';
import {offlineVault,type OfflineVault} from './vault.ts';
import {readJmapOffline,readOnlyJmap,rememberJmap} from './jmapCache.ts';
import {configureOfflinePolicy,forgetOfflineBootstrap,isOffline,observeOffline,observeOfflinePolicy,offlineNamespace,offlinePolicy,registerOfflineVerifier,registerOfflineRevoker,readOfflineBootstrap,reportOffline,saveOfflineBootstrap} from './runtime.ts';

const unavailable=(error:unknown):boolean=>error instanceof NetworkError||error instanceof TypeError||(error instanceof ApiError&&[502,503,504].includes(error.status));
/** The UI keeps local view state. Authentication and cached account data remain
 * separate: a local snapshot never authorizes replay until /me is verified. */
export function withOfflineClient(client:Client,base:()=>string,storage:()=>Promise<OfflineVault>=offlineVault):Client {
 const identities=new Map<string,Me>();const sessions=new Map<string,JmapSession>();let currentBase=base(),verifying:Promise<Me>|undefined;
 const listeners=new Set<(up:boolean)=>void>();
 const emit=(up:boolean)=>{for(const listener of listeners)listener(up);};
 const context=()=>activeLinkedMailbox()?.linkId??'';
 const syncBase=()=>{if(base()!==currentBase){currentBase=base();identities.clear();sessions.clear();verifying=undefined;reportOffline(false);}};
 const capture=()=>{const origin=base(),epoch=linkedMailboxEpoch();return()=>{if(origin!==base()||epoch!==linkedMailboxEpoch())throw new DOMException('Offline request ownership changed','AbortError');};};
 const bootKey=(link:string)=>`context:${link}`;
 const bootstrapNamespace=()=>`offline-bootstrap:${base()}`;
 observeOffline(value=>emit(!value));
 client.onNetwork(up=>{if(!up)reportOffline(true);else if(!isOffline())emit(true);});
 observeOfflinePolicy(()=>{syncBase();const primary=identities.get('');if(primary)void (offlinePolicy(primary).enabled?saveOfflineBootstrap(base(),primary,storage):forgetOfflineBootstrap(base(),storage)).catch(()=>undefined);if(offlinePolicy().enabled)for(const [link,me] of identities){const session=sessions.get(link);if(session&&offlinePolicy(me).enabled)void storage().then(vault=>vault.put(offlineNamespace(base(),me),'session',session)).catch(()=>undefined);}});
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
 registerOfflineRevoker(()=>forgetOfflineBootstrap(base(),storage).catch(()=>undefined));
 return {
  ...client,
  onNetwork(listener){listeners.add(listener);return()=>listeners.delete(listener);},
  async login(input){syncBase();const me=await client.login(input);reportOffline(false);identities.clear();await remember(me,'').catch(()=>undefined);return me;},
  async logout(){try{await client.logout();}finally{identities.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);reportOffline(false);}},
  async me(options){
   syncBase();const admitted=capture(),link=options?(options.linkId??''):context();
   try{if(typeof navigator!=='undefined'&&navigator.onLine===false)throw new NetworkError('Working offline');const me=isOffline()?await verify(link):await client.me(options);admitted();const previous=identities.get(link);if(previous&&(previous.accountId!==me.accountId||previous.storageAuthority!==me.storageAuthority))throw new ApiError(401,'The signed-in account changed');await remember(me,link).catch(()=>undefined);reportOffline(false);return me;}
   catch(error){
    if(error instanceof ApiError&&[401,403].includes(error.status)){identities.clear();await forgetOfflineBootstrap(base(),storage).catch(()=>undefined);throw error;}
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
    if(!unavailable(error)||!owner?.storageAuthority||!offlinePolicy(owner).enabled)throw error;
    admitted();const cached=await (await storage()).get<JmapSession>(offlineNamespace(base(),owner),'session');
    if(!cached)throw error;reportOffline(true);return cached;
   }
  },
  async jmap(request:JmapRequest,options?:JmapCallOptions){
   syncBase();const admitted=capture(),owner=identities.get(context());
   try{
    await ensureLive();admitted();const response=await client.jmap(request,options);
    admitted();if(owner?.storageAuthority&&offlinePolicy(owner).enabled)await rememberJmap(await storage(),offlineNamespace(base(),owner),request,response).catch(()=>undefined);
    return response;
   }catch(error){
    if(options?.signal?.aborted||!unavailable(error)||!owner?.storageAuthority||!offlinePolicy(owner).enabled||!readOnlyJmap(request))throw error;
    admitted();const cached=await readJmapOffline(await storage(),offlineNamespace(base(),owner),request);
    if(!cached)throw error;reportOffline(true);return cached;
   }
  },
 };
}
