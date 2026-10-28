// Theme MODE behaviour end-to-end through the state slice: the
// fixed/system/schedule tri-state, live OS-follow, and what lands on `:root`
// (SPEC §17.1).
//
// Lives beside the theme layer rather than next to the slice because the
// behaviour under test is the appearance engine — `state/slices/theme.ts` is
// only its DOM adapter, and the slice's own storage/override tests already sit
// in `state/slices/theme.test.ts`.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { createThemeSlice, type ThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';

const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;

/** Minimal `matchMedia` stub with a manual trigger for the dark-scheme query. */
function installMatchMedia(dark: boolean): { set(next: boolean): void } {
  const listeners = new Map<string, Set<(ev: MediaQueryListEvent) => void>>();
  const state = new Map<string, boolean>([['(prefers-color-scheme: dark)', dark]]);

  vi.stubGlobal('matchMedia', (query: string) => ({
    media: query,
    get matches() {
      return state.get(query) === true;
    },
    addEventListener(_type: string, fn: (ev: MediaQueryListEvent) => void) {
      if (!listeners.has(query)) listeners.set(query, new Set());
      listeners.get(query)?.add(fn);
    },
    removeEventListener(_type: string, fn: (ev: MediaQueryListEvent) => void) {
      listeners.get(query)?.delete(fn);
    },
  }));

  return {
    set(next: boolean) {
      const query = '(prefers-color-scheme: dark)';
      state.set(query, next);
      for (const fn of listeners.get(query) ?? []) {
        fn({ matches: next } as MediaQueryListEvent);
      }
    },
  };
}

function root(): HTMLElement {
  return document.documentElement;
}

let slices: ThemeSlice[] = [];

function makeSlice(): ThemeSlice {
  const s = createThemeSlice(ctx);
  slices.push(s);
  return s;
}

describe('theme mode tri-state', () => {
  beforeEach(() => {
    localStorage.clear();
    root().removeAttribute('data-theme');
    root().removeAttribute('data-appearance');
    root().style.removeProperty('color-scheme');
  });

  afterEach(() => {
    for (const s of slices) s.dispose();
    slices = [];
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it.each([false, true])(
    'starts a user who never chose on the Dark pack (OS dark: %s)',
    (osDark) => {
      // t32: dark is the default whatever the OS says, and it is not a choice.
      const os = installMatchMedia(osDark);
      const s = makeSlice();
      expect(s.themeMode()).toBe('fixed');
      expect(s.themeChosen()).toBe(false);
      expect(s.theme()).toBe('dark');
      expect(s.appearance()).toBe('dark');
      expect(root().getAttribute('data-theme')).toBe('dark');
      expect(root().getAttribute('data-appearance')).toBe('dark');
      expect(root().style.getPropertyValue('color-scheme')).toBe('dark');
      // An OS flip does not move a user who never chose.
      os.set(!osDark);
      expect(s.theme()).toBe('dark');
    },
  );

  it('picking "Follow the system" under a light OS gives Light and survives a reload', () => {
    installMatchMedia(false);
    const first = makeSlice();
    first.setThemeMode('system');
    expect(first.themeChosen()).toBe(true);
    expect(first.theme()).toBe('light');

    const second = makeSlice();
    expect(second.themeMode()).toBe('system');
    expect(second.theme()).toBe('light');
    expect(root().getAttribute('data-theme')).toBe('light');
  });

  it('a stored chosen system mode follows the OS signal', () => {
    localStorage.setItem(
      'mw.theme.prefs',
      JSON.stringify({ mode: 'system', lightTheme: 'light', darkTheme: 'dark', chosen: true }),
    );
    const os = installMatchMedia(false);
    const s = makeSlice();
    expect(s.theme()).toBe('light');
    os.set(true);
    expect(s.theme()).toBe('dark');
  });

  it('an old untouched default in storage boots Dark and keeps its density', () => {
    // Written by a build before t32: no `chosen`, system mode, Light/Dark pair.
    localStorage.setItem(
      'mw.theme.prefs',
      JSON.stringify({
        mode: 'system',
        theme: 'light',
        lightTheme: 'light',
        darkTheme: 'dark',
        density: 'compact',
      }),
    );
    installMatchMedia(false);
    const s = makeSlice();
    expect(s.theme()).toBe('dark');
    expect(s.themeChosen()).toBe(false);
    expect(s.density()).toBe('compact');
  });

  it('each theme setter records a choice; the other setters do not', () => {
    installMatchMedia(false);
    const untouched = makeSlice();
    untouched.setDensity('compact');
    untouched.setAccentId('red');
    untouched.setUiFont('mono');
    untouched.setLayout('ribbon');
    expect(untouched.themeChosen()).toBe(false);
    expect(untouched.appearancePrefs().chosen).toBe(false);
    expect(untouched.theme()).toBe('dark');

    const setters: Array<(s: ThemeSlice) => void> = [
      (s) => s.setTheme('ocean-light'),
      (s) => s.setThemeMode('schedule'),
      (s) => s.setLightTheme('slate-light'),
      (s) => s.setDarkTheme('plum-dark'),
      (s) => s.setSchedule({ darkStart: '19:00', darkEnd: '06:00' }),
    ];
    for (const apply of setters) {
      localStorage.clear();
      const s = makeSlice();
      expect(s.themeChosen()).toBe(false);
      apply(s);
      expect(s.themeChosen()).toBe(true);
      expect(JSON.parse(localStorage.getItem('mw.theme.prefs') ?? '{}').chosen).toBe(true);
    }
  });

  it('adopting a pre-t32 server blob keeps its explicit pick on a fresh device', () => {
    installMatchMedia(false);
    const s = makeSlice();
    s.setAppearancePrefs({ mode: 'fixed', theme: 'amoled' });
    expect(s.theme()).toBe('amoled');
    expect(s.themeChosen()).toBe(true);

    // …and the old untouched default from another device moves it to Dark.
    s.setAppearancePrefs({ mode: 'system', theme: 'light', lightTheme: 'light', darkTheme: 'dark' });
    expect(s.theme()).toBe('dark');
    expect(s.themeChosen()).toBe(false);
  });

  it('follows an OS scheme change LIVE, with no reload', () => {
    const os = installMatchMedia(false);
    const s = makeSlice();
    s.setThemeMode('system');
    expect(s.theme()).toBe('light');

    os.set(true);
    expect(s.theme()).toBe('dark');
    expect(root().getAttribute('data-theme')).toBe('dark');
    expect(root().style.getPropertyValue('color-scheme')).toBe('dark');

    os.set(false);
    expect(s.theme()).toBe('light');
    expect(root().getAttribute('data-theme')).toBe('light');
  });

  it('follows the OS inside the user’s chosen pack pair', () => {
    const os = installMatchMedia(false);
    const s = makeSlice();
    s.setLightTheme('ocean-light');
    s.setDarkTheme('plum-dark');
    s.setThemeMode('system');
    expect(s.theme()).toBe('ocean-light');
    os.set(true);
    expect(s.theme()).toBe('plum-dark');
  });

  it('picking a pack explicitly pins it and stops following the OS', () => {
    const os = installMatchMedia(false);
    const s = makeSlice();
    s.setTheme('grove-dark');
    expect(s.themeMode()).toBe('fixed');
    expect(s.theme()).toBe('grove-dark');
    expect(s.appearance()).toBe('dark');
    // The pick also seeds the pair, so returning to `system` keeps the pack.
    expect(s.darkTheme()).toBe('grove-dark');
    os.set(true);
    expect(s.theme()).toBe('grove-dark');
    os.set(false);
    expect(s.theme()).toBe('grove-dark');

    s.setThemeMode('system');
    expect(s.theme()).toBe('light');
    os.set(true);
    expect(s.theme()).toBe('grove-dark');
  });

  it('schedule mode resolves from the local clock and flips on its own timer', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 7, 10, 19, 59, 0));
    installMatchMedia(false);
    const s = makeSlice();
    s.setDarkTheme('slate-dark');
    s.setThemeMode('schedule');
    expect(s.theme()).toBe('light');

    // Default window opens at 20:00 — one minute away, no polling in between.
    vi.advanceTimersByTime(59_000);
    expect(s.theme()).toBe('light');
    vi.advanceTimersByTime(2_000);
    expect(s.theme()).toBe('slate-dark');
    expect(root().getAttribute('data-appearance')).toBe('dark');
  });

  it('schedule mode ignores the OS scheme', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 7, 10, 12, 0, 0));
    const os = installMatchMedia(false);
    const s = makeSlice();
    s.setThemeMode('schedule');
    expect(s.theme()).toBe('light');
    os.set(true);
    expect(s.theme()).toBe('light');
  });

  it('honours a custom dark window', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 7, 10, 18, 30, 0));
    installMatchMedia(false);
    const s = makeSlice();
    s.setThemeMode('schedule');
    expect(s.theme()).toBe('light');
    s.setSchedule({ darkStart: '18:00', darkEnd: '06:00' });
    expect(s.theme()).toBe('dark');
    expect(s.schedule().darkStart).toBe('18:00');
  });

  it('dispose() drops the OS listener and the pending timer', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 7, 10, 19, 59, 0));
    const os = installMatchMedia(false);
    const s = makeSlice();
    s.setThemeMode('schedule');
    s.dispose();
    vi.advanceTimersByTime(10 * 60_000);
    expect(s.theme()).toBe('light');
    os.set(true);
    expect(s.theme()).toBe('light');
  });

  it('persists the mode, the pair and the window across a reload', () => {
    installMatchMedia(true);
    const first = makeSlice();
    first.setLightTheme('slate-light');
    first.setDarkTheme('ocean-dark');
    first.setSchedule({ darkStart: '21:30', darkEnd: '05:45' });
    first.setThemeMode('system');

    const second = makeSlice();
    expect(second.themeMode()).toBe('system');
    expect(second.lightTheme()).toBe('slate-light');
    expect(second.darkTheme()).toBe('ocean-dark');
    expect(second.schedule()).toEqual({ darkStart: '21:30', darkEnd: '05:45' });
    expect(second.theme()).toBe('ocean-dark');
  });

  it('exposes prefs as one object and validates what is merged back in', () => {
    installMatchMedia(false);
    const s = makeSlice();
    const seen: string[] = [];
    const off = s.subscribePrefs((p) => seen.push(p.mode));

    s.setAppearancePrefs({ mode: 'fixed', theme: 'plum-light', density: 'compact' });
    expect(s.theme()).toBe('plum-light');
    expect(s.density()).toBe('compact');
    expect(seen).toEqual(['fixed']);

    // A hostile/garbled sync payload degrades field-by-field instead of
    // corrupting the live state.
    s.setAppearancePrefs({ theme: 'javascript:alert(1)' as never, density: 'huge' as never });
    expect(s.theme()).toBe('dark');
    expect(s.density()).toBe('cozy');

    off();
    s.setDensity('relaxed');
    expect(seen).toHaveLength(2);

    expect(s.appearancePrefs()).toMatchObject({ density: 'relaxed', theme: 'dark' });
  });

  it('is inert where matchMedia does not exist', () => {
    // jsdom's default: no matchMedia at all. Boot must not throw and must land
    // on the default Dark pack.
    const s = makeSlice();
    expect(s.theme()).toBe('dark');
    expect(s.appearance()).toBe('dark');
    expect(() => s.dispose()).not.toThrow();
  });
});
