// The theme registry — the single lookup surface for everything that needs to
// know about a theme pack (SPEC §17.1).
//
// Layering, top to bottom:
//   contract.css.ts  frozen CSS-custom-property names + the `ThemeName` union
//   tokens.ts        token VALUES per theme + `THEME_META` (labels/family/…)
//   contrast.ts      WCAG math + the pair table every pack must satisfy
//   registry.ts      ← composes the above into one `ThemeEntry` per theme
//
// UI code should read the registry, not `THEMES`/`THEME_META` directly: the
// entry carries the *resolved* palette (accent unwrapped from its
// `var(--mw-accent, …)` override form), ready-to-render preview swatches, and
// the light/dark pairing that the `system` and `schedule` theme modes need.
//
// Consumers: the theme gallery (Settings), the per-theme contrast suite, and
// `state/slices/theme.ts` for validation + mode resolution.

import type { ThemeName } from './contract.css.ts';
import {
  THEMES,
  THEME_META,
  type Appearance,
  type ThemeFamily,
  type ThemeMeta,
  type ThemeTokens,
} from './tokens.ts';
export type { Appearance, ThemeFamily } from './tokens.ts';

/** A theme's colour palette with every value resolved to a concrete colour. */
export type ResolvedPalette = ThemeTokens['color'];

export interface ThemeEntry extends ThemeMeta {
  /** The full token set bound to the contract by `themes.css.ts`. */
  readonly tokens: ThemeTokens;
  /**
   * Palette with `accent` unwrapped from `var(--mw-accent, …)` to the theme's
   * own colour. This is what contrast is judged on and what previews paint —
   * the user's accent override is a separate, orthogonal layer.
   */
  readonly palette: ResolvedPalette;
  /** Ordered preview swatches: page, panel, accent, text, border. */
  readonly swatches: readonly string[];
  /**
   * The pack's opposite-appearance sibling, used by the `system` and
   * `schedule` modes. Packs with only one variant (AMOLED) point at the
   * closest neutral counterpart.
   */
  readonly counterpart: ThemeName;
  /** True for packs designed for `prefers-contrast: more` / forced colours. */
  readonly highContrast: boolean;
}

/** Unwrap `var(--mw-accent, #xxxxxx)` to its fallback; pass anything else through. */
function unwrap(value: string): string {
  const m = /^var\(\s*--[\w-]+\s*,\s*(.+)\)$/.exec(value.trim());
  return m?.[1] !== undefined ? m[1].trim() : value;
}

function resolvePalette(color: ThemeTokens['color']): ResolvedPalette {
  const out = {} as Record<keyof ThemeTokens['color'], string>;
  for (const key of Object.keys(color) as (keyof ThemeTokens['color'])[]) {
    out[key] = unwrap(color[key]);
  }
  return out;
}

/**
 * Counterparts for packs that do not ship both appearances. A pack with a
 * light and a dark variant finds its sibling by family; these fill the gap.
 */
const COUNTERPART_FALLBACK: Partial<Record<ThemeName, ThemeName>> = {
  // AMOLED is dark-only: its light counterpart is the neutral light theme.
  amoled: 'light',
};

function siblingOf(meta: ThemeMeta): ThemeName {
  const sibling = THEME_META.find(
    (m) => m.family === meta.family && m.appearance !== meta.appearance,
  );
  return sibling?.id ?? COUNTERPART_FALLBACK[meta.id] ?? (meta.appearance === 'dark' ? 'light' : 'dark');
}

function entryOf(meta: ThemeMeta): ThemeEntry {
  const tokens = THEMES[meta.id];
  const palette = resolvePalette(tokens.color);
  return {
    ...meta,
    tokens,
    palette,
    swatches: [palette.bg, palette.surface, palette.accent, palette.text, palette.border],
    counterpart: siblingOf(meta),
    highContrast: meta.family === 'contrast',
  };
}

/** Every built-in theme in gallery order. */
export const THEME_LIST: readonly ThemeEntry[] = Object.freeze(THEME_META.map(entryOf));

/** Every built-in theme keyed by its `data-theme` value. */
export const THEME_REGISTRY: Readonly<Record<ThemeName, ThemeEntry>> = Object.freeze(
  Object.fromEntries(THEME_LIST.map((e) => [e.id, e])) as Record<ThemeName, ThemeEntry>,
);

/** Runtime guard — the only sanctioned way to trust an untrusted theme id. */
export function isThemeName(value: unknown): value is ThemeName {
  return typeof value === 'string' && Object.prototype.hasOwnProperty.call(THEME_REGISTRY, value);
}

/** Registry entry for a known theme. */
export function themeEntry(id: ThemeName): ThemeEntry {
  return THEME_REGISTRY[id];
}

/** Registry entry for an untrusted id, or `undefined`. */
export function findTheme(id: unknown): ThemeEntry | undefined {
  return isThemeName(id) ? THEME_REGISTRY[id] : undefined;
}

/** Themes painting the given appearance, in gallery order. */
export function themesByAppearance(appearance: Appearance): readonly ThemeEntry[] {
  return THEME_LIST.filter((e) => e.appearance === appearance);
}

export interface ThemeFamilyGroup {
  readonly family: ThemeFamily;
  /** Family label, taken from the pack name without its variant suffix. */
  readonly label: string;
  readonly themes: readonly ThemeEntry[];
}

const FAMILY_LABELS: Record<ThemeFamily, string> = {
  core: 'Default',
  slate: 'Slate',
  ocean: 'Ocean',
  plum: 'Plum',
  grove: 'Grove',
  amoled: 'AMOLED',
  contrast: 'High contrast',
};

/** The gallery grouping: one row per pack, its variants in appearance order. */
export const THEME_GROUPS: readonly ThemeFamilyGroup[] = Object.freeze(
  (Object.keys(FAMILY_LABELS) as ThemeFamily[]).map((family) => ({
    family,
    label: FAMILY_LABELS[family],
    themes: THEME_LIST.filter((e) => e.family === family),
  })),
);

/**
 * Pick the variant of `id`'s pack that matches `appearance`. Used when a
 * `system`/`schedule` flip should stay inside the pack the user chose — e.g.
 * Grove Light at sunrise for someone who picked Grove Dark.
 */
export function variantFor(id: ThemeName, appearance: Appearance): ThemeName {
  const entry = THEME_REGISTRY[id];
  if (entry.appearance === appearance) return entry.id;
  const counterpart = THEME_REGISTRY[entry.counterpart];
  return counterpart.appearance === appearance ? counterpart.id : entry.id;
}

/** Defaults for the `system`/`schedule` light+dark pair. */
export const DEFAULT_LIGHT_THEME: ThemeName = 'light';
export const DEFAULT_DARK_THEME: ThemeName = 'dark';
