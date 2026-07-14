// TypeScript UI-plugin tier — live host↔guest postMessage RPC broker (t10 plan §6).
//
// This is the runtime half of the security core in `host.ts`. It listens for
// `postMessage` calls from a sandboxed guest frame and, deny-by-default:
//   1. drops any message that is NOT from the exact sandboxed frame's opaque-origin
//      window (`isTrustedGuestEvent`) — foreign-origin / spoofed senders never proceed;
//   2. drops any message whose shape is not a valid `RpcRequest` (`parseRpcRequest`) —
//      no trustworthy `id` to answer, so it is silently ignored;
//   3. rejects any request whose capability is not granted (`capability-denied`) or
//      whose method is not in that capability's allowlist (`method-denied`) — LOCALLY,
//      so a denied call never touches the network;
//   4. forwards ONLY surviving allow-listed calls to the server broker
//      (`POST /api/ui-plugins/{id}/rpc`) and relays the `{v,id,ok|err}` back to the guest.
//
// The broker NEVER exposes host globals, the session cookie/token, or the mailbox client
// to the guest — the only thing that crosses back is the structured RPC response the
// guest itself asked for.

import { callUiPluginRpc } from './client';
import {
  brokerReject,
  isTrustedGuestEvent,
  parseRpcRequest,
  rpcErrorResponse,
  rpcError,
} from './host';
import type { RpcRequest, RpcResponse, UiPluginGrant } from './types';

/// Everything the broker needs to service one plugin's frame.
export interface BrokerWiring {
  /// The plugin id — the `/api/ui-plugins/{id}/rpc` path segment.
  readonly pluginId: string;
  /// The plugin's granted capabilities (deny-by-default gate input).
  readonly grants: readonly UiPluginGrant[];
  /// The sandboxed frame's `contentWindow`. Only messages whose `source` IS this window
  /// are trusted; `null` (frame not mounted) trusts nothing.
  readonly frameWindow: Window | null;
  /// Deliver a response back to the guest. Default wiring posts to `frameWindow` with
  /// targetOrigin `'*'` (an opaque-origin frame cannot be addressed by a concrete
  /// origin; the payload is only the result the guest requested).
  readonly post?: (response: RpcResponse) => void;
  /// Forward an allow-listed call to the server broker. Injectable for tests; defaults to
  /// the same-origin `callUiPluginRpc` HTTP client.
  readonly rpc?: (pluginId: string, request: RpcRequest) => Promise<RpcResponse>;
  /// Base URL for the server broker (passed through to the default `rpc`).
  readonly base?: string;
}

/// The classification of an inbound guest message, BEFORE any network call. Pure +
/// side-effect-free so tests (and the e15 escape-gate hook) can assert the decision
/// without a live window or server.
export type BrokerDecision =
  | { readonly kind: 'ignore'; readonly reason: 'foreign-origin' | 'malformed' }
  | { readonly kind: 'reject'; readonly response: RpcResponse }
  | { readonly kind: 'forward'; readonly request: RpcRequest };

/// Decide what to do with a raw `message` event WITHOUT performing the forward. This is
/// the deny-by-default heart of the broker and the documented **e15 escape-gate hook**:
/// drive it with a foreign-origin event, a spoofed `source`, or an ungranted/method-denied
/// request and assert the decision is `ignore`/`reject` — a plugin attempting to read host
/// cookies/DOM/token can only ride this channel, and it is provably blocked here.
export function classifyMessage(
  event: Pick<MessageEvent, 'source' | 'origin' | 'data'>,
  grants: readonly UiPluginGrant[],
  frameWindow: Window | null,
): BrokerDecision {
  if (!isTrustedGuestEvent(event, frameWindow)) {
    return { kind: 'ignore', reason: 'foreign-origin' };
  }
  const request = parseRpcRequest(event.data);
  if (request === null) {
    return { kind: 'ignore', reason: 'malformed' };
  }
  const denied = brokerReject(grants, request);
  if (denied) {
    return { kind: 'reject', response: rpcErrorResponse(request.id, denied) };
  }
  return { kind: 'forward', request };
}

/// Handle one inbound guest message end-to-end: classify, then (for `forward`) proxy to
/// the server broker and relay the response. `ignore` produces no reply.
export async function handleGuestMessage(
  event: Pick<MessageEvent, 'source' | 'origin' | 'data'>,
  wiring: BrokerWiring,
): Promise<void> {
  const decision = classifyMessage(event, wiring.grants, wiring.frameWindow);
  if (decision.kind === 'ignore') return;

  const post = wiring.post ?? defaultPost(wiring.frameWindow);
  if (decision.kind === 'reject') {
    post(decision.response);
    return;
  }

  const rpc = wiring.rpc ?? ((id, req) => callUiPluginRpc(id, req, wiring.base ?? ''));
  try {
    post(await rpc(wiring.pluginId, decision.request));
  } catch {
    post(rpcErrorResponse(decision.request.id, rpcError('internal', 'broker request failed')));
  }
}

/// Default response channel: post back into the sandboxed frame (targetOrigin `'*'`,
/// since an opaque-origin frame has no addressable concrete origin). No-op when the frame
/// is not mounted.
function defaultPost(frameWindow: Window | null): (response: RpcResponse) => void {
  return (response) => {
    frameWindow?.postMessage(response, '*');
  };
}

/// Attach the live broker to `target` (normally the top `window`). Every `message` event
/// is routed through `handleGuestMessage`, which itself filters to the specific frame.
/// Returns a disconnect function that removes the listener — the tier calls it on unmount
/// so a torn-down plugin can no longer drive the broker.
export function attachBroker(target: Window, wiring: BrokerWiring): () => void {
  const listener = (event: MessageEvent): void => {
    void handleGuestMessage(event, wiring);
  };
  target.addEventListener('message', listener);
  return () => target.removeEventListener('message', listener);
}
