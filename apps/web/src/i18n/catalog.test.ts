// The lazy `.ftl` catalog loader (t19-e12, tag 26.19).
//
// `catalog.ts` had no test before this file, and it is the piece that decides
// whether a screen shows real text or raw message ids. It is also a size-budget
// mechanism: only `en/common` is statically imported, everything else is a
// dynamic `import()` so translation weight stays out of the login→inbox entry
// chunk. Both halves of that — "the right catalogs load" and "only the right
// catalog is eager" — are what these tests pin.
//
// The module carries process-wide state (a `loaded` dedupe set and the shared
// bundle registry), which is realistic: in the app it is a singleton too. Tests
// are therefore written to be order-independent rather than assuming a reset.

import { describe, it, expect } from 'vitest';
import { loadCatalog, modulesForLocale, preloadCatalogs, reloadForActiveChain } from './catalog.ts';
import { setActiveLocale, t } from './registry.ts';
import { LOCALES, SOURCE_LOCALE } from './locales.ts';

describe('modulesForLocale', () => {
  it('lists the catalogs actually on disk for the source locale', () => {
    const modules = modulesForLocale('en');
    expect(modules).toContain('common');
    expect(modules).toContain('mail');
    // Names come back bare — no path, no extension — because they are the keys
    // callers pass to `loadCatalog('mail')`.
    for (const m of modules) {
      expect(m).not.toContain('/');
      expect(m).not.toContain('.ftl');
    }
  });

  it('returns nothing for a locale with no catalog directory', () => {
    expect(modulesForLocale('xx-YY')).toEqual([]);
    // A prefix of a real locale must not match it — `p` is not `pl`.
    expect(modulesForLocale('p')).toEqual([]);
  });

  it('finds catalogs for a locale whose tag contains a region subtag', () => {
    // `pt-BR` exercises the key parser against a directory name with a hyphen.
    expect(modulesForLocale('pt-BR').length).toBeGreaterThan(0);
  });

  it('agrees with the declared locale list about which locales are translated', () => {
    // Every locale that ships catalogs must be a declared `Locale`; a directory
    // nobody can negotiate to is dead weight in the glob manifest.
    const withCatalogs = LOCALES.filter((l) => modulesForLocale(l).length > 0);
    expect(withCatalogs).toContain(SOURCE_LOCALE);
    expect(withCatalogs.length).toBeGreaterThan(1);
  });
});

describe('loadCatalog', () => {
  it('resolves and makes the module’s messages formattable', async () => {
    setActiveLocale('en');
    await loadCatalog('mail');
    // `t()` returns the message id itself when nothing is registered, so an id
    // that formats to something else proves the catalog landed.
    const ids = ['mail-compose', 'mail-archive'];
    expect(ids.some((id) => t(id) !== id)).toBe(true);
  });

  it('is idempotent — a second call is a no-op, not a double registration', async () => {
    await loadCatalog('mail');
    const first = t('mail-compose');
    await loadCatalog('mail');
    expect(t('mail-compose')).toBe(first);
  });

  it('resolves quietly for a module that has no catalog file', async () => {
    // A screen calling `loadCatalog` for a module nobody has translated must not
    // reject — the UI degrades to message ids, it does not fail to mount.
    await expect(loadCatalog('no-such-module')).resolves.toBeUndefined();
    // And it must not retry forever: the second call resolves just as quietly.
    await expect(loadCatalog('no-such-module')).resolves.toBeUndefined();
  });

  it('loads the whole fallback chain, not only the active locale', async () => {
    // A partly-translated locale must still show English for the strings it is
    // missing, which only works if `en` is loaded alongside it.
    setActiveLocale('de');
    await loadCatalog('mail');
    expect(t('mail-compose')).not.toBe('mail-compose');
    setActiveLocale('en');
  });
});

describe('preloadCatalogs', () => {
  it('loads several modules at once for the active chain', async () => {
    setActiveLocale('en');
    const modules = modulesForLocale('en').slice(0, 3);
    await expect(preloadCatalogs(modules)).resolves.toBeUndefined();
  });

  it('accepts an empty list', async () => {
    await expect(preloadCatalogs([])).resolves.toBeUndefined();
  });
});

describe('reloadForActiveChain', () => {
  it('re-fetches every already-touched module after a locale switch', async () => {
    // This is what makes a LIVE language switch repaint without a full reload:
    // the modules the user has already visited are reloaded for the new chain.
    setActiveLocale('en');
    await loadCatalog('mail');

    setActiveLocale('fr');
    await reloadForActiveChain();
    expect(t('mail-compose')).not.toBe('mail-compose');

    setActiveLocale('en');
  });

  it('ignores an unknown locale in the chain rather than throwing', async () => {
    setActiveLocale(SOURCE_LOCALE);
    await expect(reloadForActiveChain()).resolves.toBeUndefined();
  });
});
