import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface TelemetryConfig {
  enabled:boolean; endpoint:string; serviceName:string; environment:string; traces:boolean; metrics:boolean;
  sampleRatio:number; exportTimeoutMs:number; metricsIntervalSecs:number; spanQueueSize:number;
  spanBatchSize:number; spanExportDelayMs:number; prometheusEnabled:boolean;
}
export interface TelemetryView { config:TelemetryConfig; effective:TelemetryConfig; active:TelemetryConfig|null; pinned:string[]; restartRequired:boolean; metricsTokenConfigured:boolean }
export interface TelemetryApi { read():Promise<{data:TelemetryView;etag:string}>; save(config:TelemetryConfig,etag:string):Promise<{data:TelemetryView;etag:string}> }
export const telemetryApi:TelemetryApi = {
  async read() { const result=await adminRequest<TelemetryView>('/telemetry');return {data:result.data,etag:result.etag??''}; },
  async save(config,etag) { const result=await adminRequest<TelemetryView>('/telemetry',{method:'PUT',body:config,ifMatch:etag});return {data:result.data,etag:result.etag??''}; },
};
