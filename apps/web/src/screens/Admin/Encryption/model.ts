import type { EncryptionSnapshot, EncryptionStore, StoreState } from '../../../api/admin/encryption.ts';

export const storeLabel = (id: string): string => `admin-encryption-store-${id}`;
export const stateLabel = (s: StoreState): string => `admin-encryption-state-${s}`;
export const evidenceLabel = (id: string): string => `admin-encryption-${id}`;
export const warningLabel = (id: string): string => `admin-encryption-warning-${id}`;
export const checkLabel = (name: string): string => `admin-encryption-check-${name}`;

/** States an operator should look at. */
export const needsAttention = (s: StoreState): boolean => s === 'plaintext' || s === 'unknown';

/** Evidence arguments flattened to strings and numbers for the message formatter. */
export function evidenceArgs(row: EncryptionStore): Record<string, string | number> {
  const out: Record<string, string | number> = {};
  for (const [k, v] of Object.entries(row.args)) {
    if (typeof v === 'number' || typeof v === 'string') out[k] = v;
    else if (typeof v === 'boolean') out[k] = v ? 'yes' : 'no';
    else if (Array.isArray(v)) out[k] = v.length;
    else if (v === null || v === undefined) out[k] = '-';
  }
  return out;
}

/** Whole days of key age, from the file's modification time or the last rotation. */
export function keyAgeDays(s: EncryptionSnapshot, now = Date.now()): number | null {
  if (s.key.source?.age_secs != null) return Math.floor(s.key.source.age_secs / 86400);
  if (s.rotation.lastRotatedAt) return Math.max(0, Math.floor((now - Date.parse(s.rotation.lastRotatedAt)) / 86400000));
  return null;
}

export function counts(stores: readonly EncryptionStore[]): Record<StoreState, number> {
  const out: Record<StoreState, number> = { encrypted: 0, 'sealed-columns-only': 0, plaintext: 0, 'memory-only': 0, 'not-stored': 0, unknown: 0 };
  for (const s of stores) out[s.state] += 1;
  return out;
}
