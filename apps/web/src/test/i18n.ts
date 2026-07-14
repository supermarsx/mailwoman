// `en`-bundle test helper (plan §6 e0, risk #2).
//
// The 579-suite gate asserts against LITERAL UI text ("Focused inbox", "Send",
// …). As e1–e4 wrap those literals in `t('…')`, the assertions must still see
// the same English strings. This helper synchronously seeds the i18n registry
// with EVERY `en/*.ftl` catalog, then makes `en` the active locale — so `t(id)`
// returns the English string with no async, no provider, no Suspense.
//
// Wired once from `src/test/setup.ts` (runs before every suite). Because it uses
// an EAGER glob, any `en/<module>.ftl` an executor adds is picked up
// automatically — no per-executor test wiring needed.

import { seedLocaleSync } from '../i18n/registry.ts';
import { SOURCE_LOCALE } from '../i18n/locales.ts';

// Eagerly inline every English catalog at test build time. `.ftl?raw` -> string.
const enCatalogs = import.meta.glob<string>('../../locales/en/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

/**
 * Seed the i18n registry with all `en` catalogs and activate `en`. Idempotent —
 * safe to call from a global setup file and/or a per-suite `beforeEach`.
 */
export function setupI18nForTests(): void {
  seedLocaleSync(SOURCE_LOCALE, Object.values(enCatalogs));
}

// Re-export so a test can assert against the same accessor components use:
//   import { t } from '../test/i18n';  expect(...).toHaveTextContent(t('common-ok'))
export { t } from '../i18n/registry.ts';
