// Admin panel client + slice (SPEC §19, plan §2.5 / §2.6, §3 e7).
//
// The admin panel drives a SEPARATE session domain (the `mw_admin_session` cookie)
// over a small REST surface under `/admin/*`, distinct from the cookie-authed JMAP
// surface the mailbox uses. This file owns the TYPED client (`AdminApi` +
// `createHttpAdminApi`) plus the reactive `AdminSlice` the screens consume. The
// wire shapes are the server's (`crates/mw-server/src/admin.rs`); each DTO below
// cites the Rust struct it mirrors.
//
// The client is an interface so component tests inject a mock; the HTTP impl is a
// thin `fetch` wrapper (same-origin, cookie-authed) that never touches the JMAP
// client, so the normal mailbox path is untouched (regression gate).

import { createSignal, type Accessor } from 'solid-js';
import { basePath } from '../../api/basePath.ts';

// ── Wire DTOs (the frozen `/admin/*` JSON contract e11 satisfies) ──────────────

/** An authenticated admin session (`GET /admin/session`). */
export interface AdminSession {
  username: string;
}

/** A managed mail domain: its name (`DomainDto`, `crates/mw-server/src/admin.rs`).
 *  The server stores more columns for a domain; nothing reads them, so they are
 *  not part of the wire shape. */
export interface Domain {
  name: string;
}

/** A per-account quota (`quotas`, 0007). A non-positive limit means "no limit". */
export interface Quota {
  bytesLimit: number;
  msgLimit: number;
}

/** Per-user feature flags (incl. the zero-access toggle — §9). */
export interface UserFeatureFlags {
  zeroAccess: boolean;
  forcePasswordChange: boolean;
  remoteCacheWipe: boolean;
  disabled: boolean;
}

/** A provisioned user row (list view). */
export interface UserSummary {
  accountId: string;
  username: string;
  domain: string;
  quota: Quota | null;
  flags: UserFeatureFlags;
}

/** The stored security-policy record as the server exposes it
 *  (`SecurityPolicyDto`, `crates/mw-server/src/admin.rs`). Both fields are stored
 *  and NOT applied by this release, so no screen offers them; the server refuses a
 *  `PUT` carrying any other field. */
export interface SecurityPolicy {
  dlpRulesJson: string;
  maxSecurityFloor: boolean;
}

/** The statuses `GET /admin/integrations` sends
 *  (`mw_admin::IntegrationStatus::as_str`, `crates/mw-admin/src/provisioning.rs`). */
export type IntegrationStatus = 'active' | 'configured' | 'not-configured' | 'unknown';

/**
 * `GET /admin/integrations` (`get_integrations`, `crates/mw-server/src/admin.rs`).
 *
 * The fields are `string`, not {@link IntegrationStatus}: a value this build does
 * not recognise must render as "status unknown" rather than fail to type-check or
 * be shown as one of the known states. `ldap` / `nextcloud` say whether the
 * deployment has CONFIGURATION for them — never that the remote service answers.
 */
export interface IntegrationsConfig {
  webhooks: string;
  apiKeyOversight: string;
  ldap: string;
  nextcloud: string;
}

/** An outbound webhook registration (oversight view; secret never returned). */
export interface WebhookInfo {
  id: string;
  accountId: string;
  url: string;
  eventFilterJson: string;
  createdAt: string;
}

/** A scoped API/MCP key (oversight view; the secret is shown-once at mint, never
 *  here). MCP keys ARE API keys (§20.3), surfaced by their `mcp:*` scopes. */
export interface ApiKeyInfo {
  id: string;
  prefix: string;
  accountId: string;
  /** The typed scope set (opaque JSON — `mw-oauth` `Scope`). */
  scopesJson: string;
  createdAt: string;
  lastUsedAt: string | null;
  expiresAt: string | null;
  revokedAt: string | null;
  /** The key's owner asked for unattended send when minting it (in the scope). */
  unattendedSendRequested: boolean;
  /** An admin approved that request (`api_keys.unattended_send`). Only an
   *  approved, unrevoked key sends through MCP without the Outbox. */
  unattendedSendApproved: boolean;
}

/** The stored telemetry record (`GET /admin/observability`). Stored, not applied:
 *  the running log filter, OTLP exporter and metrics endpoint come from the
 *  server's environment at start. No screen reads or writes it. */
export interface ObservabilityConfig {
  logLevel: string;
  otlpDsn: string | null;
  metricsEnabled: boolean;
  sentryDsn: string | null;
}

/** Who/what performed an audited action. */
export type ActorKind = 'admin' | 'user' | 'api-key' | 'system';

/** An append-only audit-log record (`audit_log`, 0007). */
export interface AuditLogEntry {
  id: string;
  ts: string;
  actor: string;
  actorKind: ActorKind;
  action: string;
  target: string | null;
  /** Structured detail (JSON), redacted of secrets + mail content (§21.1). */
  detailJson: string;
  ip: string | null;
}

/** A ban-list entry. A record: the server does not refuse a listed address. */
export interface BanEntry {
  ip: string;
  reason: string;
  bannedAt: string;
  expiresAt: string | null;
}

/** The deployment-default appearance (`GET /admin/appearance`). Held in the
 *  server's memory only — a restart resets it — so no screen reads or writes it. */
export interface Appearance {
  theme: string;
  brandName: string;
  accent: string | null;
}

/** Input to provision (or update) a user. */
export interface ProvisionInput {
  domain: string;
  username: string;
  quota: Quota;
}

/** Input to add a ban. */
export interface BanInput {
  ip: string;
  reason: string;
  expiresAt: string | null;
}

// ── The typed client (`/admin/*` surface) ─────────────────────────────────────

/**
 * The admin REST client. Component tests supply a mock; `createHttpAdminApi`
 * is the production `fetch` impl. Every method maps to exactly one `/admin/*`
 * endpoint (documented inline).
 */
export interface AdminApi {
  /**
   * Called when a request other than the session probe or the sign-in is answered
   * `401`: the admin session has ended (idle for 30 minutes, 12 hours old, or
   * signed out elsewhere). The slice sets this to return the panel to the sign-in
   * gate. Optional so a mock need not supply it.
   */
  onSessionEnded?: () => void;
  /** `GET /admin/session` → the session, or `null` on 401 (gate). */
  session(): Promise<AdminSession | null>;
  /** `POST /admin/login` → the session (401 throws `AdminApiError`). */
  login(username: string, password: string): Promise<AdminSession>;
  /** `POST /admin/logout`. */
  logout(): Promise<void>;

  /** `GET /admin/domains`. */
  listDomains(): Promise<Domain[]>;
  /** `PUT /admin/domains/{name}` — registers the name; no body is sent. */
  saveDomain(name: string): Promise<void>;
  /** `DELETE /admin/domains/{name}`. */
  deleteDomain(name: string): Promise<void>;

  /** `GET /admin/users`. */
  listUsers(): Promise<UserSummary[]>;
  /** `POST /admin/users`. */
  provisionUser(input: ProvisionInput): Promise<void>;
  /** `PUT /admin/users/{accountId}/quota`. */
  setQuota(accountId: string, quota: Quota): Promise<void>;
  /** `PUT /admin/users/{accountId}/flags`. */
  setFlags(accountId: string, flags: UserFeatureFlags): Promise<void>;
  /** `POST /admin/users/{accountId}/zero-access` → toggle zero-access (§9). */
  toggleZeroAccess(accountId: string, on: boolean): Promise<void>;
  /** `POST /admin/users/{accountId}/revoke-sessions` → count revoked. */
  revokeSessions(accountId: string): Promise<number>;

  /** `GET /admin/security-policy`. No screen calls this — see {@link SecurityPolicy}. */
  getSecurityPolicy(): Promise<SecurityPolicy>;
  /** `PUT /admin/security-policy`. No screen calls this. */
  setSecurityPolicy(policy: SecurityPolicy): Promise<void>;

  /** `GET /admin/integrations`. */
  getIntegrations(): Promise<IntegrationsConfig>;
  /** `GET /admin/webhooks`. */
  listWebhooks(): Promise<WebhookInfo[]>;
  /** `GET /admin/api-keys`. */
  listApiKeys(): Promise<ApiKeyInfo[]>;
  /** `POST /admin/api-keys/{id}/revoke`. */
  revokeApiKey(id: string): Promise<void>;
  /** `PUT /admin/api-keys/{id}/unattended-send` — approve or withdraw the admin
   *  approval of a key's unattended send. Rejects with {@link AdminApiError}:
   *  `404` for an id that is not a key (or, when approving, a key revoked in the
   *  meantime), `409` when approving a revoked key or one that did not ask. */
  setApiKeyUnattendedSend(id: string, approved: boolean): Promise<void>;

  /** `GET /admin/observability`. No screen calls this — see {@link ObservabilityConfig}. */
  getObservability(): Promise<ObservabilityConfig>;
  /** `PUT /admin/observability`. No screen calls this. */
  setObservability(cfg: ObservabilityConfig): Promise<void>;
  /** `GET /admin/audit?limit=`. */
  listAudit(limit: number): Promise<AuditLogEntry[]>;
  /** `GET /admin/audit/export?limit=` → JSONL text. */
  exportAudit(limit: number): Promise<string>;
  /** `GET /admin/bans`. */
  listBans(): Promise<BanEntry[]>;
  /** `POST /admin/bans`. */
  addBan(input: BanInput): Promise<void>;
  /** `DELETE /admin/bans/{ip}`. */
  removeBan(ip: string): Promise<void>;

  /** `GET /admin/appearance`. No screen calls this — see {@link Appearance}. */
  getAppearance(): Promise<Appearance>;
  /** `PUT /admin/appearance`. No screen calls this. */
  setAppearance(appearance: Appearance): Promise<void>;
}

/** Raised when an `/admin/*` request fails (non-2xx that isn't a session 401). */
export class AdminApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'AdminApiError';
    this.status = status;
  }
}

/**
 * The production HTTP client. Same-origin, cookie-authed against the admin
 * session domain — it shares nothing with the JMAP client, so the mailbox path
 * is byte-unchanged. `base` lets a native shell point at a remote server (as the
 * JMAP client does); it defaults to the deploy prefix, which is `''` at the
 * origin root and `/mail` under sub-path hosting, so every `/admin/*` call
 * follows the app without each call site having to know.
 */
export function createHttpAdminApi(base = basePath()): AdminApi {
  async function raw(path: string, init?: RequestInit): Promise<Response> {
    const res = await fetch(`${base}/admin${path}`, { credentials: 'same-origin', ...init });
    // `/session` answers 401 for "not signed in" and `/login` for a wrong password;
    // on every other route a 401 means a session that existed has ended.
    if (res.status === 401 && path !== '/session' && path !== '/login') api.onSessionEnded?.();
    return res;
  }
  async function getJson<T>(path: string): Promise<T> {
    const res = await raw(path);
    if (!res.ok) throw new AdminApiError(res.status, `GET ${path} failed (${res.status})`);
    return (await res.json()) as T;
  }
  async function send(path: string, method: string, body?: unknown): Promise<Response> {
    const init: RequestInit = { method };
    if (body !== undefined) {
      init.headers = { 'content-type': 'application/json' };
      init.body = JSON.stringify(body);
    }
    const res = await raw(path, init);
    if (!res.ok) throw new AdminApiError(res.status, `${method} ${path} failed (${res.status})`);
    return res;
  }

  const api: AdminApi = {
    async session() {
      const res = await raw('/session');
      if (res.status === 401) return null;
      if (!res.ok) throw new AdminApiError(res.status, `GET /session failed (${res.status})`);
      return (await res.json()) as AdminSession;
    },
    async login(username, password) {
      const res = await raw('/login', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });
      if (res.status === 401) throw new AdminApiError(401, 'invalid admin credentials');
      if (!res.ok) throw new AdminApiError(res.status, `login failed (${res.status})`);
      return (await res.json()) as AdminSession;
    },
    async logout() {
      await send('/logout', 'POST');
    },

    listDomains: () => getJson<Domain[]>('/domains'),
    async saveDomain(name) {
      await send(`/domains/${encodeURIComponent(name)}`, 'PUT');
    },
    async deleteDomain(name) {
      await send(`/domains/${encodeURIComponent(name)}`, 'DELETE');
    },

    listUsers: () => getJson<UserSummary[]>('/users'),
    async provisionUser(input) {
      await send('/users', 'POST', input);
    },
    async setQuota(accountId, quota) {
      await send(`/users/${encodeURIComponent(accountId)}/quota`, 'PUT', quota);
    },
    async setFlags(accountId, flags) {
      await send(`/users/${encodeURIComponent(accountId)}/flags`, 'PUT', flags);
    },
    async toggleZeroAccess(accountId, on) {
      await send(`/users/${encodeURIComponent(accountId)}/zero-access`, 'POST', { on });
    },
    async revokeSessions(accountId) {
      const res = await send(`/users/${encodeURIComponent(accountId)}/revoke-sessions`, 'POST');
      const out = (await res.json()) as { count: number };
      return out.count;
    },

    getSecurityPolicy: () => getJson<SecurityPolicy>('/security-policy'),
    async setSecurityPolicy(policy) {
      await send('/security-policy', 'PUT', policy);
    },

    getIntegrations: () => getJson<IntegrationsConfig>('/integrations'),
    listWebhooks: () => getJson<WebhookInfo[]>('/webhooks'),
    listApiKeys: () => getJson<ApiKeyInfo[]>('/api-keys'),
    async revokeApiKey(id) {
      await send(`/api-keys/${encodeURIComponent(id)}/revoke`, 'POST');
    },
    async setApiKeyUnattendedSend(id, approved) {
      await send(`/api-keys/${encodeURIComponent(id)}/unattended-send`, 'PUT', { approved });
    },

    getObservability: () => getJson<ObservabilityConfig>('/observability'),
    async setObservability(cfg) {
      await send('/observability', 'PUT', cfg);
    },
    listAudit: (limit) => getJson<AuditLogEntry[]>(`/audit?limit=${limit}`),
    async exportAudit(limit) {
      const res = await raw(`/audit/export?limit=${limit}`);
      if (!res.ok) throw new AdminApiError(res.status, `export audit failed (${res.status})`);
      return res.text();
    },
    listBans: () => getJson<BanEntry[]>('/bans'),
    async addBan(input) {
      await send('/bans', 'POST', input);
    },
    async removeBan(ip) {
      await send(`/bans/${encodeURIComponent(ip)}`, 'DELETE');
    },

    getAppearance: () => getJson<Appearance>('/appearance'),
    async setAppearance(appearance) {
      await send('/appearance', 'PUT', appearance);
    },
  };
  return api;
}

// ── The reactive slice (session gate + shared api handle) ──────────────────────

/**
 * The admin panel sections, in nav order.
 *
 * There is no `security` or `appearance` section: every control either screen
 * carried saved a value nothing applied, and with the controls gone the screens
 * had nothing left to do (26.20, t28-e8). Two-factor requirements are on the
 * Require two-factor screen; a user's own appearance is in their settings.
 */
export const ADMIN_SECTIONS = [
  'domains',
  'users',
  'integrations',
  'observability',
  // V7 (plan §3 e14): the plugin registry + Assist governance sections.
  'plugins',
  'assist',
] as const;

export type AdminSection = (typeof ADMIN_SECTIONS)[number];

/** Human labels for the nav rail. */
export const ADMIN_SECTION_LABELS: Record<AdminSection, string> = {
  domains: 'Domains',
  users: 'Users',
  integrations: 'Integrations',
  observability: 'Observability',
  plugins: 'Plugins',
  assist: 'Assist',
};

export interface AdminSlice {
  /** The typed client every section calls. */
  readonly api: AdminApi;
  /** The current admin session (reactive); `null` until authenticated. */
  session: Accessor<AdminSession | null>;
  /** Whether the initial session probe has completed (gates the boot spinner). */
  sessionChecked: Accessor<boolean>;
  /** True after the server ended a session this panel was using; the sign-in gate
   *  says so. Cleared by the next successful sign-in. */
  sessionEnded: Accessor<boolean>;
  /** The visible section. */
  section: Accessor<AdminSection>;
  setSection(section: AdminSection): void;
  /** Probe `/admin/session` (called once at mount). */
  loadSession(): Promise<void>;
  /** Sign in against the admin session domain. */
  login(username: string, password: string): Promise<void>;
  /** Sign out of the admin session. */
  logout(): Promise<void>;
}

/** Build the admin slice over a client (mockable). */
export function createAdminSlice(api: AdminApi): AdminSlice {
  const [session, setSession] = createSignal<AdminSession | null>(null);
  const [sessionChecked, setSessionChecked] = createSignal(false);
  const [section, setSection] = createSignal<AdminSection>('domains');
  const [sessionEnded, setSessionEnded] = createSignal(false);

  // Only a session that was in use can "end": a 401 with no session held is the
  // ordinary signed-out state and gets no message.
  api.onSessionEnded = () => {
    if (session() === null) return;
    setSession(null);
    setSessionEnded(true);
  };

  async function loadSession(): Promise<void> {
    try {
      setSession(await api.session());
    } finally {
      setSessionChecked(true);
    }
  }

  async function login(username: string, password: string): Promise<void> {
    setSession(await api.login(username, password));
    setSessionEnded(false);
  }

  async function logout(): Promise<void> {
    await api.logout();
    setSession(null);
  }

  return {
    api,
    session,
    sessionChecked,
    sessionEnded,
    section,
    setSection,
    loadSession,
    login,
    logout,
  };
}
