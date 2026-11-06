import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

/** Which statistics: image proxy latency, browser load times, rate-limit and request-protection decisions. */
export type PersistedSource='imageProxy'|'clientTiming'|'rateLimit'|'requestProtection';
export const PERSISTED_RANGES=['1h','24h','7d','30d','1y'] as const;
export type PersistedRange=(typeof PERSISTED_RANGES)[number];
/** Latency groups carry p50/p95 (milliseconds); counter groups carry counts only. */
export interface PersistedTotals {count:number;p50?:number|null;p95?:number|null;p99?:number|null;max?:number|null;mean?:number|null}
export interface PersistedGroup {
 totals:PersistedTotals;
 series:{t:number[];count:number[];p50?:(number|null)[];p95?:(number|null)[]};
 breakdown:Record<string,PersistedTotals>;
}
export interface PersistedHistory {
 source:PersistedSource;range:PersistedRange;from:number;to:number;stepSecs:number;counter:boolean;relativeError:number;
 persistence:{
  enabled:boolean;dimensionsEnabled:boolean;flushSeconds:number;
  lastFlushAt:number|null;rows:number;historySince:number|null;instances:number;
  retention:{minuteBucketsHours:number;hourBucketsDays:number;dayBucketsDays:number};
 };
 groups:Record<string,PersistedGroup>;
}
/** `GET /admin/metrics/persisted` (observability.read): stored statistics merged over every server instance. */
export async function loadPersisted(source:PersistedSource,range:PersistedRange,signal?:AbortSignal):Promise<PersistedHistory>{
 const query=new URLSearchParams({source,range});
 return (await adminRequest<PersistedHistory>(`/metrics/persisted?${query}`,signal?{signal}:{})).data;
}
/** `POST /admin/metrics/persisted/reset` (settings.write): deletes the stored buckets of one source; audit-logged. */
export async function resetPersisted(source:PersistedSource):Promise<number>{
 return (await adminRequest<{removed:number}>('/metrics/persisted/reset',{method:'POST',body:{source}})).data.removed;
}
