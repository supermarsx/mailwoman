// FROZEN SharedWorker store protocol (plan §2.6). Implemented by e6
// (worker/**). A SharedWorker owns the JMAP store per browser profile; tabs
// talk to it via this `postMessage` envelope. When SharedWorker is unavailable
// (private windows / Safari), the same `broadcast` frames travel over a
// BroadcastChannel and each tab runs the store locally. The per-tab
// `createAppState` becomes a proxy implementing the existing `AppState` over
// this protocol — `AppState`'s public shape is preserved (§2.6).

export type EnvelopeKind = 'req' | 'res' | 'broadcast';

/** Tab↔worker message envelope (frozen §2.6). */
export interface WorkerEnvelope {
  /** Correlates a `res` with its `req`; ignored for `broadcast`. */
  id: string;
  kind: EnvelopeKind;
  /** Store method for `req`, or `"state"` for a `broadcast`. */
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: unknown;
}

/** BroadcastChannel name carrying the same `broadcast` frames (SharedWorker-less). */
export const BROADCAST_CHANNEL = 'mw-store';

/**
 * The broadcast the worker fans out to all ports on any state change. `params`
 * is an `AppStateDelta` whose concrete shape e6 defines against the store.
 */
export interface StateBroadcast extends WorkerEnvelope {
  kind: 'broadcast';
  method: 'state';
  params: unknown;
}
