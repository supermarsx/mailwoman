// Lazy `.ftl` catalog loader (plan §4 — protects the 250 KB entry budget).
//
// Catalogs live at `apps/web/locales/<locale>/<module>.ftl`. Only the CRITICAL
// `en/common` catalog is statically imported (rides the entry chunk); every
// other module for every locale is a lazy `import()` pulled the first time a
// screen asks for it, so translation weight never inflates the login→inbox
// bundle the size gate measures.
//
// Vite turns `import.meta.glob(..., '?raw')` into a map of per-file dynamic
// importers, each its own chunk — exactly the per-module code-split we want.

import { createResource, type Resource } from 'solid-js';
import { registerResource, activeLocaleSignal } from './registry.ts';
import { fallbackChain, SOURCE_LOCALE, isKnownLocale, type Locale } from './locales.ts';

// CRITICAL PATH: the common `en` strings (buttons, generic errors) load
// synchronously with the app so first paint has real text, not message ids.
import enCommon from '../../locales/en/common.ftl?raw';

// Every OTHER catalog, lazily. Keys look like `../../locales/de/mail.ftl`.
const lazyCatalogs = import.meta.glob<string>('../../locales/*/*.ftl', {
  query: '?raw',
  import: 'default',
});

/** Parse a glob key into its `{ locale, module }`. */
function parseKey(key: string): { locale: string; module: string } | null {
  const m = key.match(/\/locales\/([^/]+)\/([^/]+)\.ftl$/);
  return m && m[1] && m[2] ? { locale: m[1], module: m[2] } : null;
}

const globKey = (locale: string, module: string): string =>
  `../../locales/${locale}/${module}.ftl`;

// `locale/module` pairs already registered (or eagerly bundled) — dedupes loads.
const loaded = new Set<string>([`${SOURCE_LOCALE}/common`]);

// Register the eagerly-bundled critical catalog immediately on import.
registerResource(SOURCE_LOCALE, enCommon);

/** All catalog modules present on disk for a locale (from the glob manifest). */
export function modulesForLocale(locale: string): string[] {
  const out: string[] = [];
  for (const key of Object.keys(lazyCatalogs)) {
    const parsed = parseKey(key);
    if (parsed && parsed.locale === locale) out.push(parsed.module);
  }
  return out;
}

async function loadOne(locale: Locale, module: string): Promise<void> {
  const dedupeKey = `${locale}/${module}`;
  if (loaded.has(dedupeKey)) return;
  const loader = lazyCatalogs[globKey(locale, module)];
  if (loader === undefined) {
    // No catalog file for this locale/module (e.g. a locale not yet translated
    // for that screen). Mark done so we don't retry every render.
    loaded.add(dedupeKey);
    return;
  }
  const source = await loader();
  registerResource(locale, source);
  loaded.add(dedupeKey);
}

/**
 * Load a `<module>.ftl` catalog for the active locale AND its `en` fallback (the
 * whole fallback chain), registering each into the reactive registry. Idempotent
 * and safe to call from any number of screens/effects — repeat calls no-op.
 *
 * e1–e4: call this once where your feature area mounts (`void loadCatalog('mail')`)
 * so your `t('mail-…')` ids resolve; strings fill in reactively as it settles.
 */
export async function loadCatalog(module: string): Promise<void> {
  const chain = fallbackChain(activeLocaleSignal());
  await Promise.all(chain.map((locale) => loadOne(locale, module)));
}

/**
 * Suspense-friendly variant: returns a resource that resolves once `module` has
 * loaded for the active locale chain. Read it inside `<Suspense>` to defer a
 * screen's first paint until its catalog is present (avoids an id→text flash).
 */
export function useCatalog(module: string): Resource<true> {
  const [res] = createResource(
    () => `${activeLocaleSignal()}:${module}`,
    async () => {
      await loadCatalog(module);
      return true as const;
    },
  );
  return res;
}

/** Preload several modules for the active chain (e.g. at shell boot). */
export async function preloadCatalogs(modules: readonly string[]): Promise<void> {
  await Promise.all(modules.map((m) => loadCatalog(m)));
}

/**
 * Switch which locales the on-demand loads target after a locale change: reload
 * every already-touched module for the NEW active chain, so a live language
 * switch repaints without a full reload. `en/common` is always present.
 */
export async function reloadForActiveChain(): Promise<void> {
  const chain = fallbackChain(activeLocaleSignal());
  const modules = new Set<string>();
  for (const key of loaded) {
    const module = key.split('/')[1];
    if (module) modules.add(module);
  }
  await Promise.all(
    chain
      .filter((l): l is Locale => isKnownLocale(l))
      .flatMap((locale) => [...modules].map((module) => loadOne(locale, module))),
  );
}
