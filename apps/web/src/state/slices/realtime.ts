// Realtime slice (plan §3 e6). Owns the WebSocket/EventSource push client
// (contracts/push.ts, §2.2), the SharedWorker store proxy (contracts/worker.ts,
// §2.6), the sub-tab strip, and the connection-status toast upgrade. Empty at
// scaffold time; e6 fills it, extending `AppState` in `store.ts`.

import type { SliceContext } from './context.ts';

/** Filled by e6. `never`-valued so it contributes nothing to `AppState` yet. */
export type RealtimeSlice = Record<string, never>;

export function createRealtimeSlice(_ctx: SliceContext): RealtimeSlice {
  return {};
}
