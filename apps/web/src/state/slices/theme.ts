// Theme slice: the runtime side of the appearance layer (SPEC §17.1).
//
// Owns the DOM and the listeners; every *decision* is a pure function in
// `theme/appearance.ts` and every fact about a theme comes from
// `theme/registry.ts`. What lands on `:root`:
//   • `data-theme`      — the resolved concrete pack (never `system`/`schedule`)
//   • `data-density`    — row height / base font size
//   • `data-appearance` + `color-scheme` — light or dark, for UA-painted widgets
//   • `data-accent`     — the named accent; `themes.css.ts` has one rule per
//                         theme × accent with that theme's shade (an attribute,
//                         not an inline `--mw-accent`, because an inline
//                         property would beat those rules)
//   • `--mw-ui-font`    — inline override on top of any pack
//
// TRI-STATE: `themeMode` is `fixed` | `system` | `schedule`.
//   • `system` subscribes to `prefers-color-scheme` and re-resolves LIVE, so a
//     platform theme change is reflected without a reload.
//   • `schedule` re-resolves on a timer armed at the next window boundary
//     (no polling), for platforms that never report a scheme change.
// Both modes pick from the user's light/dark pack PAIR, so following the OS
// does not mean being limited to the two neutral packs.
//
// DEFAULT (t32): a user who never made a theme decision gets the Dark pack in
// `fixed` mode, whatever the OS reports. `chosen` records that decision: the
// theme setters set it, density/accent/font/layout setters do not, and while it
// is false `parseAppearancePrefs` keeps the theme fields on the defaults.
//
// PERSISTENCE: the whole `AppearancePrefs` object is one JSON blob in
// localStorage. `subscribePrefs()` fires after every change — that is the seam
// for per-user server sync (§17.3); swapping the transport needs no other edit.
//
// Importing the style modules here (for their side effects) is what pulls the
// vanilla-extract themes + @font-face + print CSS into the bundle graph, since
// the store is reachable from `main.tsx`.
import '../../theme/themes.css.ts';
import '../../styles/print.css.ts';

import { createSignal, type Accessor } from 'solid-js';
import type { ThemeName, Density } from '../../theme/contract.css.ts';
import {
  FONT_STACKS,
  defaultAppearance,
  legacyRibbonCollapsed,
  mergeAppearancePrefs,
  msUntilNextFlip,
  parseAppearancePrefs,
  resolveAppearance,
  resolveThemeName,
  serializeAppearancePrefs,
  type AppearancePrefs,
  type BadgeColor,
  type LayoutMode,
  type ThemeMode,
  type ThemeSchedule,
  type UiFont,
} from '../../theme/appearance.ts';
import { themeEntry, type Appearance } from '../../theme/registry.ts';
import type { AccentId } from '../../theme/accentMath.ts';
import { applyCustomAccent } from '../../theme/customAccent.ts';
import {
  prefersDarkScheme,
  syncRootAppearance,
  watchColorScheme,
} from '../../shell/rootAttributes.ts';
import type { SliceContext } from './context.ts';
import { noteLegacyRibbonCollapsed } from '../legacyRibbonCollapsed.ts';

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
  /** The named accent, or `''` for the pack's own. */
  accentId: Accessor<AccentId | ''>;
  /** Unread-count badge colour (`data-badge` on :root). */
  badgeColor: Accessor<BadgeColor>;
  uiFont: Accessor<UiFont>;
  layout: Accessor<LayoutMode>;
  /**
   * Whether the user has made a theme decision. False means the theme fields
   * are the defaults (Dark), which a deployment default may replace.
   */
  themeChosen: Accessor<boolean>;

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
  setAccentId(id: AccentId | ''): void;
  setBadgeColor(c: BadgeColor): void;
  setUiFont(f: UiFont): void;
  setLayout(l: LayoutMode): void;

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
    const stored: unknown = JSON.parse(raw);
    const prefs = parseAppearancePrefs(stored);
    if (legacyRibbonCollapsed(stored)) {
      // Hand the old collapse over to the ui.bar.mailRibbon migration, then
      // drop the field from the stored blob.
      noteLegacyRibbonCollapsed();
      savePrefs(prefs);
    }
    return prefs;
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
  if (p.accentId !== '') root.setAttribute('data-accent', p.accentId);
  else root.removeAttribute('data-accent');
  applyCustomAccent(root, p.customAccent ?? '', resolved);
  if (p.badgeColor !== 'default') root.setAttribute('data-badge', p.badgeColor);
  else root.removeAttribute('data-badge');
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
  const [accentId, setAccentIdSig] = createSignal<AccentId | ''>(initial.accentId);
  const [customAccent, setCustomAccentSig] = createSignal(initial.customAccent ?? '');
  const [badgeColor, setBadgeColorSig] = createSignal<BadgeColor>(initial.badgeColor);
  const [uiFont, setUiFontSig] = createSignal<UiFont>(initial.font);
  const [layout, setLayoutSig] = createSignal<LayoutMode>(initial.layout);
  const [chosen, setChosenSig] = createSignal(initial.chosen);

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
      accentId: accentId(),
      customAccent: customAccent(),
      badgeColor: badgeColor(),
      font: uiFont(),
      layout: layout(),
      chosen: chosen(),
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
    accentId,
    badgeColor,
    uiFont,
    layout,
    themeChosen: chosen,
    setTheme(t) {
      setPickSig(t);
      setModeSig('fixed');
      // Keep the pair in step with the explicit pick.
      if (themeEntry(t).appearance === 'light') setLightSig(t);
      else setDarkSig(t);
      setChosenSig(true);
      persist();
    },
    setThemeMode(m) {
      setModeSig(m);
      setChosenSig(true);
      persist();
    },
    setLightTheme(t) {
      setLightSig(t);
      setChosenSig(true);
      persist();
    },
    setDarkTheme(t) {
      setDarkSig(t);
      setChosenSig(true);
      persist();
    },
    setSchedule(s) {
      setScheduleSig(s);
      setChosenSig(true);
      persist();
    },
    setDensity(d) {
      setDensitySig(d);
      persist();
    },
    setAccentId(id) {
      setAccentIdSig(id);
      setCustomAccentSig('');
      persist();
    },
    setBadgeColor(c) {
      setBadgeColorSig(c);
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
    appearancePrefs: snapshot,
    setAppearancePrefs(next) {
      // Validate through the same parser an untrusted payload goes through, so
      // a bad field from the sync endpoint degrades instead of corrupting. A
      // server blob written before t32 has no `chosen`; the merge infers it
      // from that blob's theme fields, not from this device's flag.
      // A server blob from a build before t38 may still carry `ribbonCollapsed`.
      if (legacyRibbonCollapsed(next)) noteLegacyRibbonCollapsed();
      const merged = mergeAppearancePrefs(snapshot(), next);
      setModeSig(merged.mode);
      setPickSig(merged.theme);
      setLightSig(merged.lightTheme);
      setDarkSig(merged.darkTheme);
      setScheduleSig(merged.schedule);
      setDensitySig(merged.density);
      setAccentIdSig(merged.accentId);
      setCustomAccentSig(merged.customAccent ?? '');
      setBadgeColorSig(merged.badgeColor);
      setUiFontSig(merged.font);
      setLayoutSig(merged.layout);
      setChosenSig(merged.chosen);
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
