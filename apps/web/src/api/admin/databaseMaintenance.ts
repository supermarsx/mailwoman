import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
import type {BackgroundDatabaseReport} from './database.ts';

export interface DatabaseMaintenanceReport extends BackgroundDatabaseReport {
 backend:'sqlite'|'postgres';
 cachedBodyExpiry?:CachedBodyExpiryReport|null;
 changeLogRetention?:{at:number;status:string;scannedRows:number;removedRows:number;oversizedBatches:number;elapsedMs:number}|null;
}

export interface CachedBodyExpiryReport {
 status:string;at:number;cutoffAt:number;scannedMessages:number;initializedMessages:number;
 detachedMessages:number;removedBodies:number;removedBytes:number;protectedMessages:number;oversizedBodies?:number;cursor:{scan:string;expire:string};elapsedMs:number;
}

export async function loadDatabaseMaintenance(signal?:AbortSignal):Promise<DatabaseMaintenanceReport>{
 return (await adminRequest<DatabaseMaintenanceReport>('/maintenance/database-background',signal?{signal}:{})).data;
}
