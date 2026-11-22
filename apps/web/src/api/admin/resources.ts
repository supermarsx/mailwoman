import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export const CPU_KERNELS=['scalar','sse2','sse3','ssse3','sse41','sse42','avx2','avx512'] as const;
export type CpuBackend=typeof CPU_KERNELS[number];

export interface CpuCapabilities {
 architecture:string;
 availableFeatures:string[];
 selectedBackend:CpuBackend;
 highestAvailableBackend?:CpuBackend;
 sseMinBytes?:number|null;
 avx2MinBytes:number;
 avx512MinBytes:number;
}

export interface ResourceSnapshot {
 cpuCapabilities?:CpuCapabilities;
 cpuUsage?:Partial<Record<CpuBackend,{calls:number;bytes:number}>>;
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
