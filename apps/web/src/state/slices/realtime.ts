// Realtime slice (plan §3 e6). Owns the WebSocket/EventSource push client
// (contracts/push.ts, §2.2), the connection-status model, the sub-tab strip,
// and the change reconciler — all bundled in a `RealtimeController`. This slice
// constructs the app-wide controller, registers it as the `useRealtime()`
// singleton so components resolve it, bridges the fetch layer's offline signal
// into the connection model, and surfaces connection status + push lifecycle on
// `AppState`. It does NOT auto-connect: the app calls `startRealtime()` once a
// session exists, keeping construction inert under jsdom (which has no
// WebSocket/EventSource) so the existing web tests are unaffected.

import type { Accessor } from 'solid-js';
import {
  createRealtimeController,
  type RealtimeController,
  type RealtimeControllerOptions,
} from '../../realtime/controller.ts';
import { setGlobalRealtime } from '../../realtime/context.ts';
import type { ConnectionState } from '../../realtime/connection.ts';
import type { PushTransport, StateChange } from '../../contracts/push.ts';
import type { SliceContext } from './context.ts';

export interface RealtimeSlice {
  /** The live push connection status, for the connection-status toast (§2.2). */
  connectionState: Accessor<ConnectionState>;
  /** The transport currently carrying pushes (ws / sse / poll / offline). */
  pushTransport: Accessor<PushTransport>;
  /** Open the push transport + begin reacting to `StateChange` (call post-login). */
  startRealtime(): void;
  /** Tear the push transport down (logout / teardown). */
  stopRealtime(): void;
  /** Force a reconnect from the top of the transport ladder (toast action). */
  reconnectRealtime(): void;
  /** Subscribe to decoded `StateChange`s (integration wires the changes refetch). */
  onRealtimeChange(handler: (change: StateChange) => void): () => void;
}

export function createRealtimeSlice(
  ctx: SliceContext,
  opts: RealtimeControllerOptions = {},
): RealtimeSlice {
  const controller: RealtimeController = createRealtimeController(opts);
  // Register as the app-wide singleton so components using `useRealtime()`
  // without an explicit provider resolve this controller.
  setGlobalRealtime(controller);

  // Bridge the fetch layer's network signal into the connection model: a dropped
  // request marks the push connection offline before the socket layer notices.
  ctx.client.onNetwork((up) => {
    if (!up) controller.connection.setOffline();
  });

  return {
    connectionState: controller.connection.state,
    pushTransport: controller.connection.transport,
    startRealtime: () => controller.start(),
    stopRealtime: () => controller.stop(),
    reconnectRealtime: () => controller.reconnect(),
    onRealtimeChange: (handler) => controller.onStateChange(handler),
  };
}
