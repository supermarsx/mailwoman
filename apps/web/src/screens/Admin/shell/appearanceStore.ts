// The admin console's own appearance (t32-cfg DC5, F22): theme, accent and mode.
//
// The console no longer borrows the webmail theme read-only. An admin picks any
// of the built-in themes, any of the ten accents (or the theme's own), and a
// mode: light, dark, or match the system. The mode chooses the light or dark
// variant of the picked theme's pack (`variantFor`), so "match the system" stays
// inside the pack the admin chose. The webmail allowed-themes policy does not
// apply here (DC5).
//
// Default: until the admin makes a choice, the console follows this browser's
// webmail theme (its stored preference, resolved now), else Dark.
//
// Storage: `localStorage['mw.admin.appearance']` behind a small store interface;
// e71b swaps in `GET/PUT /admin/me/preferences` (per admin account) without
// touching the controller. Density and the UI-font override still come from this
// browser's webmail preference, read-only, as before t32-e71a.

import { createSignal, type Accessor } from 'solid-js';
import type { ThemeName } from '../../../theme/contract.css.ts';
import { FONT_STACKS, parseAppearancePrefs, resolveThemeName, type AppearancePrefs } from '../../../theme/appearance.ts';
import { DEFAULT_DARK_THEME, isThemeName, themeEntry, variantFor } from '../../../theme/registry.ts';
import { isAccentId, type AccentId } from '../../../theme/accentMath.ts';
import { prefersDarkScheme, syncRootAppearance, watchColorScheme } from '../../../shell/rootAttributes.ts';

export type AdminThemeMode = 'light' | 'dark' | 'system';
export const ADMIN_THEME_MODES: readonly AdminThemeMode[] = ['light', 'dark', 'system'];

export interface AdminAppearancePrefs {
  theme: ThemeName;
  /** A named accent, or `''` for the theme's own. */
  accent: AccentId | '';
  mode: AdminThemeMode;
}

/** Where the console's choice is kept in this browser. */
export const ADMIN_APPEARANCE_KEY = 'mw.admin.appearance';
/** The webmail's appearance blob (`state/slices/theme.ts`), read only for the default. */
export const WEBMAIL_APPEARANCE_KEY = 'mw.theme.prefs';

/** Validate an untrusted stored value. Returns null when it is not a usable choice. */
export function parseAdminAppearance(raw: unknown): AdminAppearancePrefs | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const o = raw as Record<string, unknown>;
  if (!isThemeName(o['theme'])) return null;
  const mode = ADMIN_THEME_MODES.includes(o['mode'] as AdminThemeMode) ? (o['mode'] as AdminThemeMode) : themeEntry(o['theme']).appearance;
  const accent = isAccentId(o['accent']) ? o['accent'] : '';
  return { theme: o['theme'], accent, mode };
}

/** The default: this browser's webmail theme when one was chosen, else Dark. */
export function defaultAdminAppearance(webmail: AppearancePrefs | null, systemDark: boolean, now: Date = new Date()): AdminAppearancePrefs {
  if (webmail === null || !webmail.chosen) {
    return { theme: DEFAULT_DARK_THEME, accent: webmail?.accentId ?? '', mode: 'dark' };
  }
  const theme = resolveThemeName(webmail, { systemDark, now });
  const mode: AdminThemeMode = webmail.mode === 'fixed' ? themeEntry(theme).appearance : 'system';
  return { theme, accent: webmail.accentId, mode };
}

/** The pack painted for `prefs`, given the OS colour scheme. */
export function resolveAdminTheme(prefs: AdminAppearancePrefs, systemDark: boolean): ThemeName {
  const appearance = prefs.mode === 'system' ? (systemDark ? 'dark' : 'light') : prefs.mode;
  return variantFor(prefs.theme, appearance);
}

/** The persistence seam: localStorage now, the per-account endpoint later (e71b). */
export interface AdminAppearanceStore {
  load(): AdminAppearancePrefs | null;
  save(prefs: AdminAppearancePrefs | null): void;
}

function readJson(key: string): unknown {
  try {
    const raw = typeof localStorage === 'undefined' ? null : localStorage.getItem(key);
    return raw === null ? null : (JSON.parse(raw) as unknown);
  } catch {
    return null;
  }
}

export const localAppearanceStore: AdminAppearanceStore = {
  load: () => parseAdminAppearance(readJson(ADMIN_APPEARANCE_KEY)),
  save(prefs) {
    try {
      if (prefs === null) localStorage.removeItem(ADMIN_APPEARANCE_KEY);
      else localStorage.setItem(ADMIN_APPEARANCE_KEY, JSON.stringify(prefs));
    } catch {
      // Private mode or a full quota: the choice applies for this page only.
    }
  },
};

/** This browser's webmail appearance, or null when none is stored. */
export function readWebmailAppearance(): AppearancePrefs | null {
  const raw = readJson(WEBMAIL_APPEARANCE_KEY);
  return raw === null ? null : parseAppearancePrefs(raw);
}

export interface AdminAppearanceController {
  /** The current choice (the default until the admin picks). */
  prefs: Accessor<AdminAppearancePrefs>;
  /** The pack on `data-theme` now. */
  resolved: Accessor<ThemeName>;
  /** True once the admin made a choice (false = following the webmail theme or Dark). */
  chosen: Accessor<boolean>;
  /** Pick a theme. Outside `system` mode the mode follows the theme's appearance. */
  setTheme(theme: ThemeName): void;
  setAccent(accent: AccentId | ''): void;
  setMode(mode: AdminThemeMode): void;
  /** Forget the choice and follow the webmail theme again. */
  reset(): void;
  dispose(): void;
}

export interface AdminAppearanceOptions {
  store?: AdminAppearanceStore;
  webmail?: () => AppearancePrefs | null;
}

function apply(theme: ThemeName, accent: AccentId | '', webmail: AppearancePrefs | null): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.setAttribute('data-theme', theme);
  if (accent !== '') root.setAttribute('data-accent', accent);
  else root.removeAttribute('data-accent');
  syncRootAppearance(themeEntry(theme).appearance);
  // Density and font: this browser's webmail preference, read-only (unchanged by e71a).
  root.setAttribute('data-density', webmail?.density ?? 'cozy');
  const stack = webmail ? FONT_STACKS[webmail.font] : null;
  if (stack) root.style.setProperty('--mw-ui-font', stack);
  else root.style.removeProperty('--mw-ui-font');
}

/** Apply the console's appearance to `:root` and keep it applied. */
export function createAdminAppearance(opts: AdminAppearanceOptions = {}): AdminAppearanceController {
  const store = opts.store ?? localAppearanceStore;
  const webmail = opts.webmail ?? readWebmailAppearance;
  const [systemDark, setSystemDark] = createSignal(prefersDarkScheme());
  const stored = store.load();
  const [chosen, setChosen] = createSignal(stored !== null);
  const [prefs, setPrefs] = createSignal<AdminAppearancePrefs>(stored ?? defaultAdminAppearance(webmail(), systemDark()));
  const resolved = (): ThemeName => resolveAdminTheme(prefs(), systemDark());

  const sync = (): void => apply(resolved(), prefs().accent, webmail());
  sync();
  const stop = watchColorScheme((dark) => {
    setSystemDark(dark);
    if (!chosen()) setPrefs(defaultAdminAppearance(webmail(), dark));
    sync();
  });

  function commit(next: AdminAppearancePrefs): void {
    setPrefs(next);
    setChosen(true);
    store.save(next);
    sync();
  }

  return {
    prefs,
    resolved,
    chosen,
    setTheme(theme) {
      const p = prefs();
      commit({ ...p, theme, mode: p.mode === 'system' ? 'system' : themeEntry(theme).appearance });
    },
    setAccent(accent) {
      commit({ ...prefs(), accent });
    },
    setMode(mode) {
      commit({ ...prefs(), mode });
    },
    reset() {
      store.save(null);
      setChosen(false);
      setPrefs(defaultAdminAppearance(webmail(), systemDark()));
      sync();
    },
    dispose: stop,
  };
}
