// Admin client of the compliance routes (`crates/mw-server/src/compliance*.rs`).

import { apiFetch } from '../http.ts';
import { adminUrl } from '../adminPath.ts';
import { basePath } from '../basePath.ts';
import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export type Lang = 'en' | 'pt-PT';
export type Document = 'privacy' | 'cookies';

export interface PolicySection { id: string; title: string; body: string; overridden: boolean; missing: string[] }
export interface PolicyChange { key: string; old: unknown; new: unknown }
export interface PolicyVersion { version: number; kind: 'auto' | 'published'; createdBy: string; createdAt: number; changes: PolicyChange[] | null }
export interface PolicyView {
  domain: string; lang: Lang; document: Document; langs: Lang[];
  sections: PolicySection[]; published: PolicySection[] | null;
  warnings: string[]; version: number | null; publishedAt: number | null; pending: PolicyChange[];
  autoVersion: boolean; versions: PolicyVersion[];
  overrides: { lang: Lang; section: string; body: string }[];
}

export interface RecordRow {
  id: string; builtin: boolean; enabled: boolean; name: string; kind?: 'activity' | 'processor'; source?: string;
  purpose?: string; legalBasis?: string; subjects?: string; categories?: string; recipients?: string;
  transfers?: string; retention?: string; security?: string; location?: string; safeguards?: string; url?: string; notes?: string;
}

export interface Risk {
  title: string; description: string; mitigations: string;
  likelihood: number; severity: number; residualLikelihood?: number | null; residualSeverity?: number | null;
  score?: number | null; level?: string | null; residualScore?: number | null; residualLevel?: string | null;
}
export interface DpiaBody {
  feature: string; activityId: string; description: string; purposes: string; necessity: string; proportionality: string;
  risks: Risk[]; mitigations: string; dpoOpinion: string; consultation: string; notes: string;
  links: { label: string; url: string; text: string }[];
}
export type DpiaStatus = 'draft' | 'review' | 'approved' | 'needs-prior-consultation';
export interface Dpia {
  id: string; title: string; status: DpiaStatus; template: string; version: number; reviewAt: number | null;
  due: boolean; overdue: boolean; createdBy: string; updatedBy: string; createdAt: number; updatedAt: number; body: DpiaBody;
  history?: { version: number; status: string; changedBy: string; at: number }[];
}
export interface ComplianceSummary {
  policy: { version: number | null; pending: number; missing: string[] };
  records: { custom: number; processors: number; builtin: number };
  dpia: { total: number; byStatus: Record<string, number>; due: number; suggestions: number };
}
export interface Suggestion { feature: string; template: string; setting: string }

const query = (params: Record<string, string | undefined>): string => {
  const s = new URLSearchParams(Object.entries(params).filter((e): e is [string, string] => e[1] !== undefined)).toString();
  return s === '' ? '' : `?${s}`;
};

async function text(path: string): Promise<string> {
  const r = await apiFetch(adminUrl(path, basePath()), { credentials: 'same-origin' });
  if (!r.ok) throw new Error(`export failed (${r.status})`);
  return r.text();
}

export const complianceApi = {
  async summary(): Promise<ComplianceSummary> { return (await adminRequest<ComplianceSummary>('/compliance/summary')).data; },
  async policy(lang: Lang, document: Document, domain = ''): Promise<PolicyView> { return (await adminRequest<PolicyView>(`/compliance/policy${query({ lang, document, domain })}`)).data; },
  async saveOverride(lang: Lang, section: string, body: string | null, domain = ''): Promise<void> { await adminRequest('/compliance/policy/overrides', { method: 'PUT', body: { domain, lang, section, body } }); },
  async publish(domain = ''): Promise<{ version: number | null; changed: boolean }> { return (await adminRequest<{ version: number | null; changed: boolean }>('/compliance/policy/publish', { method: 'POST', body: { domain } })).data; },
  async records(lang: Lang): Promise<{ activities: RecordRow[]; processors: RecordRow[] }> { return (await adminRequest<{ activities: RecordRow[]; processors: RecordRow[] }>(`/compliance/records${query({ lang })}`)).data; },
  async addRecord(kind: 'activity' | 'processor', name: string, fields: Record<string, string>, enabled = true): Promise<RecordRow> { return (await adminRequest<RecordRow>('/compliance/records', { method: 'POST', body: { kind, name, fields, enabled } })).data; },
  async updateRecord(id: string, kind: 'activity' | 'processor', name: string, fields: Record<string, string>, enabled = true): Promise<RecordRow> { return (await adminRequest<RecordRow>(`/compliance/records/${encodeURIComponent(id)}`, { method: 'PUT', body: { kind, name, fields, enabled } })).data; },
  async deleteRecord(id: string): Promise<void> { await adminRequest(`/compliance/records/${encodeURIComponent(id)}`, { method: 'DELETE' }); },
  exportRecords(set: 'activities' | 'processors' | 'contract' | 'transfer' | 'control', format: 'csv' | 'json' | 'html', lang: Lang): Promise<string> { return text(`/compliance/records/export${query({ set, format, lang })}`); },
  async dpias(): Promise<Dpia[]> { return (await adminRequest<{ items: Dpia[] }>('/compliance/dpia')).data.items; },
  async dpia(id: string): Promise<Dpia> { return (await adminRequest<Dpia>(`/compliance/dpia/${encodeURIComponent(id)}`)).data; },
  async createDpia(title: string, template: string, lang: Lang): Promise<Dpia> { return (await adminRequest<Dpia>('/compliance/dpia', { method: 'POST', body: { title, template, lang } })).data; },
  async saveDpia(id: string, version: number, change: { title?: string; status?: DpiaStatus; body?: DpiaBody; reviewAt?: number }): Promise<Dpia> { return (await adminRequest<Dpia>(`/compliance/dpia/${encodeURIComponent(id)}`, { method: 'PUT', body: { version, ...change } })).data; },
  async deleteDpia(id: string): Promise<void> { await adminRequest(`/compliance/dpia/${encodeURIComponent(id)}`, { method: 'DELETE' }); },
  async suggestions(): Promise<Suggestion[]> { return (await adminRequest<{ items: Suggestion[] }>('/compliance/dpia/suggestions')).data.items; },
  exportDpia(id: string, format: 'html' | 'json'): Promise<string> { return text(`/compliance/dpia/${encodeURIComponent(id)}/export${query({ format })}`); },
};
export type ComplianceApi = typeof complianceApi;

/** Offer `content` as a download. */
export function saveText(content: string, filename: string, type: string): void {
  const url = URL.createObjectURL(new Blob([content], { type }));
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
