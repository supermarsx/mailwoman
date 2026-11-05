import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export type StoreState = 'encrypted' | 'sealed-columns-only' | 'plaintext' | 'memory-only' | 'not-stored' | 'unknown';

/** One row of `GET /admin/encryption`: a place data is kept, its state and how the state was determined. */
export interface EncryptionStore {
  id: string;
  state: StoreState;
  /** Message id suffix under `admin-encryption-evidence-`. */
  evidence: string;
  args: Record<string, unknown>;
}

export interface ColumnCoverage { table: string; column: string; what: string; sealed: boolean }

export interface EncryptionSnapshot {
  mode: 'off' | 'optional' | 'required';
  backend: 'sqlite' | 'postgres' | 'memory';
  databaseFile: string | null;
  key: {
    configured: boolean;
    source: { kind: string; detail: string; age_secs: number | null } | null;
    fingerprint: string | null;
  };
  rotation: { days: number; lastRotatedAt: string | null; encryptedAt: string | null; dueAt: string | null; overdue: boolean };
  stores: EncryptionStore[];
  postgres: null | {
    error?: string;
    serverVersion?: string | null;
    connectionSsl?: boolean | null;
    tlsVersion?: string | null;
    serverSsl?: string | null;
    dataChecksums?: string | null;
    pgcryptoInstalled?: boolean | null;
    pgcryptoAvailable?: boolean | null;
    tdeIndicators?: { settings: string[]; extensions: string[] };
    tablespaces?: { name: string; location: string }[];
  };
  columnCoverage: ColumnCoverage[];
  warnings: string[];
  keys: string[];
  commands: Record<'dryRun' | 'encrypt' | 'rekey' | 'decrypt' | 'verify' | 'backup' | 'keygen', string>;
}

export interface VerifyReport { passed: boolean; checks: { name: string; passed: boolean; detail: string }[] }
export interface Rehearsal { ok: boolean; operation?: string; tables?: number; rows?: number; bytes?: number | null; millis?: number; error?: string }

export const loadEncryption = async (): Promise<EncryptionSnapshot> => (await adminRequest<EncryptionSnapshot>('/encryption')).data;
export const verifyEncryption = async (): Promise<VerifyReport> =>
  (await adminRequest<VerifyReport>('/encryption/verify', { method: 'POST', body: {} })).data;
export const rehearseEncryption = async (): Promise<Rehearsal> =>
  (await adminRequest<Rehearsal>('/encryption/rehearse', { method: 'POST', body: {} })).data;
