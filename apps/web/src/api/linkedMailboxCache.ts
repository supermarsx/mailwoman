import type {Me} from './client.ts';
import type {LinkedAccountContext} from './linkedAccountContext.ts';
import type {JmapRequest,JmapResponse,JmapSession} from './jmap-types.ts';
import {offlineServerBase} from '../offline/cacheScope.ts';
import {onLinkedMailboxChange} from './linkedMailboxSession.ts';
export interface LinkedMailboxSnapshot {user:Me;discovery:JmapSession;context:Readonly<LinkedAccountContext>|null;folders:JmapResponse;page?:{request:JmapRequest;response:JmapResponse};at:number}
const rows=new Map<string,{value:LinkedMailboxSnapshot;bytes:number}>();let bytes=0;
const key=(root:string,link:string|null)=>JSON.stringify([offlineServerBase(),root,link]);
export function rememberLinkedMailbox(root:string,link:string|null,value:Omit<LinkedMailboxSnapshot,'at'>,limits={maxBytes:8*1024*1024,maxEntries:17}):void{
 const id=key(root,link),snapshot={...value,at:Date.now()},size=new TextEncoder().encode(JSON.stringify(snapshot)).length;
 const maxBytes=Math.max(0,Math.min(8*1024*1024,limits.maxBytes)),maxEntries=Math.max(0,Math.min(17,limits.maxEntries));
 bytes-=rows.get(id)?.bytes??0;rows.delete(id);
 if(size<=Math.min(2*1024*1024,maxBytes)&&maxEntries>0){rows.set(id,{value:structuredClone(snapshot),bytes:size});bytes+=size;}
 while(rows.size>maxEntries||bytes>maxBytes){const first=rows.keys().next().value!;bytes-=rows.get(first)!.bytes;rows.delete(first);}
}
export function peekLinkedMailbox(root:string,link:string|null,account:string,authority:string):LinkedMailboxSnapshot|undefined{
 const value=rows.get(key(root,link))?.value;
 return value&&Date.now()-value.at<120000&&value.user.accountId===account&&value.user.storageAuthority===authority?structuredClone(value):undefined;
}
export function linkedMailboxCachedPage(root:string,link:string|null,account:string,authority:string,request:JmapRequest):JmapResponse|undefined{
 const held=peekLinkedMailbox(root,link,account,authority);return held?.page&&JSON.stringify(held.page.request)===JSON.stringify(request)?held.page.response:undefined;
}
export function clearLinkedMailboxCache():void{rows.clear();bytes=0;}
onLinkedMailboxChange(change=>{if(!change.context&&!change.previous)clearLinkedMailboxCache();});
