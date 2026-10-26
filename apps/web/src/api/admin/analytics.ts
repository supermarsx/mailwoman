import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface AnalyticsConfig { enabled:boolean;provider:'umami'|'matomo'|'ga4';endpoint:string;siteId:string;allowHttp:boolean;apiSecretConfigured?:boolean }
export type AnalyticsWrite = Omit<AnalyticsConfig,'apiSecretConfigured'> & {apiSecret?:string;clearApiSecret?:boolean};
export interface AnalyticsView { config:AnalyticsConfig;status:{queued:number;delivered:number;dropped:number;mode:'event-counts';uniqueVisitors:null};allowlist:{id:string;destination:string}[] }
export interface AnalyticsApi {
  read():Promise<{data:AnalyticsView;etag:string}>;
  save(config:AnalyticsWrite,etag:string):Promise<{data:AnalyticsView;etag:string}>;
  allow(destination:string):Promise<void>;
  remove(id:string):Promise<void>;
}
export const analyticsApi:AnalyticsApi = {
  async read() { const result=await adminRequest<AnalyticsView>('/analytics'); return {data:result.data,etag:result.etag??''}; },
  async save(config,etag) { const result=await adminRequest<AnalyticsView>('/analytics',{method:'PUT',body:config,ifMatch:etag}); return {data:result.data,etag:result.etag??''}; },
  async allow(destination) { await adminRequest('/analytics/allowlist',{method:'POST',body:{destination}}); },
  async remove(id) { await adminRequest(`/analytics/allowlist/${encodeURIComponent(id)}`,{method:'DELETE'}); },
};
