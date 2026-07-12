// Tab-side store proxy (plan §2.6). The per-tab `createAppState` becomes a thin
// proxy over one of two transports, chosen by feature detection:
//
//   • SharedWorker present  → the worker owns the single store; tabs send `req`
//     frames and receive `broadcast` state deltas over the worker port.
//   • SharedWorker absent    → each tab runs the store locally and syncs deltas
//     to peers over the `mw-store` BroadcastChannel (broadcast.ts).
//
// Both are exposed behind one `StoreTransport` so the proxy — and the tests —
// don't branch on which is live. The DOM `SharedWorker`/`BroadcastChannel`
// constructors are injectable so the ladder is unit-testable under jsdom, which
// ships neither.

import {
  Correlator,
  broadcastEnvelope,
  reqEnvelope,
  type PortLike,
} from './protocol.ts';
import { broadcastChannelAvailable, openStoreChannel } from './broadcast.ts';
import type { WorkerEnvelope } from '../contracts/worker.ts';

/** The uniform surface a tab uses regardless of which transport is live. */
export interface StoreTransport {
  /** Which transport was selected — for diagnostics + the tests. */
  readonly mode: 'worker' | 'channel';
  /** Invoke a store method. In worker mode it round-trips to the worker; in
   *  channel mode it runs the tab-local store handler. */
  request(method: string, params?: unknown): Promise<unknown>;
  /** Fan a state delta out to peer tabs (channel mode); no-op in worker mode,
   *  where the worker is the sole broadcaster. */
  publish(delta: unknown): void;
  /** Subscribe to inbound state deltas from the worker / peer tabs. */
  onBroadcast(cb: (delta: unknown) => void): () => void;
  /** Detach handlers and close the underlying port/channel. */
  close(): void;
}

/** A minimal structural view of the DOM `SharedWorker` we rely on. */
export interface SharedWorkerLike {
  port: PortLike;
}
export type SharedWorkerCtor = new (url: string | URL, name?: string) => SharedWorkerLike;

function fanout(subs: Set<(delta: unknown) => void>, delta: unknown): void {
  for (const cb of subs) cb(delta);
}

/** Route inbound frames on a port to the correlator (res) or subscribers (broadcast). */
function attach(port: PortLike, correlator: Correlator, subs: Set<(d: unknown) => void>): void {
  port.onmessage = (ev: { data: unknown }): void => {
    const env = ev.data as WorkerEnvelope;
    if (env === null || typeof env !== 'object') return;
    if (env.kind === 'res') {
      correlator.handle(env);
    } else if (env.kind === 'broadcast' && env.method === 'state') {
      fanout(subs, env.params);
    }
  };
  port.start?.();
}

/** SharedWorker transport: the worker owns the store; the tab is a pure client. */
export function workerTransport(port: PortLike): StoreTransport {
  const correlator = new Correlator();
  const subs = new Set<(delta: unknown) => void>();
  attach(port, correlator, subs);
  return {
    mode: 'worker',
    request(method, params): Promise<unknown> {
      return correlator.request(reqEnvelope(method, params), (env) => port.postMessage(env));
    },
    publish(): void {
      // The worker is the sole broadcaster; a tab publishing is a no-op.
    },
    onBroadcast(cb): () => void {
      subs.add(cb);
      return () => subs.delete(cb);
    },
    close(): void {
      correlator.rejectAll(new Error('store transport closed'));
      port.onmessage = null;
      port.close?.();
    },
  };
}

/** The tab-local store the channel transport forwards requests to (fallback mode). */
export type LocalRequest = (method: string, params?: unknown) => Promise<unknown> | unknown;

/**
 * BroadcastChannel transport: no worker, so requests run against the tab-local
 * store and state deltas are gossiped to peer tabs over the channel. Inbound
 * peer deltas reach subscribers; the channel never echoes a tab's own frames.
 */
export function channelTransport(port: PortLike, local: LocalRequest): StoreTransport {
  const subs = new Set<(delta: unknown) => void>();
  port.onmessage = (ev: { data: unknown }): void => {
    const env = ev.data as WorkerEnvelope;
    if (env === null || typeof env !== 'object') return;
    if (env.kind === 'broadcast' && env.method === 'state') fanout(subs, env.params);
  };
  port.start?.();
  return {
    mode: 'channel',
    async request(method, params): Promise<unknown> {
      return local(method, params);
    },
    publish(delta): void {
      port.postMessage(broadcastEnvelope(delta));
    },
    onBroadcast(cb): () => void {
      subs.add(cb);
      return () => subs.delete(cb);
    },
    close(): void {
      port.onmessage = null;
      port.close?.();
    },
  };
}

export interface StoreProxyOptions {
  /** Worker script URL; defaults to the co-located `sharedWorker.ts` bundle. */
  workerUrl?: string | URL;
  /** Injectable `SharedWorker` ctor (tests pass a fake; jsdom has none). */
  SharedWorkerImpl?: SharedWorkerCtor;
  /** Injectable channel port (tests pass a fake `BroadcastChannel`). */
  channelPort?: PortLike | null;
  /** The tab-local store handler used only in channel/fallback mode. */
  localRequest: LocalRequest;
}

/** True when this runtime can host the shared store in a `SharedWorker`. */
export function sharedWorkerAvailable(impl?: SharedWorkerCtor): boolean {
  return (impl ?? (globalThis as { SharedWorker?: SharedWorkerCtor }).SharedWorker) !== undefined;
}

/**
 * Pick the best store transport for this runtime: SharedWorker when present,
 * else the BroadcastChannel fallback, else a channel-less local-only transport
 * (private window with neither — a lone tab still works, just without sync).
 */
export function createStoreProxy(opts: StoreProxyOptions): StoreTransport {
  const SW =
    opts.SharedWorkerImpl ?? (globalThis as { SharedWorker?: SharedWorkerCtor }).SharedWorker;
  if (SW !== undefined) {
    const url = opts.workerUrl ?? new URL('./sharedWorker.ts', import.meta.url);
    try {
      const worker = new SW(url, 'mw-store');
      return workerTransport(worker.port);
    } catch {
      // Fall through to the channel path (e.g. SharedWorker blocked by policy).
    }
  }
  const port =
    opts.channelPort !== undefined ? opts.channelPort : broadcastChannelAvailable() ? openStoreChannel() : null;
  if (port !== null) return channelTransport(port, opts.localRequest);
  // Neither transport: a single isolated tab. Requests still run locally; peer
  // sync is simply absent (nothing to sync with).
  return channelTransport(nullPort(), opts.localRequest);
}

/** A do-nothing port for the isolated-tab case (no peers to reach). */
function nullPort(): PortLike {
  return { postMessage(): void {}, onmessage: null };
}
