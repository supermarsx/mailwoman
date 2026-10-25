// Appearance preferences: the theme MODE tri-state and everything persisted
// alongside it (SPEC §17.1 — "per-user theme choice … auto light/dark by OS
// schedule; density and accent-color user overrides on top of any theme").
//
// `system` and `schedule` are modes, not themes (see `contract.css.ts`): the
// `data-theme` attribute always names a concrete pack, and this module is the
// pure function that decides WHICH pack, given the mode, the OS colour-scheme
// signal, and the wall clock. Keeping it pure and DOM-free is what makes the
// tri-state testable without a browser; `state/slices/theme.ts` owns the DOM
// and listener side, `shell/rootAttributes.ts` owns the media-query plumbing.
//
// The whole `AppearancePrefs` object is the persistence unit — one JSON blob in
// localStorage today, and the same blob for the per-user server sync (the
// `parse`/`serialize` pair here is the seam; a transport swap needs no other
// change).

import type { Density, ThemeName } from './contract.css.ts';
import {
  DEFAULT_DARK_THEME,
  DEFAULT_LIGHT_THEME,
  THEME_REGISTRY,
  isThemeName,
  variantFor,
  type Appearance,
} from './registry.ts';
import { isAccentId, nearestAccentId, type AccentId } from './accentMath.ts';

/**
 * How the active theme is chosen.
 *   • `fixed`    — the user picked one pack; it never changes on its own.
 *   • `system`   — follow the OS `prefers-color-scheme`, live.
 *   • `schedule` — follow a local clock window (SPEC §17.1 "OS schedule"), for
 *                  platforms that never report a scheme change.
 */
export type ThemeMode = 'fixed' | 'system' | 'schedule';

export const THEME_MODES: readonly ThemeMode[] = ['fixed', 'system', 'schedule'];

/** UI-font override choice. */
export type UiFont = 'default' | 'system' | 'serif' | 'mono' | 'inter' | 'roboto' | 'open-sans' | 'jost';

/** Chrome layout preset. */
export type LayoutMode = 'default' | 'ribbon';

/** UI-font override stacks; `null` = remove the override (use the theme font). */
export const FONT_STACKS: Record<UiFont, string | null> = {
  default: null,
  inter: '"Inter Variable", system-ui, sans-serif',
  roboto: '"Roboto Variable", system-ui, sans-serif',
  'open-sans': '"Open Sans Variable", system-ui, sans-serif',
  jost: '"Jost", system-ui, sans-serif',
  system: 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
  serif: '"Newsreader Variable", Georgia, "Times New Roman", serif',
  mono: '"JetBrains Mono Variable", ui-monospace, Consolas, monospace',
};

/** Local-clock window during which the dark pack is used, `HH:MM` 24-hour. */
export interface ThemeSchedule {
  readonly darkStart: string;
  readonly darkEnd: string;
}

/** Evening-to-morning default: dark from 20:00 until 07:00. */
export const DEFAULT_SCHEDULE: ThemeSchedule = { darkStart: '20:00', darkEnd: '07:00' };

const DENSITY_VALUES: readonly Density[] = ['compact', 'cozy', 'relaxed'];

/** Everything the appearance layer persists, as one serializable unit. */
export interface AppearancePrefs {
  mode: ThemeMode;
  /** The pack used in `fixed` mode; also the last resolved pack. */
  theme: ThemeName;
  /** Pack used when `system`/`schedule` resolves to a light appearance. */
  lightTheme: ThemeName;
  /** Pack used when `system`/`schedule` resolves to a dark appearance. */
  darkTheme: ThemeName;
  schedule: ThemeSchedule;
  density: Density;
  /**
   * One of the named accents (`accentMath.ts`), shaded per theme; empty string
   * = the pack's own accent. Replaces `accent`, the CSS colour that builds
   * before t32 V stored; `parseAppearancePrefs` carries that over to the
   * nearest named accent.
   */
  accentId: AccentId | '';
  font: UiFont;
  layout: LayoutMode;
  ribbonCollapsed: boolean;
  /**
   * True once the user has made a theme decision: picked a pack, a mode, a
   * pair member or a schedule. While it is false the theme fields are the
   * defaults, so a later change of the default reaches every user who never
   * chose. Density, accent, font and layout changes leave it alone.
   */
  chosen: boolean;
}

/** Fields that are reset to the defaults while `chosen` is false. */
const THEME_FIELDS = ['mode', 'theme', 'lightTheme', 'darkTheme'] as const;

/**
 * Defaults for a user with nothing stored: the Dark pack, whatever the OS
 * reports (t32, SPEC §17.1). Light applies once the user picks a light pack,
 * or picks "Follow system" while the OS is light.
 */
export function defaultAppearance(): AppearancePrefs {
  return {
    mode: 'fixed',
    theme: DEFAULT_DARK_THEME,
    lightTheme: DEFAULT_LIGHT_THEME,
    darkTheme: DEFAULT_DARK_THEME,
    schedule: DEFAULT_SCHEDULE,
    density: 'cozy',
    accentId: '',
    font: 'default',
    layout: 'default',
    ribbonCollapsed: false,
    chosen: false,
  };
}

// ── Clock helpers ────────────────────────────────────────────────────────────

const CLOCK = /^([01]\d|2[0-3]):([0-5]\d)$/;

/** `HH:MM` → minutes since local midnight, or `null` if malformed. */
export function parseClock(value: string): number | null {
  const m = CLOCK.exec(value.trim());
  if (m?.[1] === undefined || m[2] === undefined) return null;
  return Number(m[1]) * 60 + Number(m[2]);
}

function minutesOf(now: Date): number {
  return now.getHours() * 60 + now.getMinutes();
}

/**
 * Is `minutes` inside the dark window? Windows wrap past midnight
 * (20:00 → 07:00 is the normal case). A zero-length window (start === end) is
 * "never dark"; a malformed window is also "never dark" so a corrupt
 * preference degrades to light rather than pinning the UI dark.
 */
export function inDarkWindow(minutes: number, schedule: ThemeSchedule): boolean {
  const start = parseClock(schedule.darkStart);
  const end = parseClock(schedule.darkEnd);
  if (start === null || end === null || start === end) return false;
  return start < end
    ? minutes >= start && minutes < end
    : minutes >= start || minutes < end;
}

/**
 * Milliseconds until the schedule next flips appearance, or `null` when the
 * window is malformed/empty (nothing to wake up for). Always ≥ 1000 ms so a
 * boundary landing exactly on `now` cannot spin.
 */
export function msUntilNextFlip(now: Date, schedule: ThemeSchedule): number | null {
  const start = parseClock(schedule.darkStart);
  const end = parseClock(schedule.darkEnd);
  if (start === null || end === null || start === end) return null;

  const nowMs = ((now.getHours() * 60 + now.getMinutes()) * 60 + now.getSeconds()) * 1000 + now.getMilliseconds();
  const dayMs = 24 * 60 * 60 * 1000;
  const deltas = [start, end]
    .map((m) => (m * 60 * 1000 - nowMs + dayMs) % dayMs)
    .map((d) => (d === 0 ? dayMs : d));
  return Math.max(1000, Math.min(...deltas));
}

// ── Resolution ───────────────────────────────────────────────────────────────

/** Inputs the resolver needs from the outside world. */
export interface AppearanceSignals {
  /** `prefers-color-scheme: dark` at this instant. */
  readonly systemDark: boolean;
  /** Local wall clock, injected so the schedule mode is testable. */
  readonly now: Date;
}

/** Which appearance the current mode calls for. */
export function resolveAppearance(
  prefs: AppearancePrefs,
  signals: AppearanceSignals,
): Appearance {
  switch (prefs.mode) {
    case 'system':
      return signals.systemDark ? 'dark' : 'light';
    case 'schedule':
      return inDarkWindow(minutesOf(signals.now), prefs.schedule) ? 'dark' : 'light';
    case 'fixed':
    default:
      return THEME_REGISTRY[prefs.theme].appearance;
  }
}

/**
 * The concrete pack `data-theme` should carry. In `fixed` mode that is the
 * user's pick; otherwise it is their chosen pack for the resolved appearance.
 */
export function resolveThemeName(prefs: AppearancePrefs, signals: AppearanceSignals): ThemeName {
  if (prefs.mode === 'fixed') return prefs.theme;
  const appearance = resolveAppearance(prefs, signals);
  const picked = appearance === 'dark' ? prefs.darkTheme : prefs.lightTheme;
  // If the stored pair member is the wrong appearance (hand-edited prefs, or a
  // pack that lost a variant), fall back to that pack's own counterpart.
  return THEME_REGISTRY[picked].appearance === appearance
    ? picked
    : variantFor(picked, appearance);
}

// ── Persistence ──────────────────────────────────────────────────────────────

function themeOr(value: unknown, fallback: ThemeName): ThemeName {
  return isThemeName(value) ? value : fallback;
}

function scheduleOf(value: unknown, fallback: ThemeSchedule): ThemeSchedule {
  if (value === null || typeof value !== 'object') return fallback;
  const raw = value as Partial<ThemeSchedule>;
  const darkStart =
    typeof raw.darkStart === 'string' && parseClock(raw.darkStart) !== null
      ? raw.darkStart
      : fallback.darkStart;
  const darkEnd =
    typeof raw.darkEnd === 'string' && parseClock(raw.darkEnd) !== null
      ? raw.darkEnd
      : fallback.darkEnd;
  return { darkStart, darkEnd };
}

/**
 * Validate an untrusted payload (localStorage, or the per-user sync response)
 * into a complete `AppearancePrefs`. Every field falls back independently, so a
 * partial or partly-corrupt object still yields a usable set rather than
 * throwing away the fields that were fine.
 *
 * `chosen` decides whether the stored theme fields count:
 *   • `true`  — they are the user's, kept as stored.
 *   • `false` — the user never chose: the theme fields are the defaults (Dark),
 *               and density, accent, font, layout and the schedule window are
 *               kept.
 *   • absent  — written before t32 added the flag; see `inferChosen`.
 */
export function parseAppearancePrefs(input: unknown): AppearancePrefs {
  const base = defaultAppearance();
  if (input === null || typeof input !== 'object') return base;
  const p = input as Partial<AppearancePrefs>;

  const hasLegacyTheme = isThemeName(p.theme);
  // Payloads written before the tri-state existed carry a `theme` but no
  // `mode`. Those users made an explicit choice, so they migrate to `fixed`.
  // Builds before t32 started everything else in `system`.
  const mode: ThemeMode =
    typeof p.mode === 'string' && THEME_MODES.includes(p.mode as ThemeMode)
      ? (p.mode as ThemeMode)
      : hasLegacyTheme
        ? 'fixed'
        : 'system';

  const theme = themeOr(p.theme, base.theme);
  // A pre-tri-state payload has no pair; seed it from the chosen pack so a
  // later switch to `system` keeps the user inside the pack they liked.
  const lightTheme = themeOr(
    p.lightTheme,
    hasLegacyTheme ? variantFor(theme, 'light') : base.lightTheme,
  );
  const darkTheme = themeOr(p.darkTheme, hasLegacyTheme ? variantFor(theme, 'dark') : base.darkTheme);
  const chosen =
    typeof p.chosen === 'boolean' ? p.chosen : inferChosen(mode, lightTheme, darkTheme);

  const rest = {
    schedule: scheduleOf(p.schedule, base.schedule),
    density: DENSITY_VALUES.includes(p.density as Density) ? (p.density as Density) : base.density,
    accentId: accentIdOf(p),
    font: typeof p.font === 'string' && p.font in FONT_STACKS ? (p.font as UiFont) : base.font,
    layout: p.layout === 'ribbon' ? ('ribbon' as const) : ('default' as const),
    ribbonCollapsed: p.ribbonCollapsed === true,
  };

  if (!chosen) {
    const { mode: m, theme: t, lightTheme: l, darkTheme: d } = base;
    return { mode: m, theme: t, lightTheme: l, darkTheme: d, ...rest, chosen: false };
  }
  return { mode, theme, lightTheme, darkTheme, ...rest, chosen: true };
}

/**
 * The accent of a payload: a valid `accentId` as stored (`''` included, the
 * pack default); otherwise the `accent` colour that builds before t32 V stored,
 * mapped to the named accent nearest in hue (Graphite for a grey); otherwise
 * the pack default.
 */
function accentIdOf(p: Partial<AppearancePrefs> & { accent?: unknown }): AccentId | '' {
  if (p.accentId === '' || isAccentId(p.accentId)) return p.accentId;
  if (typeof p.accent === 'string' && p.accent.trim() !== '') {
    return nearestAccentId(p.accent.trim()) ?? '';
  }
  return '';
}

/**
 * Whether a payload without the `chosen` flag (written before t32) records a
 * theme decision.
 *
 * Builds before t32 started every user in `system` mode with the Light/Dark
 * pair, and the account sync pushed that set to the server on first sign-in.
 * So `system` with exactly that pair is what an untouched account stores, and
 * it counts as "never chose". A user who clicked "Follow system" without
 * changing the pair stored the same bytes and cannot be told apart; they get
 * Dark and can pick "Follow system" again. A fixed or scheduled mode, or any
 * other pair, was the user's pick.
 */
function inferChosen(mode: ThemeMode, lightTheme: ThemeName, darkTheme: ThemeName): boolean {
  if (mode !== 'system') return true;
  return lightTheme !== DEFAULT_LIGHT_THEME || darkTheme !== DEFAULT_DARK_THEME;
}

/**
 * Merge a partial payload over the current prefs and validate the result.
 *
 * A payload that carries theme fields but no `chosen` (a server blob written
 * before t32) has `chosen` inferred from its own fields rather than inheriting
 * this device's flag; otherwise an old explicit choice adopted on a fresh
 * device would be reset to the default. A payload without theme fields
 * (density only, say) keeps the current flag.
 */
export function mergeAppearancePrefs(
  current: AppearancePrefs,
  next: Partial<AppearancePrefs> & { accent?: unknown },
): AppearancePrefs {
  const merged: Partial<AppearancePrefs> = { ...current, ...next };
  if (typeof next.chosen !== 'boolean' && THEME_FIELDS.some((k) => k in next)) {
    delete merged.chosen;
  }
  // The same for the accent: a payload from a build before t32 V carries the
  // old `accent` colour and no `accentId`, and its accent wins over this
  // device's.
  if (!('accentId' in next) && 'accent' in next) delete merged.accentId;
  return parseAppearancePrefs(merged);
}

/** The JSON-safe wire/storage form. Round-trips through `parseAppearancePrefs`. */
export function serializeAppearancePrefs(prefs: AppearancePrefs): string {
  return JSON.stringify(prefs);
}
