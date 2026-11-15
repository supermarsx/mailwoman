import {createSignal} from 'solid-js';
import type {Me} from '../api/client.ts';
import {offlineVault,type OfflineVault,type VaultLimits} from './vault.ts';

export interface OfflinePolicy {enabled:boolean;budgetMb:number;retentionDays:number;eviction:'lru'|'oldest'|'manual'}
export const DEFAULT_OFFLINE_POLICY:OfflinePolicy={enabled:false,budgetMb:250,retentionDays:30,eviction:'lru'};
const accountPolicies=new Map<string,OfflinePolicy>();
const policyKey=(owner:{accountId?:string;id?:string;storageAuthority?:string})=>JSON.stringify([owner.storageAuthority,owner.accountId??owner.id]);
let policy:OfflinePolicy={...DEFAULT_OFFLINE_POLICY};
const [isOffline,setIsOffline]=createSignal(false);
export {isOffline};
const listeners=new Set<(offline:boolean)=>void>();
const policies=new Set<(value:OfflinePolicy)=>void>();
export const offlinePolicy=(owner?:{accountId?:string;id?:string;storageAuthority?:string}):OfflinePolicy=>owner?(accountPolicies.get(policyKey(owner))??policy):policy;
export function configureOfflinePolicy(value:OfflinePolicy,owner?:{id?:string;accountId?:string;storageAuthority?:string}):void {
 const next={...value,budgetMb:Math.max(0,Math.min(100000,value.budgetMb)),retentionDays:Math.max(0,Math.min(36500,value.retentionDays))};
 if(!owner)accountPolicies.clear();
 const changed=owner&&JSON.stringify(accountPolicies.get(policyKey(owner)))!==JSON.stringify(next);
 if(owner)accountPolicies.set(policyKey(owner),next);
 if(!changed&&JSON.stringify(next)===JSON.stringify(policy))return;
 policy=next;for(const listener of policies)listener(next);
}
export function observeOfflinePolicy(listener:(value:OfflinePolicy)=>void):()=>void {policies.add(listener);return()=>policies.delete(listener);}
export function reportOffline(value:boolean):void {if(isOffline()===value)return;setIsOffline(value);for(const listener of listeners)listener(value);}
export function observeOffline(listener:(value:boolean)=>void):()=>void {listeners.add(listener);return()=>listeners.delete(listener);}
export function offlineLimits(extra:Partial<VaultLimits>={}):VaultLimits {return {maxBytes:policy.budgetMb*1024*1024,retentionDays:policy.retentionDays,eviction:policy.eviction,...extra};}
export function offlineNamespace(base:string,me:Pick<Me,'accountId'|'storageAuthority'>):string {
 if(!me.storageAuthority)throw new Error('Verified offline storage authority is required');
 return JSON.stringify(['offline-account',base,me.storageAuthority,me.accountId]);
}
export interface OfflineBootstrap {me:Me;policy:OfflinePolicy;at:number}
export async function readOfflineBootstrap(base:string,storage:()=>Promise<OfflineVault>=offlineVault):Promise<OfflineBootstrap|undefined> {
 const row=await (await storage()).get<OfflineBootstrap>(`offline-bootstrap:${base}`,'primary');
 return row?.policy.enabled&&row.me.storageAuthority?row:undefined;
}
export async function saveOfflineBootstrap(base:string,me:Me,storage:()=>Promise<OfflineVault>=offlineVault):Promise<void> {
 const selected=offlinePolicy(me);if(!selected.enabled||!me.storageAuthority)return;
 await (await storage()).put(`offline-bootstrap:${base}`,'primary',{me,policy:selected,at:Date.now()} satisfies OfflineBootstrap);
}
export async function forgetOfflineBootstrap(base:string,storage:()=>Promise<OfflineVault>=offlineVault):Promise<void> {await (await storage()).clear(`offline-bootstrap:${base}`);}

let verifier:(()=>Promise<void>)|undefined;
export function registerOfflineVerifier(value:()=>Promise<void>):void {verifier=value;}
export async function verifyOfflineAuthority():Promise<void> {
 if(!isOffline())return;
 if(!verifier)throw new TypeError('Offline session has not been revalidated');
 await verifier();
}

let revoker:(()=>Promise<void>)|undefined;
export function registerOfflineRevoker(value:()=>Promise<void>):void {revoker=value;}
export async function revokeOfflineAuthorization():Promise<void> {await revoker?.();}
