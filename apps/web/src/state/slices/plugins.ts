// Plugin-registry admin client + slice (SPEC §22, plan §2.6 / §3 e7). Owns the typed
// `/admin/plugins/*` surface (the contract e9 fills + e14 mounts over the `mw-plugin`
// host + the 0008 `plugins`/`plugin_grants` tables) plus the reactive slice the Admin
// → Plugins screen consumes. Same-origin, cookie-authed against the admin session
// domain (like admin.ts) — it shares nothing with the JMAP client, so the mailbox path
// is byte-unchanged. Disjoint file — no `store.ts` collision.

import { createSignal, type Accessor } from 'solid-js';

// ── Wire DTOs (the frozen `/admin/plugins/*` JSON contract e9 satisfies) ──────────

/** The capabilities a plugin may declare (mirrors `mw_plugin::Capability`, kebab). */
export type PluginCapability =
  | 'account-backend'
  | 'net'
  | 'dlp-detector'
  | 'spam-action'
  | 'addrbook-source'
  | 'autoconfig-source'
  | 'message-pipeline'
  | 'store-kv-scoped';

/**
 * The high-power capability set (the account-backend / send-as-user class). The server
 * REFUSES any of these at grant time to a third-party (non-first-party) plugin — the
 * gate is provenance-based and cannot be overridden by admin action (26.15 t15 e6:
 * `HIGH_POWER_CAPABILITIES = [Capability::AccountBackend]`). The web surface mirrors the
 * list so it can show these as not-grantable-to-third-party rather than letting an admin
 * attempt a grant the server will reject.
 */
export const HIGH_POWER_CAPABILITIES: readonly PluginCapability[] = ['account-backend'];

/** Whether a capability is high-power (first-party-only; never grantable to third-party). */
export function isHighPowerCapability(cap: PluginCapability): boolean {
  return HIGH_POWER_CAPABILITIES.includes(cap);
}

/** Resource limits declared in a plugin manifest (mirrors `mw_plugin::PluginLimits`). */
export interface PluginLimits {
  readonly memoryMb: number;
  readonly deadlineMs: number;
  readonly fuel: number | null;
}

/** A registry plugin row (mirrors the server projection of the 0008 `plugins` table). */
export interface PluginInfo {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  /** Whether the component carries a valid detached signature over its bytes. An
   *  unsigned plugin can only run under `allowUnsigned` and raises a permanent banner. */
  readonly signed: boolean;
  /** Admin approval state (approve gate before it can be enabled). */
  readonly approved: boolean;
  readonly enabled: boolean;
  /** Whether the admin has opted this unsigned plugin in (`allow_unsigned` policy). */
  readonly allowUnsigned: boolean;
  readonly capabilities: PluginCapability[];
  readonly netAllowlist: string[];
  readonly limits: PluginLimits;
}

/** A per-account capability grant for a plugin (`plugin_grants`). */
export interface GrantInput {
  /** Omit / null ⇒ a deployment-wide grant; else scope to one account. */
  readonly accountId: string | null;
  readonly capability: PluginCapability;
}

// ── Third-party allowlist DTOs (the `/admin/plugins/allowlist` contract, e6) ──────
//
// The trust surface for the ONLY security-core loosening in 26.15: `resolve_component`
// loads a NON-first-party component only if its exact on-disk SHA-256 matches a
// non-revoked admin-approved pin. This client feeds the admin review panel — the digest
// shown here is the digest the admin is approving, computed by the server over the exact
// on-disk bytes.

/** A third-party component present on disk in `MW_THIRDPARTY_PLUGIN_DIR`, with the digest
 *  the server computed over its exact bytes (the value the admin approves). `firstParty`
 *  flags an id that collides with a first-party component — the first-party pin always
 *  takes precedence and such an id can never be third-party-approved. */
export interface AllowlistPresent {
  readonly pluginId: string;
  readonly computedDigest: string;
  readonly firstParty: boolean;
  /** True when a non-revoked pin already matches this exact computed digest. */
  readonly approved: boolean;
}

/** A stored allowlist pin (an admin-approved byte-exact identity + its provenance). A
 *  revoked pin is retained for oversight; it no longer admits the component. */
export interface AllowlistPin {
  readonly pluginId: string;
  readonly digestHex: string;
  readonly name: string | null;
  readonly version: string | null;
  readonly source: string | null;
  readonly note: string | null;
  readonly approvedBy: string;
  readonly approvedAt: string;
  readonly revoked: boolean;
}

/** The `GET /admin/plugins/allowlist` projection: present-on-disk components joined with
 *  the stored pins (including revoked rows, for oversight). */
export interface AllowlistView {
  readonly present: AllowlistPresent[];
  readonly pins: AllowlistPin[];
}

/** The empty allowlist view (initial slice state before the first load). */
export const EMPTY_ALLOWLIST: AllowlistView = { present: [], pins: [] };

/** Raised when an `/admin/plugins/*` request fails. */
export class PluginsApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'PluginsApiError';
    this.status = status;
  }
}

/**
 * The plugin-registry admin client. Component tests supply a mock; `createHttpPluginsApi`
 * is the production `fetch` impl. Endpoints (e9 to satisfy, e14 to mount):
 *   GET  /admin/plugins                    → PluginInfo[]
 *   POST /admin/plugins/{id}/approve
 *   POST /admin/plugins/{id}/enable
 *   POST /admin/plugins/{id}/disable
 *   POST /admin/plugins/{id}/grant  (GrantInput)
 */
export interface PluginsApi {
  list(): Promise<PluginInfo[]>;
  approve(id: string): Promise<void>;
  enable(id: string): Promise<void>;
  disable(id: string): Promise<void>;
  grant(id: string, input: GrantInput): Promise<void>;
  /** Toggle the `allow_unsigned` policy for an unsigned plugin. */
  setAllowUnsigned(id: string, allow: boolean): Promise<void>;
  /** GET /admin/plugins/allowlist — present-on-disk third-party components + stored pins. */
  listAllowlist(): Promise<AllowlistView>;
  /** POST /admin/plugins/allowlist — pin the exact `(pluginId, digestHex)` shown for review. */
  approveDigest(pluginId: string, digestHex: string): Promise<void>;
  /** POST /admin/plugins/allowlist/{pluginId}/{digestHex}/revoke — revoke the pin + disable. */
  revokeDigest(pluginId: string, digestHex: string): Promise<void>;
  /** POST /admin/plugins/{id}/uninstall — purge the plugin's KV, delete its pins, disable it. */
  uninstall(id: string): Promise<void>;
}

/** The production HTTP client. Same-origin, cookie-authed against the admin domain. */
export function createHttpPluginsApi(base = ''): PluginsApi {
  async function raw(path: string, init?: RequestInit): Promise<Response> {
    return fetch(`${base}/admin/plugins${path}`, { credentials: 'same-origin', ...init });
  }
  async function send(path: string, method: string, body?: unknown): Promise<void> {
    const init: RequestInit = { method };
    if (body !== undefined) {
      init.headers = { 'content-type': 'application/json' };
      init.body = JSON.stringify(body);
    }
    const res = await raw(path, init);
    if (!res.ok) throw new PluginsApiError(res.status, `${method} ${path} failed (${res.status})`);
  }
  return {
    async list() {
      const res = await raw('');
      if (!res.ok) throw new PluginsApiError(res.status, `list plugins failed (${res.status})`);
      return (await res.json()) as PluginInfo[];
    },
    approve: (id) => send(`/${encodeURIComponent(id)}/approve`, 'POST'),
    enable: (id) => send(`/${encodeURIComponent(id)}/enable`, 'POST'),
    disable: (id) => send(`/${encodeURIComponent(id)}/disable`, 'POST'),
    grant: (id, input) => send(`/${encodeURIComponent(id)}/grant`, 'POST', input),
    setAllowUnsigned: (id, allow) => send(`/${encodeURIComponent(id)}/allow-unsigned`, 'POST', { allow }),
    async listAllowlist() {
      const res = await raw('/allowlist');
      if (!res.ok) throw new PluginsApiError(res.status, `list allowlist failed (${res.status})`);
      return (await res.json()) as AllowlistView;
    },
    approveDigest: (pluginId, digestHex) => send('/allowlist', 'POST', { pluginId, digestHex }),
    revokeDigest: (pluginId, digestHex) =>
      send(`/allowlist/${encodeURIComponent(pluginId)}/${encodeURIComponent(digestHex)}/revoke`, 'POST'),
    uninstall: (id) => send(`/${encodeURIComponent(id)}/uninstall`, 'POST'),
  };
}

// ── The reactive slice ─────────────────────────────────────────────────────────

export interface PluginsSlice {
  readonly api: PluginsApi;
  plugins: Accessor<PluginInfo[]>;
  loading: Accessor<boolean>;
  /** True when any ENABLED plugin is running unsigned (drives the permanent banner). */
  hasUnsignedEnabled: Accessor<boolean>;
  load(): Promise<void>;
  approve(id: string): Promise<void>;
  enable(id: string): Promise<void>;
  disable(id: string): Promise<void>;
  grant(id: string, input: GrantInput): Promise<void>;
  setAllowUnsigned(id: string, allow: boolean): Promise<void>;
  // ── Third-party allowlist ──────────────────────────────────────────────────
  allowlist: Accessor<AllowlistView>;
  allowlistLoading: Accessor<boolean>;
  loadAllowlist(): Promise<void>;
  approveDigest(pluginId: string, digestHex: string): Promise<void>;
  revokeDigest(pluginId: string, digestHex: string): Promise<void>;
  uninstall(id: string): Promise<void>;
}

/** Whether the given set of plugins includes an enabled, unsigned one. */
export function anyUnsignedEnabled(plugins: PluginInfo[]): boolean {
  return plugins.some((p) => p.enabled && !p.signed);
}

/** Build the plugins slice over a client (mockable). */
export function createPluginsSlice(api: PluginsApi): PluginsSlice {
  const [plugins, setPlugins] = createSignal<PluginInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [allowlist, setAllowlist] = createSignal<AllowlistView>(EMPTY_ALLOWLIST);
  const [allowlistLoading, setAllowlistLoading] = createSignal(false);

  async function load(): Promise<void> {
    setLoading(true);
    try {
      setPlugins(await api.list());
    } finally {
      setLoading(false);
    }
  }

  async function loadAllowlist(): Promise<void> {
    setAllowlistLoading(true);
    try {
      setAllowlist(await api.listAllowlist());
    } finally {
      setAllowlistLoading(false);
    }
  }

  async function mutate(fn: () => Promise<void>): Promise<void> {
    await fn();
    await load();
  }

  // Allowlist mutations re-read the allowlist (and the registry, since revoke/uninstall
  // also disable the plugin, which the registry list reflects).
  async function mutateAllowlist(fn: () => Promise<void>): Promise<void> {
    await fn();
    await loadAllowlist();
  }

  return {
    api,
    plugins,
    loading,
    hasUnsignedEnabled: () => anyUnsignedEnabled(plugins()),
    load,
    approve: (id) => mutate(() => api.approve(id)),
    enable: (id) => mutate(() => api.enable(id)),
    disable: (id) => mutate(() => api.disable(id)),
    grant: (id, input) => mutate(() => api.grant(id, input)),
    setAllowUnsigned: (id, allow) => mutate(() => api.setAllowUnsigned(id, allow)),
    allowlist,
    allowlistLoading,
    loadAllowlist,
    approveDigest: (pluginId, digestHex) => mutateAllowlist(() => api.approveDigest(pluginId, digestHex)),
    revokeDigest: (pluginId, digestHex) => mutateAllowlist(() => api.revokeDigest(pluginId, digestHex)),
    uninstall: (id) => mutateAllowlist(() => api.uninstall(id)),
  };
}
