import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface RequestHistoryRange {from:number;to:number}
export interface RequestHistory {
 from:number;to:number;bucketSeconds:number;
 series:{t:number;requests:number|null;errors5xx:number|null;p50Ms:number|null;p95Ms:number|null;coverage:'complete'|'partial'|'unavailable';legacyLatencyEstimate?:boolean;p50ResolutionMs?:number|null;p95ResolutionMs?:number|null}[];
 coverage:{observedSeconds:number;requestedSeconds:number;gaps:RequestHistoryRange[]};
 policy:{defaultRangeSeconds:number;maxRangeSeconds:number;retentionDays:number};
 scope:'server';timing:'response-ready';
}
export async function loadRequestHistory(range?:RequestHistoryRange,signal?:AbortSignal):Promise<RequestHistory>{
 const query=new URLSearchParams({maxPoints:'240'});
 if(range){query.set('from',String(range.from));query.set('to',String(range.to));}
 return (await adminRequest<RequestHistory>(`/metrics/history?${query}`,signal?{signal}:{})).data;
}
