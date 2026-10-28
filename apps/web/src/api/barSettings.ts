// The four collapsible bars and their settings-registry keys (t38-e2).
//
// Each bar is `pinned` (always shown) or `auto` (collapsed to its tab names or a
// slim handle until pointed at, tapped or focused). The value lives in the
// settings registry (`crates/mw-server/src/settings_registry.rs`,
// `ui.bar.<id>`, global + domain + user scope), so an administrator can set the
// default and lock it. Reads and writes go through `api/settings.ts`
// (`useSetting`), which sends `x-mailwoman-storage-authority` on every write;
// the settings routes take no `If-Match`. Every bar starts `pinned` (t38 D2).
//
// This file is the single source of the bar ids, modes, default and key;
// `components/bars/useBarMode.ts` re-exports them.

/** A collapsible bar. */
export type BarId = 'mailTopBar' | 'workspaceTabs' | 'mailRibbon' | 'composerRibbon';

/** Where a bar stands. */
export type BarMode = 'pinned' | 'auto';

/** The bars in Settings order. */
export const BAR_IDS: readonly BarId[] = ['mailTopBar', 'workspaceTabs', 'mailRibbon', 'composerRibbon'];

/** The modes in Settings order. */
export const BAR_MODES: readonly BarMode[] = ['pinned', 'auto'];

/** The registry default for every bar, used until the settings map loads. */
export const BAR_MODE_DEFAULT: BarMode = 'pinned';

/** The registry key of a bar: `mailTopBar` -> `ui.bar.mailTopBar`. */
export function barSettingKey(id: BarId): `ui.bar.${BarId}` {
  return `ui.bar.${id}`;
}

/** Whether a wire value is a bar mode (anything else falls back to the default). */
export function isBarMode(value: unknown): value is BarMode {
  return value === 'pinned' || value === 'auto';
}

/** The kebab-case suffix of a bar's Fluent ids: `mailTopBar` -> `mail-top-bar`. */
export function barSlug(id: BarId): string {
  return id.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
}
