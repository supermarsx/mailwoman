// TypeScript UI-plugin tier — FROZEN manifest + RPC contract (t10 plan §2.3, SPEC
// §22.2). e0 freezes these shapes so the web sandbox host (e10) and the server
// registry (e11) agree. Security-first: deny-by-default, capability-gated, rendered
// only inside a cross-origin sandboxed iframe (see `host.ts`).

/// The RPC envelope protocol version. Bump only on a breaking envelope change.
export const RPC_PROTOCOL_VERSION = 1 as const;

/// Enumerated UI extension-point slots a plugin may render into. This is a CLOSED
/// allowlist — the host renders a plugin only into slots it declares AND is granted.
export type UiExtensionPoint = 'compose-action' | 'message-toolbar' | 'settings-panel';

/// Every extension-point slot, for exhaustive host-side registry construction.
export const UI_EXTENSION_POINTS: readonly UiExtensionPoint[] = [
  'compose-action',
  'message-toolbar',
  'settings-panel',
];

/// Enumerated capabilities a plugin manifest may declare and an admin may grant.
/// Deny-by-default: a grant is intersected with the manifest's declared set, and a
/// capability never declared can never be granted.
export type UiCapability =
  | 'ui:compose-action'
  | 'ui:message-toolbar'
  | 'ui:settings-panel'
  | 'net:host-allowlist'
  | 'store:kv-scoped';

/// Every capability, for exhaustive host-side gating.
export const UI_CAPABILITIES: readonly UiCapability[] = [
  'ui:compose-action',
  'ui:message-toolbar',
  'ui:settings-panel',
  'net:host-allowlist',
  'store:kv-scoped',
];

/// The frozen `ui-plugin.json` manifest (§2.3). `signature` is a detached signature
/// over the bundle (base64), `null` when unsigned (enabling an unsigned plugin
/// requires an explicit admin `allow_unsigned` + banner). `csp` is locked host-side.
export interface UiPluginManifest {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly signature: string | null;
  readonly extensionPoints: readonly UiExtensionPoint[];
  readonly capabilities: readonly UiCapability[];
  readonly csp: string;
}

/// A granted capability plus its scoped config (e.g. the `net:host-allowlist` host
/// set, or the `store:kv-scoped` namespace). Mirrors the server 0010
/// `ui_plugin_grants` row (`mw_store::UiPluginGrantRow`).
export interface UiPluginGrant {
  readonly capability: UiCapability;
  readonly params: Readonly<Record<string, unknown>>;
}

/// A registered, admin-approved plugin as the SPA tier sees it. Absent entirely when
/// no plugin is approved (the tier does not render at all).
export interface UiPluginRegistration {
  readonly manifest: UiPluginManifest;
  readonly grants: readonly UiPluginGrant[];
  readonly enabled: boolean;
  readonly approved: boolean;
}

// ── RPC envelope (FROZEN §2.3): `{v,id,cap,method,args}` → `{v,id,ok|err}` ────────

/// A guest→host RPC request. The broker rejects a request whose `cap` is not granted
/// or whose `method` is not in the per-capability method allowlist.
export interface RpcRequest {
  readonly v: typeof RPC_PROTOCOL_VERSION;
  readonly id: string;
  readonly cap: UiCapability;
  readonly method: string;
  readonly args: readonly unknown[];
}

/// A structured RPC error (never leaks host internals to the guest).
export interface RpcError {
  readonly code: RpcErrorCode;
  readonly message: string;
}

/// The closed set of RPC error codes the broker returns.
export type RpcErrorCode =
  | 'capability-denied'
  | 'method-denied'
  | 'bad-request'
  | 'not-implemented'
  | 'internal';

/// A host→guest RPC response: either `ok` (the result) or `err`.
export type RpcResponse =
  | { readonly v: typeof RPC_PROTOCOL_VERSION; readonly id: string; readonly ok: unknown }
  | { readonly v: typeof RPC_PROTOCOL_VERSION; readonly id: string; readonly err: RpcError };
