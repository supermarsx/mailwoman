// Engine-plugin registry admin client and slice (SPEC §22). The wire shapes are the
// server's and this file follows them:
//
//   crates/mw-server/src/plugins.rs       — `plugins_router` (routes), `plugin_view`
//                                           (the plugin object every answer carries),
//                                           `RegisterReq`, `GrantReq`,
//                                           `AllowUnsignedReq`, `SettingsReq`,
//                                           `test_classifier`, `refuse` (the error body)
//   crates/mw-server/src/v7_mount.rs      — `NotLoaded::wire`, `TrustPolicy::wire`,
//                                           `FIRST_PARTY_MANIFESTS`,
//                                           `HIGH_POWER_CAPABILITIES`
//   crates/mw-server/src/admin_plugins.rs — the `/admin/plugins/allowlist` routes and
//                                           `uninstall_plugin`
//
// Same-origin, cookie-authed against the admin session (like admin.ts). It shares
// nothing with the JMAP client.

import { createSignal, type Accessor } from 'solid-js';
import { basePath } from '../../api/basePath.ts';

// ── Wire DTOs ────────────────────────────────────────────────────────────────

/** `mw_plugin::Capability`, kebab-case (crates/mw-plugin/src/lib.rs). */
export type PluginCapability =
  | 'account-backend'
  | 'net'
  | 'dlp-detector'
  | 'spam-action'
  | 'addrbook-source'
  | 'autoconfig-source'
  | 'message-pipeline'
  | 'store-kv-scoped';

/** Every capability, in the enum's order. */
export const PLUGIN_CAPABILITIES: readonly PluginCapability[] = [
  'account-backend',
  'net',
  'dlp-detector',
  'spam-action',
  'addrbook-source',
  'autoconfig-source',
  'message-pipeline',
  'store-kv-scoped',
];

/**
 * The capabilities reserved to first-party components (v7_mount.rs
 * `HIGH_POWER_CAPABILITIES`). The server refuses one in a third-party registration
 * or grant, so the forms do not offer it there.
 */
export const HIGH_POWER_CAPABILITIES: readonly PluginCapability[] = ['account-backend'];

/** Whether a capability is reserved to first-party components. */
export function isHighPowerCapability(cap: PluginCapability): boolean {
  return HIGH_POWER_CAPABILITIES.includes(cap);
}

/** The ids of the first-party components (v7_mount.rs `FIRST_PARTY_MANIFESTS`). */
export const FIRST_PARTY_PLUGIN_IDS: readonly string[] = [
  'bridge-graph',
  'bridge-ews',
  'bridge-gmail',
  'languagetool',
  'nextcloud',
  'spam-rspamd',
  'spam-spamassassin',
];

/** `limits` in `plugin_view`. */
export interface PluginLimits {
  readonly memoryMb: number;
  readonly deadlineMs: number;
  readonly fuel: number | null;
}

/** `TrustPolicy::wire`. */
export type PluginTrust = 'first-party-digest' | 'admin-pinned-digest';

/** `role` in `plugin_view`. */
export type PluginRole = 'account-backend' | 'spam-classifier' | 'none';

/** `NotLoaded::wire`. */
export type NotLoadedReason =
  | 'not-approved'
  | 'disabled'
  | 'unsigned-not-allowed'
  | 'no-grant'
  | 'component-unavailable'
  | 'load-failed'
  | 'proxy-mode'
  | 'no-account-binding'
  | 'no-host-caller'
  | 'another-classifier-active';

/** One registered plugin: the object `plugin_view` builds. Every key is always present. */
export interface PluginInfo {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly firstParty: boolean;
  readonly trust: PluginTrust;
  /** The manifest carries a signature. */
  readonly signed: boolean;
  /** The stored allow-unsigned flag. Always false for a first-party component. */
  readonly allowUnsigned: boolean;
  readonly approved: boolean;
  readonly approvedBy: string | null;
  /** The stored setting. Whether it runs is `loaded`. */
  readonly enabled: boolean;
  readonly role: PluginRole;
  /** What the manifest declares. */
  readonly capabilities: PluginCapability[];
  /** What a deployment-wide instance would run with. */
  readonly granted: PluginCapability[];
  readonly netAllowlist: string[];
  readonly limits: PluginLimits;
  /** A spam classifier's daemon address; null when unset or not a classifier. */
  readonly endpoint: string | null;
  /** An instance is running in the server process. */
  readonly loaded: boolean;
  readonly loadedCapabilities: PluginCapability[];
  /** The stored state differs from what runs and only a restart applies it. */
  readonly restartRequired: boolean;
  readonly notLoadedReason: NotLoadedReason | null;
}

/** `RegisterReq`. A first-party id takes `id` and optionally `netAllowlist` only. */
export interface RegisterInput {
  readonly id: string;
  readonly name?: string;
  readonly version?: string;
  readonly capabilities?: PluginCapability[];
  readonly netAllowlist?: string[];
}

/** `GrantReq`: the complete capability set for one scope. */
export interface GrantInput {
  /** null ⇒ the deployment-wide scope. */
  readonly accountId: string | null;
  readonly capabilities: PluginCapability[];
}

/** The answer of `POST /admin/plugins/{id}/test`. */
export interface ClassifierTest {
  readonly verdict: 'spam' | 'ham' | 'unknown';
  /** The component's own answer. */
  readonly detail: unknown;
}

// ── Third-party allowlist DTOs (admin_plugins.rs `list_allowlist`) ───────────

/** A third-party component file on disk, with the digest the server computed. */
export interface AllowlistPresent {
  readonly pluginId: string;
  readonly computedDigest: string;
  readonly firstParty: boolean;
  /** A non-revoked pin already matches this digest. */
  readonly approved: boolean;
}

/** A stored allowlist pin. A revoked pin is kept for oversight. */
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

/** The body of `GET /admin/plugins/allowlist`. */
export interface AllowlistView {
  readonly present: AllowlistPresent[];
  readonly pins: AllowlistPin[];
}

/** The empty allowlist view (initial slice state before the first load). */
export const EMPTY_ALLOWLIST: AllowlistView = { present: [], pins: [] };

/** A refused or failed `/admin/plugins/*` request. `code` is the server's (`refuse`). */
export class PluginsApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = 'PluginsApiError';
    this.status = status;
    this.code = code;
  }
}

/** The plugin-registry admin client. Component tests supply a mock. */
export interface PluginsApi {
  /** GET /admin/plugins → `{ plugins }`. */
  list(): Promise<PluginInfo[]>;
  /** POST /admin/plugins. */
  register(input: RegisterInput): Promise<PluginInfo>;
  approve(id: string): Promise<PluginInfo>;
  enable(id: string): Promise<PluginInfo>;
  disable(id: string): Promise<PluginInfo>;
  /** POST /admin/plugins/{id}/grant — replaces the scope's grants. */
  grant(id: string, input: GrantInput): Promise<PluginInfo>;
  /** POST /admin/plugins/{id}/allow-unsigned `{ allow }`. */
  setAllowUnsigned(id: string, allow: boolean): Promise<PluginInfo>;
  /** POST /admin/plugins/{id}/settings `{ endpoint }`. */
  setEndpoint(id: string, endpoint: string | null): Promise<PluginInfo>;
  /** POST /admin/plugins/{id}/test. */
  testClassifier(id: string): Promise<ClassifierTest>;
  /** GET /admin/plugins/allowlist. */
  listAllowlist(): Promise<AllowlistView>;
  /** POST /admin/plugins/allowlist — pin the exact `(pluginId, digestHex)`. */
  approveDigest(pluginId: string, digestHex: string): Promise<void>;
  /** POST /admin/plugins/allowlist/{pluginId}/{digestHex}/revoke. */
  revokeDigest(pluginId: string, digestHex: string): Promise<void>;
  /** POST /admin/plugins/{id}/uninstall. */
  uninstall(id: string): Promise<void>;
}

/** The production HTTP client. `base` defaults to the deploy prefix. */
export function createHttpPluginsApi(base = basePath()): PluginsApi {
  async function call(path: string, method: string, body?: unknown): Promise<unknown> {
    const init: RequestInit = { method, credentials: 'same-origin' };
    if (body !== undefined) {
      init.headers = { 'content-type': 'application/json' };
      init.body = JSON.stringify(body);
    }
    const res = await fetch(`${base}/admin/plugins${path}`, init);
    const answer: unknown = await res.json().catch(() => null);
    if (!res.ok) {
      const fields = (answer ?? {}) as { error?: unknown; code?: unknown };
      throw new PluginsApiError(
        res.status,
        typeof fields.code === 'string' ? fields.code : 'http-error',
        typeof fields.error === 'string' ? fields.error : `${method} ${path} (${res.status})`,
      );
    }
    return answer;
  }
  const plugin = async (path: string, body?: unknown): Promise<PluginInfo> =>
    ((await call(path, 'POST', body)) as { plugin: PluginInfo }).plugin;
  const at = (id: string, action: string): string => `/${encodeURIComponent(id)}/${action}`;
  return {
    list: async () => ((await call('', 'GET')) as { plugins: PluginInfo[] }).plugins,
    register: (input) => plugin('', input),
    approve: (id) => plugin(at(id, 'approve')),
    enable: (id) => plugin(at(id, 'enable')),
    disable: (id) => plugin(at(id, 'disable')),
    grant: (id, input) => plugin(at(id, 'grant'), input),
    setAllowUnsigned: (id, allow) => plugin(at(id, 'allow-unsigned'), { allow }),
    setEndpoint: (id, endpoint) => plugin(at(id, 'settings'), { endpoint }),
    testClassifier: async (id) => (await call(at(id, 'test'), 'POST')) as ClassifierTest,
    listAllowlist: async () => (await call('/allowlist', 'GET')) as AllowlistView,
    approveDigest: async (pluginId, digestHex) => {
      await call('/allowlist', 'POST', { pluginId, digestHex });
    },
    revokeDigest: async (pluginId, digestHex) => {
      await call(`/allowlist/${encodeURIComponent(pluginId)}/${encodeURIComponent(digestHex)}/revoke`, 'POST');
    },
    uninstall: async (id) => {
      await call(at(id, 'uninstall'), 'POST');
    },
  };
}

// ── The reactive slice ───────────────────────────────────────────────────────

export interface PluginsSlice {
  readonly api: PluginsApi;
  plugins: Accessor<PluginInfo[]>;
  loading: Accessor<boolean>;
  /** The last read of the registry failed; the list shown is the one before it. */
  loadFailed: Accessor<boolean>;
  /** The last refused or failed change, until the next change succeeds. */
  lastError: Accessor<PluginsApiError | null>;
  /** A third-party plugin without a signature is loaded (drives the banner). */
  hasUnsignedLoaded: Accessor<boolean>;
  load(): Promise<void>;
  /** Each change resolves to whether the server accepted it. */
  register(input: RegisterInput): Promise<boolean>;
  approve(id: string): Promise<boolean>;
  enable(id: string): Promise<boolean>;
  disable(id: string): Promise<boolean>;
  grant(id: string, input: GrantInput): Promise<boolean>;
  setAllowUnsigned(id: string, allow: boolean): Promise<boolean>;
  setEndpoint(id: string, endpoint: string | null): Promise<boolean>;
  // ── Third-party allowlist ──────────────────────────────────────────────────
  allowlist: Accessor<AllowlistView>;
  allowlistLoading: Accessor<boolean>;
  /** The last read of the allowlist failed. */
  allowlistLoadFailed: Accessor<boolean>;
  loadAllowlist(): Promise<void>;
  approveDigest(pluginId: string, digestHex: string): Promise<void>;
  revokeDigest(pluginId: string, digestHex: string): Promise<void>;
  uninstall(id: string): Promise<void>;
}

/** Whether any third-party plugin without a signature is loaded. */
export function anyUnsignedLoaded(plugins: PluginInfo[]): boolean {
  return plugins.some((p) => p.loaded && !p.signed && !p.firstParty);
}

/** Build the plugins slice over a client (mockable). */
export function createPluginsSlice(api: PluginsApi): PluginsSlice {
  const [plugins, setPlugins] = createSignal<PluginInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [loadFailed, setLoadFailed] = createSignal(false);
  const [allowlistLoadFailed, setAllowlistLoadFailed] = createSignal(false);
  const [lastError, setLastError] = createSignal<PluginsApiError | null>(null);
  const [allowlist, setAllowlist] = createSignal<AllowlistView>(EMPTY_ALLOWLIST);
  const [allowlistLoading, setAllowlistLoading] = createSignal(false);

  async function load(): Promise<void> {
    setLoading(true);
    try {
      setPlugins(await api.list());
      setLoadFailed(false);
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }

  async function loadAllowlist(): Promise<void> {
    setAllowlistLoading(true);
    try {
      setAllowlist(await api.listAllowlist());
      setAllowlistLoadFailed(false);
    } catch {
      setAllowlistLoadFailed(true);
    } finally {
      setAllowlistLoading(false);
    }
  }

  // A change to one plugin can change another's state (one classifier seat), so the
  // whole list is read again rather than patched with the answer.
  async function change(fn: () => Promise<unknown>): Promise<boolean> {
    try {
      await fn();
      setLastError(null);
      await load();
      return true;
    } catch (e) {
      if (!(e instanceof PluginsApiError)) throw e;
      setLastError(e);
      return false;
    }
  }

  // Revoke and uninstall also change the registry (disable / remove the plugin).
  async function changeAllowlist(fn: () => Promise<void>): Promise<void> {
    await fn();
    await Promise.all([loadAllowlist(), load()]);
  }

  return {
    api,
    plugins,
    loading,
    loadFailed,
    lastError,
    hasUnsignedLoaded: () => anyUnsignedLoaded(plugins()),
    load,
    register: (input) => change(() => api.register(input)),
    approve: (id) => change(() => api.approve(id)),
    enable: (id) => change(() => api.enable(id)),
    disable: (id) => change(() => api.disable(id)),
    grant: (id, input) => change(() => api.grant(id, input)),
    setAllowUnsigned: (id, allow) => change(() => api.setAllowUnsigned(id, allow)),
    setEndpoint: (id, endpoint) => change(() => api.setEndpoint(id, endpoint)),
    allowlist,
    allowlistLoading,
    allowlistLoadFailed,
    loadAllowlist,
    approveDigest: (pluginId, digestHex) => changeAllowlist(() => api.approveDigest(pluginId, digestHex)),
    revokeDigest: (pluginId, digestHex) => changeAllowlist(() => api.revokeDigest(pluginId, digestHex)),
    uninstall: (id) => changeAllowlist(() => api.uninstall(id)),
  };
}
