// `useBarMode(id)`: whether one collapsible bar is pinned or auto-hides (t38 §2.1).
//
// The mode is a settings-registry key (`ui.bar.<id>`, `pinned|auto`, global +
// domain + user scope), so an administrator can set the default and lock it.
// Until the map loads, or against a server that does not know the key yet, the
// mode is the registry default `pinned`. A change shows at once and goes back
// with a toast when the server refuses it.
//
// Narrow screens (t39 D4): below 600 px the composer ribbon starts in `auto`
// (tab names only), so the body keeps most of the height on a phone. That is
// only a default: it applies while nobody has chosen, that is while the value
// is the registry default. A value the user stored, or one an administrator
// set or locked at any scope, always wins.

import { createSignal, onCleanup, type Accessor } from 'solid-js';
import { useSetting } from '../../screens/Settings/useSetting.ts';
import { toast } from '../toast/toast.ts';
import { t } from '../../i18n/index.ts';
import type { SettingEntry } from '../../api/settings.ts';

// The bar ids, modes, default and key all live in `api/barSettings.ts` (the
// single source, shared with Settings -> Toolbars); they are re-exported here.
import { BAR_MODE_DEFAULT, barSettingKey, isBarMode, type BarId, type BarMode } from '../../api/barSettings.ts';

export { BAR_IDS, BAR_MODES, BAR_MODE_DEFAULT, barSettingKey, barSlug, isBarMode } from '../../api/barSettings.ts';
export type { BarId, BarMode } from '../../api/barSettings.ts';

/** The registry default for every bar (user decision D2); an alias of `BAR_MODE_DEFAULT`. */
export const DEFAULT_BAR_MODE: BarMode = BAR_MODE_DEFAULT;

export interface BarModeHandle {
  /** The mode on screen: a pending change first, then the stored value, then `pinned`. */
  mode: Accessor<BarMode>;
  /** An administrator holds the value; the pin toggle is disabled. */
  locked: Accessor<boolean>;
  /** A write is in flight. */
  saving: Accessor<boolean>;
  /** The last failed write as localised text, or ''. */
  error: Accessor<string>;
  /** Show `mode` now and store it. Resolves `false` (and reverts) when the server refuses it. */
  setMode(mode: BarMode): Promise<boolean>;
}

/** Bars with a different default on narrow screens (t39 D4). */
export const NARROW_DEFAULTS: Readonly<Partial<Record<BarId, { query: string; mode: BarMode }>>> = {
  composerRibbon: { query: '(max-width: 599.98px)', mode: 'auto' },
};

/** Whether nobody has chosen a value: the entry is absent or the registry default. */
export function unchosen(entry: SettingEntry | undefined): boolean {
  return entry === undefined || (entry.source === 'default' && !entry.locked);
}

/** Whether a media query matches now, kept live while the owner is mounted. */
function useMediaMatch(query: string | undefined): Accessor<boolean> {
  if (query === undefined || typeof window === 'undefined' || typeof window.matchMedia !== 'function') return () => false;
  let list: MediaQueryList;
  try {
    list = window.matchMedia(query);
  } catch {
    return () => false;
  }
  const [matches, setMatches] = createSignal(list.matches);
  const update = (): void => {
    setMatches(list.matches);
  };
  list.addEventListener?.('change', update);
  onCleanup(() => list.removeEventListener?.('change', update));
  return matches;
}

/**
 * The mode a bar shows for a settings entry: the stored or administered value,
 * or, while nobody has chosen, the narrow-screen default when `narrow` holds.
 */
export function effectiveBarMode(id: BarId, entry: SettingEntry | undefined, narrow: boolean): BarMode {
  const narrowDefault = NARROW_DEFAULTS[id];
  if (narrow && narrowDefault && unchosen(entry)) return narrowDefault.mode;
  const value = entry?.value;
  return isBarMode(value) ? value : DEFAULT_BAR_MODE;
}

/** `effectiveBarMode` for one bar, following the viewport width live. */
export function useEffectiveBarMode(id: BarId, entry: () => SettingEntry | undefined): Accessor<BarMode> {
  const narrow = useMediaMatch(NARROW_DEFAULTS[id]?.query);
  return () => effectiveBarMode(id, entry(), narrow());
}

export function useBarMode(id: BarId): BarModeHandle {
  const setting = useSetting<BarMode>(barSettingKey(id), DEFAULT_BAR_MODE, isBarMode);
  const effective = useEffectiveBarMode(id, setting.entry);
  // The optimistic value: set on a change, cleared when the write settles.
  const [pending, setPending] = createSignal<BarMode | null>(null);
  let sequence = 0;

  return {
    mode: () => pending() ?? effective(),
    locked: setting.locked,
    saving: setting.saving,
    error: setting.error,
    async setMode(mode: BarMode): Promise<boolean> {
      if (setting.locked()) return false;
      const current = ++sequence;
      setPending(mode);
      const ok = await setting.set(mode);
      if (current === sequence) setPending(null);
      if (!ok) toast.error(setting.error() || t('bars-mode-save-failed'), { id: `bar-mode-${id}` });
      return ok;
    },
  };
}
