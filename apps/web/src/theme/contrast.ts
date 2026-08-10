// WCAG contrast primitives + the per-theme requirement table (SPEC §17.1, §24).
//
// This module is DATA + MATH only: it declares which token pairs every built-in
// theme must satisfy and how to measure them. It deliberately contains no test
// assertions — `contrast.test.ts` (t19-e12) is the automated suite that runs
// this table across `THEME_REGISTRY` and fails the build on a regression.
//
// Ratios follow WCAG 2.2 §1.4.3 (text, 4.5:1 at normal weight) and §1.4.11
// (non-text UI boundaries and state indicators, 3:1). Every value in the token
// palettes is an opaque colour, so a plain relative-luminance ratio is exact —
// no alpha compositing is involved. An unparseable or translucent colour is
// reported as a FAILURE rather than skipped, so a token that stops being
// measurable can never silently pass.

import type { ThemeTokens } from './tokens.ts';

/** sRGB 0-255 triple. */
export type Rgb = readonly [number, number, number];

/** The palette keys a contrast pair can name. */
export type ColorKey = keyof ThemeTokens['color'];

const HEX = /^#([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$/i;
const RGB_FN = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)\s*(?:[,/]\s*([\d.%]+)\s*)?\)$/i;
const VAR_FN = /^var\(\s*--[\w-]+\s*,\s*(.+)\)$/;

/**
 * Parse an opaque CSS colour to sRGB. Accepts `#rgb`/`#rrggbb`, `rgb()`, and a
 * `var(--x, fallback)` wrapper (the accent tokens are written that way so an
 * inline `--mw-accent` can override them — the fallback is the theme's own
 * value, which is what a theme's contrast is judged on).
 *
 * Returns `null` for anything translucent or unrecognised: callers treat that
 * as a failure, never as a pass.
 */
export function parseColor(css: string): Rgb | null {
  const v = css.trim();

  const varMatch = VAR_FN.exec(v);
  if (varMatch?.[1] !== undefined) return parseColor(varMatch[1]);

  if (HEX.test(v)) {
    const h = v.slice(1);
    if (h.length === 3 || h.length === 4) {
      // #rgba is translucent unless the alpha nibble is `f`.
      if (h.length === 4 && h[3]?.toLowerCase() !== 'f') return null;
      const [r, g, b] = [h[0], h[1], h[2]].map((c) => parseInt(`${c}${c}`, 16));
      return r !== undefined && g !== undefined && b !== undefined ? [r, g, b] : null;
    }
    if (h.length === 8 && h.slice(6).toLowerCase() !== 'ff') return null;
    return [
      parseInt(h.slice(0, 2), 16),
      parseInt(h.slice(2, 4), 16),
      parseInt(h.slice(4, 6), 16),
    ];
  }

  const fn = RGB_FN.exec(v);
  if (fn?.[1] !== undefined && fn[2] !== undefined && fn[3] !== undefined) {
    const alpha = fn[4];
    if (alpha !== undefined && alpha !== '1' && alpha !== '100%') return null;
    return [Number(fn[1]), Number(fn[2]), Number(fn[3])];
  }

  return null;
}

/** WCAG relative luminance of an sRGB triple. */
export function relativeLuminance([r, g, b]: Rgb): number {
  const lin = (c: number): number => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

/**
 * Contrast ratio between two CSS colours, 1..21. Returns `0` when either side
 * cannot be measured — a value no threshold accepts, so unmeasurable is
 * indistinguishable from failing.
 */
export function contrastRatio(a: string, b: string): number {
  const ca = parseColor(a);
  const cb = parseColor(b);
  if (ca === null || cb === null) return 0;
  const la = relativeLuminance(ca);
  const lb = relativeLuminance(cb);
  const [hi, lo] = la >= lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

/**
 * Which rule a pair is held to.
 *   • `text`     — WCAG §1.4.3, 4.5:1. ENFORCED: a pack failing one is a bug.
 *   • `non-text` — WCAG §1.4.11, 3:1, for colours that carry state or identify
 *                  a control. ENFORCED.
 *   • `advisory` — measured and reported, NOT enforced. See `CONTRAST_PAIRS`.
 */
export type ContrastLevel = 'text' | 'non-text' | 'advisory';

export interface ContrastPair {
  readonly fg: ColorKey;
  readonly bg: ColorKey;
  /** Minimum acceptable ratio. */
  readonly min: number;
  readonly level: ContrastLevel;
  /** Why this pair exists — the UI situation it protects. */
  readonly why: string;
}

/** Every surface a foreground colour can be painted on. */
const SURFACES: readonly ColorKey[] = ['bg', 'bgAlt', 'bgSink', 'surface'];

/** Foreground colours used as body/label text somewhere in the chrome. */
const TEXT_ON_SURFACE: readonly ColorKey[] = [
  'text',
  'textDim',
  'link',
  'danger',
  'success',
  'warning',
];

function pairs(): ContrastPair[] {
  const out: ContrastPair[] = [];
  for (const bg of SURFACES) {
    for (const fg of TEXT_ON_SURFACE) {
      out.push({ fg, bg, min: 4.5, level: 'text', why: `${fg} text rendered on ${bg}` });
    }
    // `accent` carries state — focus rings, the selected row, filled controls.
    out.push({
      fg: 'accent',
      bg,
      min: 3,
      level: 'non-text',
      why: `accent state indicator drawn on ${bg}`,
    });
    // `border` is ADVISORY, not enforced, and deliberately so: this one token
    // draws both decorative panel dividers (exempt from §1.4.11) and the
    // boundaries of real controls (3:1 required). Enforcing 3:1 on the shared
    // token would darken every divider in the app — the correct fix is a
    // component-level audit that splits the control boundary onto its own
    // token. Until that lands, the ratio is MEASURED and REPORTED so the gap
    // stays visible instead of being quietly dropped from the table.
    out.push({
      fg: 'border',
      bg,
      min: 3,
      level: 'advisory',
      why: `border drawn on ${bg} (decorative + control boundaries share this token)`,
    });
  }
  // `accentText` is the label colour for every FILLED control, which in this
  // codebase means the accent button and the status badges alike
  // (`modules/keys/styles`: verified/revoked badges paint it on success/danger).
  for (const bg of ['accent', 'success', 'danger'] as const) {
    out.push({
      fg: 'accentText',
      bg,
      min: 4.5,
      level: 'text',
      why: `label on a filled ${bg} control`,
    });
  }
  // Selected text keeps its own colour over the selection highlight.
  out.push({
    fg: 'text',
    bg: 'selection',
    min: 4.5,
    level: 'text',
    why: 'text under a selection highlight',
  });
  return out;
}

/**
 * The full requirement table every built-in theme is measured against.
 * Frozen shape: `contrast.test.ts` iterates it, so adding a pair here
 * immediately tightens the gate for all themes.
 */
export const CONTRAST_PAIRS: readonly ContrastPair[] = Object.freeze(pairs());

export interface ContrastResult {
  readonly pair: ContrastPair;
  /** Measured ratio, rounded to 2dp. `0` when a colour was unmeasurable. */
  readonly ratio: number;
  readonly ok: boolean;
}

/** Measure every declared pair against one theme's colour palette. */
export function checkPalette(color: ThemeTokens['color']): ContrastResult[] {
  return CONTRAST_PAIRS.map((pair) => {
    const raw = contrastRatio(color[pair.fg], color[pair.bg]);
    const ratio = Math.round(raw * 100) / 100;
    return { pair, ratio, ok: ratio >= pair.min };
  });
}

/**
 * ENFORCED pairs a palette fails — empty means the pack is contrast-viable and
 * is the condition every built-in theme must satisfy.
 */
export function failingPairs(color: ThemeTokens['color']): ContrastResult[] {
  return checkPalette(color).filter((r) => !r.ok && r.pair.level !== 'advisory');
}

/** Advisory shortfalls: reported, never fatal. See `ContrastLevel`. */
export function advisoryFailures(color: ThemeTokens['color']): ContrastResult[] {
  return checkPalette(color).filter((r) => !r.ok && r.pair.level === 'advisory');
}
