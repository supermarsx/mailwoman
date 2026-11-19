export interface BackgroundDatabaseReport {
 running:boolean;enabled?:boolean;nextRunAt?:number|null;lastError?:string|null;
 options?:Record<string,number|boolean>;
 lastReport?:{at:number;status:string;vacuumStatus:string;scanned:number;marked:number;removed:number;removedBytes:number;reclaimedPages:number;autoVacuum?:number;elapsedMs:number}|null;
}
import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export interface DatabaseReport {
 observedAt:string;driver:string;
 backgroundMaintenance?:BackgroundDatabaseReport;
 growth?:{status:string;bytesPerMinute:number|null;thresholdBytesPerMinute:number;windowMinutes:number;samples:{at:number;dbBytes:number;walBytes:number;occupiedBytes?:number}[];writeOrigins:{origin:string;statements:number;affectedRows:number;inputBytes:number}[]};
 database:{backend:string;serverVersion:string|null;name:string|null;host:string|null;port:number|null;journalMode:string|null};
 diagnostics:{dbBytes:number|null;walBytes:number|null;dbPingMs:number;migrationApplied:number;migrationExpected:number};
 pool:{open:number;idle:number;busy:number;minimum:number;maximum:number;acquireTimeoutMs:number;idleTimeoutMs:number|null;maxLifetimeMs:number|null;closed:boolean};
 storage:{pageSize?:number;pageCount?:number;freePages?:number;freeBytes?:number;occupiedBytes?:number;autoVacuum?:number;synchronous?:number;busyTimeoutMs?:number;cacheSize?:number;foreignKeys?:number;mmapBytes?:number;tls?:boolean|null};
 migrations:{applied:number;expected:number;pending:unknown[];failed:unknown[];unknown:unknown[]};
 details:null|{observedAt:string;limited:boolean;notes:string[];bodies:null|{count:number|null;bytes:number|null;orphanCount:number|null;orphanBytes:number|null;liveCount?:number|null;liveBytes?:number|null};tables:{name:string;bytes:number|null;rows:number|null;rowsEstimated:boolean;growthBytes?:number;growthSince?:string;payloadBytes?:number|null;indexBytes?:number|null;columns?:{name:string;bytes:number}[];deadRows?:number}[]};
}
export async function loadDatabaseReport(detailed=false):Promise<DatabaseReport>{
 return (await adminRequest<DatabaseReport>(`/observability/database${detailed?'?detailed=true':''}`)).data;
}
