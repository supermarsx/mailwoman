import { adminRequest, type AdminResponse } from '../../screens/Admin/shell/adminApi.ts';
/** `GET/PUT {console}/signin-policy`, `GET .../signin-policy/preview` and `{console}/preprovisioned-users` (signin.newUsers and friends). */
export type SigninRule = 'open' | 'preprovisioned' | 'provisioned' | 'closed';
export interface SigninPolicyView {
  keys: { rules: string[]; other: string[] };
  global: Record<string, unknown>;
  domains: Record<string, Record<string, unknown>>;
  defaults: Record<string, unknown>;
  warnings: { code: string; key?: string }[];
}
export interface SigninChange { key: string; scope?: 'global' | 'domain'; domain?: string; value: unknown }
export interface SigninPreview { address: string; type: string; rule: SigninRule; ruleKey: string; ruleScope: string; wouldBeRefused: boolean }
export interface PendingEntry { address: string; domain: string; origin: string; note: string | null; createdAt: number; createdBy: string | null; expiresAt: number | null; consumedAt: number | null; state: 'pending' | 'expired' | 'consumed' }
export interface PendingList { entries: PendingEntry[]; ttlDays: number; maxTtlDays: number; bulkCap: number }
export interface PendingAddResult { results: { address: string; result: string }[]; added: number }
export interface SigninPolicyApi {
  read(): Promise<AdminResponse<SigninPolicyView>>;
  save(changes: SigninChange[], etag: string | null, dryRun?: boolean): Promise<AdminResponse<SigninPolicyView>>;
  preview(address: string, type: string): Promise<SigninPreview>;
  pending(): Promise<PendingList>;
  add(addresses: string[], note: string, ttlDays: number): Promise<PendingAddResult>;
  extend(address: string, ttlDays: number): Promise<void>;
  revoke(address: string): Promise<void>;
}
export const signinPolicyApi: SigninPolicyApi = {
  read: () => adminRequest('/signin-policy'),
  save: (changes, ifMatch, dryRun) => adminRequest('/signin-policy', { method: 'PUT', body: { changes, ...(dryRun ? { dryRun: true } : {}) }, ifMatch }),
  preview: async (address, type) => (await adminRequest<SigninPreview>(`/signin-policy/preview?address=${encodeURIComponent(address)}&type=${encodeURIComponent(type)}`)).data,
  pending: async () => (await adminRequest<PendingList>('/preprovisioned-users')).data,
  add: async (addresses, note, ttlDays) => (await adminRequest<PendingAddResult>('/preprovisioned-users', { method: 'POST', body: { addresses, ...(note ? { note } : {}), ttlDays } })).data,
  extend: async (address, ttlDays) => { await adminRequest(`/preprovisioned-users/${encodeURIComponent(address)}`, { method: 'PUT', body: { ttlDays } }); },
  revoke: async (address) => { await adminRequest(`/preprovisioned-users/${encodeURIComponent(address)}`, { method: 'DELETE' }); },
};
