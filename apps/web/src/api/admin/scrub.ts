import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export interface ScrubKeyInfo { gen: number; createdAt: number; retiredAt: number | null; destroyAfter: number | null }
export interface ChainReport { rows: number; unsealed: number; gaps: number; ok: boolean; firstBadSeq: number | null; headMatches: boolean; rechains: number }
export interface ChainAnchor {
  id: string; ts: number; scrubId: string; fromSeq: number; toSeq: number;
  oldHeadHash: string | null; newHeadHash: string | null; rowsChanged: number; algorithm: string; keyGen: number;
}
export interface RegisteredBackup {
  id: string; kind: string; label: string; location: string; createdAt: number;
  tombstoneWatermark: number;
  /** People erased or scrubbed after the backup was made; they are still in it. */
  erasedSince: number;
}
export interface ScrubState {
  settings: {
    enabled: boolean; pseudonymAlgorithm: string; includeLogs: boolean; autoRunOnErasure: boolean;
    keyRetentionDays: number; propagateErasure: boolean; tombstoneRetentionDays: number;
  };
  keys: ScrubKeyInfo[];
  auditChain: ChainReport;
  anchors: ChainAnchor[];
  tombstoneWatermark: number;
  backups: RegisteredBackup[];
  algorithms: string[];
}
export interface ScrubResult {
  dryRun: boolean;
  counts: Record<string, number>;
  errors: string[];
  pseudonym: string;
  keyGen: number;
  anchor: string | null;
  tombstone: number | null;
}

export const loadScrubState = async (): Promise<ScrubState> => (await adminRequest<ScrubState>('/privacy/scrub')).data;
export const previewScrub = async (address: string): Promise<ScrubResult> =>
  (await adminRequest<ScrubResult>('/privacy/scrub/preview', { method: 'POST', body: { address } })).data;
export const runScrub = async (address: string): Promise<ScrubResult> =>
  (await adminRequest<ScrubResult>('/privacy/scrub/run', { method: 'POST', body: { address } })).data;
export const rotateScrubKey = async (): Promise<{ generation: number; keys: ScrubKeyInfo[] }> =>
  (await adminRequest<{ generation: number; keys: ScrubKeyInfo[] }>('/privacy/scrub/rotate-key', { method: 'POST', body: {} })).data;
export const sealAudit = async (): Promise<{ sealed: number; auditChain: ChainReport }> =>
  (await adminRequest<{ sealed: number; auditChain: ChainReport }>('/privacy/scrub/seal-audit', { method: 'POST', body: {} })).data;
export const registerBackup = async (input: { label: string; location: string; tombstoneWatermark?: number }): Promise<{ backups: RegisteredBackup[] }> =>
  (await adminRequest<{ backups: RegisteredBackup[] }>('/privacy/backups', { method: 'POST', body: input })).data;
export const removeBackup = async (id: string): Promise<{ backups: RegisteredBackup[] }> =>
  (await adminRequest<{ backups: RegisteredBackup[] }>(`/privacy/backups/${encodeURIComponent(id)}`, { method: 'DELETE' })).data;
