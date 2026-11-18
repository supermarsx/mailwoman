import {createConnectionStability} from './connectionStability.ts';
import {batch,createSignal} from 'solid-js';
import {primaryMailboxAuthority} from '../api/linkedMailboxSession.ts';
import type {Me} from '../api/client.ts';
export type AccountHealthState='unknown'|'online'|'syncing'|'offline'|'degraded'|'auth-expired';
export interface AccountHealth {id:string;root:string;accountId:string;authority:string;linkId:string|null;label:string;state:AccountHealthState;updatedAt:number;retryAt?:number;unstable?:boolean}
const [records,setRecords]=createSignal<readonly AccountHealth[]>([]);
export const accountHealth=records;
const stability=new Map<string,{authority:string;unstable:boolean;tracker:ReturnType<typeof createConnectionStability>}>();
function forgetStability(id:string):void{const entry=stability.get(id);stability.delete(id);entry?.tracker.reset();}
function pruneStability():void{const ids=new Set(records().map(row=>row.id));for(const id of stability.keys())if(!ids.has(id))forgetStability(id);}
function observeStability(id:string,authority:string,state:AccountHealthState):boolean{
 let entry=stability.get(id);
 if(entry&&entry.authority!==authority){forgetStability(id);entry=undefined;}
 if(!entry){
  const created={authority,unstable:false,tracker:createConnectionStability(value=>{
   if(stability.get(id)!==created)return;
   created.unstable=value;
   setRecords(rows=>rows.map(row=>row.id===id&&row.authority===authority?{...row,unstable:value}:row));
  })};entry=created;stability.set(id,entry);
 }
 if(state==='unknown'||state==='auth-expired')entry.tracker.reset();
 else if(state!=='syncing')entry.tracker.observe(state==='online');
 return entry.unstable;
}

export function reportAccountHealth(owner:Me,linkId:string|null,state:AccountHealthState,retrySeconds?:number):void{
 const authority=owner.storageAuthority;if(!authority)return;
 batch(()=>{
 const root=primaryMailboxAuthority()??authority,id=JSON.stringify([root,linkId?'linked':'primary',linkId??owner.accountId]);
 const unstable=observeStability(id,authority,state);
 const value:AccountHealth={unstable,id,root,accountId:owner.accountId,authority,linkId,label:owner.username,state,updatedAt:Date.now(),...(retrySeconds?{retryAt:Date.now()+Math.min(86400,Math.max(1,retrySeconds))*1000}:{})};
 setRecords(previous=>{const old=previous.find(row=>row.id===id);if(old?.state===state&&old.authority===authority&&old.unstable===unstable&&!retrySeconds)return previous;return [value,...previous.filter(row=>row.id!==id&&!(row.root===root&&row.linkId===linkId&&row.accountId===owner.accountId))].slice(0,64);});pruneStability();
 });
}
export function clearAccountHealth():void{for(const id of stability.keys())forgetStability(id);setRecords([]);}

export function reconcileLinkedHealth(root:string,accounts:readonly {id:string;accountId:string;storageAuthority?:string;displayName:string;address?:string;status:string}[]):void{
 const ids=new Set(accounts.filter(row=>row.status!=='disabled').map(row=>row.id));
 setRecords(previous=>previous.filter(row=>row.root!==root||row.linkId===null||ids.has(row.linkId)));pruneStability();
 for(const account of accounts){
  if(account.status==='disabled')continue;
  const previous=records().find(row=>row.root===root&&row.linkId===account.id);
  // A saved, usable credential is not evidence of a successful mailbox request.
  const state:AccountHealthState=account.status==='reauthRequired'?'auth-expired':previous?.state==='auth-expired'||!previous?'unknown':previous.state;
  if(previous?.state===state&&previous.authority===(account.storageAuthority??''))continue;
  if(previous)forgetStability(previous.id);
  const value:AccountHealth={id:JSON.stringify([root,'linked',account.id]),root,accountId:account.accountId,authority:account.storageAuthority??'',linkId:account.id,label:account.displayName||account.address||account.accountId,state,updatedAt:Date.now()};
  setRecords(rows=>[value,...rows.filter(row=>!(row.root===root&&row.linkId===account.id))].slice(0,64));
 }
}
