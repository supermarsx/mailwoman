export interface BackgroundDatabaseReport {
 running:boolean;enabled?:boolean;nextRunAt?:number|null;lastError?:string|null;
 options?:Record<string,number|boolean>;
 lastReport?:{at:number;status:string;vacuumStatus:string;scanned:number;marked:number;removed:number;removedBytes:number;reclaimedPages:number;autoVacuum?:number;elapsedMs:number;cachedBodyExpiry?:import('./databaseMaintenance.ts').CachedBodyExpiryReport;walRecovery?:{beforeBytes:number;afterBytes:number;status:string}}|null;
}
import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export interface DatabaseOperationSnapshot {calls:number;failures:number;totalMicros:number;maxMicros:number}
export interface DatabaseCleanupReceipt {
 removedBodies:number;removedBodyBytes:number;cleanedEnvelopes:number;removedEnvelopeBytes:number;
 removedChangeRows:number;reclaimedPages:number;retiredFileBytes:number;
}
export interface DatabaseReport {
 observedAt:string;driver:string;
 backgroundMaintenance?:BackgroundDatabaseReport;
 growth?:{status:string;bytesPerMinute:number|null;thresholdBytesPerMinute:number;windowMinutes:number;
  basis?:'occupied-pages'|'database-allocation';knownCleanupTotal?:DatabaseCleanupReceipt;
  changes?:{from:number;to:number;seconds:number;occupiedBytes:number|null;dbBytes:number|null;walBytes:number|null;freeBytes:number|null;knownCleanup:DatabaseCleanupReceipt|null}|null;
  samples:{at:number;dbBytes:number|null;walBytes:number|null;occupiedBytes?:number|null;freeBytes?:number|null;segment?:number;breakReason?:string|null}[];
  writeOperations?:{operation:string;statements:number;affectedRows:number;inputBytes:number}[];
  writeOrigins:{origin:string;statements:number;affectedRows:number;inputBytes:number}[]};
 database:{backend:string;serverVersion:string|null;name:string|null;host:string|null;port:number|null;journalMode:string|null};
 diagnostics:{dbBytes:number|null;walBytes:number|null;dbPingMs:number;migrationApplied:number;migrationExpected:number};
 pool:{open:number;idle:number;busy:number;minimum:number;maximum:number;acquireTimeoutMs:number;idleTimeoutMs:number|null;maxLifetimeMs:number|null;closed:boolean;scaling?:string;ceilingChangesRequireRestart?:boolean;dynamicCeilingResize?:boolean;telemetryScope?:string;operations?:Record<'read'|'write'|'other'|'acquire',DatabaseOperationSnapshot>};
 storage:{pageSize?:number;pageCount?:number;freePages?:number;freeBytes?:number;occupiedBytes?:number;autoVacuum?:number;synchronous?:number;busyTimeoutMs?:number;cacheSize?:number;foreignKeys?:number;mmapBytes?:number;tls?:boolean|null};
 migrations:{applied:number;expected:number;pending:unknown[];failed:unknown[];unknown:unknown[]};
 details:null|{observedAt:string;limited:boolean;notes:string[];bodies:null|{count:number|null;countLowerBound?:boolean;bytes:number|null;bytesLowerBound?:boolean;sampleRows?:number;orphanCount:number|null;orphanBytes:number|null;liveCount?:number|null;liveBytes?:number|null};tables:{name:string;bytes:number|null;rows:number|null;rowsEstimated:boolean;rowsLowerBound?:boolean;growthBytes?:number;growthSince?:string;payloadBytes?:number|null;payloadBytesLowerBound?:boolean;payloadSampleRows?:number;measurementSource?:string;indexBytes?:number|null;columns?:{name:string;bytes:number;bytesLowerBound?:boolean}[];deadRows?:number}[]};
}
export async function loadDatabaseReport(detailed=false):Promise<DatabaseReport>{
 return (await adminRequest<DatabaseReport>(`/observability/database${detailed?'?detailed=true':''}`)).data;
}
