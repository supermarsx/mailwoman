import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
import type { AdminSetting } from './settings.ts';
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

// ── t97: the policy editor and the simulator ──────────────────────────────────
//   GET  {console}/bans/protection/policy   → { rules: AdminSetting[], presets, canEdit, domainScopes }
//   PUT  {console}/bans/protection/policy   ← { changes: [{key, value|null, etag}], preset? } (step-up)
//   POST {console}/bans/protection/simulate ← { subjectKind, subjectValue?, sample, draft? }
export type PresetName = 'relaxed' | 'balanced' | 'strict';
export interface ProtectionPolicy { rules: AdminSetting[]; presets: Record<PresetName, Record<string, unknown>>; canEdit: boolean; domainScopes: boolean }
export interface PolicyChange { key: string; value: unknown; etag?: string | undefined }
export type SimSignal = 'normal' | 'malformed' | 'oversized' | 'credential-rejected' | 'permission-denied' | 'body-deadline' | 'admin-route-miss';
export interface SimSample { requestsInWindow: number; short: ProtectionCounts; long: ProtectionCounts; signal: SimSignal; previouslyThrottled: boolean }
export interface SimRequest { subjectKind: 'client' | 'account' | 'admin'; subjectValue?: string; sample: SimSample; draft?: Record<string, unknown> }
export type SimRuleId = 'request-rate' | 'short-absolute' | 'short-ratio' | 'long-absolute' | 'long-ratio';
export interface SimResult { action: 'allow' | 'throttle' | 'block'; reason: SimRuleId | null; enforced: boolean; durationSecs: number; signalCounted: boolean; off: boolean; exempt: boolean; rules: { rule: SimRuleId; enabled: boolean; value: number; threshold: number; fired: boolean }[] }
export interface ProtectionPolicyApi {
 policy(): Promise<ProtectionPolicy>;
 save(changes: PolicyChange[], preset?: PresetName): Promise<AdminSetting[]>;
 simulate(request: SimRequest): Promise<SimResult>;
}
export const protectionPolicyApi: ProtectionPolicyApi = {
 async policy(){return(await adminRequest<ProtectionPolicy>('/bans/protection/policy')).data;},
 async save(changes,preset){return(await adminRequest<{rules:AdminSetting[]}>('/bans/protection/policy',{method:'PUT',body:{changes,...(preset?{preset}:{})}})).data.rules;},
 async simulate(request){return(await adminRequest<SimResult>('/bans/protection/simulate',{method:'POST',body:request})).data;},
};
