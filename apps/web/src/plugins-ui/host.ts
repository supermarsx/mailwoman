// TypeScript UI-plugin tier — sandboxed-iframe host + deny-by-default broker gate
// (t10 plan §2.3/§6, SPEC §22.2). This is the security core the whole tier hangs off;
// the live postMessage wiring lives in `broker.ts`, the HTTP registry/broker client in
// `client.ts`, and the SolidJS tier + trust banner in `Tier.tsx`.
//
// SECURITY MODEL (frozen — every consumer must preserve it):
//   * A plugin renders ONLY inside a sandboxed `<iframe srcdoc sandbox="allow-scripts">`
//     with NO `allow-same-origin`. The browser therefore assigns the frame an OPAQUE
//     origin (`event.origin === "null"`): it cannot reach the host's cookies,
//     localStorage, DOM, session token, or `window.parent`.
//   * The frame's own CSP (`LOCKED_PLUGIN_CSP`, injected host-side into the srcdoc — the
//     manifest's `csp` is advisory only) sets `connect-src 'none'`, so the guest can make
//     NO direct network request; all egress is host-proxied under a `net:host-allowlist`
//     grant via the broker.
//   * Guest→host calls arrive as `postMessage`. The host validates the message ORIGIN
//     (must be the exact sandboxed frame's window, opaque origin), validates the message
//     SHAPE, then forwards ONLY allow-listed capability methods to the server broker.
//     Everything else is rejected `method-denied`/`capability-denied` — deny-by-default,
//     mirroring the e11 server broker (`CAP_METHOD_ALLOWLIST` ⇔ e11 `cap_methods`).

import {
  RPC_PROTOCOL_VERSION,
  UI_CAPABILITIES,
  type RpcError,
  type RpcErrorCode,
  type RpcRequest,
  type RpcResponse,
  type UiCapability,
  type UiPluginGrant,
  type UiPluginManifest,
  type UiPluginRegistration,
} from './types';

/// The iframe `sandbox` attribute value. Deliberately `allow-scripts` ONLY — adding
/// `allow-same-origin` would collapse the opaque-origin barrier and is FORBIDDEN.
export const PLUGIN_IFRAME_SANDBOX = 'allow-scripts' as const;

/// The host-owned Content-Security-Policy injected into every guest document. Locked
/// host-side (§2.3): the manifest's `csp` is advisory and never widens this. `connect-src
/// 'none'` is the load-bearing line — the guest cannot open ANY socket, so the only path
/// to the network is the host broker's `net:host-allowlist/fetch` proxy.
export const LOCKED_PLUGIN_CSP =
  "default-src 'none'; " +
  "script-src 'unsafe-inline'; " +
  "style-src 'unsafe-inline'; " +
  "img-src data:; " +
  "font-src data:; " +
  "connect-src 'none'; " +
  "form-action 'none'; " +
  "base-uri 'none'; " +
  "frame-ancestors 'none'";

/// The per-capability method allowlist (deny-by-default). A method not listed for a
/// granted capability is rejected by the broker. Mirrors the e11 server `cap_methods`
/// EXACTLY — a divergence would let one tier admit a call the other denies. `ui:*` render
/// capabilities expose no guest-initiated methods (render is host-driven).
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

/// Wrap an `RpcError` as the `{v,id,err}` response envelope returned to the guest.
export function rpcErrorResponse(id: string, error: RpcError): RpcResponse {
  return { v: RPC_PROTOCOL_VERSION, id, err: error };
}

/// Whether `granted` includes `cap`.
function hasCapability(granted: readonly UiPluginGrant[], cap: UiCapability): boolean {
  return granted.some((g) => g.capability === cap);
}

/// The broker gate: decide whether a request is permitted for a plugin's grants.
/// Returns `null` when allowed, or the `RpcError` to return to the guest otherwise.
/// PURE + deny-by-default; `broker.ts` runs this before ANY network forward.
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

/// Validate + narrow an untrusted `postMessage` payload to an `RpcRequest`. Returns
/// `null` for anything malformed (wrong version, missing/invalid fields, or an
/// unknown capability) — the broker drops those without answering, since a payload it
/// cannot trust has no trustworthy `id` to reply to.
export function parseRpcRequest(data: unknown): RpcRequest | null {
  if (typeof data !== 'object' || data === null) return null;
  const d = data as Record<string, unknown>;
  if (d.v !== RPC_PROTOCOL_VERSION) return null;
  if (typeof d.id !== 'string' || d.id === '') return null;
  if (typeof d.cap !== 'string') return null;
  if (typeof d.method !== 'string') return null;
  if (!(UI_CAPABILITIES as readonly string[]).includes(d.cap)) return null;
  const args = Array.isArray(d.args) ? (d.args as readonly unknown[]) : [];
  return { v: RPC_PROTOCOL_VERSION, id: d.id, cap: d.cap as UiCapability, method: d.method, args };
}

/// Whether a `message` event is a trusted guest→host call from `frameWindow`. The
/// primary check is SOURCE IDENTITY — the event must come from the exact sandboxed
/// frame's `contentWindow`. Because that frame is opaque-origin, its `origin` MUST be
/// `"null"`; a message presenting any concrete origin is NOT our sandboxed guest and is
/// rejected. This is the inbound half of the escape barrier (see the e15 hook).
export function isTrustedGuestEvent(
  event: Pick<MessageEvent, 'source' | 'origin'>,
  frameWindow: Window | null,
): boolean {
  if (frameWindow === null) return false;
  if (event.source !== frameWindow) return false;
  // Opaque origins serialize to the literal string "null". Anything else (a concrete,
  // potentially same-origin sender) is not our sandboxed frame → reject.
  return event.origin === 'null' || event.origin === '';
}

/// Build the guest document served into the sandboxed iframe's `srcdoc`. The host injects
/// its LOCKED CSP (not the manifest's) plus the guest SDK shim — a tiny bridge that lets
/// guest code call `mailwoman.rpc(cap, method, ...args)` over `postMessage` and awaits the
/// host's response. `bootstrap` is the (already host-vetted) guest entry script, if any.
export function buildGuestSrcdoc(manifest: UiPluginManifest, bootstrap = ''): string {
  const guestSdk = `
(() => {
  const pending = new Map();
  let seq = 0;
  window.addEventListener('message', (e) => {
    const m = e.data;
    if (!m || typeof m !== 'object' || typeof m.id !== 'string') return;
    const p = pending.get(m.id);
    if (!p) return;
    pending.delete(m.id);
    if ('ok' in m) p.resolve(m.ok);
    else p.reject((m.err && m.err.code) || 'internal');
  });
  window.mailwoman = {
    pluginId: ${JSON.stringify(manifest.id)},
    rpc(cap, method, ...args) {
      const id = ${JSON.stringify(manifest.id)} + ':' + (++seq);
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        parent.postMessage({ v: ${RPC_PROTOCOL_VERSION}, id, cap, method, args }, '*');
      });
    },
  };
})();`;
  return (
    '<!doctype html><html><head><meta charset="utf-8">' +
    `<meta http-equiv="Content-Security-Policy" content="${LOCKED_PLUGIN_CSP}">` +
    '</head><body><div id="mw-plugin-root"></div>' +
    `<script>${guestSdk}</script>` +
    (bootstrap ? `<script>${bootstrap}</script>` : '') +
    '</body></html>'
  );
}

/// The sandboxed-iframe host for one plugin. Knows how to build the correctly-sandboxed
/// opaque-origin frame; `broker.ts` wires the live postMessage broker to its
/// `contentWindow`. Rendering is normally done declaratively by `Tier.tsx`; this
/// imperative builder exists for non-Solid callers and for the unit tests that assert the
/// origin barrier.
export class UiPluginHost {
  private readonly registration: UiPluginRegistration;

  constructor(registration: UiPluginRegistration) {
    this.registration = registration;
  }

  /// The plugin this host renders.
  get id(): string {
    return this.registration.manifest.id;
  }

  /// The plugin's granted capabilities (deny-by-default: only what the admin granted).
  get grants(): readonly UiPluginGrant[] {
    return this.registration.grants;
  }

  /// Whether the plugin is approved + enabled (the tier renders nothing otherwise).
  get active(): boolean {
    return this.registration.approved && this.registration.enabled;
  }

  /// Create the opaque-origin sandboxed iframe the plugin renders in. Returns `null`
  /// outside a DOM (SSR/tests without `document`). The frame carries the guest document
  /// via `srcdoc` ONLY — never a `src` to a host origin, and never `allow-same-origin`.
  createSandboxedFrame(bootstrap = ''): HTMLIFrameElement | null {
    if (typeof document === 'undefined') {
      return null;
    }
    const frame = document.createElement('iframe');
    frame.setAttribute('sandbox', PLUGIN_IFRAME_SANDBOX);
    frame.setAttribute('title', `plugin:${this.registration.manifest.id}`);
    frame.referrerPolicy = 'no-referrer';
    frame.srcdoc = buildGuestSrcdoc(this.registration.manifest, bootstrap);
    return frame;
  }
}
