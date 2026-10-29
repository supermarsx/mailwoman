// Admin → AI assistant → provider profiles (t39 N8). The server side is
// `crates/mw-server/src/assist_profiles.rs`; the shapes below are its wire shapes.
//
//   GET    /admin/assist/presets                 → { presets: Preset[] }
//   GET    /admin/assist/profiles                → { profiles: Profile[], assignmentKey }
//   POST   /admin/assist/profiles                ProfileDraft → Profile
//   PUT    /admin/assist/profiles/{id}           ProfileDraft → Profile
//   DELETE /admin/assist/profiles/{id}           → { removed }
//   POST   /admin/assist/profiles/{id}/test      → TestOutcome
//   GET    /admin/assist/egress-grants           → { grants: Grant[] }
//   POST   /admin/assist/egress-grants           { destination, note } → Grant   (step-up)
//   DELETE /admin/assist/egress-grants/{id}      → { removed }                     (step-up)
//
// The API key is write-only: a profile carries `hasApiKey`; a draft's `apiKey`
// left out keeps the stored key, a string replaces it, `""` removes it. Which
// profile applies to whom is the settings key `assist.profile`.

import { adminRequest } from '../shell/adminApi.ts';

export type Dialect = 'openai' | 'azure-openai' | 'anthropic';
export type Verification = 'mock-wire' | 'unverified';

export interface Preset {
  readonly id: string;
  readonly label: string;
  readonly dialect: Dialect;
  readonly endpoint: string;
  readonly models: readonly string[];
  readonly keyRequired: boolean;
  readonly local: boolean;
  readonly verification: Verification;
}

export interface ProfileParams {
  readonly maxTokens?: number;
  readonly temperature?: number;
  readonly timeoutMs?: number;
  readonly apiVersion?: string;
  readonly anthropicVersion?: string;
  readonly transcriptionModel?: string;
}

export interface Profile {
  readonly id: string;
  readonly name: string;
  readonly preset: string;
  readonly dialect: Dialect;
  readonly endpoint: string;
  readonly model: string;
  readonly params: ProfileParams;
  readonly hasApiKey: boolean;
  readonly destination: string | null;
  readonly egress: 'public' | 'granted';
  readonly updatedBy: string;
  readonly updatedAt: number;
}

export interface ProfileDraft {
  readonly name: string;
  readonly preset: string;
  readonly endpoint: string;
  readonly model: string;
  readonly params: ProfileParams;
  /** Leave out to keep the stored key; `""` removes it. */
  readonly apiKey?: string;
}

export type TestErrorKind =
  | 'egress-refused'
  | 'timeout'
  | 'connect'
  | 'unauthorized'
  | 'not-found'
  | 'rate-limited'
  | 'rejected'
  | 'upstream'
  | 'bad-response'
  | 'declined'
  | 'unsupported';

export interface TestOutcome {
  readonly ok: boolean;
  readonly latencyMs: number;
  readonly reply: string | null;
  readonly error: { readonly kind: TestErrorKind; readonly status: number | null; readonly upstreamCode: string | null } | null;
}

export interface Grant {
  readonly id: string;
  readonly destination: string;
  readonly note: string;
  readonly createdBy: string;
  readonly createdAt: number;
}

export interface ProfilesApi {
  presets(): Promise<Preset[]>;
  list(): Promise<Profile[]>;
  create(draft: ProfileDraft): Promise<Profile>;
  update(id: string, draft: ProfileDraft): Promise<Profile>;
  remove(id: string): Promise<void>;
  test(id: string): Promise<TestOutcome>;
  grants(): Promise<Grant[]>;
  addGrant(destination: string, note: string): Promise<Grant>;
  removeGrant(id: string): Promise<void>;
}

const enc = encodeURIComponent;

export const httpProfilesApi: ProfilesApi = {
  async presets() {
    return (await adminRequest<{ presets: Preset[] }>('/assist/presets')).data.presets;
  },
  async list() {
    return (await adminRequest<{ profiles: Profile[] }>('/assist/profiles')).data.profiles;
  },
  async create(draft) {
    return (await adminRequest<Profile>('/assist/profiles', { method: 'POST', body: draft })).data;
  },
  async update(id, draft) {
    return (await adminRequest<Profile>(`/assist/profiles/${enc(id)}`, { method: 'PUT', body: draft })).data;
  },
  async remove(id) {
    await adminRequest(`/assist/profiles/${enc(id)}`, { method: 'DELETE' });
  },
  async test(id) {
    return (await adminRequest<TestOutcome>(`/assist/profiles/${enc(id)}/test`, { method: 'POST' })).data;
  },
  async grants() {
    return (await adminRequest<{ grants: Grant[] }>('/assist/egress-grants')).data.grants;
  },
  async addGrant(destination, note) {
    return (await adminRequest<Grant>('/assist/egress-grants', { method: 'POST', body: { destination, note } })).data;
  },
  async removeGrant(id) {
    await adminRequest(`/assist/egress-grants/${enc(id)}`, { method: 'DELETE' });
  },
};

/** The `host:port` an endpoint connects to, as grants spell it (mirrors `destination_of`). */
export function destinationOf(endpoint: string): string | null {
  try {
    const u = new URL(endpoint);
    if (u.protocol !== 'http:' && u.protocol !== 'https:') return null;
    const port = u.port || (u.protocol === 'https:' ? '443' : '80');
    return `${u.hostname.toLowerCase()}:${port}`;
  } catch {
    return null;
  }
}
