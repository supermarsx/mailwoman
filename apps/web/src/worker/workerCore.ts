// SharedWorker-side store core (plan §2.6). The worker owns the single store
// per browser profile; every tab connects a port. The core answers `req` frames
// from a handler map and, whenever a handler reports a state delta, fans a
// `broadcast` frame out to *all* connected ports so every tab stays in sync.
//
// The store handlers themselves (backed by the JMAP client) are wired at
// integration; this core is store-agnostic and driven entirely by the injected
// map, which keeps it unit-testable with plain fake ports under jsdom.

import { resEnvelope, serializeError, type PortLike } from './protocol.ts';
import { broadcastEnvelope } from './protocol.ts';
import type { WorkerEnvelope } from '../contracts/worker.ts';

/** A store method: returns its result; the second arg publishes a state delta. */
export type StoreHandler = (
  params: unknown,
  broadcast: (delta: unknown) => void,
) => Promise<unknown> | unknown;

export interface WorkerCore {
  /** Register a freshly-connected tab port. */
  connect(port: PortLike): void;
  /** Broadcast a delta to every connected port (e.g. from a push StateChange). */
  broadcast(delta: unknown): void;
  /** Number of live ports — for teardown assertions/diagnostics. */
  portCount(): number;
}

export function createWorkerCore(handlers: Record<string, StoreHandler>): WorkerCore {
  const ports = new Set<PortLike>();

  function broadcast(delta: unknown): void {
    const frame = broadcastEnvelope(delta);
    for (const p of ports) p.postMessage(frame);
  }

  async function dispatch(port: PortLike, env: WorkerEnvelope): Promise<void> {
    const handler = env.method !== undefined ? handlers[env.method] : undefined;
    if (handler === undefined) {
      port.postMessage(resEnvelope(env.id, undefined, { message: `unknown method "${env.method ?? '?'}"` }));
      return;
    }
    try {
      const result = await handler(env.params, broadcast);
      port.postMessage(resEnvelope(env.id, result));
    } catch (err) {
      port.postMessage(resEnvelope(env.id, undefined, serializeError(err)));
    }
  }

  return {
    connect(port): void {
      ports.add(port);
      port.onmessage = (ev: { data: unknown }): void => {
        const env = ev.data as WorkerEnvelope;
        if (env === null || typeof env !== 'object' || env.kind !== 'req') return;
        void dispatch(port, env);
      };
      port.start?.();
    },
    broadcast,
    portCount(): number {
      return ports.size;
    },
  };
}
