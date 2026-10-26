// Settings under the settings model C (worker f01): held appearance keys disable
// their controls, say "Set by your administrator" and apply the held value; the
// language picker saves `ui.language`, keeps it for the next start, and is held
// like the rest.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { Settings } from './Settings.tsx';
import { AppContext } from '../state/context.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppState } from '../state/store.ts';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';
import { Language } from './Settings/Language.tsx';
import { LOCALE_STORAGE_KEY, bootLocale } from './Settings/language.ts';
import { t } from '../test/i18n.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

function renderSettings() {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const app = {...createThemeSlice(ctx),accountId:()=>null} as unknown as AppState;
  render(() => (
    <AppContext.Provider value={app}>
      <Settings onClose={vi.fn()} />
    </AppContext.Provider>
  ));
  return app;
}

const held = (value: unknown) => ({ value, source: 'global', locked: true, lockedBy: 'global' });
const free = (value: unknown) => ({ value, source: 'default', locked: false });

describe('Settings › held appearance keys', () => {
  beforeEach(() => {
    localStorage.clear();
    clearSettings();
  });

  it('nothing held: every appearance control is enabled and no note shows', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () =>
        json(200, { settings: { 'appearance.theme': free('dark'), 'appearance.density': free('cozy') } }),
      ),
    });
    renderSettings();
    expect(screen.getByRole('button', { name: 'Grove Dark' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Compact' })).toBeEnabled();
    expect(screen.queryAllByTestId('setting-lock-note')).toHaveLength(0);
  });

  it('a held theme, accent and density: disabled, noted, and applied', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () =>
        json(200, {
          settings: {
            'appearance.theme': held('hc-dark'),
            'appearance.accent': held('teal'),
            'appearance.density': held('relaxed'),
          },
        }),
      ),
    });
    const app = renderSettings();
    await waitFor(() => expect(app.theme()).toBe('hc-dark'));
    expect(app.themeMode()).toBe('fixed');
    expect(app.accentId()).toBe('teal');
    expect(app.density()).toBe('relaxed');

    expect(screen.getByRole('button', { name: 'Grove Dark' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Compact' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Green' })).toBeDisabled();
    const notes = screen.getAllByTestId('setting-lock-note');
    expect(notes.map((n) => n.id).sort()).toEqual([
      'settings-accent-lock',
      'settings-density-lock',
      'settings-theme-lock',
    ]);
    for (const n of notes) expect(n).toHaveTextContent(t('settings-policy-locked'));
    expect(screen.getByRole('group', { name: 'Density' })).toHaveAttribute('aria-describedby', 'settings-density-lock');

    // A click on a disabled control changes nothing.
    fireEvent.click(screen.getByRole('button', { name: 'Grove Dark' }));
    expect(app.theme()).toBe('hc-dark');
  });
});

describe('Settings › language', () => {
  beforeEach(() => {
    localStorage.clear();
    clearSettings();
  });

  it('saves the choice, keeps it for the next start and applies it', async () => {
    const fetcher = vi.fn<Fetcher>(async (_u, init) =>
      init?.method === 'PUT'
        ? json(200, { value: 'de', source: 'user', locked: false })
        : json(200, { settings: { 'ui.language': free('browser') } }),
    );
    await loadSettings({ fetcher });
    const onLocale = vi.fn();
    render(() => <Language onLocale={onLocale} />);
    const select = screen.getByLabelText(t('settings-policy-language'));
    expect(select).toHaveValue('browser');
    expect(select).toBeEnabled();
    fireEvent.change(select, { target: { value: 'de' } });
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(t('settings-policy-language-saved')));
    expect(onLocale).toHaveBeenLastCalledWith('de');
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('de');
    expect(bootLocale()).toBe('de');
    const put = fetcher.mock.calls.find(([, init]) => init?.method === 'PUT');
    expect(put?.[0]).toBe('/api/settings/ui.language');
  });

  it('a held language is applied, stored and not editable', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () => json(200, { settings: { 'ui.language': held('fr') } })),
    });
    const onLocale = vi.fn();
    render(() => <Language onLocale={onLocale} />);
    const select = screen.getByLabelText(t('settings-policy-language'));
    expect(select).toHaveValue('fr');
    expect(select).toBeDisabled();
    expect(screen.getByTestId('setting-lock-note')).toHaveAttribute('id', 'settings-language-lock');
    expect(onLocale).toHaveBeenLastCalledWith('fr');
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe('fr');
  });

  it('junk in storage is ignored at start', () => {
    localStorage.setItem(LOCALE_STORAGE_KEY, 'klingon');
    expect(bootLocale()).toBeNull();
    localStorage.setItem(LOCALE_STORAGE_KEY, 'browser');
    expect(bootLocale()).toBeNull();
  });
});
