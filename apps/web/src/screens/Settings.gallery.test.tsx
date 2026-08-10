// The theme gallery + mode tri-state in the Settings panel (t19 e13, SPEC §17.1).
//
// A separate file from `Settings.test.tsx` on purpose: that file pins the panel's
// pre-existing appearance controls and dialog behaviour and is deliberately left
// untouched, so a regression there stays legible as a regression rather than as
// this lane's churn. Sync transport behaviour lives in `api/prefs.test.ts`.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, fireEvent, screen, within } from '@solidjs/testing-library';
import { Settings } from './Settings.tsx';
import { AppContext } from '../state/context.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import { stopAppearanceSync } from '../api/prefs.ts';
import { THEME_LIST, themeEntry } from '../theme/registry.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppState } from '../state/store.ts';

function renderSettings() {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const app = createThemeSlice(ctx) as unknown as AppState;
  const utils = render(() => (
    <AppContext.Provider value={app}>
      <Settings onClose={vi.fn()} />
    </AppContext.Provider>
  ));
  return { app, ...utils };
}

/** The gallery card for a pack, found by its accessible name (the pack label). */
function card(label: string): HTMLElement {
  return screen.getByRole('button', { name: label });
}

describe('theme gallery', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
    stopAppearanceSync();
  });
  afterEach(() => stopAppearanceSync());

  it('renders one card per registered theme', async () => {
    renderSettings();
    // The gallery is generated from the registry, so a pack shipped later shows
    // up without touching this screen — the failure this replaces was a
    // hardcoded list that silently omitted six packs.
    await vi.waitFor(() => expect(card('Grove Dark')).toBeInTheDocument());
    for (const entry of THEME_LIST) expect(card(entry.label)).toBeInTheDocument();
  });

  it('groups the cards by pack family', async () => {
    renderSettings();
    await vi.waitFor(() => expect(card('Ocean Light')).toBeInTheDocument());
    const group = screen.getByRole('group', { name: 'Ocean' });
    expect(within(group).getByRole('button', { name: 'Ocean Light' })).toBeInTheDocument();
    expect(within(group).queryByRole('button', { name: 'Plum Light' })).toBeNull();
  });

  it('a card shows the pack description and its light/dark nature', async () => {
    renderSettings();
    await vi.waitFor(() => expect(card('Slate Dark')).toBeInTheDocument());
    expect(card('Slate Dark').textContent).toContain(themeEntry('slate-dark').description);
    expect(card('Slate Dark').textContent).toContain('Dark');
  });

  it('the preview is painted in the pack own colours and is decorative', async () => {
    renderSettings();
    await vi.waitFor(() => expect(card('Plum Light')).toBeInTheDocument());
    const preview = card('Plum Light').querySelector('[aria-hidden="true"]');
    // Decorative: the card's accessible name is the pack label alone, so a
    // screen reader is not read a wall of colour swatches.
    expect(preview).not.toBeNull();
    expect(card('Plum Light')).toHaveAttribute('aria-label', 'Plum Light');
    expect((preview as HTMLElement).style.background).not.toBe('');
  });

  it('picking a card applies the pack and switches to the fixed mode', async () => {
    const { app } = renderSettings();
    // A fresh profile follows the system, so this also covers the case where the
    // click has to take the user OUT of an automatic mode.
    expect(app.themeMode()).toBe('system');

    await vi.waitFor(() => expect(card('Ocean Dark')).toBeInTheDocument());
    fireEvent.click(card('Ocean Dark'));

    expect(app.theme()).toBe('ocean-dark');
    expect(app.themeMode()).toBe('fixed');
    expect(document.documentElement.getAttribute('data-theme')).toBe('ocean-dark');
    expect(card('Ocean Dark')).toHaveAttribute('aria-pressed', 'true');
  });

  it('picking a card seeds that appearance pair member', async () => {
    const { app } = renderSettings();
    await vi.waitFor(() => expect(card('Plum Dark')).toBeInTheDocument());
    fireEvent.click(card('Plum Dark'));

    // Going back to an automatic mode keeps the user inside the pack they liked
    // instead of dropping them on the neutral dark theme.
    expect(app.darkTheme()).toBe('plum-dark');
    app.setThemeMode('system');
    expect(app.darkTheme()).toBe('plum-dark');
  });
});

describe('theme mode tri-state', () => {
  beforeEach(() => {
    localStorage.clear();
    stopAppearanceSync();
  });
  afterEach(() => stopAppearanceSync());

  it('offers the three modes and switches between them', async () => {
    const { app } = renderSettings();
    await vi.waitFor(() =>
      expect(screen.getByRole('button', { name: 'One theme' })).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole('button', { name: 'By time of day' }));
    expect(app.themeMode()).toBe('schedule');
    expect(screen.getByRole('button', { name: 'By time of day' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );

    fireEvent.click(screen.getByRole('button', { name: 'One theme' }));
    expect(app.themeMode()).toBe('fixed');
  });

  it('shows the light/dark pair only in the automatic modes', async () => {
    const { app } = renderSettings();
    await vi.waitFor(() =>
      expect(screen.getByRole('button', { name: 'One theme' })).toBeInTheDocument(),
    );

    // Following the system (the default) needs a pair to switch between…
    expect(screen.getByRole('combobox', { name: 'Light theme' })).toBeInTheDocument();

    // …a single fixed theme does not.
    fireEvent.click(screen.getByRole('button', { name: 'One theme' }));
    expect(screen.queryByRole('combobox', { name: 'Light theme' })).toBeNull();
    expect(app.themeMode()).toBe('fixed');
  });

  it('the pair selects drive the light and dark packs', async () => {
    const { app } = renderSettings();
    await vi.waitFor(() =>
      expect(screen.getByRole('combobox', { name: 'Dark theme' })).toBeInTheDocument(),
    );

    const dark = screen.getByRole('combobox', { name: 'Dark theme' }) as HTMLSelectElement;
    fireEvent.change(dark, { target: { value: 'slate-dark' } });
    expect(app.darkTheme()).toBe('slate-dark');

    // Only dark packs are offered for the dark half of the pair.
    const offered = [...dark.options].map((o) => o.value);
    expect(offered).toContain('amoled');
    expect(offered).not.toContain('slate-light');
  });

  it('the dark-hours window appears only in the schedule mode and is editable', async () => {
    const { app } = renderSettings();
    await vi.waitFor(() =>
      expect(screen.getByRole('button', { name: 'By time of day' })).toBeInTheDocument(),
    );
    expect(screen.queryByLabelText('From')).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: 'By time of day' }));
    const from = screen.getByLabelText('From');
    expect(from).toHaveValue('20:00');

    fireEvent.change(from, { target: { value: '21:30' } });
    expect(app.schedule().darkStart).toBe('21:30');
    // The other end of the window is untouched by editing one side.
    expect(app.schedule().darkEnd).toBe('07:00');
  });
});

describe('appearance sync status', () => {
  beforeEach(() => {
    localStorage.clear();
    stopAppearanceSync();
  });
  afterEach(() => stopAppearanceSync());

  it('states where the appearance is saved', async () => {
    renderSettings();
    // No server in a unit test, so the honest answer is "on this device" — the
    // panel never claims a save it did not make.
    await vi.waitFor(() => {
      const text = screen.getByRole('dialog').textContent ?? '';
      expect(/Saved on this device|Could not reach the server|Checking your account/.test(text)).toBe(
        true,
      );
    });
  });

  it('offers to forget the appearance stored for the account', async () => {
    renderSettings();
    await vi.waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'Forget the appearance saved for my account' }),
      ).toBeInTheDocument(),
    );
  });
});
