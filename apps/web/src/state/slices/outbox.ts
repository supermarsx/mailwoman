// Outbox slice (plan §3 e7). Owns the visible send queue: undo-send, send-later,
// and the honest Outbox backed by `EmailSubmission/query` (§2.1). Empty at
// scaffold time; e7 fills it, extending `AppState` in `store.ts`.

import type { SliceContext } from './context.ts';

/** Filled by e7. `never`-valued so it contributes nothing to `AppState` yet. */
export type OutboxSlice = Record<string, never>;

export function createOutboxSlice(_ctx: SliceContext): OutboxSlice {
  return {};
}
