// Language picker (settings key `ui.language`, worker f01; t32-admin §2.8).
//
// A native <select> (keyboard and screen readers for free): "Same as the
// browser", then each shipped language under its own name. A choice is saved
// to the account through the settings model, kept in localStorage for the next
// start, and applied at once. When an administrator holds the key the select is
// disabled, says so (LockNote), and the held language is the one applied.

import { createEffect, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { formatLanguageName, loadCatalog, t, useI18n, type Locale } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import {
  isLanguageChoice,
  LANGUAGE_CHOICES,
  localeFor,
  storedLanguage,
  storeLanguage,
  type LanguageChoice,
} from './language.ts';
import * as css from './styles.css.ts';

export interface LanguageProps {
  /** Apply a locale (defaults to the LocaleProvider's `setLocale`). */
  onLocale?: (locale: Locale) => void;
}

function providerSetLocale(): ((l: Locale) => void) | null {
  try {
    return useI18n().setLocale;
  } catch {
    return null; // rendered outside a LocaleProvider (unit tests)
  }
}

export function Language(props: LanguageProps): JSX.Element {
  onMount(() => void loadCatalog('settings-policy'));
  const apply = props.onLocale ?? providerSetLocale() ?? ((): void => undefined);
  const setting = useSetting<LanguageChoice>('ui.language', storedLanguage() ?? 'browser', isLanguageChoice);
  const [saved, setSaved] = createSignal(false);

  // Whatever the account's effective value is (the user's, the domain's or a
  // held one), it is the language in force: apply it and keep it for the next start.
  createEffect(() => {
    const entry = setting.entry();
    if (entry === undefined) return;
    const v = setting.value();
    storeLanguage(v);
    apply(localeFor(v));
  });

  async function choose(v: string): Promise<void> {
    if (!isLanguageChoice(v)) return;
    setSaved(false);
    storeLanguage(v);
    apply(localeFor(v));
    if (await setting.set(v)) setSaved(true);
  }

  const label = (v: LanguageChoice): string =>
    v === 'browser' ? t('settings-policy-language-browser') : formatLanguageName(v, v);

  return (
    <section class={css.section}>
      <div class={css.field}>
        <h3 class={css.heading}>
          <label for="settings-language">{t('settings-policy-language')}</label>
        </h3>
        <select
          id="settings-language"
          class={css.select}
          aria-describedby="settings-language-help settings-language-lock"
          disabled={setting.locked() || setting.saving()}
          value={setting.value()}
          onChange={(e) => void choose(e.currentTarget.value)}
        >
          <For each={LANGUAGE_CHOICES}>
            {(v) => (
              <option value={v} lang={v === 'browser' ? undefined : v}>
                {label(v)}
              </option>
            )}
          </For>
        </select>
        <p class={css.meta} id="settings-language-help">
          {t('settings-policy-language-help')}
        </p>
        <LockNote id="settings-language-lock" locked={setting.locked()} />
        <p class={css.meta} role="status" aria-live="polite">
          <Show when={saved()}>{t('settings-policy-language-saved')}</Show>
        </p>
        <Show when={setting.error() !== ''}>
          <p class={css.error} role="alert">
            {setting.error()}
          </p>
        </Show>
      </div>
    </section>
  );
}
