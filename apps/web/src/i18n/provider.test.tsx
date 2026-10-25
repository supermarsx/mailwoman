// LocaleProvider boot with a non-English browser (t32 f37, FULL).
//
// The provider negotiates the active locale from navigator.languages at start-up.
// Only `en/common` rides the entry chunk; the negotiated locale's catalogs are
// lazy. Before this test the boot path switched the locale without loading the
// catalogs already in use for it, so `common` text (the generic error screen,
// buttons) stayed English until a later language switch. These tests boot with
// a German browser and expect German from `common` and from a catalog a screen
// requested before the provider mounted.

import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup } from '@solidjs/testing-library';
import { LocaleProvider } from './provider.tsx';
import { loadCatalog } from './catalog.ts';
import { setActiveLocale } from './registry.ts';
import { SOURCE_LOCALE } from './locales.ts';
import { AsyncError } from '../components/AsyncState.tsx';
import { t } from './registry.ts';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  setActiveLocale(SOURCE_LOCALE);
});

function germanBrowser(): void {
  vi.stubGlobal('navigator', { ...navigator, language: 'de-DE', languages: ['de-DE', 'de'] });
}

describe('LocaleProvider boot', () => {
  it('shows the error screen in the browser language', async () => {
    germanBrowser();
    render(() => (
      <LocaleProvider>
        <AsyncError error={new Error('boom')} />
      </LocaleProvider>
    ));
    expect(await screen.findByText('Etwas ist schiefgelaufen')).toBeInTheDocument();
    expect(document.documentElement.lang).toBe('de');
  });

  it('loads the browser language for catalogs requested before the provider mounted', async () => {
    // The store requests `mail` while the locale is still en (before boot).
    await loadCatalog('mail');
    germanBrowser();
    render(() => (
      <LocaleProvider>
        <p data-testid="out">{t('mail-maxsec-label')}</p>
      </LocaleProvider>
    ));
    await vi.waitFor(() => expect(screen.getByTestId('out')).toHaveTextContent('Sicherheitsstufe'));
  });
});
