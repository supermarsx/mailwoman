// Admin client of the cookie routes (`crates/mw-server/src/cookie_consent.rs`), under
// the admin session through `adminRequest`.

import { apiFetch } from '../../../api/http.ts';
import { adminUrl } from '../../../api/adminPath.ts';
import { basePath } from '../../../api/basePath.ts';
import { adminRequest } from '../shell/adminApi.ts';
import type { Category, StorageKind } from '../../../cookies/policy.ts';
import type { ConsentStats, DetectedRow, InventoryView } from './model.ts';

export const cookiesApi = {
  async inventory(): Promise<InventoryView> { return (await adminRequest<InventoryView>('/cookies/inventory')).data; },
  async detected(): Promise<DetectedRow[]> { return (await adminRequest<{ items: DetectedRow[] }>('/cookies/detected')).data.items; },
  async stats(from: number, to: number): Promise<ConsentStats> { return (await adminRequest<ConsentStats>(`/cookies/stats?from=${from}&to=${to}`)).data; },
  async saveRule(rule: { kind: StorageKind; pattern: string; category: Category; note?: string }): Promise<void> { await adminRequest('/cookies/rules', { method: 'POST', body: { note: '', ...rule } }); },
  async deleteRule(id: string): Promise<void> { await adminRequest(`/cookies/rules/${encodeURIComponent(id)}`, { method: 'DELETE' }); },
  async classify(id: string, category: Category, note = ''): Promise<void> { await adminRequest(`/cookies/detected/${encodeURIComponent(id)}/classify`, { method: 'POST', body: { category, note } }); },
  async ignore(id: string): Promise<void> { await adminRequest(`/cookies/detected/${encodeURIComponent(id)}/ignore`, { method: 'POST', body: {} }); },
  async remove(id: string): Promise<void> { await adminRequest(`/cookies/detected/${encodeURIComponent(id)}`, { method: 'DELETE' }); },
  async scan(items: { kind: StorageKind; name: string }[]): Promise<number> { return (await adminRequest<{ recorded: number }>('/cookies/detected', { method: 'POST', body: { items: items.slice(0, 100) } })).data.recorded; },
  /** The consent log as CSV text. */
  async exportCsv(from: number, to: number): Promise<string> {
    const response = await apiFetch(adminUrl(`/cookies/consents.csv?from=${from}&to=${to}`, basePath()), { credentials: 'same-origin', headers: { Accept: 'text/csv' } });
    if (!response.ok) throw new Error(`export failed (${response.status})`);
    return response.text();
  },
};
export type CookiesApi = typeof cookiesApi;
