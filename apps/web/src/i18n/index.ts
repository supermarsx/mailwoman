// Public i18n surface (plan §4). Import from here, not the internal modules.
//
// Typical use in a screen/component (e1–e4):
//   import { t, loadCatalog } from '../i18n';
//   onMount(() => void loadCatalog('mail'));   // pull this area's catalog
//   <button>{t('common-ok')}</button>          // formatted, reactive
//   <span>{t('mail-from', { name: isolate(sender) })}</span>  // isolate untrusted
//
// The `t` accessor is a plain function backed by a reactive registry, so reading
// it inside JSX makes the node re-render when a catalog loads or the locale flips.

export { LocaleProvider, useI18n, type I18nContext } from './provider.tsx';
export { t, type TArgs } from './registry.ts';
export { loadCatalog, useCatalog, preloadCatalogs } from './catalog.ts';
export { isolate, isolateDir, stripIsolates } from './bidi.ts';
export {
  LOCALES,
  SOURCE_LOCALE,
  negotiateLocale,
  resolveDir,
  baseLanguage,
  isKnownLocale,
  fallbackChain,
  type Locale,
  type Dir,
} from './locales.ts';
