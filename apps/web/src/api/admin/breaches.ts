import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export type BreachStatus = 'detected' | 'assessing' | 'contained' | 'notified-authority' | 'notified-subjects' | 'closed' | 'not-notifiable';
export type BreachSeverity = 'unassessed' | 'none' | 'risk' | 'high';
export type ClockPhase = 'not-started' | 'running' | 'overdue' | 'stopped-on-time' | 'stopped-late' | 'not-required';

export interface BreachClock {
  phase: ClockPhase;
  awareAt: number | null;
  deadlineAt: number | null;
  /** Seconds to the deadline; negative once overdue. */
  remainingSecs: number | null;
  elapsedSecs: number | null;
  notifiedAt: number | null;
}

export interface BreachSummary {
  id: string;
  reference: string;
  title: string;
  domain: string;
  status: BreachStatus;
  severity: BreachSeverity;
  source: string;
  type: string;
  detectedAt: number;
  awareAt: number | null;
  deadlineAt: number | null;
  assignee: string | null;
  updatedAt: number;
  clock: BreachClock;
  authorityNotifiedAt: number | null;
  affectedCount: number;
  draft: boolean;
}

export interface BreachStats {
  total: number;
  open: number;
  overdue: number;
  dueSoon: number;
  drafts: number;
  notifiedLate: number;
  byStatus: Record<string, number>;
  bySeverity: Record<string, number>;
  created30d: { t: number; count: number }[];
}

export interface BreachList {
  incidents: BreachSummary[];
  matched: number;
  stats: BreachStats;
}

export interface BreachEntry { id: string; ts: number; actor: string; kind: string; text: string }
export interface BreachEvidence { id: string; ts: number; actor: string; kind: 'text' | 'link'; label: string; value: string }
export interface BreachAuthorityRecord { id: string; phase: string; recordedAt: number; recordedBy: string; submittedAt: number; reference: string; late: boolean; delayReasons: string }
export interface BreachAffected { address: string; accountId: string | null }

export interface BreachBody {
  incidentType: string;
  description: string;
  reporter: string;
  subjectCategories: string[];
  dataCategories: string[];
  approxSubjects: number | null;
  approxRecords: number | null;
  consequences: string;
  measuresTaken: string;
  measuresProposed: string;
  answers: Record<string, string>;
  assessment: { score: number; suggested: string; chosen: string; rationale: string; by: string; at: number; complete: boolean } | null;
  affected: BreachAffected[];
  affectedDomains: string[];
  timeline: BreachEntry[];
  evidence: BreachEvidence[];
  notNotifiableReason: string;
  closeReason: string;
  delayReasons: string;
  authorityNotices: BreachAuthorityRecord[];
  subjectNotices: { id: string; ts: number; by: string; audience: string; recipients: number; dryRun: boolean }[];
  escalatedAt: number | null;
  auto: Record<string, unknown> | null;
}

export interface BreachSuggestion { score: number; level: BreachSeverity; complete: boolean; answered: number; total: number }
export interface PlaybookItem { id: string; done: boolean; by: string; at: number }

export interface BreachIncident extends BreachSummary {
  rev: number;
  createdAt: number;
  body: BreachBody;
  suggestion: BreachSuggestion;
  playbook: PlaybookItem[];
  noticeHours: number;
  delayReasonsRequired: boolean;
  overdue: boolean;
}

export interface BreachTrail { id: string; ts: number; actor: string; kind: string; detail: Record<string, unknown> | null }
export interface BreachDelivery { id: string; noticeId: string; recipient: string; channel: string; status: string; detail: string; ts: number }
export interface BreachDetail { incident: BreachIncident; trail: BreachTrail[]; deliveries: BreachDelivery[] }

export interface BreachMeta {
  statuses: BreachStatus[];
  sources: string[];
  severities: BreachSeverity[];
  types: string[];
  phases: string[];
  questions: { id: string; options: string[] }[];
  variables: string[];
  defaults: { subject: Record<string, string>; body: Record<string, string> };
  enabled: boolean;
  noticeHours: number;
}

export interface BreachFilter { status?: string; severity?: string; domain?: string; q?: string; due?: string; year?: string }

export interface NewBreach {
  title: string;
  description?: string;
  incidentType?: string;
  source?: string;
  domain?: string;
  draft?: boolean;
  awareAt?: number;
  detectedAt?: number;
  affected?: string[];
  dataCategories?: string[];
  approxSubjects?: number | null;
}

export interface Audience { kind: 'all' | 'selected' | 'domain'; addresses?: string[]; domain?: string }
export interface SubjectNoticePlan {
  dryRun?: boolean;
  recipients: { address: string; accountId: string | null; locale: string; channels: string[]; skipped: { channel: string; reason: string }[] }[];
  channels: string[];
  preview: { subject: string; body: string; locale: string; unknownVariables: { subject: string[]; body: string[] } };
  noticeId?: string;
}
export interface LockdownResult { dryRun: boolean; action: string; results: { address: string; status: string; sessions?: number; detail?: unknown }[] }

const qs = (f: object): string => {
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(f)) if (v) p.set(k, String(v));
  const s = p.toString();
  return s ? `?${s}` : '';
};
const enc = encodeURIComponent;
const post = async <T>(id: string, action: string, body: object = {}): Promise<T> =>
  (await adminRequest<T>(`/breaches/${enc(id)}/${action}`, { method: 'POST', body })).data;
const incident = async (id: string, action: string, body: object = {}): Promise<BreachIncident> =>
  (await post<{ incident: BreachIncident }>(id, action, body)).incident;

export const listBreaches = async (f: BreachFilter = {}, signal?: AbortSignal): Promise<BreachList> =>
  (await adminRequest<BreachList>(`/breaches${qs(f)}`, signal ? { signal } : {})).data;
export const runBreachPass = async (): Promise<void> => { await adminRequest<{ ok: boolean }>('/breaches/run', { method: 'POST', body: {} }); };
export const breachMeta = async (): Promise<BreachMeta> => (await adminRequest<BreachMeta>('/breaches/meta')).data;
export const loadBreach = async (id: string): Promise<BreachDetail> => (await adminRequest<BreachDetail>(`/breaches/${enc(id)}`)).data;
export const createBreach = async (input: NewBreach): Promise<BreachIncident> =>
  (await adminRequest<{ incident: BreachIncident }>('/breaches', { method: 'POST', body: input })).data.incident;

export const updateBreach = (id: string, details: Partial<Pick<BreachBody, 'description' | 'consequences' | 'measuresTaken' | 'measuresProposed' | 'delayReasons' | 'reporter'>> & { title?: string; incidentType?: string; domain?: string; subjectCategories?: string[]; dataCategories?: string[]; approxSubjects?: number | null; approxRecords?: number | null; affected?: string[]; affectedDomains?: string[] }) => incident(id, 'details', details);
export const assignBreach = (id: string, assignee: string | null) => incident(id, 'assign', { assignee });
export const confirmBreach = (id: string, awareAt?: number) => incident(id, 'confirm', awareAt === undefined ? {} : { awareAt });
export const changeAwareAt = (id: string, awareAt: number, reason: string) => incident(id, 'aware-at', { awareAt, reason });
export const assessBreach = (id: string, answers: Record<string, string>, severity: BreachSeverity | undefined, rationale: string) => incident(id, 'assess', { answers, ...(severity ? { severity } : {}), rationale });
export const moveBreach = (id: string, status: BreachStatus, reason: string) => incident(id, 'status', { status, reason });
export const addTimeline = (id: string, kind: string, text: string) => incident(id, 'timeline', { kind, text });
export const addEvidence = (id: string, kind: 'text' | 'link', label: string, value: string) => incident(id, 'evidence', { kind, label, value });
export const recordAuthorityNotice = (id: string, input: { phase?: string; submittedAt?: number; reference?: string; delayReasons?: string }) => incident(id, 'authority-notice', input);
export const checkPlaybook = (id: string, item: string, done: boolean) => incident(id, 'playbook', { item, done });
export const previewSubjectNotice = async (id: string, input: { audience: Audience; locale?: string; advice?: string; subjectTemplate?: string; bodyTemplate?: string }): Promise<SubjectNoticePlan> =>
  post<SubjectNoticePlan>(id, 'subject-notice', { ...input, dryRun: true });
export const sendSubjectNotice = async (id: string, input: { audience: Audience; locale?: string; advice?: string; subjectTemplate?: string; bodyTemplate?: string }): Promise<SubjectNoticePlan> =>
  post<SubjectNoticePlan>(id, 'subject-notice', { ...input, dryRun: false });
export const lockdown = (id: string, input: { action: 'revoke-sessions' | 'force-password-reset'; dryRun: boolean; confirm?: boolean; addresses?: string[] }): Promise<LockdownResult> =>
  post<LockdownResult>(id, 'lockdown', input);
export const authorityNotice = async (id: string, phase?: string): Promise<{ notice: Record<string, unknown>; phase: string }> =>
  (await adminRequest<{ notice: Record<string, unknown>; phase: string }>(`/breaches/${enc(id)}/authority-notice${qs({ phase })}`)).data;

/** A file fetched with the admin session (a plain link would skip the session header). */
export async function fetchBreachFile(path: string): Promise<string> {
  const { adminUrl } = await import('../adminPath.ts');
  const { basePath } = await import('../basePath.ts');
  const { apiFetch } = await import('../http.ts');
  const r = await apiFetch(adminUrl(path, basePath()), { credentials: 'same-origin' });
  if (!r.ok) throw new Error(`download failed (${r.status})`);
  return r.text();
}
export const registerFile = (year: string, format: 'csv' | 'json' | 'html'): Promise<string> => fetchBreachFile(`/breaches/export${qs({ year, format })}`);
export const reportFile = (id: string, format: 'html' | 'json' | 'csv', locale = 'en'): Promise<string> => fetchBreachFile(`/breaches/${enc(id)}/report${qs({ format, locale })}`);
export const authorityFile = (id: string, format: 'html' | 'json-file', phase?: string, locale = 'en'): Promise<string> => fetchBreachFile(`/breaches/${enc(id)}/authority-notice${qs({ format, phase, locale })}`);
