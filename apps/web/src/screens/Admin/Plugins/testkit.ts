// A `PluginsApi` for the Plugins screen tests. NOT a test suite (vitest collects
// only `*.{test,spec}`).
//
// Every object it returns has the server's shape, and its refusals use the
// server's codes. It keeps just enough of the server's rules for the screen to be
// driven through a whole flow; the rules themselves are proven against the real
// router in `crates/mw-server/tests/t28_plugin_register.rs`. Shapes and rules come
// from:
//
//   `plugin_view`       crates/mw-server/src/plugins.rs:164   the plugin object
//   `register`          crates/mw-server/src/plugins.rs:494   409 `already-registered`
//   `enable`            crates/mw-server/src/plugins.rs:615   400 `not-approved`,
//                                                             403 `unsigned-not-allowed`
//   `grantable`         crates/mw-server/src/plugins.rs:695   400 for an undeclared capability
//   `allow_unsigned`    crates/mw-server/src/plugins.rs:784   400 `first-party`
//   `test_classifier`   crates/mw-server/src/plugins.rs:905   409 `not-loaded`
//   `refuse`            crates/mw-server/src/plugins.rs:115   the `{ error, code }` body
//   `plugin_status`     crates/mw-server/src/v7_mount.rs:2272 loaded / notLoadedReason
//   `uninstall_plugin`  crates/mw-server/src/admin_plugins.rs:270

import {
  EMPTY_ALLOWLIST,
  PluginsApiError,
  type AllowlistView,
  type ClassifierTest,
  type PluginInfo,
  type PluginsApi,
} from '../../../state/slices/plugins.ts';

/**
 * A plugin as `POST /admin/plugins {"id":"spam-rspamd"}` answers it: registered,
 * not approved, not enabled, nothing granted. Pass `over` for any other state.
 */
export function pluginInfo(over: Partial<PluginInfo> = {}): PluginInfo {
  return {
    id: 'spam-rspamd',
    name: 'Rspamd spam classifier',
    version: '26.10.0',
    firstParty: true,
    trust: 'first-party-digest',
    signed: false,
    allowUnsigned: false,
    approved: false,
    approvedBy: null,
    enabled: false,
    role: 'spam-classifier',
    capabilities: ['spam-action', 'net', 'store-kv-scoped'],
    granted: [],
    netAllowlist: ['rspamd'],
    limits: { memoryMb: 32, deadlineMs: 10000, fuel: null },
    endpoint: null,
    loaded: false,
    loadedCapabilities: [],
    restartRequired: false,
    notLoadedReason: 'not-approved',
    ...over,
  };
}

/** A third-party, unsigned plugin as registered (digest already pinned). */
export function thirdPartyInfo(over: Partial<PluginInfo> = {}): PluginInfo {
  return pluginInfo({
    id: 'acme-spam',
    name: 'Acme spam filter',
    version: '1.0.0',
    firstParty: false,
    trust: 'admin-pinned-digest',
    capabilities: ['spam-action', 'net'],
    netAllowlist: [],
    limits: { memoryMb: 64, deadlineMs: 5000, fuel: null },
    ...over,
  });
}

/** `plugin_status`, for the roles and reasons the mock can produce. */
function withStatus(p: PluginInfo): PluginInfo {
  const stopped = (notLoadedReason: PluginInfo['notLoadedReason']): PluginInfo => ({
    ...p,
    loaded: false,
    loadedCapabilities: [],
    notLoadedReason,
  });
  if (p.role === 'none') return stopped('no-host-caller');
  if (!p.approved) return stopped('not-approved');
  if (!p.enabled) return stopped('disabled');
  if (!p.signed && !p.firstParty && !p.allowUnsigned) return stopped('unsigned-not-allowed');
  if (p.role === 'account-backend') return stopped('no-account-binding');
  if (!p.granted.includes('spam-action')) return stopped('no-grant');
  return { ...p, loaded: true, loadedCapabilities: [...p.granted].sort(), notLoadedReason: null };
}

export type MockPluginsApi = PluginsApi & {
  /** Every change the screen asked for, as `name:id[:json]`. */
  readonly calls: string[];
};

export function mockPluginsApi(initial: PluginInfo[] = [], allowlist: AllowlistView = EMPTY_ALLOWLIST): MockPluginsApi {
  const calls: string[] = [];
  let current = initial;
  let view = allowlist;
  const find = (id: string): PluginInfo => {
    const p = current.find((x) => x.id === id);
    if (p === undefined) throw new PluginsApiError(404, 'unknown-plugin', `unknown plugin '${id}'`);
    return p;
  };
  const put = (next: PluginInfo): PluginInfo => {
    const done = withStatus(next);
    current = current.map((x) => (x.id === done.id ? done : x));
    return done;
  };
  return {
    calls,
    async list() {
      return current;
    },
    async register(input) {
      calls.push(`register:${JSON.stringify(input)}`);
      if (current.some((p) => p.id === input.id)) {
        throw new PluginsApiError(409, 'already-registered', `plugin '${input.id}' is already registered`);
      }
      const base = input.name === undefined ? pluginInfo({ id: input.id }) : thirdPartyInfo({ id: input.id });
      const created = withStatus({
        ...base,
        name: input.name ?? base.name,
        version: input.version ?? base.version,
        capabilities: input.capabilities ?? base.capabilities,
        netAllowlist: input.netAllowlist ?? base.netAllowlist,
      });
      current = [...current, created];
      return created;
    },
    async approve(id) {
      calls.push(`approve:${id}`);
      return put({ ...find(id), approved: true, approvedBy: 'root' });
    },
    async enable(id) {
      calls.push(`enable:${id}`);
      const p = find(id);
      if (!p.approved) {
        throw new PluginsApiError(400, 'not-approved', 'plugin must be approved before it can be enabled');
      }
      if (!p.signed && !p.firstParty && !p.allowUnsigned) {
        throw new PluginsApiError(
          403,
          'unsigned-not-allowed',
          'this plugin is unsigned and has not been allowed to run unsigned',
        );
      }
      return put({ ...p, enabled: true });
    },
    async disable(id) {
      calls.push(`disable:${id}`);
      return put({ ...find(id), enabled: false });
    },
    async grant(id, input) {
      calls.push(`grant:${id}:${JSON.stringify(input)}`);
      const p = find(id);
      const undeclared = input.capabilities.find((c) => !p.capabilities.includes(c));
      if (undeclared !== undefined) {
        throw new PluginsApiError(
          400,
          'bad-request',
          `capability '${undeclared}' is not declared by this plugin's manifest`,
        );
      }
      return put({ ...p, granted: input.capabilities });
    },
    async setAllowUnsigned(id, allow) {
      calls.push(`allow:${id}:${allow}`);
      const p = find(id);
      if (p.firstParty) {
        throw new PluginsApiError(400, 'first-party', 'the allow-unsigned flag does not apply to it');
      }
      return put({ ...p, allowUnsigned: allow });
    },
    async setEndpoint(id, endpoint) {
      calls.push(`endpoint:${id}:${endpoint}`);
      return put({ ...find(id), endpoint });
    },
    async testClassifier(id): Promise<ClassifierTest> {
      calls.push(`test:${id}`);
      const p = find(id);
      if (!p.loaded) {
        throw new PluginsApiError(409, 'not-loaded', 'plugin is not the loaded spam classifier');
      }
      // Without the `net` grant the guest's call is refused by the host and the
      // component answers `unknown` (the rspamd guest's fail-soft verdict).
      return p.granted.includes('net')
        ? { verdict: 'spam', detail: { verdict: 'spam', score: 15 } }
        : { verdict: 'unknown', detail: { verdict: 'unknown', note: 'net capability not granted' } };
    },
    async listAllowlist() {
      return view;
    },
    async approveDigest(pluginId, digestHex) {
      calls.push(`approveDigest:${pluginId}:${digestHex}`);
      view = {
        present: view.present.map((p) =>
          p.pluginId === pluginId && p.computedDigest === digestHex ? { ...p, approved: true } : p,
        ),
        pins: view.pins,
      };
    },
    async revokeDigest(pluginId, digestHex) {
      calls.push(`revokeDigest:${pluginId}:${digestHex}`);
      view = {
        present: view.present.map((p) => (p.pluginId === pluginId ? { ...p, approved: false } : p)),
        pins: view.pins,
      };
      // The server disables the plugin too.
      current = current.map((p) => (p.id === pluginId ? withStatus({ ...p, enabled: false }) : p));
    },
    async uninstall(id) {
      calls.push(`uninstall:${id}`);
      current = current.filter((p) => p.id !== id);
    },
  };
}
