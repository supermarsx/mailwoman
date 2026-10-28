// `useBarMode(id)`: whether one collapsible bar is pinned or auto-hides (t38 §2.1).
//
// The mode is a settings-registry key (`ui.bar.<id>`, `pinned|auto`, global +
// domain + user scope), so an administrator can set the default and lock it.
// Until the map loads, or against a server that does not know the key yet, the
// mode is the registry default `pinned`. A change shows at once and goes back
// with a toast when the server refuses it.

import { createSignal, type Accessor } from 'solid-js';
import { useSetting } from '../../screens/Settings/useSetting.ts';
import { toast } from '../toast/toast.ts';
import { t } from '../../i18n/index.ts';

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

export function useBarMode(id: BarId): BarModeHandle {
  const setting = useSetting<BarMode>(barSettingKey(id), DEFAULT_BAR_MODE, isBarMode);
  // The optimistic value: set on a change, cleared when the write settles.
  const [pending, setPending] = createSignal<BarMode | null>(null);
  let sequence = 0;

  return {
    mode: () => pending() ?? setting.value(),
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
