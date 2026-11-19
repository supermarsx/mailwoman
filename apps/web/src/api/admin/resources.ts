import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export interface ResourceSnapshot {
 observedAt:number;
 effectiveCpuCores:number;
 effectiveMemoryLimitBytes:number|null;
 effectiveMemoryAvailableBytes:number|null;
 hostMemoryTotalBytes:number|null;
 hostMemoryAvailableBytes:number|null;
 processResidentBytes:number|null;
 processCpuPercent:number|null;
 cpuSampleSeconds:number|null;
 filesystemTotalBytes:number|null;
 filesystemAvailableBytes:number|null;
 cgroupDetected:boolean;
}

export async function loadResourceSnapshot(signal?:AbortSignal):Promise<ResourceSnapshot>{
 return (await adminRequest<ResourceSnapshot>('/observability/resources',signal?{signal}:{})).data;
}
