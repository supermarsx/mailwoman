import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
export const receiptModes=['never','ask','always','contactsOnly','orgOnly','orgAlwaysAskOthers'] as const;
export type ReceiptMode=typeof receiptModes[number];
export interface ReceiptPolicy {policy:ReceiptMode;locked:boolean;allowedModes:readonly ReceiptMode[]}
export interface ReceiptPolicyApi {load(signal:AbortSignal):Promise<ReceiptPolicy>;save(policy:ReceiptMode,signal:AbortSignal):Promise<void>}
export class ReceiptPolicyError extends Error {constructor(readonly status:number){super(`Receipt settings request failed (${status})`);}}
function mode(value:unknown):value is ReceiptMode {return typeof value==='string'&&receiptModes.some(item=>item===value);}
export const receiptPolicyApi:ReceiptPolicyApi={
 async load(signal){const response=await apiFetch(withBase('/api/account/mdn-policy'),{credentials:'same-origin',signal});if(!response.ok)throw new ReceiptPolicyError(response.status);const value:unknown=await response.json();
  if(!value||typeof value!=='object'||!('policy' in value)||!mode(value.policy)||!('locked' in value)||typeof value.locked!=='boolean')throw new Error('Invalid receipt policy');
  const supplied='allowedModes' in value?value.allowedModes:undefined;
  const allowedModes:readonly ReceiptMode[]=supplied===undefined?['never','ask','always']:Array.isArray(supplied)&&supplied.length>0&&supplied.every(mode)?supplied:[];
  if(!allowedModes.length||(!value.locked&&!allowedModes.includes(value.policy)))throw new Error('Invalid receipt policy modes');
  return {policy:value.policy,locked:value.locked,allowedModes:value.locked?[value.policy]:allowedModes};
 },
 async save(policy,signal){const response=await apiFetch(withBase('/api/account/mdn-policy'),{method:'PUT',credentials:'same-origin',signal,headers:{'Content-Type':'application/json'},body:JSON.stringify({policy})});if(!response.ok)throw new ReceiptPolicyError(response.status);},
};

export interface ReceiptDecision {accountId:string;emailId:string;requestedBy:string;handled:boolean;action:'none'|'ask'|'automatic'}
export async function loadReceiptDecision(accountId:string,emailId:string,signal:AbortSignal):Promise<ReceiptDecision|null>{
 const response=await apiFetch(withBase(`/api/account/mdn-decision/${encodeURIComponent(emailId)}`),{credentials:'same-origin',signal});
 if(response.status===204||response.status===404)return null;if(!response.ok)throw new ReceiptPolicyError(response.status);
 const value=await response.json() as Partial<ReceiptDecision>;
 if(value.accountId!==accountId||value.emailId!==emailId||typeof value.requestedBy!=='string'||typeof value.handled!=='boolean'||!['none','ask','automatic'].includes(value.action??''))throw new Error('Invalid receipt decision');
 return value as ReceiptDecision;
}

export interface ReceiptSenderRule {address:string;decision:'always'|'never';updatedAt:number}
export interface ReceiptSenderSnapshot {revision:string;rules:ReceiptSenderRule[]}
export async function loadReceiptSenders(signal:AbortSignal):Promise<ReceiptSenderSnapshot>{
 const response=await apiFetch(withBase('/api/account/mdn-senders'),{credentials:'same-origin',signal});if(!response.ok)throw new ReceiptPolicyError(response.status);
 const value=await response.json() as {rules?:unknown};const revision=response.headers.get('etag');
 if(!revision||!Array.isArray(value.rules)||value.rules.length>500||!value.rules.every((row:unknown)=>!!row&&typeof row==='object'&&'address'in row&&typeof row.address==='string'&&'decision'in row&&(row.decision==='always'||row.decision==='never')&&'updatedAt'in row&&typeof row.updatedAt==='number'))throw new Error('Invalid sender rules');
 return {revision,rules:value.rules as ReceiptSenderRule[]};
}
export async function saveReceiptSender(address:string,decision:'always'|'never'|null,revision:string,signal:AbortSignal):Promise<void>{
 const response=await apiFetch(withBase('/api/account/mdn-senders'),{method:'PUT',credentials:'same-origin',signal,headers:{'Content-Type':'application/json','If-Match':revision},body:JSON.stringify({address,decision})});if(!response.ok)throw new ReceiptPolicyError(response.status);
}
