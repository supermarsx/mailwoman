// Admin security policy and OAuth client registration policy (t32-admin §3.1 e16).
//
// `GET/PUT /admin/security-policy` (`get_policy` / `set_policy`,
// crates/mw-server/src/admin.rs): the TLS floor, the Argon2 cost, the stored DLP
// rules, the reading floor and capture policy, and the global two-factor rule.
// A PUT carries all eight fields and answers with the policy now in force. The
// server refuses a field an environment variable pins with
// `409 {error, field, variable}` and a value out of range with `422 {error}`;
// both are turned into an `AdminProblemError` whose `field` points at the field,
// so the screen shows the server's sentence there (t32-cfg UX5).
//
// `GET/PUT /admin/oauth-dcr` (`dcr_admin_get` / `dcr_admin_put`,
// crates/mw-server/src/oauth.rs): dynamic client registration (RFC 7591) on or
// off, the redirect host suffixes a self-registered client may use, the scope
// every such client gets, and whether registration needs the shared
// `MW_DCR_INITIAL_ACCESS_TOKEN`.
//
// Every URL goes through `adminRequest` → `adminUrl()` (ADMIN PATH CONTRACT).

import type { DlpRule } from '../crypto-types.ts';
import { AdminProblemError, adminRead, adminRequest, type AdminReadResult } from '../../screens/Admin/shell/adminApi.ts';

export type MinTls = '1.2' | '1.3';
export type CapturePolicy = 'off' | 'protect';
export type SecurityFloor = 'full-sanitized' | 'sanitized-no-media' | 'plain-text';

/** The eight fields a `PUT /admin/security-policy` writes, all required. */
export interface SecurityPolicyWrite {
  minTls: MinTls;
  require2fa: boolean;
  argon2MCost: number;
  argon2TCost: number;
  argon2PCost: number;
  capturePolicy: CapturePolicy;
  maxSecurityFloor: SecurityFloor | null;
  /** A JSON array of DLP rules. */
  dlpRulesJson: string;
}

/** The GET body: the policy in force, the `MW_DLP_RULES` rules, the pinned fields. */
export interface SecurityPolicy extends SecurityPolicyWrite {
  envDlpRules: DlpRule[];
  pinnedByEnv: ('minTls' | 'require2fa')[];
}

/** The environment variable that pins each pinnable field (`PinnedField`, mw-admin policy.rs). */
export const POLICY_PIN_VARS: Readonly<Record<'minTls' | 'require2fa', string>> = {
  minTls: 'MW_ADMIN_MIN_TLS',
  require2fa: 'MW_ADMIN_REQUIRE_2FA',
};

/** Argon2 bounds the server checks (`argon_bounds`, crates/mw-admin/src/policy.rs). */
export const ARGON_BOUNDS = {
  minMemoryKib: 19_456,
  maxMemoryKib: 1_048_576,
  minMemoryKibSinglePass: 47_104,
  maxPasses: 10,
  maxParallelism: 16,
} as const;

/** The field a server sentence is about, for a 422 that names none. */
export function policyFieldFor(message: string): string | undefined {
  const m = message.toLowerCase();
  if (m.includes('argon2 memory') || m.includes('single pass')) return 'argon2MCost';
  if (m.includes('argon2 passes')) return 'argon2TCost';
  if (m.includes('argon2 parallelism')) return 'argon2PCost';
  if (m.includes('dlp')) return 'dlpRules';
  if (m.includes('mintls')) return 'minTls';
  if (m.includes('capturepolicy')) return 'capturePolicy';
  if (m.includes('maxsecurityfloor')) return 'maxSecurityFloor';
  return undefined;
}

/** A 409/422 of these routes as a problem that names its field. */
function placed(err: unknown): unknown {
  if (!(err instanceof AdminProblemError) || (err.status !== 409 && err.status !== 422)) return err;
  const sentence = err.detail ?? err.message;
  const field = err.field ?? policyFieldFor(sentence);
  const body: Record<string, unknown> = { detail: sentence };
  if (field !== undefined) body.field = field.startsWith('/') ? field : `/${field}`;
  if (err.status === 409) body.code = 'pinned';
  return new AdminProblemError(err.status, body, err.currentEtag ?? null);
}

export function loadSecurityPolicy(base?: string): Promise<AdminReadResult<SecurityPolicy>> {
  return adminRead<SecurityPolicy>('/security-policy', base === undefined ? {} : { base });
}

export async function saveSecurityPolicy(policy: SecurityPolicyWrite, base?: string): Promise<SecurityPolicy> {
  try {
    const res = await adminRequest<SecurityPolicy>('/security-policy', { method: 'PUT', body: policy, ...(base === undefined ? {} : { base }) });
    return res.data;
  } catch (err) {
    throw placed(err);
  }
}

// ── OAuth dynamic client registration ────────────────────────────────────────

/** `ScopeSelector` (mw-oauth, kebab-case): every id, or a listed subset. */
export type ScopeSelector = 'all' | { subset: string[] };

/** The scope a self-registered client is granted (`mw_oauth::Scope`, snake_case on the wire). */
export interface OAuthScope {
  read: boolean;
  send: boolean;
  delete: boolean;
  accounts: ScopeSelector;
  folders: ScopeSelector;
  mail: boolean;
  pim: boolean;
  scripts: boolean;
  ip_allowlist: string[];
  expires_at: string | null;
  rate_limit: number | null;
  mcp_tools: string[];
  unattended_send: boolean;
}

/** `mw_oauth::dcr::no_scope()`: the scope the server applies when none is stored. */
export function noScope(): OAuthScope {
  return {
    read: false,
    send: false,
    delete: false,
    accounts: { subset: [] },
    folders: { subset: [] },
    mail: false,
    pim: false,
    scripts: false,
    ip_allowlist: [],
    expires_at: null,
    rate_limit: null,
    mcp_tools: [],
    unattended_send: false,
  };
}

/** A stored scope read leniently: missing or malformed members take `noScope()`'s value. */
export function readScope(raw: unknown): OAuthScope {
  const base = noScope();
  if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) return base;
  const o = raw as Record<string, unknown>;
  const bool = (k: keyof OAuthScope): boolean => (typeof o[k] === 'boolean' ? (o[k] as boolean) : (base[k] as boolean));
  const selector = (v: unknown): ScopeSelector => {
    if (v === 'all') return 'all';
    if (typeof v === 'object' && v !== null && Array.isArray((v as { subset?: unknown }).subset)) {
      return { subset: ((v as { subset: unknown[] }).subset).filter((s): s is string => typeof s === 'string') };
    }
    return { subset: [] };
  };
  const strings = (v: unknown): string[] => (Array.isArray(v) ? v.filter((s): s is string => typeof s === 'string') : []);
  return {
    read: bool('read'),
    send: bool('send'),
    delete: bool('delete'),
    accounts: selector(o.accounts),
    folders: selector(o.folders),
    mail: bool('mail'),
    pim: bool('pim'),
    scripts: bool('scripts'),
    ip_allowlist: strings(o.ip_allowlist),
    expires_at: typeof o.expires_at === 'string' ? o.expires_at : null,
    rate_limit: typeof o.rate_limit === 'number' ? o.rate_limit : null,
    mcp_tools: strings(o.mcp_tools),
    unattended_send: bool('unattended_send'),
  };
}

/** `GET /admin/oauth-dcr` (`policy_view`). `defaultScope` is null when no policy is stored. */
export interface DcrPolicy {
  enabled: boolean;
  requireInitialAccessToken: boolean;
  allowedRedirectHostSuffixes: string[];
  defaultScope: unknown;
  updatedAt: string | null;
}

export interface DcrPolicyWrite {
  enabled: boolean;
  requireInitialAccessToken: boolean;
  allowedRedirectHostSuffixes: string[];
  defaultScope: OAuthScope;
}

export function loadDcrPolicy(base?: string): Promise<AdminReadResult<DcrPolicy>> {
  return adminRead<DcrPolicy>('/oauth-dcr', base === undefined ? {} : { base });
}

export async function saveDcrPolicy(policy: DcrPolicyWrite, base?: string): Promise<DcrPolicy> {
  const res = await adminRequest<DcrPolicy>('/oauth-dcr', { method: 'PUT', body: policy, ...(base === undefined ? {} : { base }) });
  return res.data;
}
