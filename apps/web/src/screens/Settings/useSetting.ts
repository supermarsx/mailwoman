// `useSetting(key, fallback)` — one settings-model key for a Settings control
// (t32-admin §2.8, worker f01). Reads the shared map in `api/settings.ts`
// (loaded once per sign-in) and writes through `PUT /api/settings/<key>`.
//
// A control shows `value()`, disables itself while `locked()` and puts a
// `LockNote` next to it. Until the map loads, or when the server does not know
// the key, `value()` is the caller's `fallback` and nothing is locked, so a
// control keeps its device-local behaviour signed out or against an older
// server. Errors come back as localised text (`error()`), never the server's
// English.

import { createSignal, onMount } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import {
  loadSettings,
  resetSetting,
  settings,
  SettingsApiError,
  writeSetting,
  type SettingEntry,
} from '../../api/settings.ts';

export interface SettingHandle<T> {
  /** The effective value, or the fallback while unknown. */
  value(): T;
  /** The server's entry, when the key is known. */
  entry(): SettingEntry | undefined;
  /** An administrator holds the value: the control must not offer an edit. */
  locked(): boolean;
  /** A write is in flight. */
  saving(): boolean;
  /** The last write's failure as localised text, or ''. */
  error(): string;
  /** Store the user's value. Resolves `false` (and sets `error`) on failure. */
  set(value: T): Promise<boolean>;
  /** Remove the user's value. Resolves `false` (and sets `error`) on failure. */
  reset(): Promise<boolean>;
}

/** The localised text for a failed settings request. */
export function settingErrorMessage(e: unknown): string {
  if (e instanceof SettingsApiError) {
    if (e.code === 'setting_locked') return t('settings-policy-error-locked');
    if (e.code === 'invalidValue' || e.status === 422) return t('settings-policy-error-invalid');
    if (e.status === 401) return t('settings-policy-error-signed-out');
  }
  return t('settings-policy-error-generic');
}

export function useSetting<T>(
  key: string,
  fallback: T,
  accept: (v: unknown) => v is T = (v: unknown): v is T => typeof v === typeof fallback,
): SettingHandle<T> {
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal('');
  let operation = 0;
  onMount(() => { void loadCatalog('settings-policy'); void loadSettings(); });

  const entry = (): SettingEntry | undefined => settings()[key];

  async function run(op: () => Promise<SettingEntry>): Promise<boolean> {
    const current = ++operation;
    setSaving(true);
    setError('');
    try {
      await op();
      return true;
    } catch (e) {
      if (current === operation) setError(settingErrorMessage(e));
      return false;
    } finally {
      if (current === operation) setSaving(false);
    }
  }

  return {
    value: () => {
      const v = entry()?.value;
      return accept(v) ? v : fallback;
    },
    entry,
    locked: () => entry()?.locked === true,
    saving,
    error,
    set: (value: T) => {
      if (entry()?.locked === true) {
        setError(t('settings-policy-error-locked'));
        return Promise.resolve(false);
      }
      return run(() => writeSetting(key, value));
    },
    reset: () => run(() => resetSetting(key)),
  };
}
