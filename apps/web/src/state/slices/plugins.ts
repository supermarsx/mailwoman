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
}

/** Whether the given set of plugins includes an enabled, unsigned one. */
export function anyUnsignedEnabled(plugins: PluginInfo[]): boolean {
  return plugins.some((p) => p.enabled && !p.signed);
}

/** Build the plugins slice over a client (mockable). */
export function createPluginsSlice(api: PluginsApi): PluginsSlice {
  const [plugins, setPlugins] = createSignal<PluginInfo[]>([]);
  const [loading, setLoading] = createSignal(false);

  async function load(): Promise<void> {
    setLoading(true);
    try {
      setPlugins(await api.list());
    } finally {
      setLoading(false);
    }
  }

  async function mutate(fn: () => Promise<void>): Promise<void> {
    await fn();
    await load();
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
  };
}
