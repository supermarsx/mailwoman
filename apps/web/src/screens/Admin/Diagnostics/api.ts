// t105: the diagnostics wire types and the HTTP client behind the Diagnostics screen.
//
// Endpoints (crates/mw-server/src/diagnostics*.rs, domain_probe.rs):
//   POST {console}/diag/run         the server suite (database, network, posture)
//   GET  {console}/diag/runs        saved history, newest first, cursor paged
//   GET  {console}/diag/catalogue   suites, limits, canRunBackends
//   POST {console}/diag/jobs        the saved-backends job (202, then polled)
//   GET  {console}/diag/jobs/{id}   the job snapshot
//   POST {console}/diag/jobs/{id}/cancel
//   GET  {console}/diag/report      the allowlisted troubleshooting report
//   POST {console}/diag/upstream    TLS + handshake of one saved domain's endpoints
//   POST {console}/diag/dns         MX / SPF / DMARC / SRV of one saved domain
// Rate limits, job dedupe and the short-429 retry are the server's and
// adminRequest's (t104); this client sends one request per click.

import { adminRead, adminRequest, type AdminReadResult } from '../shell/adminApi.ts';
import { domainsApi, type DnsReport, type ProbeReport } from '../../../api/admin/domains.ts';

export interface Check {
  id: string;
  status: string;
  durationMs: number;
  /** Exact time in μs (servers from 26.20); null for a check that never ran. */
  durationUs?: number | null;
  summary: string;
  domain?: string;
  backendId?: string;
  protocol?: string;
  category?: string;
  fixId?: string;
  /** Backend checks: the handshake detail (host, port, TLS, certificate, error). */
  detail?: Record<string, unknown>;
  termination?: string;
  omitted?: number;
}
export interface Run { id: string; suite: string; startedAt: number; finishedAt?: number; status: string; results: Check[]; triggeredBy?: string }
export interface History { runs: Run[]; nextCursor: string | null }
export interface Job { run: Run; total: number; completed: number; limited: boolean; historySaved: boolean; auditSaved?: boolean; termination?: string }
export interface Catalogue {
  canRunBackends?: boolean;
  limits?: { concurrency: number; checks: number; runSeconds: number; checkSeconds: number };
  suites: { id: string; supported: boolean; description?: string; reason?: string }[];
}
export interface ServerRunResult { run: Run; historySaved: boolean; auditSaved: boolean; retentionApplied?: boolean }
export interface ReportOptions { omitDomains?: boolean; selected?: string | undefined }
export interface DetectResult { name: string; source: string; requiresOAuth: boolean; upstream: unknown }

export interface DiagnosticsApi {
  history: (cursor?: string) => Promise<AdminReadResult<History>>;
  run: () => Promise<ServerRunResult>;
  start?: () => Promise<Job>;
  job?: (id: string) => Promise<Job>;
  cancel?: (id: string) => Promise<unknown>;
  report?: (options?: ReportOptions) => Promise<unknown>;
  catalogue?: () => Promise<Catalogue>;
  /** Saved domain names, for the per-domain tools. */
  domains?: () => Promise<{ name: string; enabled: boolean; failover: boolean; backends: number }[]>;
  probe?: (domain: string) => Promise<ProbeReport>;
  dns?: (domain: string) => Promise<DnsReport>;
  detect?: (name: string) => Promise<DetectResult>;
}

export const httpDiagnosticsApi: DiagnosticsApi = {
  history: (cursor) => adminRead<History>(`/diag/runs${cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''}`),
  run: async () => (await adminRequest<ServerRunResult>('/diag/run', { method: 'POST', body: { suite: 'server' } })).data,
  // The server answers the session's running job for a repeat (t104), so a resend is safe.
  start: async () => (await adminRequest<Job>('/diag/jobs', { method: 'POST', body: { suite: 'backends' }, idempotent: true })).data,
  job: async (id) => (await adminRequest<Job>(`/diag/jobs/${encodeURIComponent(id)}`)).data,
  cancel: async (id) => adminRequest(`/diag/jobs/${encodeURIComponent(id)}/cancel`, { method: 'POST', body: {} }),
  report: async (options) => {
    const query = new URLSearchParams();
    if (options?.omitDomains) query.set('omitDomains', 'true');
    if (options?.selected) query.set('selected', options.selected);
    const suffix = query.toString();
    return (await adminRequest<unknown>(`/diag/report${suffix ? `?${suffix}` : ''}`)).data;
  },
  catalogue: async () => (await adminRequest<Catalogue>('/diag/catalogue')).data,
  domains: async () => (await domainsApi.list()).map((d) => ({
    name: d.name,
    enabled: d.enabled,
    failover: d.upstream.failover === true,
    backends: d.upstream.version === 2 ? (d.upstream.backends ?? []).filter((b) => b.role === 'mail').length : (['imap', 'pop3', 'jmap'] as const).filter((p) => d.upstream[p]).length,
  })),
  probe: (domain) => domainsApi.probe(domain),
  dns: (domain) => domainsApi.dns(domain),
  detect: async (name) => await domainsApi.detect(name) as DetectResult,
};
