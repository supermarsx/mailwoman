// t85: the two badge colours derived from the live accent.
//
//   accent-lighter   the accent with its OKLCH lightness raised by 0.2
//   accent-darker    the accent with its OKLCH lightness lowered by 0.2
//
// OKLCH L runs 0..1, so 0.2 is 20% of its range; the result is clamped to the
// range, keeps the accent's hue and chroma, and lowers the chroma only when the
// colour would fall outside sRGB (`oklchToHex`). The count text is black or
// white, whichever contrasts more with the badge: one of the two always clears
// 4.5:1 (the worst case, a mid grey, still gives about 4.6:1).
//
// The accent is the one the page paints: the custom accent when one is set,
// else the named accent, else the theme's own, in the ACTIVE theme's shade. The
// theme slice writes the result to `--mw-badge-derived` / `--mw-badge-derived-text`
// on :root through the CSSOM, and `themes.css.ts` maps them to `--mw-badge`.

import { oklchToHex, toOklch, type AccentId } from './accentMath.ts';
import { contrastRatio } from './contrast.ts';
import { customAccentShade } from './customAccent.ts';
import { accentShade } from './registry.ts';
import type { ThemeName } from './contract.css.ts';

export type DerivedBadge = 'accent-lighter' | 'accent-darker';
export const DERIVED_BADGES: readonly DerivedBadge[] = ['accent-lighter', 'accent-darker'];

/** How far the badge moves from the accent, in OKLCH L (range 0..1). */
export const BADGE_LIGHTNESS_STEP = 0.2;

export const DERIVED_BADGE_PROPERTY = '--mw-badge-derived';
export const DERIVED_BADGE_TEXT_PROPERTY = '--mw-badge-derived-text';

const INKS = ['#ffffff', '#000000'] as const;

export function isDerivedBadge(value: unknown): value is DerivedBadge {
  return value === 'accent-lighter' || value === 'accent-darker';
}

/** Black or white, whichever has the higher contrast with `background`. */
export function contrastingText(background: string): string {
  const [white, black] = INKS;
  return contrastRatio(white, background) >= contrastRatio(black, background) ? white : black;
}

/**
 * The badge colour and its text for `accent` (an opaque CSS colour), or null
 * when the accent cannot be read.
 */
export function deriveBadge(accent: string, which: DerivedBadge): { badge: string; text: string } | null {
  const lch = toOklch(accent);
  if (lch === null) return null;
  const step = which === 'accent-lighter' ? BADGE_LIGHTNESS_STEP : -BADGE_LIGHTNESS_STEP;
  const l = Math.min(1, Math.max(0, lch.l + step));
  const badge = oklchToHex(l, lch.c, lch.h);
  return { badge, text: contrastingText(badge) };
}

/** The accent the page paints for these prefs on `theme`. */
export function effectiveAccent(theme: ThemeName, accentId: AccentId | '', customAccent: string): string {
  return customAccentShade(customAccent, theme)?.accent ?? accentShade(theme, accentId).accent;
}

/** `deriveBadge` of the effective accent, for the theme slice and the picker swatches. */
export function derivedBadgeFor(
  theme: ThemeName,
  accentId: AccentId | '',
  customAccent: string,
  which: DerivedBadge,
): { badge: string; text: string } | null {
  return deriveBadge(effectiveAccent(theme, accentId, customAccent), which);
}

/**
 * Write (or clear) the derived badge colour on `root` through the CSSOM. Called
 * by the theme slice on every appearance change, so an accent or theme switch
 * repaints the badges without a reload.
 */
export function applyDerivedBadge(
  root: HTMLElement,
  badgeColor: string,
  theme: ThemeName,
  accentId: AccentId | '',
  customAccent: string,
): void {
  const derived = isDerivedBadge(badgeColor) ? derivedBadgeFor(theme, accentId, customAccent, badgeColor) : null;
  if (derived === null) {
    root.style.removeProperty(DERIVED_BADGE_PROPERTY);
    root.style.removeProperty(DERIVED_BADGE_TEXT_PROPERTY);
    return;
  }
  root.style.setProperty(DERIVED_BADGE_PROPERTY, derived.badge);
  root.style.setProperty(DERIVED_BADGE_TEXT_PROPERTY, derived.text);
}
