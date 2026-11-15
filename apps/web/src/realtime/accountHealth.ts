import {createSignal} from 'solid-js';
import {primaryMailboxAuthority} from '../api/linkedMailboxSession.ts';
import type {Me} from '../api/client.ts';
export type AccountHealthState='online'|'syncing'|'offline'|'degraded'|'auth-expired';
export interface AccountHealth {id:string;root:string;accountId:string;authority:string;linkId:string|null;label:string;state:AccountHealthState;updatedAt:number;retryAt?:number}
const [records,setRecords]=createSignal<readonly AccountHealth[]>([]);
export const accountHealth=records;
export function reportAccountHealth(owner:Me,linkId:string|null,state:AccountHealthState,retrySeconds?:number):void{
 if(!owner.storageAuthority)return;
 const root=primaryMailboxAuthority()??owner.storageAuthority,id=JSON.stringify([root,linkId?'linked':'primary',linkId??owner.accountId]);
 const value:AccountHealth={id,root,accountId:owner.accountId,authority:owner.storageAuthority,linkId,label:owner.username,state,updatedAt:Date.now(),...(retrySeconds?{retryAt:Date.now()+Math.min(86400,Math.max(1,retrySeconds))*1000}:{})};
 setRecords(previous=>{const old=previous.find(row=>row.id===id);if(old?.state===state&&!retrySeconds)return previous;return [value,...previous.filter(row=>row.id!==id&&!(row.root===root&&row.linkId===linkId&&row.accountId===owner.accountId))].slice(0,64);});
}
export function clearAccountHealth():void{setRecords([]);}

export function reconcileLinkedHealth(root:string,accounts:readonly {id:string;accountId:string;storageAuthority?:string;displayName:string;address?:string;status:string}[]):void{
 const ids=new Set(accounts.filter(row=>row.status!=='disabled').map(row=>row.id));
 setRecords(previous=>previous.filter(row=>row.root!==root||row.linkId===null||ids.has(row.linkId)));
 for(const account of accounts){
  if(account.status==='disabled')continue;
  const previous=records().find(row=>row.root===root&&row.linkId===account.id);
  const state:AccountHealthState=account.status==='reauthRequired'?'auth-expired':previous?.state==='auth-expired'||!previous?'online':previous.state;
  if(previous?.state===state)continue;
  const value:AccountHealth={id:JSON.stringify([root,'linked',account.id]),root,accountId:account.accountId,authority:account.storageAuthority??'',linkId:account.id,label:account.displayName||account.address||account.accountId,state,updatedAt:Date.now()};
  setRecords(rows=>[value,...rows.filter(row=>!(row.root===root&&row.linkId===account.id))].slice(0,64));
 }
}
