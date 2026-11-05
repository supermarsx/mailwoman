import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

/** One store in `GET /admin/retention`. */
export interface RetentionStore {
  id: string;
  owner: 'sweeper' | 'existing';
  unit: 'age' | 'grace' | 'ip';
  /** The setting holding the limit; null for a limit fixed in code or a policy elsewhere. */
  key: string | null;
  days: number | null;
  defaultDays: number | null;
  fixed: boolean;
  rows: number | null;
  bytes: number | null;
  lastSweepAt: number | null;
  lastRows: number | null;
  purged: number | null;
  error: string | null;
}

export interface RetentionSnapshot {
  sweeper: {
    enabled: boolean;
    nextRunAt: number | null;
    lastRunAt: number | null;
    lastTotal: number | null;
    vacuumed: boolean | null;
    running: boolean;
    keys: Record<string, string>;
  };
  stores: RetentionStore[];
  logs: {
    status: { fileEnabled: boolean; directory: string; currentBytes: number; rotatedFiles: number; rotatedBytes: number };
    keys: string[];
  };
}

export interface DryRun {
  dryRun: boolean;
  total: number;
  heldAccounts: number;
  stores: { id: string; rows: number; more: boolean; skipped: string | null; error: string | null }[];
}

export const loadRetention = async (): Promise<RetentionSnapshot> => (await adminRequest<RetentionSnapshot>('/retention')).data;
export const previewRetention = async (): Promise<DryRun> =>
  (await adminRequest<DryRun>('/retention/run', { method: 'POST', body: { dryRun: true } })).data;
export const runRetention = async (): Promise<{ runId: string }> =>
  (await adminRequest<{ runId: string }>('/retention/run', { method: 'POST', body: { dryRun: false } })).data;

// ── classes, archive tier and user rules ─────────────────────────────────────

export interface ClassStore {
  id: string;
  key: string | null;
  ownDays: number | null;
  days: number | null;
  keptForever: boolean;
  exceedsClass: boolean;
  archivable: boolean;
  /** Row counts per age bucket, `edgesDays.length + 1` long; null when the store has no age column to read. */
  ageBuckets: number[] | null;
}

export interface RetentionClass {
  id: 'short' | 'medium' | 'long';
  key: string;
  days: number;
  effectiveDays: number;
  forever: boolean;
  entries: ClassStore[];
  ageBuckets: number[];
}

export interface ClassesSnapshot {
  enforce: boolean;
  enforceKey: string;
  edgesDays: number[];
  warnings: ('mediumBelowShort' | 'longBelowMedium' | 'longForever')[];
  classes: RetentionClass[];
}

export interface ArchiveSegment {
  id: string;
  storeId: string;
  seq: number;
  rowCount: number;
  firstTs: number;
  lastTs: number;
  bytes: number;
  keyGen: number;
  state: string;
  createdAt: number;
  verifiedAt: number;
  verifyStatus: string;
}

export interface ArchiveSnapshot {
  enabled: boolean;
  directory: string;
  directoryValid: boolean;
  keyGeneration: number;
  legalHold: boolean;
  longDays: number;
  longForever: boolean;
  totals: { segments: number; rows: number; bytes: number; oldestVerification: number; oldestRow: number; failing: number };
  stores: { id: string; segments: number; rows: number; bytes: number }[];
  segments: ArchiveSegment[];
  status: { verifiedAt?: number; checked?: number; ok?: number; failed?: { segment: string; verdict: string }[]; purged?: number } | null;
  tables: string[];
  keys: string[];
}

export interface ArchiveVerify {
  checked: number;
  ok: number;
  failed: [string, string][];
  purged: number;
}

export interface ArchiveHit {
  segmentId: string;
  row: Record<string, string | number>;
}

export interface UserRulesOverview {
  enabled: boolean;
  usersWithRules: number;
  rules: number;
  enabledRules: number;
  runs30d: number;
  matched30d: number;
  messagesChanged30d: number;
  lastJob: { at: number; accounts: number; changed: number; errors: number; skipped: number } | null;
  keys: string[];
}

export const loadClasses = async (): Promise<ClassesSnapshot> => (await adminRequest<ClassesSnapshot>('/retention/classes')).data;
export const loadArchive = async (): Promise<ArchiveSnapshot> => (await adminRequest<ArchiveSnapshot>('/retention/archive')).data;
export const loadUserRules = async (): Promise<UserRulesOverview> => (await adminRequest<UserRulesOverview>('/retention/user-rules')).data;
export const verifyArchive = async (): Promise<ArchiveVerify> =>
  (await adminRequest<ArchiveVerify>('/retention/archive/verify', { method: 'POST', body: {} })).data;
export const searchArchive = async (store: string, query: { key?: string; account?: string }): Promise<{ hits: ArchiveHit[] }> => {
  const q = new URLSearchParams({ store });
  if (query.key) q.set('key', query.key);
  if (query.account) q.set('account', query.account);
  return (await adminRequest<{ hits: ArchiveHit[] }>(`/retention/archive/search?${q.toString()}`)).data;
};
export const restoreArchive = async (segmentId: string): Promise<{ restored: number }> =>
  (await adminRequest<{ restored: number }>('/retention/archive/restore', { method: 'POST', body: { segmentId } })).data;
/** The path (relative to the admin API) that streams one segment as JSON lines. */
export const archiveExportPath = (segmentId: string): string => `/retention/archive/export?segment=${encodeURIComponent(segmentId)}`;
