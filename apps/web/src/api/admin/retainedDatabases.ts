import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface RetainedDatabase {
 generationId:string;fileName:string;path:string;bytes:number;modifiedAt:number|null;
 fileIdentity:string;deletable:boolean;blockedReason:string|null;
}
export interface RetainedDatabaseReport {
 supported:boolean;activeGeneration:string|null;observedAt:number;retained:RetainedDatabase[];
}
export interface RetireDatabaseRequest {
 generationId:string;expectedActiveGeneration:string;fileIdentity:string;confirmation:string;
}
export const loadRetainedDatabases=async(signal?:AbortSignal):Promise<RetainedDatabaseReport>=>(await adminRequest<RetainedDatabaseReport>('/maintenance/database-retained',signal?{signal}:{})).data;
export const retireDatabase=async(request:RetireDatabaseRequest):Promise<{generationId:string;fileName:string;removedBytes:number}>=>(await adminRequest<{generationId:string;fileName:string;removedBytes:number}>('/maintenance/database-retained',{method:'DELETE',body:request})).data;
