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
