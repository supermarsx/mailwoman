import type {PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {MailCachePolicy} from './mailCachePolicy.ts';
export type MailCacheKind='mailbox'|'message';
export type PrefetchTrigger='hover'|'focus'|'idle';
export interface PrefetchOptions {trigger:PrefetchTrigger;folderId?:string;folderRole?:string}
type Loader<T>=(signal:AbortSignal)=>Promise<T>;
interface Entry<T>{value:T;at:number;bytes:number;kind:MailCacheKind;key:string}
interface Job<T>{key:string;kind:MailCacheKind;owner:PreferenceAccount;loader:Loader<T>;controller:AbortController;promise:Promise<T>;resolve:(value:T)=>void;reject:(error:unknown)=>void;speculative:boolean;started:boolean;ready:boolean;timer?:ReturnType<typeof setTimeout>;options?:PrefetchOptions;epoch:number;waiters:number;publish:boolean}
const aborted=()=>new DOMException('Mail cache request cancelled','AbortError');
const sameOwner=(a:PreferenceAccount|null,b:PreferenceAccount|null)=>a===b||!!a&&!!b&&a.id===b.id&&a.generation===b.generation&&a.storageAuthority===b.storageAuthority;
/** Session-only cache. All requests use caller-supplied authenticated transport. */
export function createMailReadCache<T>(options:{policy:()=>MailCachePolicy;currentOwner:()=>PreferenceAccount|null;now?:()=>number;canPrefetch?:()=>boolean}){
 const now=options.now??Date.now;
 const entries=new Map<string,Entry<T>>(),jobs=new Map<string,Job<T>>();
 const detached=new Set<Job<T>>();
 let bytes=0,epoch=0,disposed=false,owner=options.currentOwner(),fingerprint='';
 const id=(key:string,kind:MailCacheKind)=>JSON.stringify([kind,key]);
 const enabled=(kind:MailCacheKind,p:MailCachePolicy)=>p.enabled&&(kind==='mailbox'?p.mailboxes:p.messages);
 function remove(key:string){const entry=entries.get(key);if(entry){bytes-=entry.bytes;entries.delete(key);}}
 function cancel(job:Job<T>){if(job.timer!==undefined)clearTimeout(job.timer);job.controller.abort();if(jobs.get(id(job.key,job.kind))===job)jobs.delete(id(job.key,job.kind));job.reject(aborted());}
 function clear(){epoch++;entries.clear();bytes=0;for(const job of new Set([...jobs.values(),...detached]))cancel(job);detached.clear();}
 function syncPolicy(){
  const next=options.currentOwner(),policy=options.policy(),stamp=JSON.stringify(policy);
  if(!sameOwner(owner,next)){clear();owner=next;}
  if(fingerprint!==stamp){invalidate(undefined,{abort:false});fingerprint=stamp;}
  return policy;
 }
 function admitted(captured:PreferenceAccount){return !disposed&&sameOwner(captured,options.currentOwner())&&sameOwner(captured,owner);}
 function trim(p:MailCachePolicy){for(const [key,e] of entries)if(now()-e.at>p.maxAgeSeconds*1000)remove(key);while(entries.size>p.maxEntries||bytes>p.maxBytes){const first=entries.keys().next().value;if(first===undefined)break;remove(first);}}
 function peek(captured:PreferenceAccount,key:string,kind:MailCacheKind):{value:T;fresh:boolean}|undefined{
  const p=syncPolicy();if(!admitted(captured)||!enabled(kind,p))return;
  trim(p);const cacheKey=id(key,kind),entry=entries.get(cacheKey);if(!entry)return;
  entries.delete(cacheKey);entries.set(cacheKey,entry);
  return {value:structuredClone(entry.value),fresh:now()-entry.at<=p.freshSeconds*1000};
 }
 function store(job:Job<T>,value:T){
  const p=syncPolicy();if(!job.publish||job.epoch!==epoch||!admitted(job.owner)||job.controller.signal.aborted||!enabled(job.kind,p))return;
  let copy:T,size:number;try{copy=structuredClone(value);size=new TextEncoder().encode(JSON.stringify(copy)).byteLength;}catch{return;}
  if(size>p.maxBytes)return;
  const cacheKey=id(job.key,job.kind);remove(cacheKey);entries.set(cacheKey,{value:copy,at:now(),bytes:size,kind:job.kind,key:job.key});bytes+=size;trim(p);
 }
 function speculativeAllowed(kind:MailCacheKind,o:PrefetchOptions,p:MailCachePolicy){
  if(!enabled(kind,p)||!p.prefetchEnabled||(kind==='message'&&!p.preloadMessages))return false;
  if(o.trigger==='hover'&&!p.hoverEnabled||o.trigger==='focus'&&!p.focusEnabled||o.trigger==='idle'&&!p.idleEnabled)return false;
  if(o.folderId&&p.excludedFolders.includes(`id:${o.folderId}`)||o.folderRole&&p.excludedFolders.includes(`role:${o.folderRole}`))return false;
  if(options.canPrefetch&&!options.canPrefetch())return false;
  if(typeof navigator!=='undefined'&&(navigator.onLine===false||(navigator as Navigator&{connection?:{saveData?:boolean}}).connection?.saveData))return false;
  return typeof document==='undefined'||document.visibilityState!=='hidden';
 }
 function start(job:Job<T>){
  if(job.started)return;job.started=true;
  (async()=>{if(!admitted(job.owner)||job.controller.signal.aborted||job.epoch!==epoch)throw aborted();return await job.loader(job.controller.signal);})().then(value=>{
   if(!admitted(job.owner)||job.controller.signal.aborted||job.epoch!==epoch)throw aborted();
   store(job,value);if(job.controller.signal.aborted||job.epoch!==epoch)throw aborted();job.resolve(value);
  }).catch(job.reject).finally(()=>{detached.delete(job);if(jobs.get(id(job.key,job.kind))===job)jobs.delete(id(job.key,job.kind));pump();});
 }
 function pump(){
  const p=syncPolicy();if(disposed)return;
  if([...jobs.values(),...detached].some(job=>!job.speculative))return;
  let running=[...jobs.values()].filter(job=>job.started).length;
  for(const job of [...jobs.values()]){
   if(job.started||!job.ready)continue;
   if(!job.options||!speculativeAllowed(job.kind,job.options,p)||!admitted(job.owner)){cancel(job);continue;}
   if(running>=p.maxConcurrency)break;running++;start(job);
  }
 }
 function make(captured:PreferenceAccount,key:string,kind:MailCacheKind,loader:Loader<T>,speculative:boolean):Job<T>{
  let resolve!:(value:T)=>void,reject!:(error:unknown)=>void;
  const promise=new Promise<T>((yes,no)=>{resolve=yes;reject=no;});promise.catch(()=>undefined);
  const job:Job<T>={key,kind,owner:captured,loader,controller:new AbortController(),promise,resolve,reject,speculative,started:false,ready:!speculative,epoch,waiters:0,publish:true};
  jobs.set(id(key,kind),job);return job;
 }
 function load(captured:PreferenceAccount,key:string,kind:MailCacheKind,loader:Loader<T>,request:{force?:boolean;signal?:AbortSignal}={}):Promise<T>{
  syncPolicy();if(!admitted(captured)||request.signal?.aborted)return Promise.reject(aborted());
  const hit=peek(captured,key,kind);if(!request.force&&hit?.fresh)return Promise.resolve(hit.value);
  const existing=jobs.get(id(key,kind));
  for(const job of [...jobs.values()])if(job.speculative&&job!==existing)cancel(job);
  const job=existing??make(captured,key,kind,loader,false);job.speculative=false;
  if(job.timer!==undefined)clearTimeout(job.timer);job.ready=true;start(job);
  job.waiters++;
  return new Promise<T>((resolve,reject)=>{
   let done=false;
   const finish=()=>{if(done)return false;done=true;request.signal?.removeEventListener('abort',onAbort);job.waiters--;return true;};
   const onAbort=()=>{if(!finish())return;reject(aborted());if(job.waiters===0&&!job.speculative)cancel(job);};
   request.signal?.addEventListener('abort',onAbort,{once:true});
   job.promise.then(value=>{if(finish())resolve(structuredClone(value));},error=>{if(finish())reject(error);});
  });
 }
 function prefetch(captured:PreferenceAccount,key:string,kind:MailCacheKind,loader:Loader<T>,request:PrefetchOptions):()=>void{
  const p=syncPolicy();if(!admitted(captured)||!speculativeAllowed(kind,request,p)||peek(captured,key,kind)?.fresh||jobs.has(id(key,kind)))return ()=>undefined;
  if([...jobs.values()].filter(job=>job.speculative&&!job.started).length>=p.maxQueued)return ()=>undefined;
  const job=make(captured,key,kind,loader,true);job.options=request;
  job.timer=setTimeout(()=>{job.ready=true;pump();},request.trigger==='idle'?p.idleDelayMs:p.dwellMs);
  return ()=>{if(job.speculative){cancel(job);pump();}};
 }
 function invalidate(predicate?:(key:string,kind:MailCacheKind)=>boolean,request:{abort?:boolean}={}){
  for(const [key,entry] of entries)if(!predicate||predicate(entry.key,entry.kind))remove(key);
  for(const job of [...jobs.values()])if(!predicate||predicate(job.key,job.kind)){
   if(request.abort!==false||job.speculative)cancel(job);
   else{job.publish=false;jobs.delete(id(job.key,job.kind));detached.add(job);}
  }
 }
 function dispose(){disposed=true;clear();}
 return {peek,load,prefetch,invalidate,clear,dispose,syncPolicy,stats:()=>({entries:entries.size,bytes,queued:[...jobs.values()].filter(j=>!j.started).length,inflight:[...jobs.values()].filter(j=>j.started).length})};
}
