// Admin > Observability > Client errors: browser error reports the web app sent
// (`GET /admin/client-errors`, `observability.read`). Resolve, ignore, reopen and
// delete need `settings.write`.
import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';

export type ClientErrorStatus='open'|'resolved'|'ignored';
export interface ClientErrorGroup {
 id:string;fingerprint:string;kind:string;name:string;message:string;boundary:string;routeTemplate:string;
 firstSeen:number;lastSeen:number;count:number;lastBuild:string;status:ClientErrorStatus;resolvedBuild:string|null;
 reopenedCount:number;
 /** Distinct page visits among the stored events: an estimate of affected sessions. */
 sessions:number;
 /** Hourly `[hourStart, occurrences]` pairs; present on the first 100 groups. */
 series?:[number,number][];
}
export interface ClientErrorList {
 groups:ClientErrorGroup[];since:number;
 policy:{enabled:boolean;sourceMaps:boolean;forward:boolean;includeAccountHash:boolean};
}
export interface ClientErrorEvent {
 id:string;time:number;build:string;occurrences:number;stackRaw:string;stackSymbolicated:string|null;
 uaFamily:string;osFamily:string;viewport:string;locale:string;accountHash:string|null;
 extra:{route?:string;uaVersion?:string;online?:boolean;reducedMotion?:boolean;theme?:string;componentStack?:string}|null;
}
export interface ClientErrorDetail {
 group:ClientErrorGroup;events:ClientErrorEvent[];series:[number,number][];
 breakdown:Record<'browser'|'os'|'build'|'viewport'|'route',[string,number][]>;
}
export interface ClientErrorFilter {status?:ClientErrorStatus|'';build?:string;boundary?:string;rangeHours?:number}

const qs=(f:ClientErrorFilter):string=>{
 const q=new URLSearchParams();
 if(f.status)q.set('status',f.status);
 if(f.build)q.set('build',f.build);
 if(f.boundary)q.set('boundary',f.boundary);
 if(f.rangeHours)q.set('rangeHours',String(f.rangeHours));
 const s=q.toString();return s?`?${s}`:'';
};
export const listClientErrors=async(f:ClientErrorFilter={},signal?:AbortSignal):Promise<ClientErrorList>=>(await adminRequest<ClientErrorList>(`/client-errors${qs(f)}`,signal?{signal}:{})).data;
export const loadClientError=async(id:string,signal?:AbortSignal):Promise<ClientErrorDetail>=>(await adminRequest<ClientErrorDetail>(`/client-errors/${encodeURIComponent(id)}`,signal?{signal}:{})).data;
export const setClientErrorStatus=async(id:string,status:ClientErrorStatus):Promise<void>=>{await adminRequest(`/client-errors/${encodeURIComponent(id)}/status`,{method:'POST',body:{status}});};
export const deleteClientError=async(id:string):Promise<void>=>{await adminRequest(`/client-errors/${encodeURIComponent(id)}`,{method:'DELETE'});};
