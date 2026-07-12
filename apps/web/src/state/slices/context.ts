// Shared wiring passed to every store slice (plan §3 e0 store-slices refactor).
//
// The store is split into disjoint slices under `state/slices/` so the five web
// executors (e4 theme, e5 offline, e6 realtime, e7 mail/tags/outbox) own
// separate files and never collide on `store.ts`. Each slice is a factory
// `createXxxSlice(ctx) => XxxSlice`; `store.ts` composes them into the frozen
// `AppState` (its public accessors + actions are UNCHANGED by this refactor).

import type { Client } from '../../api/client.ts';
import type { Email } from '../../api/jmap-types.ts';
import type { OutboundType } from '../../contracts/offline.ts';
import type { OfflineQuery } from '../../offline/search.ts';
import type { ToastKind } from '../store.ts';

/** Cross-slice dependencies handed to each slice factory by `store.ts`. */
export interface SliceContext {
  readonly client: Client;
  /** Transient toast (auto-clears after `ttlMs`). Owned by the store core. */
  showToast(kind: ToastKind, message: string, ttlMs?: number): void;

  // ── V2 integration seams (t4-e13). All OPTIONAL and additive: `store.ts`
  // populates them once the offline/core slices exist; a slice built in
  // isolation (its unit tests) simply gets `undefined` and takes the direct/
  // online path, so the frozen per-slice behaviour is unchanged. ──
  /** Live network status; when `false`, mutating slices queue instead of calling. */
  online?(): boolean;
  /** Queue a mutation for offline replay (offline slice; drained on reconnect). */
  enqueueOffline?(type: OutboundType, payload: unknown): Promise<void>;
  /** Reduced offline search over the cached header slice (offline slice). */
  searchOffline?(query: OfflineQuery): Email[];
  /** Notify peer tabs a mutation happened so they refetch (multi-window sync). */
  broadcastChange?(): void;
}
