import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface PublicAssetCacheStatus {
 scope:'public-static-representations-only';
 config:{enabled:boolean;budgetMiB:number;maxFileMiB:number;maxEntries:number;revalidateSeconds:number;preload:'off'|'shell'|'core'};
 effectivePreload:'off'|'shell'|'core';generation:number;residentOwnedBytes:number;entries:number;effectiveMaxFileBytes:number;
 pendingUnique:number;runningLoads:number;queuedUnique:number;maxQueuedUnique:number;maxRunningLoads:number;
 configurationStale:boolean;lastConfigAt:number|null;
 counters:{hits:number;misses:number;loads:number;coalesced:number;evictions:number;expired:number;oversize:number;busy:number;loadFailures:number;staleInserts:number;clears:number;preloadLoaded:number;preloadSkipped:number};
 hardRssLimit:false;embeddedRawBytesDuplicated:false;
}
export async function loadPublicAssetCache(signal?:AbortSignal):Promise<PublicAssetCacheStatus>{
 return (await adminRequest<PublicAssetCacheStatus>('/maintenance/web-assets',signal?{signal}:{})).data;
}
export async function clearPublicAssetCache(signal?:AbortSignal):Promise<PublicAssetCacheStatus>{
 return (await adminRequest<PublicAssetCacheStatus>('/maintenance/web-assets/clear',{method:'POST',...(signal?{signal}:{})})).data;
}
