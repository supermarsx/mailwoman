// Offline slice (plan §3 e5). Owns the Service-Worker app-shell state, the OPFS
// encrypted cache + IndexedDB outbound queue (contracts/offline.ts, §2.5), and
// the reduced offline-search surface. Empty at scaffold time; e5 fills it,
// extending `AppState` in `store.ts`.

import type { SliceContext } from './context.ts';

/** Filled by e5. `never`-valued so it contributes nothing to `AppState` yet. */
export type OfflineSlice = Record<string, never>;

export function createOfflineSlice(_ctx: SliceContext): OfflineSlice {
  return {};
}
