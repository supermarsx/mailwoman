// Console language picker (t32-admin.md §2.1 footer, §2.9).
//
// Lists every shipped locale by its own name ("Deutsch", "日本語"), switches the
// app locale through the i18n provider, and remembers the admin's choice in this
// browser (`mw.admin.locale`), separately from the webmail's language. Without a
// stored choice the provider's negotiation from the browser languages applies.
// Most non-English catalogs are still being translated, so missing strings show
// in English until then.
//
// Outside a `LocaleProvider` (component tests) it renders nothing.

import { For, Show, createUniqueId, onMount, type JSX } from 'solid-js';
import { LOCALES, formatLanguageName, isKnownLocale, t, useI18n, type I18nContext, type Locale } from '../../../i18n';
import * as css from './shell.css.ts';

const KEY = 'mw.admin.locale';

function stored(): Locale | null {
  try {
    const v = localStorage.getItem(KEY);
    return v !== null && isKnownLocale(v) ? v : null;
  } catch {
    return null;
  }
}

function store(locale: Locale): void {
  try {
    localStorage.setItem(KEY, locale);
  } catch {
    /* private mode: the choice lasts for this page only */
  }
}

/** The i18n context, or undefined outside a provider. */
function useOptionalI18n(): I18nContext | undefined {
  try {
    return useI18n();
  } catch {
    return undefined;
  }
}

/** Apply the admin's stored language, if any. Call once where the console mounts. */
export function useStoredAdminLocale(): void {
  const i18n = useOptionalI18n();
  onMount(() => {
    const want = stored();
    if (i18n && want !== null && want !== i18n.locale()) i18n.setLocale(want);
  });
}

export function LanguagePicker(): JSX.Element {
  const i18n = useOptionalI18n();
  const id = createUniqueId();
  return (
    <Show when={i18n}>
      {(ctx) => (
        <div class={css.field}>
          <label class={css.fieldLabel} for={id}>
            {t('admin-shell-language-label')}
          </label>
          <select
            id={id}
            value={ctx().locale()}
            onChange={(e) => {
              const next = e.currentTarget.value;
              if (!isKnownLocale(next)) return;
              store(next);
              ctx().setLocale(next);
            }}
          >
            <For each={LOCALES}>
              {(code) => (
                <option value={code} lang={code}>
                  {formatLanguageName(code, code)}
                </option>
              )}
            </For>
          </select>
        </div>
      )}
    </Show>
  );
}

/** The localStorage key of the admin's language (tests). */
export const ADMIN_LOCALE_KEY = KEY;
