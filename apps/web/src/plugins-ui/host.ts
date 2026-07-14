// TypeScript UI-plugin tier — inert sandboxed-iframe host + RPC broker (t10 plan
// §2.3/§6, SPEC §22.2). e0 scaffolds the security-critical shapes; e10 fills the
// live postMessage/MessageChannel wiring and the extension-point registry.
//
// SECURITY MODEL (frozen; e10 must preserve):
//   * A plugin renders ONLY inside a cross-origin sandboxed `<iframe srcdoc
//     sandbox="allow-scripts">` — NO `allow-same-origin`, so the iframe gets an
//     opaque origin: no access to host cookies, localStorage, the DOM, or
//     `window.parent`.
//   * The iframe's own CSP sets `connect-src 'none'`; all network is host-proxied
//     under the `net:host-allowlist` grant only.
//   * Guest→host calls go over a MessageChannel; the broker rejects any request
//     whose capability is not granted, or whose method is not in that capability's
//     method allowlist (deny-by-default).

import {
  RPC_PROTOCOL_VERSION,
  type RpcError,
  type RpcErrorCode,
  type RpcRequest,
  type RpcResponse,
  type UiCapability,
  type UiPluginGrant,
  type UiPluginRegistration,
} from './types';

/// The iframe `sandbox` attribute value. Deliberately `allow-scripts` ONLY — adding
/// `allow-same-origin` would collapse the origin barrier and is FORBIDDEN.
export const PLUGIN_IFRAME_SANDBOX = 'allow-scripts' as const;

/// The per-capability method allowlist (deny-by-default). A method not listed for a
/// granted capability is rejected by the broker. e10/e11 extend these lists as the
/// guest SDK grows; a `ui:*` render capability exposes no host methods (render is
/// driven by structured RPC from the host, not guest-initiated calls).
export const CAP_METHOD_ALLOWLIST: Readonly<Record<UiCapability, readonly string[]>> = {
  'ui:compose-action': [],
  'ui:message-toolbar': [],
  'ui:settings-panel': [],
  'net:host-allowlist': ['fetch'],
  'store:kv-scoped': ['get', 'put'],
};

/// Build a structured RPC error.
export function rpcError(code: RpcErrorCode, message: string): RpcError {
  return { code, message };
}

/// Whether `granted` includes `cap`.
function hasCapability(granted: readonly UiPluginGrant[], cap: UiCapability): boolean {
  return granted.some((g) => g.capability === cap);
}

/// The broker gate: decide whether a request is permitted for a plugin's grants.
/// Returns `null` when allowed, or the `RpcError` to return to the guest otherwise.
/// PURE + inert — e10 wires this into the live MessageChannel handler.
export function brokerReject(
  granted: readonly UiPluginGrant[],
  req: RpcRequest,
): RpcError | null {
  if (req.v !== RPC_PROTOCOL_VERSION) {
    return rpcError('bad-request', 'unsupported RPC protocol version');
  }
  if (!hasCapability(granted, req.cap)) {
    return rpcError('capability-denied', `capability not granted: ${req.cap}`);
  }
  const methods = CAP_METHOD_ALLOWLIST[req.cap];
  if (!methods.includes(req.method)) {
    return rpcError('method-denied', `method not allowed for ${req.cap}: ${req.method}`);
  }
  return null;
}

/// Wrap a broker outcome as an `RpcResponse` for the guest. The `ok` path is
/// deliberately `not-implemented` in the scaffold — e10 supplies the real handlers.
export function brokerRespond(
  granted: readonly UiPluginGrant[],
  req: RpcRequest,
): RpcResponse {
  const denied = brokerReject(granted, req);
  if (denied) {
    return { v: RPC_PROTOCOL_VERSION, id: req.id, err: denied };
  }
  return {
    v: RPC_PROTOCOL_VERSION,
    id: req.id,
    err: rpcError('not-implemented', 'the UI-plugin RPC broker is not wired in this build'),
  };
}

/// The inert sandboxed-iframe host. e10 fills the live rendering + MessageChannel
/// setup; today it only knows how to build the correctly-sandboxed iframe element
/// and gate requests through the broker, so the security posture is testable early.
export class UiPluginHost {
  private readonly registration: UiPluginRegistration;

  constructor(registration: UiPluginRegistration) {
    this.registration = registration;
  }

  /// The plugin this host renders.
  get id(): string {
    return this.registration.manifest.id;
  }

  /// Whether the plugin is approved + enabled (the tier renders nothing otherwise).
  get active(): boolean {
    return this.registration.approved && this.registration.enabled;
  }

  /// Create the cross-origin sandboxed iframe the plugin renders in. Returns `null`
  /// outside a DOM (SSR/tests without `document`). e10 sets `srcdoc` to the wrapped
  /// guest bundle + the locked per-plugin CSP; here the element is created with the
  /// correct sandbox + no `src`, proving the origin barrier is in place.
  createSandboxedFrame(): HTMLIFrameElement | null {
    if (typeof document === 'undefined') {
      return null;
    }
    const frame = document.createElement('iframe');
    frame.setAttribute('sandbox', PLUGIN_IFRAME_SANDBOX);
    frame.setAttribute('title', `plugin:${this.registration.manifest.id}`);
    // No `allow-same-origin`, no `src` to a host origin — e10 sets `srcdoc` only.
    frame.referrerPolicy = 'no-referrer';
    return frame;
  }

  /// Gate + respond to a guest RPC request through the broker (inert in the scaffold).
  handleRequest(req: RpcRequest): RpcResponse {
    return brokerRespond(this.registration.grants, req);
  }
}
