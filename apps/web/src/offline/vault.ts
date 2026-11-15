import {openDB, type IDBPDatabase} from 'idb';
import {EncryptedCache, opfsAvailable, opfsBackend, type BlobBackend} from './opfs.ts';
import {generateProfileKey} from './crypto.ts';

export interface VaultLimits {
 maxBytes: number; maxEntries?: number; retentionDays?: number;
 budgetGroup?: string; eviction?: 'lru'|'oldest'|'manual'; pinned?: boolean; flagged?: boolean;
}
export interface VaultEntry {id:string; namespace:string; group?:string; bytes:number; created:number; accessed:number; protected:boolean; expires?:number|null}
export interface VaultIndex {
 all(namespace:string):Promise<VaultEntry[]>;
 group?(group:string):Promise<VaultEntry[]>;
 put(entry:VaultEntry):Promise<void>;
 remove(id:string):Promise<void>;
}
export interface OfflineVault {
 get<T>(namespace:string,key:string):Promise<T|undefined>;
 put(namespace:string,key:string,value:unknown,limits?:VaultLimits):Promise<boolean>;
 list<T>(namespace:string):Promise<{key:string;value:T}[]>;
 remove(namespace:string,key:string):Promise<void>;
 clear(namespace:string):Promise<void>;
 usage(namespace:string):Promise<{entries:number;bytes:number;protectedBytes:number}>;
}
const hash=async(value:string):Promise<string>=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(value))),b=>b.toString(16).padStart(2,'0')).join('');

/** The manifest carries opaque hashes and sizes only. Actual keys and values
 * are authenticated inside the encrypted record, including its namespace. */
export function createOfflineVault(cache:EncryptedCache,index:VaultIndex,now=Date.now):OfflineVault {
 let pending:Promise<unknown>=Promise.resolve();
 const serial=<T>(run:()=>Promise<T>):Promise<T>=>{const result=pending.then(async():Promise<T>=>{if(typeof navigator!=='undefined'&&navigator.locks){return await navigator.locks.request('mw-offline-vault',run);}return run();});pending=result.catch(()=>undefined);return result;};
 const identifiers=async(namespace:string,key:string)=>({namespace:await hash(namespace),id:await hash(JSON.stringify([namespace,key]))});
 async function read<T>(namespace:string,id:string):Promise<{key:string;value:T}|undefined>{
  const data=await cache.getRecord<{namespace:string;key:string;value:T}>(id);
  if(!data||data.namespace!==namespace||(await identifiers(namespace,data.key)).id!==id)return;
  return {key:data.key,value:data.value};
 }
 return {
  get<T>(namespace:string,key:string){return serial(async()=>{
   const identity=await identifiers(namespace,key);
   const entry=(await index.all(identity.namespace)).find(row=>row.id===identity.id);
   if(!entry)return;
   if(!entry.protected&&entry.expires&&entry.expires<now()){await cache.removeRecord(entry.id);await index.remove(entry.id);return;}
   const record=await read<T>(namespace,identity.id);
   if(!record)return;
   if(entry)await index.put({...entry,accessed:now()});
   return record.value;
  });},
  put(namespace,key,value,limits){return serial(async()=>{
   const identity=await identifiers(namespace,key),timestamp=now();
   // Measure the complete encrypted envelope, including its IV and GCM tag.
   const bytes=new TextEncoder().encode(JSON.stringify({id:identity.id,value:{namespace,key,value}})).length+28;
   const group=await hash(limits?.budgetGroup??namespace);
   const rows=index.group?await index.group(group):await index.all(identity.namespace),old=rows.find(row=>row.id===identity.id);
   const protectedRecord=limits?.pinned===true||limits?.flagged===true;
   const remaining=rows.filter(row=>row.id!==identity.id);let total=remaining.reduce((sum,row)=>sum+row.bytes,bytes);
   const remove:VaultEntry[]=[];
   const strategy=limits?.eviction??'lru';
   const expired=limits?.retentionDays?timestamp-limits.retentionDays*86400000:0;
   remaining.sort((a,b)=>strategy==='oldest'?a.created-b.created:a.accessed-b.accessed);
   for(const row of remaining){
    if(row.protected)continue;
    if((expired>0&&row.created<expired)||(limits&&strategy!=='manual'&&(total>limits.maxBytes||remaining.length-remove.length>=(limits.maxEntries??Infinity)))){remove.push(row);total-=row.bytes;}
   }
   if(limits&&(total>limits.maxBytes||remaining.length-remove.length>=(limits.maxEntries??Infinity)))return false;
   await cache.putRecord(identity.id,{namespace,key,value});
   await index.put({...identity,group,bytes,created:old?.created??timestamp,accessed:timestamp,protected:protectedRecord,expires:limits?.retentionDays?timestamp+limits.retentionDays*86400000:null});
   for(const row of remove){await cache.removeRecord(row.id);await index.remove(row.id);}
   return true;
  });},
  list<T>(namespace:string){return serial(async()=>{
   const result:{key:string;value:T}[]=[];
   for(const entry of await index.all(await hash(namespace))){if(!entry.protected&&entry.expires&&entry.expires<now()){await cache.removeRecord(entry.id);await index.remove(entry.id);continue;}try{const row=await read<T>(namespace,entry.id);if(row)result.push(row);}catch{/* Corrupt ciphertext is never shown. */}}
   return result;
  });},
  remove(namespace,key){return serial(async()=>{const {id}=await identifiers(namespace,key);await cache.removeRecord(id);await index.remove(id);});},
  clear(namespace){return serial(async()=>{const scope=await hash(namespace),rows=[...await index.all(scope),...(index.group?await index.group(scope):[])];for(const entry of new Map(rows.map(row=>[row.id,row])).values()){await cache.removeRecord(entry.id);await index.remove(entry.id);}});},
  usage(namespace){return serial(async()=>{const scope=await hash(namespace),all=[...await index.all(scope),...(index.group?await index.group(scope):[])],rows=[...new Map(all.map(row=>[row.id,row])).values()];return {entries:rows.length,bytes:rows.reduce((n,row)=>n+row.bytes,0),protectedBytes:rows.filter(row=>row.protected).reduce((n,row)=>n+row.bytes,0)};});},
 };
}

const openings=new Map<string,Promise<OfflineVault>>();
/** OPFS is preferred; browsers without OPFS keep the same encrypted records in
 * IndexedDB. Unsupported storage never falls back to plaintext persistence. */
export function offlineVault(mode:'auto'|'indexeddb'='auto'):Promise<OfflineVault> {
 const existing=openings.get(mode);if(existing)return existing;
 const opening=(async()=>{
  if(typeof indexedDB==='undefined'||!crypto.subtle)throw new Error('Encrypted offline storage is unavailable');
  const database:IDBPDatabase=await openDB(mode==='auto'?'mailwoman-offline-vault':'mailwoman-offline-vault-idb',2,{upgrade(db,_old,_new,transaction){if(!db.objectStoreNames.contains('key'))db.createObjectStore('key');if(!db.objectStoreNames.contains('blobs'))db.createObjectStore('blobs');const store=db.objectStoreNames.contains('index')?transaction.objectStore('index'):db.createObjectStore('index',{keyPath:'id'});if(!store.indexNames.contains('namespace'))store.createIndex('namespace','namespace');if(!store.indexNames.contains('group'))store.createIndex('group','group');}});
  const candidate=await generateProfileKey(),tx=database.transaction('key','readwrite');
  const stored=await tx.store.get('profile') as CryptoKey|undefined;
  if(!stored)await tx.store.put(candidate,'profile');await tx.done;
  let filesystem:BlobBackend|undefined;
  if(mode==='auto'&&opfsAvailable()){try{await navigator.storage.getDirectory();filesystem=opfsBackend();}catch{/* Use encrypted IndexedDB when OPFS is unavailable in this context. */}}
  const backend:BlobBackend=filesystem??{
   async read(path){return (await database.get('blobs',path) as Uint8Array|undefined)??null;},
   async write(path,bytes){await database.put('blobs',bytes,path);},
   async remove(path){await database.delete('blobs',path);},
  };
  return createOfflineVault(new EncryptedCache(backend,stored??candidate),{
   all:namespace=>database.getAllFromIndex('index','namespace',namespace),group:group=>database.getAllFromIndex('index','group',group),
   async put(row){await database.put('index',row);},async remove(id){await database.delete('index',id);},
  });
 })().catch(error=>{openings.delete(mode);throw error;});
 openings.set(mode,opening);return opening;
}
