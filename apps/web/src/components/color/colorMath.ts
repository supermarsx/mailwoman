// Colour maths of the app colour picker (t81): hex parsing, HSV <-> RGB and the
// WCAG contrast hint. Pure functions, no DOM; `colorMath.test.ts` covers them.

import { contrastRatio } from '../../theme/contrast.ts';

/** sRGB 0-255 channels plus alpha 0-1. */
export interface Rgba { r: number; g: number; b: number; a: number }
/** Hue 0-360, saturation and value 0-1, alpha 0-1. */
export interface Hsva { h: number; s: number; v: number; a: number }

const clamp = (value: number, min: number, max: number): number => Math.min(max, Math.max(min, value));
const byte = (value: number): number => clamp(Math.round(value), 0, 255);

/**
 * Parse what a person types in the hex field: `#rgb`, `#rgba`, `#rrggbb` or
 * `#rrggbbaa`, with or without the `#`, any case, surrounding space ignored.
 * Alpha forms are accepted only when `alpha` is true. Anything else is `null`.
 */
export function parseHex(input: string, alpha = false): Rgba | null {
  const raw = input.trim().replace(/^#/, '');
  if (!/^[0-9a-f]+$/i.test(raw)) return null;
  const lengths = alpha ? [3, 4, 6, 8] : [3, 6];
  if (!lengths.includes(raw.length)) return null;
  const full = raw.length <= 4 ? [...raw].map((c) => c + c).join('') : raw;
  const n = (i: number): number => parseInt(full.slice(i, i + 2), 16);
  return { r: n(0), g: n(2), b: n(4), a: full.length === 8 ? Math.round((n(6) / 255) * 100) / 100 : 1 };
}

/** `#rrggbb`, or `#rrggbbaa` when `alpha` is on and the colour is translucent. Lower case. */
export function toHex(c: Rgba, alpha = false): string {
  const h = (n: number): string => byte(n).toString(16).padStart(2, '0');
  const base = `#${h(c.r)}${h(c.g)}${h(c.b)}`;
  return alpha && c.a < 1 ? `${base}${h(c.a * 255)}` : base;
}

/** Normalise a stored value to the field's canonical form, or `null` when it is not a colour. */
export function normalizeHex(input: string | null | undefined, alpha = false): string | null {
  if (!input) return null;
  const parsed = parseHex(input, alpha);
  return parsed ? toHex(parsed, alpha) : null;
}

export function rgbToHsv({ r, g, b, a }: Rgba): Hsva {
  const [rr, gg, bb] = [r / 255, g / 255, b / 255];
  const max = Math.max(rr, gg, bb);
  const min = Math.min(rr, gg, bb);
  const d = max - min;
  let h = 0;
  if (d !== 0) {
    if (max === rr) h = ((gg - bb) / d) % 6;
    else if (max === gg) h = (bb - rr) / d + 2;
    else h = (rr - gg) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  return { h, s: max === 0 ? 0 : d / max, v: max, a };
}

export function hsvToRgb({ h, s, v, a }: Hsva): Rgba {
  const hh = ((h % 360) + 360) % 360;
  const c = v * s;
  const x = c * (1 - Math.abs(((hh / 60) % 2) - 1));
  const m = v - c;
  const [r, g, b] =
    hh < 60 ? [c, x, 0] : hh < 120 ? [x, c, 0] : hh < 180 ? [0, c, x] : hh < 240 ? [0, x, c] : hh < 300 ? [x, 0, c] : [c, 0, x];
  return { r: byte((r + m) * 255), g: byte((g + m) * 255), b: byte((b + m) * 255), a };
}

/** Composite a translucent colour over an opaque backdrop (for the contrast hint). */
export function flatten(c: Rgba, backdrop: Rgba): Rgba {
  const mix = (f: number, b: number): number => byte(f * c.a + b * (1 - c.a));
  return { r: mix(c.r, backdrop.r), g: mix(c.g, backdrop.g), b: mix(c.b, backdrop.b), a: 1 };
}

/** How a colour is used, which decides the WCAG threshold of the hint. */
export type ContrastUse = 'text' | 'non-text';

export interface ContrastHint {
  ratio: number;
  /** `aa`: passes AA for the use; `large`: text passes only at large size (3:1); `fail`. */
  level: 'aa' | 'large' | 'fail';
}

/**
 * The contrast of `color` against `against` (both CSS colours the contrast
 * module parses, i.e. opaque hex or `rgb()`), judged for `use`: text needs
 * 4.5:1 (3:1 counts as large-text only), a badge or boundary needs 3:1.
 * Returns `null` when either side cannot be measured.
 */
export function contrastHint(color: string, against: string, use: ContrastUse): ContrastHint | null {
  const ratio = contrastRatio(color, against);
  if (ratio === 0) return null;
  const rounded = Math.round(ratio * 100) / 100;
  if (use === 'text') return { ratio: rounded, level: ratio >= 4.5 ? 'aa' : ratio >= 3 ? 'large' : 'fail' };
  return { ratio: rounded, level: ratio >= 3 ? 'aa' : 'fail' };
}

/** Clamp helpers used by the keyboard handlers. */
export const clampUnit = (value: number): number => clamp(value, 0, 1);
export const clampHue = (value: number): number => clamp(value, 0, 360);
