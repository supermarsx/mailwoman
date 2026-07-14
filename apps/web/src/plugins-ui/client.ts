// TypeScript UI-plugin tier — same-origin, fail-soft HTTP client (t10 plan §3 e10).
//
// Consumes EXACTLY the two e11 web-host endpoints (`.orchestration/logs/t10-e11.md`):
//   • `GET  /api/ui-plugins`          → approved+enabled `UiPluginRegistration[]`
//                                       + `unsignedBanner: string[]`.
//   • `POST /api/ui-plugins/{id}/rpc` → the capability broker (the frozen
//                                       `{v,id,cap,method,args}` → `{v,id,ok|err}` envelope).
//
// Mirrors the established `modules/sso/client.ts` posture: best-effort + fail-soft. Any
// error (endpoint absent, offline, non-2xx, or `fetch` unavailable under jsdom) resolves
// to the EMPTY registry, so a deployment with no UI plugins renders exactly the baseline
// mailbox — the additive tier never breaks the mail path.

import {
  EMPTY_REGISTRY,
  RPC_PROTOCOL_VERSION,
  UI_CAPABILITIES,
  UI_EXTENSION_POINTS,
  type RpcRequest,
  type RpcResponse,
  type UiCapability,
  type UiExtensionPoint,
  type UiPluginGrant,
  type UiPluginManifest,
  type UiPluginRegistration,
  type UiPluginRegistry,
} from './types';
import { rpcError, rpcErrorResponse } from './host';

/// Fetch the approved+enabled UI-plugin tier (`GET /api/ui-plugins`). Fail-soft: resolves
/// to `EMPTY_REGISTRY` on ANY error, so no-plugins-configured renders the baseline.
export async function listUiPlugins(base = ''): Promise<UiPluginRegistry> {
  if (typeof fetch === 'undefined') return EMPTY_REGISTRY;
  try {
    const res = await fetch(`${base}/api/ui-plugins`, { credentials: 'same-origin' });
    if (!res.ok) return EMPTY_REGISTRY;
    const body = (await res.json()) as unknown;
    return normalizeRegistry(body);
  } catch {
    return EMPTY_REGISTRY;
  }
}

/// Call the server capability broker (`POST /api/ui-plugins/{id}/rpc`). Returns the
/// server's `{v,id,ok|err}` envelope; a transport/non-2xx failure is surfaced as a
/// structured `internal` RPC error (never throws) so the host broker can relay it to the
/// guest uniformly.
export async function callUiPluginRpc(
  pluginId: string,
  request: RpcRequest,
  base = '',
): Promise<RpcResponse> {
  if (typeof fetch === 'undefined') {
    return rpcErrorResponse(request.id, rpcError('internal', 'fetch unavailable'));
  }
  try {
    const res = await fetch(`${base}/api/ui-plugins/${encodeURIComponent(pluginId)}/rpc`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(request),
    });
    if (!res.ok) {
      return rpcErrorResponse(request.id, rpcError('internal', `rpc failed (${res.status})`));
    }
    const body = (await res.json()) as RpcResponse;
    return body;
  } catch {
    return rpcErrorResponse(request.id, rpcError('internal', 'rpc transport error'));
  }
}

// ── defensive wire → domain normalization ────────────────────────────────────────

/// Coerce the untrusted `GET /api/ui-plugins` body into a `UiPluginRegistry`, dropping any
/// malformed registration. Tolerant of a bare array (forward-compat, like `listSsoProviders`).
function normalizeRegistry(body: unknown): UiPluginRegistry {
  const rawPlugins = Array.isArray(body)
    ? body
    : ((body as { plugins?: unknown })?.plugins ?? []);
  const rawBanner = Array.isArray(body)
    ? []
    : ((body as { unsignedBanner?: unknown })?.unsignedBanner ?? []);
  if (!Array.isArray(rawPlugins)) return EMPTY_REGISTRY;

  const plugins: UiPluginRegistration[] = [];
  for (const raw of rawPlugins) {
    const reg = normalizeRegistration(raw);
    if (reg !== null) plugins.push(reg);
  }
  const unsignedBanner = Array.isArray(rawBanner)
    ? rawBanner.filter((x): x is string => typeof x === 'string')
    : [];
  return { plugins, unsignedBanner };
}

function normalizeRegistration(raw: unknown): UiPluginRegistration | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const r = raw as Record<string, unknown>;
  const manifest = normalizeManifest(r.manifest);
  if (manifest === null) return null;
  const grants = Array.isArray(r.grants)
    ? r.grants.map(normalizeGrant).filter((g): g is UiPluginGrant => g !== null)
    : [];
  return {
    manifest,
    grants,
    enabled: r.enabled === true,
    approved: r.approved === true,
  };
}

function normalizeManifest(raw: unknown): UiPluginManifest | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const m = raw as Record<string, unknown>;
  if (typeof m.id !== 'string' || m.id === '') return null;
  const extensionPoints = Array.isArray(m.extensionPoints)
    ? m.extensionPoints.filter((x): x is UiExtensionPoint =>
        (UI_EXTENSION_POINTS as readonly string[]).includes(x as string),
      )
    : [];
  const capabilities = Array.isArray(m.capabilities)
    ? m.capabilities.filter((x): x is UiCapability =>
        (UI_CAPABILITIES as readonly string[]).includes(x as string),
      )
    : [];
  return {
    id: m.id,
    name: typeof m.name === 'string' ? m.name : m.id,
    version: typeof m.version === 'string' ? m.version : '0.0.0',
    signature: typeof m.signature === 'string' ? m.signature : null,
    extensionPoints,
    capabilities,
    csp: typeof m.csp === 'string' ? m.csp : '',
  };
}

function normalizeGrant(raw: unknown): UiPluginGrant | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const g = raw as Record<string, unknown>;
  if (
    typeof g.capability !== 'string' ||
    !(UI_CAPABILITIES as readonly string[]).includes(g.capability)
  ) {
    return null;
  }
  const params =
    typeof g.params === 'object' && g.params !== null
      ? (g.params as Record<string, unknown>)
      : {};
  return { capability: g.capability as UiCapability, params };
}

/// Build a well-formed RPC request envelope (used by the guest SDK + tests).
export function makeRpcRequest(
  id: string,
  cap: UiCapability,
  method: string,
  args: readonly unknown[] = [],
): RpcRequest {
  return { v: RPC_PROTOCOL_VERSION, id, cap, method, args };
}
