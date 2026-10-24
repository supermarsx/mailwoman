// The ten named accents and the math that shades them for each theme
// (t32-ext §V).
//
// An accent is a hue, a chroma and the lightness at which it looks most like
// itself, all in OKLCH. A theme cannot always show it at that lightness: on a
// near-black page a mid blue is below 3:1, and on a white page a light yellow
// is. `shadeAccent` walks the accent's lightness in OKLCH (hue fixed, chroma
// pulled into the sRGB gamut) and keeps the shade nearest to the preferred
// lightness that
//   • clears 3:1 against every surface of the theme (WCAG 1.4.11), and
//   • carries an ink (`accentText`) at 4.5:1 (WCAG 1.4.3), where the ink also
//     clears 4.5:1 on the theme's success and danger fills, because the same
//     token labels those.
//
// `scripts/theme/accents.mjs` runs `computeAccentTable` over every theme and
// commits the result as `accents.ts`; `accents.test.ts` recomputes it and fails
// when the committed table and this module disagree. Pure functions only, no
// DOM, so the admin accent editor (t32 V2) can call the same code in the
// browser for a custom theme.

import { contrastRatio, parseColor } from './contrast.ts';

/** The ten accent ids, in picker order. */
export const ACCENT_IDS = [
  'red',
  'orange',
  'yellow',
  'green',
  'teal',
  'blue',
  'indigo',
  'violet',
  'magenta',
  'graphite',
] as const;

export type AccentId = (typeof ACCENT_IDS)[number];

export interface AccentDef {
  readonly id: AccentId;
  /** Fluent id of the user-visible name (`locales/<lang>/appearance.ftl`). */
  readonly labelId: string;
  /** OKLCH hue, degrees. */
  readonly hue: number;
  /** Chroma asked for; lowered where the sRGB gamut cannot reach it. */
  readonly chroma: number;
  /** OKLCH lightness at which the accent looks most like itself. */
  readonly lightness: number;
}

export const ACCENTS: readonly AccentDef[] = [
  { id: 'red', labelId: 'appearance-accent-red', hue: 27, chroma: 0.21, lightness: 0.6 },
  { id: 'orange', labelId: 'appearance-accent-orange', hue: 47, chroma: 0.18, lightness: 0.7 },
  { id: 'yellow', labelId: 'appearance-accent-yellow', hue: 92, chroma: 0.17, lightness: 0.86 },
  { id: 'green', labelId: 'appearance-accent-green', hue: 145, chroma: 0.17, lightness: 0.66 },
  { id: 'teal', labelId: 'appearance-accent-teal', hue: 185, chroma: 0.12, lightness: 0.66 },
  { id: 'blue', labelId: 'appearance-accent-blue', hue: 258, chroma: 0.19, lightness: 0.58 },
  { id: 'indigo', labelId: 'appearance-accent-indigo', hue: 276, chroma: 0.2, lightness: 0.5 },
  { id: 'violet', labelId: 'appearance-accent-violet', hue: 300, chroma: 0.2, lightness: 0.56 },
  { id: 'magenta', labelId: 'appearance-accent-magenta', hue: 335, chroma: 0.22, lightness: 0.6 },
  { id: 'graphite', labelId: 'appearance-accent-graphite', hue: 260, chroma: 0.012, lightness: 0.5 },
];

/** Runtime guard for an untrusted accent id. */
export function isAccentId(value: unknown): value is AccentId {
  return typeof value === 'string' && (ACCENT_IDS as readonly string[]).includes(value);
}

/** One theme × accent cell: the accent colour and the ink drawn on it. */
export interface AccentShade {
  readonly accent: string;
  readonly accentText: string;
}

/** The colours of a theme the shade has to work against. */
export interface AccentContext {
  /** Every surface the accent can be drawn on (bg, bgAlt, bgSink, surface). */
  readonly surfaces: readonly string[];
  /** Fills the ink also labels (the theme's success and danger). */
  readonly fills: readonly string[];
  /** Inks to try, in order of preference (the theme's own `accentText` first). */
  readonly inks: readonly string[];
}

// Margins over the WCAG minimums, so a shade is never accepted on a rounding
// edge. The test suite checks the plain minimums.
const UI_MIN = 3.05;
const TEXT_MIN = 4.55;

// ── OKLCH ↔ sRGB (Björn Ottosson's OKLab, D65) ──────────────────────────────

function toLinear(c: number): number {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function fromLinear(c: number): number {
  return c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
}

/** OKLCH to linear sRGB (may fall outside 0..1 when out of gamut). */
function oklchToLinear(l: number, c: number, h: number): [number, number, number] {
  const a = c * Math.cos((h * Math.PI) / 180);
  const b = c * Math.sin((h * Math.PI) / 180);
  const l_ = Math.pow(l + 0.3963377774 * a + 0.2158037573 * b, 3);
  const m_ = Math.pow(l - 0.1055613458 * a - 0.0638541728 * b, 3);
  const s_ = Math.pow(l - 0.0894841775 * a - 1.291485548 * b, 3);
  return [
    4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
    -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
    -0.0041960863 * l_ - 0.7034186147 * m_ + 1.707614701 * s_,
  ];
}

function inGamut(rgb: readonly number[]): boolean {
  return rgb.every((v) => v >= -1e-6 && v <= 1 + 1e-6);
}

function hex2(v: number): string {
  const n = Math.round(Math.min(1, Math.max(0, fromLinear(v))) * 255);
  return n.toString(16).padStart(2, '0');
}

/**
 * The sRGB hex of `oklch(l c h)`, with the chroma lowered (hue and lightness
 * kept) until the colour fits the sRGB gamut.
 */
export function oklchToHex(l: number, c: number, h: number): string {
  let rgb = oklchToLinear(l, c, h);
  if (!inGamut(rgb)) {
    let lo = 0;
    let hi = c;
    for (let i = 0; i < 24; i++) {
      const mid = (lo + hi) / 2;
      if (inGamut(oklchToLinear(l, mid, h))) lo = mid;
      else hi = mid;
    }
    rgb = oklchToLinear(l, lo, h);
  }
  return `#${hex2(rgb[0])}${hex2(rgb[1])}${hex2(rgb[2])}`;
}

/** OKLCH of an opaque CSS colour, or `null` when it cannot be read. */
export function toOklch(css: string): { l: number; c: number; h: number } | null {
  const rgb = parseColor(css);
  if (rgb === null) return null;
  const [r, g, b] = rgb.map((v) => toLinear(v / 255)) as [number, number, number];
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const h = (Math.atan2(B, A) * 180) / Math.PI;
  return { l: L, c: Math.hypot(A, B), h: h < 0 ? h + 360 : h };
}

// ── Shading ──────────────────────────────────────────────────────────────────

/** Whether `accent` with `ink` on it meets every requirement of `ctx`. */
export function shadeMeets(accent: string, ink: string, ctx: AccentContext): boolean {
  return (
    ctx.surfaces.every((s) => contrastRatio(accent, s) >= UI_MIN) &&
    contrastRatio(ink, accent) >= TEXT_MIN &&
    ctx.fills.every((f) => contrastRatio(ink, f) >= TEXT_MIN)
  );
}

/**
 * The shade of `def` for one theme: the lightness nearest to the accent's own
 * that meets `ctx`, trying each ink and keeping the one that lets the shade
 * stay closest (the earlier ink on a tie). Throws when no lightness works with
 * any ink, so a theme that cannot carry an accent fails the generator and the
 * test rather than shipping an unreadable cell.
 */
export function shadeAccent(def: AccentDef, ctx: AccentContext): AccentShade {
  let best: { shade: AccentShade; distance: number } | null = null;
  for (const ink of ctx.inks) {
    for (let step = 0; step <= 720; step++) {
      // Walk outwards from the preferred lightness, 0.0025 at a time.
      const offset = Math.ceil(step / 2) * 0.0025 * (step % 2 === 0 ? 1 : -1);
      const l = def.lightness + offset;
      if (l <= 0.05 || l >= 0.99) continue;
      const distance = Math.abs(offset);
      if (best !== null && distance >= best.distance) break;
      const accent = oklchToHex(l, def.chroma, def.hue);
      if (shadeMeets(accent, ink, ctx)) {
        best = { shade: { accent, accentText: ink }, distance };
        break;
      }
    }
  }
  if (best === null) {
    throw new Error(`accent ${def.id}: no shade meets the theme's contrast requirements`);
  }
  return best.shade;
}

/** The palette fields `accentContextOf` reads. */
export interface AccentPalette {
  readonly bg: string;
  readonly bgAlt: string;
  readonly bgSink: string;
  readonly surface: string;
  readonly success: string;
  readonly danger: string;
  readonly accentText: string;
}

/** Near-white and near-black inks tried after the theme's own. */
const FALLBACK_INKS = ['#ffffff', '#111111'] as const;

/** What an accent has to clear in a theme with this palette. */
export function accentContextOf(p: AccentPalette): AccentContext {
  const own = parseColor(p.accentText) === null ? [] : [unwrap(p.accentText)];
  return {
    surfaces: [p.bg, p.bgAlt, p.bgSink, p.surface].map(unwrap),
    fills: [p.success, p.danger].map(unwrap),
    inks: [...own, ...FALLBACK_INKS.filter((i) => !own.includes(i))],
  };
}

function unwrap(value: string): string {
  const m = /^var\(\s*--[\w-]+\s*,\s*(.+)\)$/.exec(value.trim());
  return m?.[1] !== undefined ? m[1].trim() : value;
}

/** Every accent shaded for every theme in `palettes`. */
export function computeAccentTable<T extends string>(
  palettes: Readonly<Record<T, AccentPalette>>,
): Record<T, Record<AccentId, AccentShade>> {
  const out = {} as Record<T, Record<AccentId, AccentShade>>;
  for (const theme of Object.keys(palettes) as T[]) {
    const ctx = accentContextOf(palettes[theme]);
    const row = {} as Record<AccentId, AccentShade>;
    for (const def of ACCENTS) row[def.id] = shadeAccent(def, ctx);
    out[theme] = row;
  }
  return out;
}

// Below this OKLCH chroma a colour reads as grey, whatever its hue.
const GREY_CHROMA = 0.04;

/**
 * The named accent nearest to a CSS colour: Graphite for a grey, otherwise
 * the accent with the closest hue. Used to carry an accent stored as a colour
 * (before the named accents) over to an id. `null` when the colour cannot be
 * read.
 */
export function nearestAccentId(css: string): AccentId | null {
  const lch = toOklch(css);
  if (lch === null) return null;
  if (lch.c < GREY_CHROMA) return 'graphite';
  let best: AccentId = 'blue';
  let bestDistance = Infinity;
  for (const def of ACCENTS) {
    if (def.id === 'graphite') continue;
    const d = Math.abs(((lch.h - def.hue + 540) % 360) - 180);
    if (d < bestDistance) {
      best = def.id;
      bestDistance = d;
    }
  }
  return best;
}
