// One-time migration of the legacy mail-ribbon collapse (t38 §3.1).
//
// Before t38 the mail ribbon's collapse was `ribbonCollapsed` in the appearance
// blob (localStorage, and the blind per-account appearance sync). t38 made it
// the settings-registry key `ui.bar.mailRibbon` (`pinned|auto`), which an
// administrator can default and lock. The theme slice drops the old field when
// it reads a blob that carries `ribbonCollapsed: true` and leaves this marker;
// once the settings map has loaded, `migrateLegacyRibbonCollapsed` writes
// `auto` if the user has no stored value of their own (and the key is not
// locked), then removes the marker. A user who already chose keeps the choice.

import { barSettingKey } from '../api/barSettings.ts';
import { settings, settingsLoadStatus, writeSetting } from '../api/settings.ts';

export const LEGACY_RIBBON_MARKER = 'mw.migrate.ribbonCollapsed';

function storage(): Storage | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

/** A blob with `ribbonCollapsed: true` was read; migrate it once settings load. */
export function noteLegacyRibbonCollapsed(): void {
  try {
    storage()?.setItem(LEGACY_RIBBON_MARKER, '1');
  } catch {
    /* private mode / quota: the old collapse is lost, the bar stays pinned */
  }
}

export function legacyRibbonCollapsedPending(): boolean {
  try {
    return storage()?.getItem(LEGACY_RIBBON_MARKER) === '1';
  } catch {
    return false;
  }
}

function clearMarker(): void {
  try {
    storage()?.removeItem(LEGACY_RIBBON_MARKER);
  } catch {
    /* nothing to do */
  }
}

let running: Promise<void> | null = null;

/**
 * Run the migration if it is pending and the settings map is ready. Safe to
 * call from an effect on every settings change: it does nothing until the
 * map holds `ui.bar.mailRibbon`, and at most one write is in flight.
 */
export function migrateLegacyRibbonCollapsed(): Promise<void> {
  if (running) return running;
  if (!legacyRibbonCollapsedPending() || settingsLoadStatus() !== 'ready') return Promise.resolve();
  const key = barSettingKey('mailRibbon');
  const entry = settings()[key];
  // A server without the key: keep the marker for a later build.
  if (!entry) return Promise.resolve();
  if (entry.userValue !== undefined || entry.locked) {
    clearMarker();
    return Promise.resolve();
  }
  running = writeSetting(key, 'auto')
    .then(() => clearMarker())
    .catch(() => {
      /* keep the marker and try again next time */
    })
    .finally(() => {
      running = null;
    });
  return running;
}
