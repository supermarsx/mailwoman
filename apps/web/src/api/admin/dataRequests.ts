import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
import type {DataRequest,RequestKind} from '../dataRequests.ts';

/** A request as the administrator sees it. */
export interface AdminDataRequest extends Omit<DataRequest,'requestedBy'> {
 address:string;domain:string;assignee:string|null;requestedBy:string;
 verifiedAt:number|null;verifiedBy:string|null;approvedAt:number|null;approvedBy:string|null;
 originalDueAt:number;answeredAt:number|null;error:string|null;
 legalHold:{active:boolean;source?:'policy'|'request';reason?:string};
 requiresVerification:boolean;approvalMode:string;
}
export interface DataRequestStats {
 total:number;open:number;overdue:number;dueSoon:number;answered:number;onTime:number;avgAnswerHours:number|null;
 byStatus:Record<string,number>;byKind:Record<string,number>;created30d:{t:number;count:number}[];
}
export interface DataRequestList {requests:AdminDataRequest[];matched:number;/** Present while more rows follow; pass it back as `cursor`. */nextCursor?:string|null;stats:DataRequestStats;kinds:RequestKind[]}
export interface DataRequestFilter {status?:string;kind?:string;domain?:string;assignee?:string;q?:string;due?:string;/** Comma-separated request ids (the CSV of a selection). */ids?:string;/** The `nextCursor` of the previous page. */cursor?:string;limit?:string}
export interface TrailRow {id:string;ts:number;actor:string;kind:string;detail:Record<string,unknown>|null}

const qs=(f:DataRequestFilter)=>{const p=new URLSearchParams();for(const [k,v] of Object.entries(f))if(v)p.set(k,v);const s=p.toString();return s?`?${s}`:'';};
export const listDataRequests=async(f:DataRequestFilter={},signal?:AbortSignal):Promise<DataRequestList>=>(await adminRequest<DataRequestList>(`/dsr${qs(f)}`,signal?{signal}:{})).data;
export const loadDataRequest=async(id:string):Promise<{request:AdminDataRequest;trail:TrailRow[]}>=>(await adminRequest<{request:AdminDataRequest;trail:TrailRow[]}>(`/dsr/${encodeURIComponent(id)}`)).data;
const act=async(id:string,action:string,body:Record<string,unknown>={}):Promise<AdminDataRequest>=>(await adminRequest<{request:AdminDataRequest}>(`/dsr/${encodeURIComponent(id)}/${action}`,{method:'POST',body})).data.request;
export const assignDataRequest=(id:string,assignee:string|null)=>act(id,'assign',{assignee});
export const verifyDataRequest=(id:string,note:string)=>act(id,'verify',{note});
export const approveDataRequest=(id:string,note='')=>act(id,'approve',{note});
export const rejectDataRequest=(id:string,reason:string)=>act(id,'reject',{reason});
export const extendDataRequest=(id:string,days:number,reason:string)=>act(id,'extend',{days,reason});
export const completeDataRequest=(id:string,note:string)=>act(id,'complete',{note});
export const holdDataRequest=(id:string,reason:string)=>act(id,'hold',{reason});
export const releaseDataRequestHold=(id:string)=>act(id,'release-hold');
export const runDataRequest=(id:string)=>act(id,'run');
export type BulkAction='approve'|'reject'|'extend'|'assign'|'hold'|'release-hold';
export interface BulkRowResult {id:string;ok:boolean;status?:string|null;code?:string;detail?:string}
export interface BulkResult {action:BulkAction;results:BulkRowResult[];done:number;failed:number}
/** One action over several requests; every row is answered on its own. A missing step-up proof asks once (the admin dialog) and the call is repeated. */
export const bulkDataRequests=async(input:{action:BulkAction;ids:string[];note?:string;reason?:string;days?:number;assignee?:string|null}):Promise<BulkResult>=>(await adminRequest<BulkResult>('/dsr/bulk',{method:'POST',body:input})).data;
export const createDataRequestOnBehalf=async(input:{address:string;kind:RequestKind;details?:string;includeMail?:boolean}):Promise<AdminDataRequest>=>(await adminRequest<{request:AdminDataRequest}>('/dsr',{method:'POST',body:input})).data.request;
/** The CSV of the filtered log, fetched with the admin session (a link would skip the session header). */
export const exportDataRequestLog=async(f:DataRequestFilter):Promise<string>=>{
 const {adminUrl}=await import('../adminPath.ts');const {basePath}=await import('../basePath.ts');const {apiFetch}=await import('../http.ts');
 const r=await apiFetch(adminUrl(`/dsr/export${qs(f)}`,basePath()),{credentials:'same-origin'});
 if(!r.ok)throw new Error(`export failed (${r.status})`);return r.text();
};
