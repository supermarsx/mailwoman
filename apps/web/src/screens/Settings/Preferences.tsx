import {offlineCacheUsage,purgeOfflineReadCache} from '../../offline/cacheManagement.ts';
import {formatBytes} from '../../i18n/format.ts';
import {SettingRow} from './layout/SettingRow.tsx';
// Device preferences (t16 e15): keyboard presets (W14), offline eviction policy
// (W16), and interface direction (W20). These are per-device, not account state —
// persisted to localStorage via `prefs.ts` and consumed by the shell keymap, the
// OPFS offline cache, and the Settings panel's direction. The parent owns the
// working copy so the whole account surface can mirror when RTL is chosen.
//
// t32 settings model C (worker f01): an administrator can hold the offline
// budget and retention (`offline.budgetMb`, `offline.retentionDays`). A held
// value replaces the device value, and its input is disabled with a LockNote.

import { createEffect, createSignal, Show, For, onMount, type Accessor, type JSX } from 'solid-js';
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
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
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
  const enabled=useSetting<boolean>('offline.enabled',false);
  const headersWindow=useSetting<string>('sync.headersWindow','90'),bodies=useSetting<string>('sync.bodies','on-demand'),attachments=useSetting<string>('sync.attachments','on-demand');
  const evictionHold=useSetting<EvictionStrategy>('offline.eviction',props.prefs().eviction);
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

  const [usage,setUsage]=createSignal<Awaited<ReturnType<typeof offlineCacheUsage>>>();
  const [purging,setPurging]=createSignal(false),[cacheError,setCacheError]=createSignal(false);
  const refreshUsage=()=>offlineCacheUsage().then(value=>setUsage(value)).catch(()=>setCacheError(true));
  onMount(()=>void refreshUsage());
  async function purgeOfflineCache():Promise<void>{
    if(purging())return;setPurging(true);setCacheError(false);
    try{await purgeOfflineReadCache();await refreshUsage();window.dispatchEvent(new CustomEvent('mw:offline-purge'));}
    catch{setCacheError(true);}finally{setPurging(false);}
  }

  const previewDir = (): 'ltr' | 'rtl' => (props.prefs().direction === 'rtl' ? 'rtl' : 'ltr');

  return (
    <>
      {/* W14 — keyboard shortcut presets */}
      <SettingsCard id="prefs-card-keyboard" title={t('settings-kbd-preset-title')} description={t('settings-kbd-intro')}>
        <ControlRow controlId="prefs-keyboard-preset" group stacked label={t('settings-kbd-preset-title')}>{() =>
          <div class={css.options} role="group" aria-labelledby="prefs-keyboard-preset">
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
        }</ControlRow>
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
      </SettingsCard>

      {/* W16 — offline cache eviction policy */}
      <SettingsCard id="prefs-card-offline" title={t('settings-offline-title')} description={t('settings-offline-intro')}>
        <SettingRow controlId="prefs-offline-enabled" label={t('settings-offline-enabled')} help={t('settings-offline-enabled-help')} handle={enabled}>{describedBy=><input id="prefs-offline-enabled" type="checkbox" class={css.checkbox} checked={enabled.value()} disabled={!enabled.entry()||enabled.locked()||enabled.saving()} aria-describedby={describedBy()} onChange={event=>void enabled.set(event.currentTarget.checked)}/>}</SettingRow>
        <For each={[
          {id:'headers-window',handle:headersWindow,options:['30','90','365','all']},
          {id:'bodies',handle:bodies,options:['on-demand','prefetch']},
          {id:'attachments',handle:attachments,options:['on-demand','inline','all']},
        ]}>{field=><SettingRow controlId={`prefs-sync-${field.id}`} label={t(`settings-sync-${field.id}`)} help={t(`settings-sync-${field.id}-help`)} handle={field.handle}>{description=><select id={`prefs-sync-${field.id}`} class={css.input} aria-describedby={description()} value={field.handle.value()} disabled={!field.handle.entry()||field.handle.locked()||field.handle.saving()} onChange={event=>void field.handle.set(event.currentTarget.value)}><For each={field.options}>{option=><option value={option}>{t(`settings-sync-option-${field.id==='attachments'&&option==='all'?'all-attachments':option}`)}</option>}</For></select>}</SettingRow>}</For>
        <ControlRow controlId="prefs-offline-budget" label={t('settings-offline-budget-label')} note={<LockNote id="settings-offline-budget-lock" locked={budgetHold.locked()} />}>{() =>
          <input
            id="prefs-offline-budget"
            class={css.input}
            type="number"
            min="0"
            aria-describedby={budgetHold.locked() ? 'settings-offline-budget-lock' : undefined}
            disabled={budgetHold.locked()}
            value={props.prefs().offlineBudgetMb}
            onInput={(e) => patch({ offlineBudgetMb: Math.max(0, Number(e.currentTarget.value) || 0) })}
          />
        }</ControlRow>
        <ControlRow controlId="prefs-offline-retention" label={t('settings-offline-retention-label')} note={<LockNote id="settings-offline-retention-lock" locked={retentionHold.locked()} />}>{() =>
          <input
            id="prefs-offline-retention"
            class={css.input}
            type="number"
            min="0"
            aria-describedby={retentionHold.locked() ? 'settings-offline-retention-lock' : undefined}
            disabled={retentionHold.locked()}
            value={props.prefs().offlineRetentionDays}
            onInput={(e) => patch({ offlineRetentionDays: Math.max(0, Number(e.currentTarget.value) || 0) })}
          />
        }</ControlRow>
        <ControlRow controlId="prefs-offline-strategy" group label={t('settings-offline-strategy-label')} note={<LockNote id="settings-offline-strategy-lock" locked={evictionHold.locked()} />}>{() =>
          <div class={css.options} role="group" aria-labelledby="prefs-offline-strategy">
            <For each={EVICTION_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={(evictionHold.locked()?evictionHold.value():props.prefs().eviction) === o.value}
                  disabled={evictionHold.locked()||evictionHold.saving()}
                  onClick={() => {patch({ eviction: o.value });if(evictionHold.entry())void evictionHold.set(o.value);}}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
        }</ControlRow>
        <Show when={usage()}>{value=><p role="status">{t('settings-offline-usage',{size:formatBytes(value().bytes),count:value().entries,protected:formatBytes(value().protectedBytes)})}</p>}</Show>
        <Show when={cacheError()}><p role="alert">{t('settings-offline-storage-error')}</p></Show>
        <div class={css.dangerZone}>
          <button type="button" class={css.danger} onClick={()=>void purgeOfflineCache()} disabled={purging()} data-testid="offline-purge">
            {t('settings-offline-purge')}
          </button>
        </div>
      </SettingsCard>

      {/* W20 — interface direction (ships an RTL locale + mirrored layout) */}
      <SettingsCard id="prefs-card-direction" title={t('settings-dir-title')} description={t('settings-dir-intro')}>
        <ControlRow controlId="prefs-direction" group label={t('settings-dir-title')}>{() =>
          <div class={css.options} role="group" aria-labelledby="prefs-direction">
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
        }</ControlRow>
        <div class={css.previewFrame} dir={previewDir()} data-testid="dir-preview" data-dir={previewDir()}>
          <span class={css.itemName}>{t('settings-dir-preview-title')}</span>
          <span class={css.meta}>{t('settings-dir-preview-body')}</span>
        </div>
      </SettingsCard>
    </>
  );
}

export default Preferences;
