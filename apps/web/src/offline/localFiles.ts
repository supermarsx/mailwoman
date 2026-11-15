import type {JmapRequest} from '../api/jmap-types.ts';
import {offlineVault} from './vault.ts';
interface Owner {accountId:string;storageAuthority:string;base:string}
interface LocalFile {owner:Owner;name:string;type:string;data:string;size:number;uploaded?:{blobId:string;size:number;type:string}}
const PREFIX='mw-local:';
const encode=(bytes:Uint8Array)=>{let text='';for(let offset=0;offset<bytes.length;offset+=8192)text+=String.fromCharCode(...bytes.subarray(offset,offset+8192));return btoa(text);};
const decode=(value:string)=>Uint8Array.from(atob(value),character=>character.charCodeAt(0));
const key=(owner:Owner,id:string)=>JSON.stringify([owner.base,owner.storageAuthority,owner.accountId,id]);
export async function stageLocalFile(owner:Owner,file:Blob&{name?:string}):Promise<{accountId:string;blobId:string;size:number;type:string}>{
 if(file.size>50*1024*1024)throw new Error('Offline attachments are limited to 50 MiB per file');
 const id=PREFIX+crypto.randomUUID(),type=file.type||'application/octet-stream',vault=await offlineVault('indexeddb');
 await vault.put('pending-files',key(owner,id),{owner,name:file.name??'attachment',type,size:file.size,data:encode(new Uint8Array(await file.arrayBuffer()))} satisfies LocalFile);
 return {accountId:owner.accountId,blobId:id,size:file.size,type};
}
/** Resolve local blob handles immediately before replay. Each successful upload
 * is durably mapped, so retrying a send uses the same uploaded attachments. */
export async function prepareLocalFiles(request:JmapRequest,owner:Owner,upload:(file:Blob&{name:string})=>Promise<{blobId:string;size:number;type:string}>):Promise<JmapRequest>{
 if(!JSON.stringify(request).includes(PREFIX))return request;
 const result=structuredClone(request),vault=await offlineVault('indexeddb');
 for(const [,args] of result.methodCalls){
  if(args['accountId']!==owner.accountId)throw new Error('Local attachment account changed');
  const stack:unknown[]=[args];let visited=0;
  while(stack.length){
   if(++visited>100000)throw new Error('Attachment request is too large');
   const value=stack.pop();if(!value||typeof value!=='object')continue;
   const object=value as Record<string,unknown>,id=object['blobId'];
   if(typeof id==='string'&&id.startsWith(PREFIX)){
    const record=await vault.get<LocalFile>('pending-files',key(owner,id));
    if(!record||(record.owner.base!==owner.base||record.owner.accountId!==owner.accountId||record.owner.storageAuthority!==owner.storageAuthority))throw new Error('Offline attachment is unavailable for this account');
    if(!record.uploaded){
     const blob=Object.assign(new Blob([decode(record.data).slice().buffer],{type:record.type}),{name:record.name});
     const uploaded=await upload(blob);if(uploaded.blobId.startsWith(PREFIX))throw new Error('Attachment upload is still offline');
     record.uploaded=uploaded;await vault.put('pending-files',key(owner,id),record);
    }
    object['blobId']=record.uploaded.blobId;if('size' in object)object['size']=record.uploaded.size;
   }
   for(const child of Object.values(object))if(child&&typeof child==='object')stack.push(child);
  }
 }
 return result;
}

export async function readLocalFile(owner:Owner,id:string):Promise<Blob|undefined>{
 const record=await (await offlineVault('indexeddb')).get<LocalFile>('pending-files',key(owner,id));
 if(!record||record.owner.base!==owner.base||record.owner.accountId!==owner.accountId||record.owner.storageAuthority!==owner.storageAuthority)return;
 return new Blob([decode(record.data).slice().buffer],{type:record.type});
}
