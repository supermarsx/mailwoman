// Stable names for the theme contract (t32-ext §V2).
//
// vanilla-extract hashes the contract's custom-property names
// (`--color-bg__1x2y3z`), and the hash changes whenever `contract.css.ts` does,
// so nothing stored outside the build can name them. A custom theme (admin
// theme editor, t32 V2) is therefore stored keyed by the DOTTED token path of
// the contract (`color.bg`, `border.rule`, `elevation.2`, …), which never
// changes, and applied in the browser through this map: each dotted key resolves
// to the hashed name of the running build, so a stored theme works on any
// build without a rebuild, a `<style>` element or a CSP change.
//
// How to apply one (for the V2 lanes):
//   1. Validate the colours with `contrast.ts` (`failingPairs` over the palette)
//      before saving; the server repeats the same pairs.
//   2. Set `data-theme="custom:<id>"` on :root. No stylesheet rule matches that
//      value, so every token must come from step 3: start from a built-in
//      pack's full token set (`flattenTokens(THEMES[x])`) and overwrite it.
//   3. For each `[name, value]` of `tokenProperties(flat)`, call
//      `document.documentElement.style.setProperty(name, value)`. The list holds
//      the contract names and, for colours, the legacy bridge names that the
//      plain `.css` files read (`--bg`, `--accent`, …).
//   4. Inline properties beat every stylesheet rule, including the
//      `[data-accent]` cells, so a user accent on a custom theme is applied the
//      same way: put the custom theme's shade of that accent in `color.accent`
//      and `color.accentText` (and in `color.focus` where the focus follows the
//      accent) before step 3. `shadeAccent` in `accentMath.ts` computes it.
//   5. To leave the custom theme, remove every name `tokenProperties` returned.
//
// `a11y.focusRingColor` and `a11y.focusRing` are derived from `color.focus` in
// the built-in packs (`tokens.ts` a11yOf); a custom theme sets them the same way.

import { vars } from './contract.css.ts';
import type { ThemeTokens } from './tokens.ts';

type Palette = ThemeTokens['color'];

/**
 * Legacy custom properties for each palette colour. `themes.css.ts` emits them
 * per pack for `app.css` and the other plain `.css` files, which cannot read the
 * hashed contract names.
 */
export const LEGACY_COLOR_PROPERTIES: Readonly<Record<keyof Palette, string>> = {
  bg: '--bg',
  bgAlt: '--bg-alt',
  bgSink: '--bg-sink',
  surface: '--surface',
  border: '--border',
  text: '--text',
  textDim: '--text-dim',
  accent: '--accent',
  accentText: '--accent-text',
  danger: '--danger',
  success: '--success',
  warning: '--warning',
  link: '--link',
  selection: '--selection',
  borderStrong: '--border-strong',
  signal: '--signal',
  signalText: '--signal-text',
  focus: '--focus',
  scrim: '--scrim',
  scrimText: '--scrim-text',
};

/** `var(--name)` → `--name`. */
function propertyOf(ref: string): string {
  const m = /^var\((--[^,)]+)\)$/.exec(ref.trim());
  if (m?.[1] === undefined) throw new Error(`not a custom-property reference: ${ref}`);
  return m[1];
}

function flatten(node: unknown, prefix: string, out: Record<string, string>): void {
  if (typeof node === 'string') {
    out[prefix] = node;
    return;
  }
  for (const [k, v] of Object.entries(node as Record<string, unknown>)) {
    flatten(v, prefix === '' ? k : `${prefix}.${k}`, out);
  }
}

const refs: Record<string, string> = {};
flatten(vars, '', refs);

/**
 * Every contract token, by its dotted path, to the custom-property name the
 * running build uses for it (`'color.bg'` → `'--color-bg__1x2y3z'`).
 */
export const CONTRACT_PROPERTIES: Readonly<Record<string, string>> = Object.freeze(
  Object.fromEntries(Object.entries(refs).map(([k, ref]) => [k, propertyOf(ref)])),
);

/** The dotted paths of every contract token, in contract order. */
export const PUBLIC_TOKEN_KEYS: readonly string[] = Object.freeze(Object.keys(CONTRACT_PROPERTIES));

/** A full token set as `{ 'color.bg': '#111111', 'border.rule': '2px', … }`. */
export function flattenTokens(tokens: ThemeTokens): Record<string, string> {
  const out: Record<string, string> = {};
  flatten(tokens, '', out);
  return out;
}

/**
 * The `[property, value]` pairs that apply `flat` to the page: the contract
 * name for each known dotted key, plus the legacy bridge name for each colour.
 * Unknown keys, inherited keys and non-string values are skipped, so a stored
 * theme from an older or newer build degrades instead of throwing.
 */
export function tokenProperties(flat: Readonly<Record<string, unknown>>): Array<[string, string]> {
  const out: Array<[string, string]> = [];
  for (const key of PUBLIC_TOKEN_KEYS) {
    if (!Object.prototype.hasOwnProperty.call(flat, key)) continue;
    const value = flat[key];
    if (typeof value !== 'string') continue;
    out.push([CONTRACT_PROPERTIES[key] as string, value]);
    if (key.startsWith('color.')) {
      const legacy = LEGACY_COLOR_PROPERTIES[key.slice('color.'.length) as keyof Palette];
      if (legacy !== undefined) out.push([legacy, value]);
    }
  }
  return out;
}
