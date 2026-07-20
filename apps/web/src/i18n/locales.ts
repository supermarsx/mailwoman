// The shipped locales + RTL/direction plumbing (plan §4, SPEC §24).
//
// This is the single source of truth for WHICH locales exist and WHICH way each
// one reads. The set is the 1.0 gate's twelve plus `ar`; only `ar` is RTL, and
// the `dir` resolver is written against a general RTL language set so
// `he`/`fa`/`ur` also become first-class the moment their catalog dirs are added
// — no code change, just a new `locales/<lang>/` tree. `ar` ships a stub catalog
// (settings only); missing keys fall back to `en` via the fallback chain.

import { negotiateLanguages } from '@fluent/langneg';

/** Every locale Mailwoman ships a catalog dir for (the 1.0 gate's twelve + `ar`). */
export const LOCALES = [
  'en',
  'de',
  'fr',
  'es',
  'pt-BR',
  'nl',
  'it',
  'pl',
  'ru',
  'uk',
  'zh',
  'ja',
  'ar',
] as const;

export type Locale = (typeof LOCALES)[number];

/** The source locale every other catalog is translated from + falls back to. */
export const SOURCE_LOCALE: Locale = 'en';

export type Dir = 'ltr' | 'rtl';

// Base languages that read right-to-left. `ar` is now in `LOCALES`; the others
// are not shipped yet, but the resolver consults this set so adding e.g.
// `locales/he/` is enough to get a mirrored UI. Kept as base-language codes
// (compared against the primary subtag).
const RTL_BASE_LANGUAGES = new Set(['ar', 'he', 'fa', 'ur', 'ps', 'dv', 'yi', 'ckb']);

/** The primary language subtag of a BCP-47 tag (`pt-BR` -> `pt`, `zh-Hans` -> `zh`). */
export function baseLanguage(locale: string): string {
  return locale.toLowerCase().split('-')[0] ?? locale.toLowerCase();
}

/** Resolve the writing direction for a locale. Unknown locales default to LTR. */
export function resolveDir(locale: string): Dir {
  return RTL_BASE_LANGUAGES.has(baseLanguage(locale)) ? 'rtl' : 'ltr';
}

/** Is this a locale we ship a catalog for? */
export function isKnownLocale(locale: string): locale is Locale {
  return (LOCALES as readonly string[]).includes(locale);
}

/**
 * Pick the best shipped locale for a set of user-requested tags (typically
 * `navigator.languages`), always ending on `en`. Uses Fluent's language
 * negotiation (filtering strategy) so `pt` matches `pt-BR`, `de-AT` matches `de`,
 * etc. Returns the single best match — the runtime fallback CHAIN (best -> en)
 * is assembled by the registry.
 */
export function negotiateLocale(requested: readonly string[]): Locale {
  const [best] = negotiateLanguages(requested as string[], LOCALES as unknown as string[], {
    defaultLocale: SOURCE_LOCALE,
    strategy: 'filtering',
  });
  return (best && isKnownLocale(best) ? best : SOURCE_LOCALE) as Locale;
}

/** The fallback chain for an active locale: [active, en] (deduped). */
export function fallbackChain(active: Locale): Locale[] {
  return active === SOURCE_LOCALE ? [SOURCE_LOCALE] : [active, SOURCE_LOCALE];
}
