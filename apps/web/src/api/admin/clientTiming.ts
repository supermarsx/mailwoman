import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
import type {RequestHistoryRange} from './requestHistory.ts';

export type ClientTimingMetric='timeToUsable'|'ttfb'|'dcl'|'load'|'fcp'|'lcp'|'folderOpen'|'folderLoadCold'|'folderLoadCached'|'messageLoadCold'|'messageLoadCached';
export const CLIENT_TIMING_METRICS:readonly ClientTimingMetric[]=['timeToUsable','fcp','lcp','load','ttfb','dcl','folderOpen','folderLoadCold','folderLoadCached','messageLoadCold','messageLoadCached'];
export const CLIENT_TIMING_DEVICES=['desktop','mobile','unknown'] as const;
/** Percentiles are null below the admin-set minimum sample count. */
export interface ClientTimingStats {count:number;sufficient:boolean;meanMs:number|null;p50Ms:number|null;p75Ms:number|null;p95Ms:number|null;p99Ms:number|null}
export interface ClientTimingPoint {t:number;count:number;p50Ms:number|null;p95Ms:number|null}
export interface ClientTimingSeries {all:ClientTimingStats;devices:Partial<Record<(typeof CLIENT_TIMING_DEVICES)[number],ClientTimingStats>>;series:ClientTimingPoint[]}
/** `GET /admin/metrics/client-timing` (observability.read): browser-measured, sampled, aggregate. */
export interface ClientTimingHistory {
 from:number;to:number;bucketSeconds:number;
 metrics:Record<ClientTimingMetric,ClientTimingSeries>;
 policy:{enabled:boolean;folderLoads:boolean;messageLoads:boolean;sampleRatePercent:number;minSamples:number;retentionDays:number};
 resolution:{bins:number;maxRelativeError:number};
 scope:'client';sampled:true;
}
export async function loadClientTiming(range?:RequestHistoryRange,signal?:AbortSignal):Promise<ClientTimingHistory>{
 const query=new URLSearchParams({maxPoints:'96'});
 if(range){query.set('from',String(range.from));query.set('to',String(range.to));}
 return (await adminRequest<ClientTimingHistory>(`/metrics/client-timing?${query}`,signal?{signal}:{})).data;
}
