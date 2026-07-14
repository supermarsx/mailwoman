// Admin panel client + slice (SPEC §19, plan §2.5 / §2.6, §3 e7).
//
// The admin panel drives a SEPARATE session domain (`mw_admin_session` cookie or a
// separate port; passkey-capable — plan §2.5) over a small REST surface under
// `/admin/*`, distinct from the cookie-authed JMAP surface the mailbox uses. This
// file owns the TYPED client (`AdminApi` + `createHttpAdminApi`) — the exact wire
// shape e11 mounts against — plus the reactive `AdminSlice` the screens consume.
//
// The client is an interface so component tests inject a mock; the HTTP impl is a
// thin `fetch` wrapper (same-origin, cookie-authed) that never touches the JMAP
// client, so the normal mailbox path is untouched (regression gate).

import { createSignal, type Accessor } from 'solid-js';

// ── Wire DTOs (the frozen `/admin/*` JSON contract e11 satisfies) ──────────────

/** An authenticated admin session (`GET /admin/session`). */
export interface AdminSession {
  username: string;
}

/** A managed mail domain (`domains`, 0007). */
export interface Domain {
  name: string;
  /** Upstream routing config (opaque JSON blob per §19). */
  upstreamJson: string;
  allowlist: string[];
  blocklist: string[];
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

/** The security-policy model (§19 security-policy section). */
export interface SecurityPolicy {
  minTls: string;
  require2fa: boolean;
  argon2MCost: number;
  argon2TCost: number;
  argon2PCost: number;
  dlpRulesJson: string;
  maxSecurityFloor: boolean;
  capturePolicy: string;
}

/** Whether an integration is live now or a deferred (inert) config surface. */
export type IntegrationStatus = 'active' | 'deferred';

/** The integrations surface (§19): webhooks + API-key oversight live; LDAP/
 *  Nextcloud shown inert/deferred. */
export interface IntegrationsConfig {
  webhooks: IntegrationStatus;
  apiKeyOversight: IntegrationStatus;
  ldap: IntegrationStatus;
  nextcloud: IntegrationStatus;
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
}

/** Observability configuration (§19 observability section). */
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

/** A banned source (login-monitor / ban-list, fail2ban-compatible). */
export interface BanEntry {
  ip: string;
  reason: string;
  bannedAt: string;
  expiresAt: string | null;
}

/** Admin-managed appearance (§19 appearance section). */
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

// ── The typed client (frozen `/admin/*` surface) ──────────────────────────────

/**
 * The admin REST client. Component tests supply a mock; `createHttpAdminApi`
 * is the production `fetch` impl. Every method maps to exactly one `/admin/*`
 * endpoint (documented inline — this is the contract e11 mounts).
 */
export interface AdminApi {
  /** `GET /admin/session` → the session, or `null` on 401 (gate). */
  session(): Promise<AdminSession | null>;
  /** `POST /admin/login` → the session (401 throws `AdminApiError`). */
  login(username: string, password: string): Promise<AdminSession>;
  /** `POST /admin/logout`. */
  logout(): Promise<void>;

  /** `GET /admin/domains`. */
  listDomains(): Promise<Domain[]>;
  /** `PUT /admin/domains/{name}`. */
  saveDomain(domain: Domain): Promise<void>;
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

  /** `GET /admin/security-policy`. */
  getSecurityPolicy(): Promise<SecurityPolicy>;
  /** `PUT /admin/security-policy`. */
  setSecurityPolicy(policy: SecurityPolicy): Promise<void>;

  /** `GET /admin/integrations`. */
  getIntegrations(): Promise<IntegrationsConfig>;
  /** `GET /admin/webhooks`. */
  listWebhooks(): Promise<WebhookInfo[]>;
  /** `GET /admin/api-keys`. */
  listApiKeys(): Promise<ApiKeyInfo[]>;
  /** `POST /admin/api-keys/{id}/revoke`. */
  revokeApiKey(id: string): Promise<void>;

  /** `GET /admin/observability`. */
  getObservability(): Promise<ObservabilityConfig>;
  /** `PUT /admin/observability`. */
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

  /** `GET /admin/appearance`. */
  getAppearance(): Promise<Appearance>;
  /** `PUT /admin/appearance`. */
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
 * JMAP client does), defaulting to same-origin in the browser.
 */
export function createHttpAdminApi(base = ''): AdminApi {
  async function raw(path: string, init?: RequestInit): Promise<Response> {
    return fetch(`${base}/admin${path}`, { credentials: 'same-origin', ...init });
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

  return {
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
    async saveDomain(domain) {
      await send(`/domains/${encodeURIComponent(domain.name)}`, 'PUT', domain);
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
}

// ── The reactive slice (session gate + shared api handle) ──────────────────────

/** The admin panel sections (§19), in nav order. */
export const ADMIN_SECTIONS = [
  'domains',
  'users',
  'security',
  'integrations',
  'observability',
  'appearance',
  // V7 (plan §3 e14): the plugin registry + Assist governance sections.
  'plugins',
  'assist',
] as const;

export type AdminSection = (typeof ADMIN_SECTIONS)[number];

/** Human labels for the nav rail. */
export const ADMIN_SECTION_LABELS: Record<AdminSection, string> = {
  domains: 'Domains',
  users: 'Users',
  security: 'Security policy',
  integrations: 'Integrations',
  observability: 'Observability',
  appearance: 'Appearance',
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

  async function loadSession(): Promise<void> {
    try {
      setSession(await api.session());
    } finally {
      setSessionChecked(true);
    }
  }

  async function login(username: string, password: string): Promise<void> {
    setSession(await api.login(username, password));
  }

  async function logout(): Promise<void> {
    await api.logout();
    setSession(null);
  }

  return {
    api,
    session,
    sessionChecked,
    section,
    setSection,
    loadSession,
    login,
    logout,
  };
}
