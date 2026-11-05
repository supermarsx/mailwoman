import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
import type { AdminSetting } from './settings.ts';
export interface ProtectionCounts { normal:number; anomalous:number }
export interface ProtectionSnapshot { short:ProtectionCounts; long:ProtectionCounts; shortWindowSecs:number; longWindowSecs:number; activeUntil:number; stage:number; lastAnomaly:number|null; lastSignal:string|null; rateCount:number; rateWindowSecs:number; countryCode?:string|null; sourceIp?:string|null }
export interface ProtectionSubject { subjectKey:string;subjectKind:'client'|'account'|'admin'|`ua-${string}`;updatedAt:number;expiresAt:number;revision:string;snapshot:ProtectionSnapshot }
export interface ProtectionEvent { id:string;subjectKey:string;subjectKind:string;observedAt:number;reason:string;action:string;enforced:boolean;retryAfterSecs:number;snapshot:ProtectionSnapshot;policy:{short:{seconds:number;minimum_samples:number;absolute_anomalies:number;ratio_percent:number};long:{seconds:number;minimum_samples:number;absolute_anomalies:number;ratio_percent:number}}|null }
export interface ProtocolGuardRow { class:string;mode:'off'|'detect'|'enforce';detected:number;enforced:number;lastSeen:number|null }
export interface ProtectionStatus { protocolGuard?:{classes:ProtocolGuardRow[];scope:string};policy:{rateMode:string;adaptiveMode:string;clientRequests:number;accountRequests:number;adminRequests:number;rateWindowSecs:number};subjectCapacity:number;eventCapacity:number;eventRetentionSecs:number;propagationSecs:number;coverage:string;preRouterHeadersObserved:boolean }
export type ProtectionRange = '15m' | '1h' | '6h' | '24h';
export const PROTECTION_RANGES: readonly ProtectionRange[] = ['15m', '1h', '6h', '24h'];
export interface ProtectionActionCounts { throttleObserved: number; throttleEnforced: number; blockObserved: number; blockEnforced: number }
export interface ProtectionHistoryPoint { t: number; allowed: number; throttled: number; blocked: number; actions: ProtectionActionCounts }
export interface ProtectionOutcomes { allowed: number; throttled: number; blocked: number }
/** Counts only (no address, account or user-agent text); kept in memory per minute for 24 hours since the server started. */
export interface ProtectionHistory {
 rangeSecs: number; stepSecs: number; start: number; scope: string;
 points: ProtectionHistoryPoint[];
 totals: ProtectionOutcomes & { evaluated: number };
 budgets: (ProtectionOutcomes & { budget: 'client' | 'account' | 'admin' | 'userAgent' })[];
 userAgents: (ProtectionOutcomes & { class: string })[];
 tiers: { unknown: number; known: number; authenticated: number };
 actions: ProtectionActionCounts;
 rules: { rule: string; count: number }[];
}
export interface ProtectionApi {
 status():Promise<ProtectionStatus>;
 /** Newest first. `after` is the previous page's cursor; `total` counts every retained row. */
 subjects(after?:string,limit?:number):Promise<{subjects:ProtectionSubject[];cursor:string|null;total:number}>;
 events(after?:string,limit?:number):Promise<{events:ProtectionEvent[];cursor:string|null;total:number}>;
 clear(subject:ProtectionSubject):Promise<void>;
 /** `GET /bans/protection/history?range=`; absent on a mock or an older server. */
 history?(range:ProtectionRange):Promise<ProtectionHistory>;
}
/** `?limit=&after=` for the paged lists (the server caps `limit` at 250). */
export function pageQuery(after?:string,limit?:number):string{
 const params=new URLSearchParams();if(limit)params.set('limit',String(limit));if(after)params.set('after',after);
 const text=params.toString();return text?`?${text}`:'';
}
export const protectionApi:ProtectionApi={
 async status(){return(await adminRequest<ProtectionStatus>('/bans/protection')).data;},
 async subjects(after,limit){return(await adminRequest<{subjects:ProtectionSubject[];cursor:string|null;total:number}>(`/bans/protection/subjects${pageQuery(after,limit)}`)).data;},
 async events(after,limit){return(await adminRequest<{events:ProtectionEvent[];cursor:string|null;total:number}>(`/bans/protection/events${pageQuery(after,limit)}`)).data;},
 async history(range){return(await adminRequest<ProtectionHistory>(`/bans/protection/history?range=${range}`)).data;},
 async clear(subject){await adminRequest(`/bans/protection/subjects/${encodeURIComponent(subject.subjectKey)}`,{method:'DELETE',ifMatch:`"${subject.revision}"`});},
};

// ── t97: the policy editor and the simulator ──────────────────────────────────
//   GET  {console}/bans/protection/policy   → { rules: AdminSetting[], presets, canEdit, domainScopes }
//   PUT  {console}/bans/protection/policy   ← { changes: [{key, value|null, etag}], preset? } (step-up)
//   POST {console}/bans/protection/simulate ← { subjectKind, subjectValue?, sample, draft? }
export type PresetName = 'relaxed' | 'balanced' | 'strict';
export interface ProtectionPolicy { rules: AdminSetting[]; presets: Record<PresetName, Record<string, unknown>>; canEdit: boolean; domainScopes: boolean }
export interface PolicyChange { key: string; value: unknown; etag?: string | undefined }
export type SimSignal = 'normal' | 'malformed' | 'oversized' | 'credential-rejected' | 'permission-denied' | 'body-deadline' | 'admin-route-miss' | 'probe-path' | 'bot-user-agent';
export interface SimSample { requestsInWindow: number; short: ProtectionCounts; long: ProtectionCounts; signal: SimSignal; previouslyThrottled: boolean }
export type SimTierName = 'authenticated' | 'known' | 'unknown';
export type SimUaClass = 'normal' | 'missing' | 'uncommon' | 'automation' | 'search' | 'ai-training' | 'ai-search' | 'ai-user' | 'custom';
export type SimCountryState = 'found' | 'notFound' | 'noDatabase' | 'local';
export interface SimTier { tier: SimTierName; uaClass?: SimUaClass; country?: string; countryState?: SimCountryState; adminProbe?: boolean }
export interface SimRequest { subjectKind: 'client' | 'account' | 'admin'; subjectValue?: string; sample: SimSample; draft?: Record<string, unknown>; userAgent?: string; tier?: SimTier }
export type SimAdminProbeResult = 'disabled' | 'notApplicable' | 'localNetwork' | 'knownCountry' | 'unresolvedNotBlocked' | 'counted';
export interface SimTierResult {
  tier: SimTierName; percent: number;
  requestBudget: { base: number; effective: number; windowSecs: number } | null;
  userAgentBudget: { base: number; effective: number; windowSecs: number } | null;
  signinCooldown: { baseSecs: number; effectiveSecs: number; heightenedSecs: number | null };
  challenge: { baseDifficulty: number; effectiveDifficulty: number; forcedByHeightened: boolean };
  userAgentAction: { class: string; configured: 'allow' | 'limit' | 'block'; effective: 'allow' | 'limit' | 'block'; signedInNeverBlocked: boolean } | null;
  heightened: { on: boolean; forcesChallenge: boolean; pacePercent: number | null };
  adminProbe: { enabled: boolean; threshold: number; windowSecs: number; blockSecs: number; cause: string; result: SimAdminProbeResult } | null;
}
export type SimRuleId = 'request-rate' | 'short-absolute' | 'short-ratio' | 'long-absolute' | 'long-ratio' | 'probe-path' | 'bot-user-agent';
export interface SimResult { tier?: SimTierResult | null; userAgent?: { classification: string | null; claimedOnly: true; exempt: boolean }; action: 'allow' | 'throttle' | 'block'; reason: SimRuleId | null; enforced: boolean; durationSecs: number; signalCounted: boolean; off: boolean; exempt: boolean; rules: { rule: SimRuleId; enabled: boolean; value: number; threshold: number; fired: boolean }[] }
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
