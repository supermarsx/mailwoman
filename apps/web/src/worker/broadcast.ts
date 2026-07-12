// BroadcastChannel fallback wiring (plan §2.6). When a SharedWorker is
// unavailable (private windows / Safari), each tab runs the store locally and
// syncs `broadcast` frames over a BroadcastChannel named `mw-store`. This module
// feature-detects the channel and adapts it to the `PortLike` shape the proxy
// and worker core speak, so both transports share one code path.

import { BROADCAST_CHANNEL } from '../contracts/worker.ts';
import type { PortLike } from './protocol.ts';

/** True when the runtime provides `BroadcastChannel` (jsdom does not). */
export function broadcastChannelAvailable(): boolean {
  return typeof globalThis.BroadcastChannel !== 'undefined';
}

/**
 * Open the shared `mw-store` BroadcastChannel as a `PortLike`, or `null` when
 * `BroadcastChannel` is unavailable. A real `BroadcastChannel` does not deliver
 * a message back to the sender, so peers never see their own broadcasts.
 */
export function openStoreChannel(name = BROADCAST_CHANNEL): PortLike | null {
  if (!broadcastChannelAvailable()) return null;
  const channel = new BroadcastChannel(name);
  const port: PortLike = {
    postMessage(data: unknown): void {
      channel.postMessage(data);
    },
    onmessage: null,
    close(): void {
      channel.close();
    },
  };
  channel.onmessage = (ev: MessageEvent): void => {
    port.onmessage?.({ data: ev.data });
  };
  return port;
}
