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
  /** t98: the editor group (`compose`, `pim`, ...). */
  category?: string;
  /** `toggle`, `limit` (numbers only) or `list` (allowed values). */
  control?: string;
  capabilityKeys: string[];
  limitKeys: string[];
  settingKeys?: string[];
  /** `enforced`, `partial` (some limits not yet counted by the server) or `ui` (hidden only). */
  enforcement: string;
}

/** A registry key a profile may hold (`SettingDef::describe` on the server). */
export interface ProfileKeyDef {
  key: string;
  kind: { type: string; values?: string[]; min?: number; max?: number; maxItems?: number; maxLength?: number };
  default: unknown;
  category: string;
}

export interface FeaturePreset {
  id: string;
  values: Record<string, ProfileValue>;
}

/** One key as it resolves for a previewed subject. */
export interface EffectiveKey {
  value: unknown;
  source: string;
  locked: boolean;
  lockedBy?: string;
  profile?: string;
}

export interface EffectivePreview {
  subject: { account: string; address: string; domain: string };
  profiles: string[];
  groups: string[];
  features: { id: string; category: string; enabled: boolean; enforcement: string; keys: Record<string, EffectiveKey> }[];
}

/** The exported profile document (`mailwoman.featureProfile`, version 1). */
export interface ProfileDocument {
  format: string;
  version: number;
  name: string;
  values: Record<string, ProfileValue>;
}

export const PROFILE_FORMAT = 'mailwoman.featureProfile';
export const PROFILE_FORMAT_VERSION = 1;

export function profileDocument(name: string, values: Record<string, ProfileValue>): ProfileDocument {
  return { format: PROFILE_FORMAT, version: PROFILE_FORMAT_VERSION, name, values };
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
  /** t98 (an older server omits these). */
  categories?: string[];
  profileKeys?: ProfileKeyDef[];
  defaults?: Record<string, unknown>;
  presets?: FeaturePreset[];
}

export interface FeaturesApi {
  load(): Promise<{ data: FeatureCatalogue; etag: string | null }>;
  putProfile(id: string, body: { name: string; values: Record<string, ProfileValue> }, etag: string): Promise<string | null>;
  deleteProfile(id: string, etag: string): Promise<string | null>;
  assign(a: ProfileAssignment, etag: string): Promise<string | null>;
  unassign(a: ProfileAssignment, etag: string): Promise<string | null>;
  /** Check an exported document; answers its normalised name and values (nothing is stored). */
  importDocument?(doc: unknown): Promise<{ name: string; values: Record<string, ProfileValue> }>;
  /** What every feature resolves to for one native user or one domain. */
  effective?(subject: { account: string } | { domain: string }): Promise<EffectivePreview>;
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
  async importDocument(doc) {
    return (await adminRequest<{ name: string; values: Record<string, ProfileValue> }>('/features/profiles/import', { method: 'POST', body: doc })).data;
  },
  async effective(subject) {
    const q = 'account' in subject ? `account=${enc(subject.account)}` : `domain=${enc(subject.domain)}`;
    return (await adminRequest<EffectivePreview>(`/features/effective?${q}`)).data;
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
