// Admin → Feature profiles: the client for `/admin/features/**` (t39 N9).
//
// The catalogue is one document with one revision (`fg1_<sha256>`, sent back as
// the ETag). Every write sends that tag as `If-Match`; a stale tag is refused
// with 412 and the screen reloads. A write answers with the new revision.

import { adminRequest } from '../shell/adminApi.ts';

export interface ProfileValue {
  value: unknown;
  locked: boolean;
}

export interface FeatureProfile {
  id: string;
  name: string;
  values: Record<string, ProfileValue>;
  builtin: boolean;
  updatedAt: number;
}

export type ScopeKind = 'domain' | 'group' | 'user';

export interface ProfileAssignment {
  scopeKind: ScopeKind;
  scopeId: string;
  profileId: string;
  priority: number;
}

export interface UserGroup {
  id: string;
  name: string;
  domain: string;
  source: string;
}

export interface NativeTarget {
  accountId: string;
  address: string;
  domain: string;
}

export interface FeatureDef {
  id: string;
  capabilityKeys: string[];
  limitKeys: string[];
  enforcement: string;
}

export interface FeatureCatalogue {
  revision: string;
  profiles: FeatureProfile[];
  groups: UserGroup[];
  assignments: ProfileAssignment[];
  memberships: Record<string, string[]>;
  nativeTargets: NativeTarget[];
  domains: string[];
  catalogue: FeatureDef[];
}

export interface FeaturesApi {
  load(): Promise<{ data: FeatureCatalogue; etag: string | null }>;
  putProfile(id: string, body: { name: string; values: Record<string, ProfileValue> }, etag: string): Promise<string | null>;
  deleteProfile(id: string, etag: string): Promise<string | null>;
  assign(a: ProfileAssignment, etag: string): Promise<string | null>;
  unassign(a: ProfileAssignment, etag: string): Promise<string | null>;
}

const enc = encodeURIComponent;

export const featuresApi: FeaturesApi = {
  async load() {
    const r = await adminRequest<FeatureCatalogue>('/features');
    return { data: r.data, etag: r.etag };
  },
  async putProfile(id, body, etag) {
    return (await adminRequest(`/features/profiles/${enc(id)}`, { method: 'PUT', body, ifMatch: etag })).etag;
  },
  async deleteProfile(id, etag) {
    return (await adminRequest(`/features/profiles/${enc(id)}`, { method: 'DELETE', ifMatch: etag })).etag;
  },
  async assign(a, etag) {
    return (await adminRequest('/features/assignments', { method: 'PUT', body: a, ifMatch: etag })).etag;
  },
  async unassign(a, etag) {
    return (await adminRequest('/features/assignments', { method: 'DELETE', body: a, ifMatch: etag })).etag;
  },
};

/** A new profile id from its name: lowercase ASCII words joined by `-`, with a
 *  short random suffix when the name has none or the id is taken. */
export function profileIdFor(name: string, taken: readonly string[]): string {
  const base = name
    .normalize('NFKD')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 48);
  let id = base || 'profile';
  while (taken.includes(id)) id = `${base || 'profile'}-${Math.random().toString(36).slice(2, 6)}`;
  return id;
}
