import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export interface CpuCapabilities {
 architecture:string;
 availableFeatures:string[];
 selectedBackend:'scalar'|'avx2'|'avx512';
 avx2MinBytes:number;
 avx512MinBytes:number;
}

export interface ResourceSnapshot {
 cpuCapabilities?:CpuCapabilities;
 cpuUsage?:Record<'scalar'|'avx2'|'avx512',{calls:number;bytes:number}>;
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
