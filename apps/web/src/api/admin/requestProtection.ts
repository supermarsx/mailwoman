import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface ProtectionCounts { normal:number; anomalous:number }
export interface ProtectionSnapshot { short:ProtectionCounts; long:ProtectionCounts; shortWindowSecs:number; longWindowSecs:number; activeUntil:number; stage:number; lastAnomaly:number|null; lastSignal:string|null; rateCount:number; rateWindowSecs:number }
export interface ProtectionSubject { subjectKey:string;subjectKind:'client'|'account'|'admin';updatedAt:number;expiresAt:number;revision:string;snapshot:ProtectionSnapshot }
export interface ProtectionEvent { id:string;subjectKey:string;subjectKind:string;observedAt:number;reason:string;action:string;enforced:boolean;retryAfterSecs:number;snapshot:ProtectionSnapshot;policy:{short:{seconds:number;minimum_samples:number;absolute_anomalies:number;ratio_percent:number};long:{seconds:number;minimum_samples:number;absolute_anomalies:number;ratio_percent:number}}|null }
export interface ProtectionStatus { policy:{rateMode:string;adaptiveMode:string;clientRequests:number;accountRequests:number;adminRequests:number;rateWindowSecs:number};subjectCapacity:number;eventCapacity:number;eventRetentionSecs:number;propagationSecs:number;coverage:string;preRouterHeadersObserved:boolean }
export interface ProtectionApi {
 status():Promise<ProtectionStatus>;
 subjects(after?:string):Promise<{subjects:ProtectionSubject[];cursor:string|null}>;
 events(after?:string):Promise<{events:ProtectionEvent[];cursor:string|null}>;
 clear(subject:ProtectionSubject):Promise<void>;
}
export const protectionApi:ProtectionApi={
 async status(){return(await adminRequest<ProtectionStatus>('/bans/protection')).data;},
 async subjects(after){return(await adminRequest<{subjects:ProtectionSubject[];cursor:string|null}>(`/bans/protection/subjects${after?`?after=${encodeURIComponent(after)}`:''}`)).data;},
 async events(after){return(await adminRequest<{events:ProtectionEvent[];cursor:string|null}>(`/bans/protection/events${after?`?after=${encodeURIComponent(after)}`:''}`)).data;},
 async clear(subject){await adminRequest(`/bans/protection/subjects/${encodeURIComponent(subject.subjectKey)}`,{method:'DELETE',ifMatch:`"${subject.revision}"`});},
};
