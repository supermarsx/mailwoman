import {openDB,type IDBPDatabase} from 'idb';
import type {PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {MailCacheKind} from '../state/mailReadCache.ts';
import {decryptJson,encryptJson,generateProfileKey} from './crypto.ts';

export interface PersistentMailStore<T> {
 get(owner:PreferenceAccount,key:string,kind:MailCacheKind):Promise<T|undefined>;
 put(owner:PreferenceAccount,key:string,kind:MailCacheKind,value:T,limits:{maxBytes:number;maxEntries:number}):Promise<void>;
 clear(owner:PreferenceAccount,predicate?:(key:string,kind:MailCacheKind,value?:T)=>boolean):Promise<void>;
}
interface Row {id:string;partition:string;key:string;kind:MailCacheKind;at:number;bytes:number;encrypted:Uint8Array}
/** Encrypted snapshots are never aged out. Live reads still refresh normally.
 * Each partition uses a verified server-issued storage authority and account ID.
 * Storage is opened lazily, only after the user opts in. */
export function persistentMailStore<T>():PersistentMailStore<T> {
 let opening:Promise<IDBPDatabase>|undefined;
 let encryptionKey:Promise<CryptoKey>|undefined;
 let pending:Promise<unknown>=Promise.resolve();
 const db=()=>opening??=openDB('mailwoman-mail-cache',1,{upgrade(database){
  database.createObjectStore('key');
  database.createObjectStore('entries',{keyPath:'id'}).createIndex('partition','partition');
 }});
 const partition=(owner:PreferenceAccount)=>owner.storageAuthority?JSON.stringify([owner.storageAuthority,owner.id]):undefined;
 const id=(scope:string,key:string,kind:MailCacheKind)=>JSON.stringify([scope,kind,key]);
 const key=()=>encryptionKey??=(async()=>{
  const database=await db();
  const existing=await database.get('key','aes');if(existing)return existing as CryptoKey;
  // Generate before the transaction; transactions cannot wait for WebCrypto.
  const candidate=await generateProfileKey(),tx=database.transaction('key','readwrite');
  const stored=await tx.store.get('aes') as CryptoKey|undefined;
  if(!stored)await tx.store.put(candidate,'aes');await tx.done;return stored??candidate;
 })();
 function serial<R>(run:()=>Promise<R>):Promise<R>{const result=pending.then(run);pending=result.catch(()=>undefined);return result;}
 return {
  get(owner,cacheKey,kind){return serial(async()=>{
   const scope=partition(owner);if(!scope||typeof indexedDB==='undefined')return;
   const database=await db(),record=await database.get('entries',id(scope,cacheKey,kind)) as Row|undefined;
   if(!record)return;
   try {
    const payload=await decryptJson<{id:string;value:T}>(await key(),record.encrypted);
    if(payload.id!==record.id)return;
    record.at=Date.now();await database.put('entries',record);return payload.value;
   }catch{await database.delete('entries',record.id);return;}
  });},
  put(owner,cacheKey,kind,value,limits){return serial(async()=>{
   const scope=partition(owner);if(!scope||typeof indexedDB==='undefined')return;
   const rowId=id(scope,cacheKey,kind),encrypted=await encryptJson(await key(),{id:rowId,value});
   if(encrypted.byteLength>limits.maxBytes)return;
   const database=await db(),tx=database.transaction('entries','readwrite');
   const rows=(await tx.store.index('partition').getAll(scope) as Row[]).filter(row=>row.id!==rowId).sort((a,b)=>a.at-b.at);
   let bytes=rows.reduce((total,row)=>total+row.bytes,encrypted.byteLength);
   while(rows.length>=limits.maxEntries||bytes>limits.maxBytes){const row=rows.shift();if(!row)break;bytes-=row.bytes;await tx.store.delete(row.id);}
   await tx.store.put({id:rowId,partition:scope,key:cacheKey,kind,at:Date.now(),bytes:encrypted.byteLength,encrypted} satisfies Row);
   await tx.done;
  });},
  clear(owner,predicate){return serial(async()=>{
   const scope=partition(owner);if(!scope||typeof indexedDB==='undefined')return;
   const database=await db(),rows=await database.getAllFromIndex('entries','partition',scope) as Row[];
   for(const row of rows){
    let remove=!predicate||predicate(row.key,row.kind);
    if(!remove&&predicate){
     try{const payload=await decryptJson<{id:string;value:T}>(await key(),row.encrypted);remove=payload.id===row.id&&predicate(row.key,row.kind,payload.value);}catch{remove=true;}
    }
    if(remove)await database.delete('entries',row.id);
   }
  });},
 };
}
