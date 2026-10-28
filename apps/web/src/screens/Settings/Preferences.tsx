// Device preferences (t16 e15): keyboard presets (W14), offline eviction policy
// (W16), and interface direction (W20). These are per-device, not account state —
// persisted to localStorage via `prefs.ts` and consumed by the shell keymap, the
// OPFS offline cache, and the Settings panel's direction. The parent owns the
// working copy so the whole account surface can mirror when RTL is chosen.
//
// t32 settings model C (worker f01): an administrator can hold the offline
// budget and retention (`offline.budgetMb`, `offline.retentionDays`). A held
// value replaces the device value, and its input is disabled with a LockNote.

import { createEffect, For, onMount, type Accessor, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import {
  PRESET_BINDINGS,
  type DirectionPref,
  type EvictionStrategy,
  type KeyboardPreset,
  type SettingsPrefs,
} from './prefs.ts';
import * as css from './styles.css.ts';

export interface PreferencesProps {
  prefs: Accessor<SettingsPrefs>;
  onChange: (next: SettingsPrefs) => void;
}

const KEYBOARD_OPTIONS: ReadonlyArray<{ value: KeyboardPreset; label: string }> = [
  { value: 'default', label: 'settings-kbd-default' },
  { value: 'gmail', label: 'settings-kbd-gmail' },
  { value: 'outlook', label: 'settings-kbd-outlook' },
  { value: 'vim', label: 'settings-kbd-vim' },
  { value: 'off', label: 'settings-kbd-off' },
];

const EVICTION_OPTIONS: ReadonlyArray<{ value: EvictionStrategy; label: string }> = [
  { value: 'lru', label: 'settings-offline-lru' },
  { value: 'oldest', label: 'settings-offline-oldest' },
  { value: 'manual', label: 'settings-offline-manual' },
];

const DIRECTION_OPTIONS: ReadonlyArray<{ value: DirectionPref; label: string }> = [
  { value: 'auto', label: 'settings-dir-auto' },
  { value: 'ltr', label: 'settings-dir-ltr' },
  { value: 'rtl', label: 'settings-dir-rtl' },
];

export function Preferences(props: PreferencesProps): JSX.Element {
  onMount(() => void loadCatalog('settings'));
  onMount(() => void loadCatalog('settings-policy'));

  const patch = (next: Partial<SettingsPrefs>): void => props.onChange({ ...props.prefs(), ...next });
  const budgetHold = useSetting('offline.budgetMb', props.prefs().offlineBudgetMb);
  const retentionHold = useSetting('offline.retentionDays', props.prefs().offlineRetentionDays);
  // A held value is the device's value too.
  createEffect(() => {
    const next: Partial<SettingsPrefs> = {};
    if (budgetHold.locked() && props.prefs().offlineBudgetMb !== budgetHold.value()) {
      next.offlineBudgetMb = budgetHold.value();
    }
    if (retentionHold.locked() && props.prefs().offlineRetentionDays !== retentionHold.value()) {
      next.offlineRetentionDays = retentionHold.value();
    }
    if (Object.keys(next).length > 0) patch(next);
  });

  function purgeOfflineCache(): void {
    // Decoupled seam: the OPFS offline cache listens for this and drops its store.
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('mw:offline-purge'));
    }
  }

  const previewDir = (): 'ltr' | 'rtl' => (props.prefs().direction === 'rtl' ? 'rtl' : 'ltr');

  return (
    <>
      {/* W14 — keyboard shortcut presets */}
      <section class={css.section} aria-label={t('settings-kbd-title')}>
        <h2 class={css.heading}>{t('settings-kbd-title')}</h2>
        <p class={css.prose}>{t('settings-kbd-intro')}</p>
        <div class={css.options} role="group" aria-label={t('settings-kbd-title')}>
          <For each={KEYBOARD_OPTIONS}>
            {(o) => (
              <button
                type="button"
                class={css.option}
                aria-pressed={props.prefs().keyboardPreset === o.value}
                onClick={() => patch({ keyboardPreset: o.value })}
              >
                {t(o.label)}
              </button>
            )}
          </For>
        </div>
        <details><summary>{t('settings-kbd-reference')}</summary><p class={css.meta}>{t('settings-kbd-reference-help')}</p>
        <table class="settings-shortcut-table" data-testid="kbd-preview"><thead><tr><th>{t('settings-shortcuts-action')}</th><th>{t('settings-shortcuts-binding')}</th></tr></thead><tbody>
          <For each={PRESET_BINDINGS[props.prefs().keyboardPreset]}>
            {(binding) => (
              <tr>
                <td>{t(`settings-kbd-action-${binding.action}`)}</td>
                <td><kbd class={css.badge}>{binding.keys}</kbd></td>
              </tr>
            )}
          </For>
        </tbody></table></details>
      </section>

      {/* W16 — offline cache eviction policy */}
      <section class={css.section} aria-label={t('settings-offline-title')}>
        <h2 class={css.heading}>{t('settings-offline-title')}</h2>
        <p class={css.prose}>{t('settings-offline-intro')}</p>
        <div class={css.row}>
          <label class={css.field}>
            <span class={css.label}>{t('settings-offline-budget-label')}</span>
            <input
              class={css.input}
              type="number"
              min="0"
              aria-label={t('settings-offline-budget-label')}
              aria-describedby={budgetHold.locked() ? 'settings-offline-budget-lock' : undefined}
              disabled={budgetHold.locked()}
              value={props.prefs().offlineBudgetMb}
              onInput={(e) => patch({ offlineBudgetMb: Math.max(0, Number(e.currentTarget.value) || 0) })}
            />
            <LockNote id="settings-offline-budget-lock" locked={budgetHold.locked()} />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-offline-retention-label')}</span>
            <input
              class={css.input}
              type="number"
              min="0"
              aria-label={t('settings-offline-retention-label')}
              aria-describedby={retentionHold.locked() ? 'settings-offline-retention-lock' : undefined}
              disabled={retentionHold.locked()}
              value={props.prefs().offlineRetentionDays}
              onInput={(e) => patch({ offlineRetentionDays: Math.max(0, Number(e.currentTarget.value) || 0) })}
            />
            <LockNote id="settings-offline-retention-lock" locked={retentionHold.locked()} />
          </label>
        </div>
        <div class={css.field}>
          <span class={css.label}>{t('settings-offline-strategy-label')}</span>
          <div class={css.options} role="group" aria-label={t('settings-offline-strategy-label')}>
            <For each={EVICTION_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={props.prefs().eviction === o.value}
                  onClick={() => patch({ eviction: o.value })}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
        </div>
        <div class={css.dangerZone}>
          <button type="button" class={css.danger} onClick={purgeOfflineCache} data-testid="offline-purge">
            {t('settings-offline-purge')}
          </button>
        </div>
      </section>

      {/* W20 — interface direction (ships an RTL locale + mirrored layout) */}
      <section class={css.section} aria-label={t('settings-dir-title')}>
        <h2 class={css.heading}>{t('settings-dir-title')}</h2>
        <p class={css.prose}>{t('settings-dir-intro')}</p>
        <div class={css.options} role="group" aria-label={t('settings-dir-title')}>
          <For each={DIRECTION_OPTIONS}>
            {(o) => (
              <button
                type="button"
                class={css.option}
                aria-pressed={props.prefs().direction === o.value}
                onClick={() => patch({ direction: o.value })}
              >
                {t(o.label)}
              </button>
            )}
          </For>
        </div>
        <div class={css.previewFrame} dir={previewDir()} data-testid="dir-preview" data-dir={previewDir()}>
          <span class={css.itemName}>{t('settings-dir-preview-title')}</span>
          <span class={css.meta}>{t('settings-dir-preview-body')}</span>
        </div>
      </section>
    </>
  );
}

export default Preferences;
