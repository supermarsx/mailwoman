// The accent as link text. `link` is a fixed colour per theme, so a chosen or
// custom accent left links (breadcrumbs, the shared `a` rule) on the theme's
// blue. When an accent is set, the slice derives a link colour from it that
// clears 4.5:1 on every surface of the active theme and writes it to
// `--mw-link-derived` on :root; `linkColor` reads it with the theme link as the
// fallback, so no accent (or an unreadable one) keeps the theme's own link.

import { accentContextOf, accentLinkColor, type AccentId } from './accentMath.ts';
import { effectiveAccent } from './badgeDerived.ts';
import { vars, type ThemeName } from './contract.css.ts';
import { themeEntry } from './registry.ts';

export const DERIVED_LINK_PROPERTY = '--mw-link-derived';

/** The accent-derived link colour for these prefs, or null when no accent is set. */
export function derivedLinkFor(theme: ThemeName, accentId: AccentId | '', customAccent: string): string | null {
  if (accentId === '' && customAccent === '') return null;
  const surfaces = accentContextOf(themeEntry(theme).palette).surfaces;
  return accentLinkColor(effectiveAccent(theme, accentId, customAccent), surfaces);
}

/** Write (or clear) the derived link colour on `root`. */
export function applyDerivedLink(root: HTMLElement, theme: ThemeName, accentId: AccentId | '', customAccent: string): void {
  const color = derivedLinkFor(theme, accentId, customAccent);
  if (color === null) root.style.removeProperty(DERIVED_LINK_PROPERTY);
  else root.style.setProperty(DERIVED_LINK_PROPERTY, color);
}

/** CSS value for link text: the accent-derived link, else the theme's link. */
export const linkColor = `var(${DERIVED_LINK_PROPERTY}, ${vars.color.link})`;
