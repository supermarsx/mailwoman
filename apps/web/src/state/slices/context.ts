// Shared wiring passed to every store slice (plan §3 e0 store-slices refactor).
//
// The store is split into disjoint slices under `state/slices/` so the five web
// executors (e4 theme, e5 offline, e6 realtime, e7 mail/tags/outbox) own
// separate files and never collide on `store.ts`. Each slice is a factory
// `createXxxSlice(ctx) => XxxSlice`; `store.ts` composes them into the frozen
// `AppState` (its public accessors + actions are UNCHANGED by this refactor).

import type { Client } from '../../api/client.ts';
import type { ToastKind } from '../store.ts';

/** Cross-slice dependencies handed to each slice factory by `store.ts`. */
export interface SliceContext {
  readonly client: Client;
  /** Transient toast (auto-clears after `ttlMs`). Owned by the store core. */
  showToast(kind: ToastKind, message: string, ttlMs?: number): void;
}
