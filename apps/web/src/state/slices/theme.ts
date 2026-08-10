// Theme slice: the runtime side of the appearance layer (SPEC §17.1).
//
// Owns the DOM and the listeners; every *decision* is a pure function in
// `theme/appearance.ts` and every fact about a theme comes from
// `theme/registry.ts`. What lands on `:root`:
//   • `data-theme`      — the resolved concrete pack (never `system`/`schedule`)
//   • `data-density`    — row height / base font size
//   • `data-appearance` + `color-scheme` — light or dark, for UA-painted widgets
//   • `--mw-accent` / `--mw-ui-font` — inline overrides on top of any pack
//
// TRI-STATE: `themeMode` is `fixed` | `system` | `schedule`.
//   • `system` subscribes to `prefers-color-scheme` and re-resolves LIVE, so a
//     platform theme change is reflected without a reload.
//   • `schedule` re-resolves on a timer armed at the next window boundary
//     (no polling), for platforms that never report a scheme change.
// Both modes pick from the user's light/dark pack PAIR, so following the OS
// does not mean being limited to the two neutral packs.
//
// PERSISTENCE: the whole `AppearancePrefs` object is one JSON blob in
// localStorage. `subscribePrefs()` fires after every change — that is the seam
// for per-user server sync (§17.3); swapping the transport needs no other edit.
//
// Importing the style modules here (for their side effects) is what pulls the
// vanilla-extract themes + @font-face + print CSS into the bundle graph, since
// the store is reachable from `main.tsx`.
import '../../theme/themes.css.ts';
import '../../styles/fonts.css.ts';
import '../../styles/print.css.ts';

import { createSignal, type Accessor } from 'solid-js';
import type { ThemeName, Density } from '../../theme/contract.css.ts';
import {
  FONT_STACKS,
  defaultAppearance,
  msUntilNextFlip,
  parseAppearancePrefs,
  resolveAppearance,
  resolveThemeName,
  serializeAppearancePrefs,
  type AppearancePrefs,
  type LayoutMode,
  type ThemeMode,
  type ThemeSchedule,
  type UiFont,
} from '../../theme/appearance.ts';
import { themeEntry, type Appearance } from '../../theme/registry.ts';
import {
  prefersDarkScheme,
  syncRootAppearance,
  watchColorScheme,
} from '../../shell/rootAttributes.ts';
import type { SliceContext } from './context.ts';

// Re-exported so existing importers (`screens/Settings.tsx`) keep their import
// path; the canonical definitions live in the theme layer.
export type { LayoutMode, UiFont, ThemeMode, ThemeSchedule, AppearancePrefs };

const STORAGE_KEY = 'mw.theme.prefs';

/** The theme portion of `AppState`. */
export interface ThemeSlice {
  /** The RESOLVED pack currently on `data-theme`. */
  theme: Accessor<ThemeName>;
  /** Whether the resolved pack paints a light or dark page. */
  appearance: Accessor<Appearance>;
  /** How the pack is chosen: an explicit pick, the OS, or a clock window. */
  themeMode: Accessor<ThemeMode>;
  /** Pack used when `system`/`schedule` resolves light. */
  lightTheme: Accessor<ThemeName>;
  /** Pack used when `system`/`schedule` resolves dark. */
  darkTheme: Accessor<ThemeName>;
  /** The local dark-window used by `schedule` mode. */
  schedule: Accessor<ThemeSchedule>;
  density: Accessor<Density>;
  accent: Accessor<string>;
  uiFont: Accessor<UiFont>;
  layout: Accessor<LayoutMode>;
  ribbonCollapsed: Accessor<boolean>;

  /**
   * Pick a pack explicitly. This is a user decision, so it switches the mode to
   * `fixed` and records the pack as the pair member for its own appearance —
   * a later switch back to `system` then stays inside the pack they chose.
   */
  setTheme(t: ThemeName): void;
  setThemeMode(m: ThemeMode): void;
  setLightTheme(t: ThemeName): void;
  setDarkTheme(t: ThemeName): void;
  setSchedule(s: ThemeSchedule): void;
  setDensity(d: Density): void;
  setAccent(hex: string): void;
  setUiFont(f: UiFont): void;
  setLayout(l: LayoutMode): void;
  setRibbonCollapsed(v: boolean): void;

  /** Current preferences as one serializable object (server sync, export). */
  appearancePrefs(): AppearancePrefs;
  /** Merge a (possibly partial) preference set in — used to hydrate from sync. */
  setAppearancePrefs(next: Partial<AppearancePrefs>): void;
  /** Observe every preference change. Returns an unsubscribe. */
  subscribePrefs(fn: (prefs: AppearancePrefs) => void): () => void;
  /** Drop the OS listener and any pending schedule timer. */
  dispose(): void;
}

function loadPrefs(): AppearancePrefs {
  if (typeof localStorage === 'undefined') return defaultAppearance();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === null) return defaultAppearance();
    return parseAppearancePrefs(JSON.parse(raw));
  } catch {
    return defaultAppearance();
  }
}

function savePrefs(p: AppearancePrefs): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, serializeAppearancePrefs(p));
  } catch {
    /* private mode / quota — prefs are best-effort */
  }
}

/** Reflect the resolved state onto :root (attributes + inline var overrides). */
function apply(p: AppearancePrefs, resolved: ThemeName, appearance: Appearance): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.setAttribute('data-theme', resolved);
  root.setAttribute('data-density', p.density);
  syncRootAppearance(appearance);
  if (p.accent !== '') root.style.setProperty('--mw-accent', p.accent);
  else root.style.removeProperty('--mw-accent');
  const stack = FONT_STACKS[p.font];
  if (stack !== null) root.style.setProperty('--mw-ui-font', stack);
  else root.style.removeProperty('--mw-ui-font');
}

export function createThemeSlice(_ctx: SliceContext): ThemeSlice {
  const initial = loadPrefs();

  const [mode, setModeSig] = createSignal<ThemeMode>(initial.mode);
  // The explicit pick, distinct from the RESOLVED pack below: in `system` /
  // `schedule` mode the two differ.
  const [pick, setPickSig] = createSignal<ThemeName>(initial.theme);
  const [lightTheme, setLightSig] = createSignal<ThemeName>(initial.lightTheme);
  const [darkTheme, setDarkSig] = createSignal<ThemeName>(initial.darkTheme);
  const [schedule, setScheduleSig] = createSignal<ThemeSchedule>(initial.schedule);
  const [density, setDensitySig] = createSignal<Density>(initial.density);
  const [accent, setAccentSig] = createSignal(initial.accent);
  const [uiFont, setUiFontSig] = createSignal<UiFont>(initial.font);
  const [layout, setLayoutSig] = createSignal<LayoutMode>(initial.layout);
  const [ribbonCollapsed, setRibbonCollapsedSig] = createSignal(initial.ribbonCollapsed);

  const [systemDark, setSystemDark] = createSignal(prefersDarkScheme());
  const [theme, setThemeSig] = createSignal<ThemeName>(initial.theme);
  const [appearance, setAppearanceSig] = createSignal<Appearance>('light');

  const listeners = new Set<(prefs: AppearancePrefs) => void>();
  let flipTimer: ReturnType<typeof setTimeout> | undefined;

  function snapshot(): AppearancePrefs {
    return {
      mode: mode(),
      theme: pick(),
      lightTheme: lightTheme(),
      darkTheme: darkTheme(),
      schedule: schedule(),
      density: density(),
      accent: accent(),
      font: uiFont(),
      layout: layout(),
      ribbonCollapsed: ribbonCollapsed(),
    };
  }

  /**
   * Re-resolve the active pack from the current prefs + OS signal + clock, push
   * it to the DOM, and (in `schedule` mode) arm a timer for the next boundary.
   * Called on every preference change and on every OS colour-scheme change.
   */
  function resync(prefs: AppearancePrefs): void {
    const signals = { systemDark: systemDark(), now: new Date() };
    const resolved = resolveThemeName(prefs, signals);
    const nextAppearance = resolveAppearance(prefs, signals);
    setThemeSig(resolved);
    setAppearanceSig(nextAppearance);
    apply(prefs, resolved, nextAppearance);
    armFlipTimer(prefs);
  }

  function armFlipTimer(prefs: AppearancePrefs): void {
    if (flipTimer !== undefined) {
      clearTimeout(flipTimer);
      flipTimer = undefined;
    }
    if (prefs.mode !== 'schedule') return;
    const delay = msUntilNextFlip(new Date(), prefs.schedule);
    if (delay === null) return;
    flipTimer = setTimeout(() => {
      flipTimer = undefined;
      // Re-read from the signals: prefs may have moved while we waited.
      resync(snapshot());
    }, delay);
  }

  function persist(): void {
    const p = snapshot();
    resync(p);
    savePrefs(p);
    for (const fn of listeners) fn(p);
  }

  resync(initial);

  // Live OS-follow. Subscribed unconditionally (one listener for the page life)
  // so switching INTO `system` mode later needs no re-subscription; `resync`
  // ignores the signal in the other modes.
  const stopColorScheme = watchColorScheme((dark) => {
    setSystemDark(dark);
    resync(snapshot());
  });

  return {
    theme,
    appearance,
    themeMode: mode,
    lightTheme,
    darkTheme,
    schedule,
    density,
    accent,
    uiFont,
    layout,
    ribbonCollapsed,
    setTheme(t) {
      setPickSig(t);
      setModeSig('fixed');
      // Keep the pair in step with the explicit pick.
      if (themeEntry(t).appearance === 'light') setLightSig(t);
      else setDarkSig(t);
      persist();
    },
    setThemeMode(m) {
      setModeSig(m);
      persist();
    },
    setLightTheme(t) {
      setLightSig(t);
      persist();
    },
    setDarkTheme(t) {
      setDarkSig(t);
      persist();
    },
    setSchedule(s) {
      setScheduleSig(s);
      persist();
    },
    setDensity(d) {
      setDensitySig(d);
      persist();
    },
    setAccent(hex) {
      setAccentSig(hex);
      persist();
    },
    setUiFont(f) {
      setUiFontSig(f);
      persist();
    },
    setLayout(l) {
      setLayoutSig(l);
      persist();
    },
    setRibbonCollapsed(v) {
      setRibbonCollapsedSig(v);
      persist();
    },
    appearancePrefs: snapshot,
    setAppearancePrefs(next) {
      // Validate through the same parser an untrusted payload goes through, so
      // a bad field from the sync endpoint degrades instead of corrupting.
      const merged = parseAppearancePrefs({ ...snapshot(), ...next });
      setModeSig(merged.mode);
      setPickSig(merged.theme);
      setLightSig(merged.lightTheme);
      setDarkSig(merged.darkTheme);
      setScheduleSig(merged.schedule);
      setDensitySig(merged.density);
      setAccentSig(merged.accent);
      setUiFontSig(merged.font);
      setLayoutSig(merged.layout);
      setRibbonCollapsedSig(merged.ribbonCollapsed);
      persist();
    },
    subscribePrefs(fn) {
      listeners.add(fn);
      return () => listeners.delete(fn);
    },
    dispose() {
      stopColorScheme();
      if (flipTimer !== undefined) {
        clearTimeout(flipTimer);
        flipTimer = undefined;
      }
      listeners.clear();
    },
  };
}
