// Theme slice (plan §3 e4). Owns the design-token theme/density/accent state
// (theme/contract.css.ts, §2.3), the `data-theme`/`data-density` runtime switch
// synced to server settings, and the ribbon layout preset. Empty at scaffold
// time; e4 fills it, extending `AppState` in `store.ts`.

import type { SliceContext } from './context.ts';

/** Filled by e4. `never`-valued so it contributes nothing to `AppState` yet. */
export type ThemeSlice = Record<string, never>;

export function createThemeSlice(_ctx: SliceContext): ThemeSlice {
  return {};
}
