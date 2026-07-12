// Tags/labels slice (plan §3 e7). Owns the label + color-registry state: tag
// pickers, pins, snooze, sweep, follow-up — reading the extra `Email` props in
// jmap-types.ts (§2.1). Empty at scaffold time; e7 fills it, extending
// `AppState` in `store.ts` as it adds accessors/actions.

import type { SliceContext } from './context.ts';

/** Filled by e7. `never`-valued so it contributes nothing to `AppState` yet. */
export type TagsSlice = Record<string, never>;

export function createTagsSlice(_ctx: SliceContext): TagsSlice {
  return {};
}
