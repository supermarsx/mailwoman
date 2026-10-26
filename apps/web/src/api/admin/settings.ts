// Admin client of the settings model C (t32-admin §2.4; worker f01). Server:
// `crates/mw-server/src/settings_routes.rs`, under the admin session, every URL
// through `adminUrl()` (via the UX seam's `adminRequest`).
//
//   GET    /admin/settings                         → { settings: AdminSetting[], families, propagationSecs }
//   GET    /admin/settings/<key>                   → AdminSetting (ETag)
//   PUT    /admin/settings/<key>                   ← { globalValue?, allowUserOverride?, locked?, dryRun? }
//   DELETE /admin/settings/<key>                   → AdminSetting (the stored policy removed)
//   GET    /admin/settings/<key>/values?scope=…    → { values: ScopedValue[], nextCursor }
//   PUT    /admin/settings/<key>/domains/<domain>  ← { value, locked?, dryRun? }
//   DELETE /admin/settings/<key>/domains/<domain>
//   PUT    /admin/settings/<key>/users/<id>        ← { value }
//   DELETE /admin/settings/<key>/users/<id>
//
// Refusals arrive as `AdminProblemError` (`422 {code, detail, field}`, `412 stale`,
// `409 pinnedByEnv`, `403 useDedicatedRoute | setting_locked`).

import {
  adminRead,
  adminRequest,
  parseImpact,
  type AdminReadResult,
  type ImpactItem,
} from '../../screens/Admin/shell/adminApi.ts';

/** A value's type and bounds, as the registry declares it. */
export interface SettingKind {
  type: 'folderRole' | 'bool' | 'int' | 'float' | 'enum' | 'text' | 'locale' | 'localeMap' | 'intList' | 'stringList' | 'quietHours' | 'links';
  min?: number;
  max?: number;
  values?: string[];
  format?: 'plain' | 'slug' | 'httpsUrl' | 'httpUrl' | 'upstreamUrl' | 'email' | 'domain' | 'contact';
  maxLength?: number;
  maxItems?: number;
  locales?: string[];
}

export type SettingScopes = 'global' | 'global+domain' | 'global+domain+admin_user' | 'global+domain+user';

/** One key of `GET /admin/settings`. */
export interface AdminSetting {
  key: string;
  family: boolean;
  kind: SettingKind;
  default: unknown;
  nullable: boolean;
  scopes: SettingScopes;
  userOverride: boolean;
  category: string;
  mapsTo: string;
  env: { var: string; precedence: 'pin' | 'fallback' | 'boot' } | null;
  /** The admin route (full path) that writes this key instead of the generic one. */
  dedicatedRoute: string | null;
  scopeId: 'account' | 'address';
  hot: boolean;
  ceiling: boolean;
  userVisible: boolean;
  globalValue: unknown;
  allowUserOverride: boolean;
  globalLocked: boolean;
  updatedBy: string | null;
  /** Unix seconds. */
  updatedAt: number | null;
  effective: { value: unknown; source: string };
  envState: { var: string; precedence: string; set: boolean; value: unknown } | null;
  /** Narrower values per scope: `{domain: 2, user: 5, …}`. */
  overrides: Partial<Record<'domain' | 'profile' | 'group' | 'admin_user' | 'user', number>>;
  etag: string;
}

/** One narrower value. */
export interface ScopedValue {
  scope: 'domain' | 'profile' | 'group' | 'admin_user' | 'user';
  scopeId: string;
  value: unknown;
  locked: boolean;
  updatedBy: string;
  updatedAt: number;
  etag: string;
}

export interface GlobalWrite {
  confirmLastSuperadminLoss?: boolean;
  /** Leave out to keep; `null` = back to the default. */
  globalValue?: unknown;
  allowUserOverride?: boolean;
  locked?: boolean;
}

const enc = encodeURIComponent;

/** `GET /admin/settings`; `unavailable` on a server without the settings model. */
export async function listAdminSettings(): Promise<AdminReadResult<AdminSetting[]>> {
  const r = await adminRead<{ settings?: AdminSetting[] }>('/settings');
  if (r.kind === 'unavailable') return r;
  return { kind: 'ok', data: Array.isArray(r.data?.settings) ? r.data.settings : [], etag: r.etag };
}

export async function getAdminSetting(key: string): Promise<{ setting: AdminSetting; etag: string | null }> {
  const r = await adminRequest<AdminSetting>(`/settings/${enc(key)}`);
  return { setting: r.data, etag: r.etag ?? r.data.etag };
}

export async function putGlobal(
  key: string,
  body: GlobalWrite,
  ifMatch: string | null,
): Promise<{ setting: AdminSetting; etag: string | null }> {
  const r = await adminRequest<AdminSetting>(`/settings/${enc(key)}`, { method: 'PUT', body, ifMatch });
  return { setting: r.data, etag: r.etag ?? r.data.etag };
}

/** The dry run of a global write: what a lock would overrule. */
export async function previewGlobal(key: string, body: GlobalWrite): Promise<Record<string, ImpactItem> | null> {
  const r = await adminRequest<unknown>(`/settings/${enc(key)}`, { method: 'PUT', body: { ...body, dryRun: true } });
  return parseImpact(r.data);
}

export async function deleteGlobal(key: string, ifMatch: string | null): Promise<{ setting: AdminSetting; etag: string | null }> {
  const r = await adminRequest<AdminSetting>(`/settings/${enc(key)}`, { method: 'DELETE', ifMatch });
  return { setting: r.data, etag: r.etag ?? r.data.etag };
}

export async function listScopedValues(
  key: string,
  scope: ScopedValue['scope'],
  cursor?: string,
): Promise<{ values: ScopedValue[]; nextCursor: string | null }> {
  const q = new URLSearchParams({ scope });
  if (cursor !== undefined) q.set('cursor', cursor);
  const r = await adminRequest<{ values?: ScopedValue[]; nextCursor?: string | null }>(
    `/settings/${enc(key)}/values?${q.toString()}`,
  );
  return { values: r.data.values ?? [], nextCursor: r.data.nextCursor ?? null };
}

export async function putDomainValue(
  key: string,
  domain: string,
  body: { value: unknown; locked?: boolean },
  ifMatch: string | null,
): Promise<ScopedValue> {
  const r = await adminRequest<ScopedValue>(`/settings/${enc(key)}/domains/${enc(domain)}`, {
    method: 'PUT',
    body,
    ifMatch,
  });
  return r.data;
}

/** The dry run of a domain write (a new lock over per-user values). */
export async function previewDomain(
  key: string,
  domain: string,
  body: { value: unknown; locked?: boolean },
): Promise<Record<string, ImpactItem> | null> {
  const r = await adminRequest<unknown>(`/settings/${enc(key)}/domains/${enc(domain)}`, {
    method: 'PUT',
    body: { ...body, dryRun: true },
  });
  return parseImpact(r.data);
}

export async function deleteDomainValue(key: string, domain: string, ifMatch: string | null): Promise<void> {
  await adminRequest(`/settings/${enc(key)}/domains/${enc(domain)}`, { method: 'DELETE', ifMatch });
}

export async function putUserValue(key: string, id: string, value: unknown, ifMatch: string | null): Promise<ScopedValue> {
  const r = await adminRequest<ScopedValue>(`/settings/${enc(key)}/users/${enc(id)}`, {
    method: 'PUT',
    body: { value },
    ifMatch,
  });
  return r.data;
}

export async function deleteUserValue(key: string, id: string, ifMatch: string | null): Promise<void> {
  await adminRequest(`/settings/${enc(key)}/users/${enc(id)}`, { method: 'DELETE', ifMatch });
}
