// Admin-probe protection (t100 server, t97 console). Server:
// `crates/mw-server/src/admin_probe/api.rs`. Reads need bans.read; writes need
// bans.write for every domain plus a fresh step-up (the shared dialog in
// api/http.ts answers it and retries).
//
//   GET    {console}/security/admin-probe              → AdminProbeOverview (+ ETag of the policy)
//   PUT    {console}/security/admin-probe/policy       ← { values: { <key>: value | null } }, If-Match
//   GET    {console}/security/admin-probe/blocks       → { blocks: AdminProbeSource[] }
//   DELETE {console}/security/admin-probe/blocks/{id}  → 204

import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export type AdminProbeSignal = 'probe-path' | 'slug-guess' | 'admin-signin-failed' | 'admin-unauthenticated';
export interface AdminProbePolicy {
  enabled: boolean;
  graceAllowance: number;
  windowSecs: number;
  blockAfter: number;
  tarpitDelayMs: number;
  blockSecs: number;
  maxBlockSecs: number;
  repeatWindowSecs: number;
  blockStatus: '429' | '403';
  signals: AdminProbeSignal[];
  probePaths: string[];
  allowlist: string[];
  auditIp: 'masked' | 'full';
  fail2banLog: boolean;
}
export type AdminProbeKey = keyof AdminProbePolicy;
export interface AdminProbeSource {
  /** Opaque id for unblocking (32 hex). */
  id: string;
  /** The source, masked to its network unless the policy logs full addresses. */
  source: string;
  score: number;
  graceAllowance: number;
  signals: number;
  lastSignal: AdminProbeSignal | null;
  lastSeen: string | null;
  strikes: number;
  blocked: boolean;
  blockedAt: string | null;
  blockedUntil: string | null;
  retryAfterSecs: number | null;
}
export interface AdminProbeStats {
  since: string | null;
  blocksTotal: number;
  blocksLast24h: number;
  activeBlocks: number;
  refusedRequests: number;
  tarpittedRequests: number;
  clearedBySignin: number;
  signals: Partial<Record<AdminProbeSignal, number>>;
  trackedSources: number;
  capacity: number;
  evictedSources: number;
}
export interface AdminProbeOverview {
  policy: AdminProbePolicy;
  stats: AdminProbeStats;
  topOffenders: AdminProbeSource[];
  scope: string;
  propagationSecs: number;
}
export interface AdminProbeApi {
  overview(): Promise<{ data: AdminProbeOverview; etag: string | null }>;
  save(values: Partial<Record<AdminProbeKey, unknown>>, etag: string | null): Promise<{ data: AdminProbeOverview; etag: string | null }>;
  blocks(): Promise<AdminProbeSource[]>;
  unblock(id: string): Promise<void>;
}
export const adminProbeApi: AdminProbeApi = {
  async overview() {
    const r = await adminRequest<AdminProbeOverview>('/security/admin-probe');
    return { data: r.data, etag: r.etag };
  },
  async save(values, etag) {
    const r = await adminRequest<AdminProbeOverview>('/security/admin-probe/policy', { method: 'PUT', body: { values }, ifMatch: etag });
    return { data: r.data, etag: r.etag };
  },
  async blocks() {
    return (await adminRequest<{ blocks: AdminProbeSource[] }>('/security/admin-probe/blocks')).data.blocks;
  },
  async unblock(id) {
    await adminRequest(`/security/admin-probe/blocks/${encodeURIComponent(id)}`, { method: 'DELETE' });
  },
};
