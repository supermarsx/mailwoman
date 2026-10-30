import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface RequestHistoryRange {from:number;to:number}
/** Where the 404 series exists; `notTracked` periods predate 404 tracking (never zero). */
export interface NotFoundCoverage {observedSeconds:number;trackedSince:number|null;notTracked:RequestHistoryRange[]}
export interface RequestHistory {
 from:number;to:number;bucketSeconds:number;
 series:{t:number;requests:number|null;errors5xx:number|null;notFound?:number|null;notFoundCoverage?:'complete'|'partial'|'unavailable'|'notTracked';p50Ms:number|null;p95Ms:number|null;coverage:'complete'|'partial'|'unavailable';legacyLatencyEstimate?:boolean;p50ResolutionMs?:number|null;p95ResolutionMs?:number|null}[];
 coverage:{observedSeconds:number;requestedSeconds:number;gaps:RequestHistoryRange[];notFound?:NotFoundCoverage};
 policy:{defaultRangeSeconds:number;maxRangeSeconds:number;retentionDays:number};
 scope:'server';timing:'response-ready';latencySampleUnit?:'integer-milliseconds'|'microseconds';
}
/** One redacted `method + path` pair; `count` is an upper and `minCount` a lower bound. */
/** `source` (t102): `client` = a shape the app's own clients request; `external` = scanners, probes, links from elsewhere. */
export interface NotFoundPath {method:string;path:string;count:number;minCount:number;source?:'client'|'external'}
/** `GET /admin/metrics/not-found`: top paths over the whole hours overlapping the window. */
export interface NotFoundPaths {
 from:number;to:number;pathsFrom:number;pathsTo:number;bucketSeconds:number;limit:number;
 total:number|null;approximate:boolean;paths:NotFoundPath[];coverage:NotFoundCoverage;scope:'server';
 /** t102: lower-bound totals of the listed paths by source. */
 listedBySource?:{client:number;external:number};
}
export async function loadRequestHistory(range?:RequestHistoryRange,signal?:AbortSignal):Promise<RequestHistory>{
 const query=new URLSearchParams({maxPoints:'240'});
 if(range){query.set('from',String(range.from));query.set('to',String(range.to));}
 return (await adminRequest<RequestHistory>(`/metrics/history?${query}`,signal?{signal}:{})).data;
}
/** Needs unscoped Observability read; rejects with an `AdminProblemError` (403) otherwise. */
export async function loadNotFoundPaths(range?:RequestHistoryRange,signal?:AbortSignal):Promise<NotFoundPaths>{
 const query=new URLSearchParams();
 if(range){query.set('from',String(range.from));query.set('to',String(range.to));}
 const suffix=range?`?${query}`:'';
 return (await adminRequest<NotFoundPaths>(`/metrics/not-found${suffix}`,signal?{signal}:{})).data;
}
