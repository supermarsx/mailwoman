// The interface language (settings key `ui.language`, worker f01; t32-admin §2.8).
//
// The value is `"browser"` (follow the browser's languages) or a shipped locale.
// It is a settings-model key, so an administrator can set it for a domain or
// hold it. The last value applied is also kept in localStorage `mw.locale`, so
// the next start (before sign-in, when the server cannot be asked) opens in the
// same language; `main.tsx` reads it for `LocaleProvider`.

import { isKnownLocale, LOCALES, negotiateLocale, type Locale } from '../../i18n';

/** The localStorage key holding the last applied `ui.language` value. */
export const LOCALE_STORAGE_KEY = 'mw.locale';

/** A value of `ui.language`. */
export type LanguageChoice = 'browser' | Locale;

export const LANGUAGE_CHOICES: readonly LanguageChoice[] = ['browser', ...LOCALES];

export function isLanguageChoice(v: unknown): v is LanguageChoice {
  return v === 'browser' || (typeof v === 'string' && isKnownLocale(v));
}

function storage(): Storage | null {
  try {
    return typeof localStorage !== 'undefined' ? localStorage : null;
  } catch {
    return null;
  }
}

/** The stored choice, or `null` when none (or junk) is stored. */
export function storedLanguage(): LanguageChoice | null {
  try {
    const v = storage()?.getItem(LOCALE_STORAGE_KEY) ?? null;
    return isLanguageChoice(v) ? v : null;
  } catch {
    return null;
  }
}

/** The locale to start in: the stored locale, or `null` to negotiate. */
export function bootLocale(): Locale | null {
  const v = storedLanguage();
  return v === null || v === 'browser' ? null : v;
}

export function storeLanguage(v: LanguageChoice): void {
  try {
    storage()?.setItem(LOCALE_STORAGE_KEY, v);
  } catch {
    /* blocked storage: the choice still applies for this session */
  }
}

/** The browser's preferred locales, `['en']` off-browser. */
function requested(): readonly string[] {
  return typeof navigator !== 'undefined' && Array.isArray(navigator.languages) ? navigator.languages : ['en'];
}

/** The locale a choice means right now. */
export function localeFor(v: LanguageChoice): Locale {
  return v === 'browser' ? negotiateLocale(requested()) : v;
}
