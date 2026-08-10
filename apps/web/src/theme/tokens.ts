// Design-token VALUES for every built-in theme (plan §3 e4, §2.3).
//
// Single source of truth for the palettes: `themes.css.ts` binds these to the
// frozen `contract.css.ts` via `createGlobalTheme` (parent document, hashed
// vars), and `themeCssVars.ts` emits the same values as a stable-named
// `--mw-*` block for the sandboxed message iframe (a separate opaque-origin
// document that can NOT inherit the parent's CSS vars). Keep both in sync by
// deriving from here — never hard-code a colour twice.

import type { ThemeName } from './contract.css.ts';

/** Full token set the contract expects — every leaf is a CSS value string. */
export interface ThemeTokens {
  color: {
    bg: string;
    bgAlt: string;
    bgSink: string;
    surface: string;
    border: string;
    text: string;
    textDim: string;
    accent: string;
    accentText: string;
    danger: string;
    success: string;
    warning: string;
    link: string;
    selection: string;
  };
  space: Record<0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8, string>;
  radius: { sm: string; md: string; lg: string; pill: string };
  elevation: Record<0 | 1 | 2 | 3, string>;
  texture: { grain: string; paper: string };
  font: { ui: string; reading: string; mono: string };
  fontSize: { base: string };
  density: { rowH: string };
  a11y: {
    focusRing: string;
    focusRingWidth: string;
    focusRingColor: string;
    touchTarget: string;
    motionDuration: string;
    motionDurationSlow: string;
  };
}

/** Just the per-theme colour palette; structure is shared across themes. */
type Palette = ThemeTokens['color'];

// ── Structural tokens (identical across every theme) ─────────────────────────
const space: ThemeTokens['space'] = {
  0: '0',
  1: '2px',
  2: '4px',
  3: '8px',
  4: '12px',
  5: '16px',
  6: '24px',
  7: '32px',
  8: '48px',
};

const radius: ThemeTokens['radius'] = { sm: '4px', md: '6px', lg: '10px', pill: '999px' };

const elevation: ThemeTokens['elevation'] = {
  0: 'none',
  1: '0 1px 2px rgba(0, 0, 0, 0.08)',
  2: '0 4px 12px rgba(0, 0, 0, 0.12)',
  3: '0 12px 32px rgba(0, 0, 0, 0.24)',
};

// Self-hosted families (see fonts/manifest.json + styles/fonts.css.ts) with
// system fallbacks so nothing breaks before `mailwoman fonts pull` populates
// the binaries. font-src 'self' — no remote URL ever.
const font: ThemeTokens['font'] = {
  ui: '"Inter", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
  reading: '"Newsreader", Georgia, "Times New Roman", serif',
  mono: '"JetBrains Mono", ui-monospace, "Cascadia Code", Consolas, monospace',
};

const fontSize: ThemeTokens['fontSize'] = { base: '14px' };

// Accent is wrapped so an inline `--mw-accent` on :root overrides any theme's
// default without touching the contract (plan §2.3 accent override).
function accent(base: string): string {
  return `var(--mw-accent, ${base})`;
}

// Grove textures are served same-origin from /themes/*.svg (img-src 'self');
// `none` everywhere else. Gated off under reduced-transparency / HC / data-saver
// by media queries in themes.css.ts. These two constants are the ONLY texture
// URLs in the theme layer.
//
// SUB-PATH HOSTING (t20 B4) — leave the leading `/` ALONE. It was expected that
// these would need a runtime base prefix; they do not, and adding one would break
// them. `public/themes/` is a public-dir asset and `vite.config.ts` now sets
// `base: './'`, so Vite resolves these root-absolute references at build time and
// emits them RELATIVE to the stylesheet that carries them —
// `url(../themes/grove-grain.svg)` in `dist/assets/index-*.css`. Served from
// `/mail/assets/index-*.css` that resolves to `/mail/themes/grove-grain.svg`;
// served from the root it resolves to `/themes/grove-grain.svg`, exactly as
// before. Verified against real build output, not assumed.
//
// The rewrite is keyed on the root-absolute form: writing `./themes/…` or
// interpolating a runtime prefix here would defeat it and hard-code one deploy
// path into the bundle.
const GROVE_GRAIN = "url('/themes/grove-grain.svg')";
const GROVE_PAPER = "url('/themes/grove-paper.svg')";
const NO_TEXTURE = { grain: 'none', paper: 'none' } as const;

// ── Per-theme palettes ───────────────────────────────────────────────────────
const light: Palette = {
  bg: '#ffffff',
  bgAlt: '#f4f5f7',
  bgSink: '#eceef1',
  surface: '#ffffff',
  border: '#d8dbe0',
  text: '#1c1e21',
  // WCAG 1.4.3 — muted text ≥ 4.5:1 on every light surface it lands on
  // (5.27:1 on bgSink #eceef1, 5.62:1 on bgAlt #f4f5f7, 6.13:1 on #ffffff).
  // #6b7280 fell to 4.16–4.43:1 on the tinted surfaces.
  textDim: '#5b6270',
  accent: accent('#2563eb'),
  accentText: '#ffffff',
  danger: '#b91c1c',
  // Status/link colours are held to 4.5:1 on the DARKEST light surface
  // (bgSink #eceef1), not just on white — #15803d/#b45309/#2563eb sat at
  // 4.31-4.45:1 there. See contrast.ts CONTRAST_PAIRS.
  success: '#146c33',
  warning: '#9a4708',
  link: '#1d4ed8',
  selection: '#cfe0ff',
};

const dark: Palette = {
  bg: '#16181d',
  bgAlt: '#1e2127',
  bgSink: '#12141a',
  surface: '#1e2127',
  border: '#2c3038',
  text: '#e6e8eb',
  textDim: '#9aa1ab',
  accent: accent('#3b82f6'),
  // Dark packs put DARK ink on their filled controls: the accent has to stay
  // bright enough to read as a state indicator against a near-black page
  // (≥3:1), which leaves it too bright to carry white text (#ffffff on #3b82f6
  // is 3.68:1). The same token labels the success/danger badges, where white
  // was far worse (1.9:1 on #4ade80).
  accentText: '#0b1220',
  danger: '#f87171',
  success: '#4ade80',
  warning: '#fbbf24',
  link: '#60a5fa',
  selection: '#24406b',
};

// High-contrast: WCAG AAA-leaning, pure black/white frame, no textures.
const hcLight: Palette = {
  bg: '#ffffff',
  bgAlt: '#ffffff',
  bgSink: '#ffffff',
  surface: '#ffffff',
  border: '#000000',
  text: '#000000',
  textDim: '#1a1a1a',
  accent: accent('#0b3d91'),
  accentText: '#ffffff',
  danger: '#8b0000',
  success: '#0a5d00',
  warning: '#6b4200',
  link: '#0b3d91',
  selection: '#ffd54a',
};

const hcDark: Palette = {
  bg: '#000000',
  bgAlt: '#000000',
  bgSink: '#000000',
  surface: '#000000',
  border: '#ffffff',
  text: '#ffffff',
  textDim: '#eaeaea',
  accent: accent('#7db3ff'),
  accentText: '#000000',
  danger: '#ff6b6b',
  success: '#6dffa8',
  warning: '#ffd35c',
  link: '#9cc3ff',
  selection: '#004a8f',
};

const amoled: Palette = {
  bg: '#000000',
  bgAlt: '#0a0a0c',
  bgSink: '#000000',
  surface: '#0d0f12',
  border: '#23262d',
  text: '#e6e8eb',
  textDim: '#8b929c',
  accent: accent('#3b82f6'),
  // Same reasoning as the Dark pack: dark ink on filled controls.
  accentText: '#0b1220',
  danger: '#f87171',
  success: '#4ade80',
  warning: '#fbbf24',
  link: '#60a5fa',
  selection: '#1e3a63',
};

// Grove: warm, woody. Paper-cream light + walnut dark, mossy-green accent.
const groveLight: Palette = {
  bg: '#f5efe4',
  bgAlt: '#efe6d6',
  bgSink: '#e7dcc8',
  surface: '#fbf6ec',
  border: '#cbb894',
  text: '#3a2f24',
  // Grove's light surfaces are tinted cream, so muted/status colours need to be
  // darker than they would on white to clear 4.5:1 on bgSink #e7dcc8.
  textDim: '#63563f',
  accent: accent('#55703a'),
  accentText: '#ffffff',
  danger: '#953823',
  success: '#3c5c2c',
  warning: '#75500f',
  link: '#6b4f2a',
  selection: '#dcd0af',
};

const groveDark: Palette = {
  bg: '#211c16',
  bgAlt: '#2a2319',
  bgSink: '#191510',
  surface: '#2f2820',
  border: '#4a3f30',
  text: '#ece2d0',
  textDim: '#a9977c',
  accent: accent('#9cb87a'),
  accentText: '#1a1509',
  danger: '#e08a6f',
  success: '#9ccf7a',
  warning: '#e0b25f',
  link: '#cdae7e',
  selection: '#4a3d26',
};

// Slate: cool blue-grey neutrals, softer than the default white/near-black
// frame. Light side sits on a tinted grey page rather than pure white.
const slateLight: Palette = {
  bg: '#eef1f5',
  bgAlt: '#e4e8ee',
  bgSink: '#d9dee6',
  surface: '#f8fafc',
  border: '#9aa5b4',
  text: '#1b2230',
  textDim: '#4d5666',
  accent: accent('#33547c'),
  accentText: '#ffffff',
  danger: '#9e1c1c',
  success: '#1a6135',
  warning: '#7d500c',
  link: '#284a76',
  selection: '#c3d4ea',
};

const slateDark: Palette = {
  bg: '#151a21',
  bgAlt: '#1c222b',
  bgSink: '#10141a',
  surface: '#1f2630',
  border: '#414c5b',
  text: '#e3e8ef',
  textDim: '#9aa5b5',
  accent: accent('#7aa2d6'),
  accentText: '#0e1218',
  danger: '#f08a8a',
  success: '#6fd08c',
  warning: '#e9b45c',
  link: '#8ab4e8',
  selection: '#2b3d55',
};

// Ocean: blue-green ground, teal accent. Cooler and more saturated than Slate.
const oceanLight: Palette = {
  bg: '#f2f8f9',
  bgAlt: '#e6f0f2',
  bgSink: '#d8e7ea',
  surface: '#fbfdfd',
  border: '#8fabb1',
  text: '#0f2b30',
  textDim: '#3f585f',
  accent: accent('#0c5a66'),
  accentText: '#ffffff',
  danger: '#9c2323',
  success: '#15603c',
  warning: '#7a4a0c',
  link: '#0a5177',
  selection: '#bfe0e6',
};

const oceanDark: Palette = {
  bg: '#0d171b',
  bgAlt: '#122127',
  bgSink: '#0a1215',
  surface: '#15252c',
  border: '#37525c',
  text: '#dceaee',
  textDim: '#93aab2',
  accent: accent('#4fc3d4'),
  accentText: '#04191d',
  danger: '#f38b8b',
  success: '#68d3a0',
  warning: '#e8b45f',
  link: '#6fc9e0',
  selection: '#1e3d49',
};

// Plum: muted purple ground, violet accent.
const plumLight: Palette = {
  bg: '#f7f4fa',
  bgAlt: '#efe9f4',
  bgSink: '#e5dcee',
  surface: '#fdfbfe',
  border: '#a698b6',
  text: '#241a2e',
  textDim: '#554764',
  accent: accent('#5d3690'),
  accentText: '#ffffff',
  danger: '#9d2033',
  success: '#1e6042',
  warning: '#7b4a0c',
  link: '#5d3690',
  selection: '#dccdec',
};

const plumDark: Palette = {
  bg: '#17131d',
  bgAlt: '#1e1826',
  bgSink: '#110e16',
  surface: '#231c2c',
  border: '#4b3f5c',
  text: '#e9e3f0',
  textDim: '#a89ab9',
  accent: accent('#b28ce0'),
  accentText: '#180f22',
  danger: '#f08a9b',
  success: '#72d3a3',
  warning: '#e6b45f',
  link: '#c0a2e8',
  selection: '#3a2c4d',
};

// Shared a11y token values. Structural constants (touch target, motion
// durations) are identical across themes; the focus ring is derived per-theme so
// it stays visible against that theme's own background + uses its accent. The
// ring is a two-stop box-shadow: a bg-coloured spacer, then the accent ring, so
// it reads on any surface. Motion durations are switched to ~0 under
// prefers-reduced-motion by themes.css.ts (not here).
function a11yOf(color: Palette): ThemeTokens['a11y'] {
  return {
    focusRing: `0 0 0 2px ${color.bg}, 0 0 0 4px ${color.accent}`,
    focusRingWidth: '2px',
    focusRingColor: color.accent,
    touchTarget: '24px',
    motionDuration: '150ms',
    motionDurationSlow: '240ms',
  };
}

/** Assemble a full token set from a palette + texture pack. */
function themeOf(color: Palette, texture: ThemeTokens['texture']): ThemeTokens {
  return {
    color,
    space,
    radius,
    elevation,
    texture,
    font,
    fontSize,
    density: { rowH: '56px' },
    a11y: a11yOf(color),
  };
}

/** Every built-in theme, keyed by its frozen `data-theme` value. */
export const THEMES: Record<ThemeName, ThemeTokens> = {
  light: themeOf(light, NO_TEXTURE),
  dark: themeOf(dark, NO_TEXTURE),
  'hc-light': themeOf(hcLight, NO_TEXTURE),
  'hc-dark': themeOf(hcDark, NO_TEXTURE),
  amoled: themeOf(amoled, NO_TEXTURE),
  'grove-light': themeOf(groveLight, { grain: GROVE_GRAIN, paper: GROVE_PAPER }),
  'grove-dark': themeOf(groveDark, { grain: GROVE_GRAIN, paper: GROVE_PAPER }),
  'slate-light': themeOf(slateLight, NO_TEXTURE),
  'slate-dark': themeOf(slateDark, NO_TEXTURE),
  'ocean-light': themeOf(oceanLight, NO_TEXTURE),
  'ocean-dark': themeOf(oceanDark, NO_TEXTURE),
  'plum-light': themeOf(plumLight, NO_TEXTURE),
  'plum-dark': themeOf(plumDark, NO_TEXTURE),
};

// ── Theme metadata (presentation layer) ──────────────────────────────────────
// Lives here rather than in `registry.ts` so `tokens.ts` stays dependency-free
// and `registry.ts` can compose metadata × palettes without an import cycle.
// `registry.ts` is what UI code should consume; this array is the raw ordering.

/** A theme family — a pack that ships a light and/or dark variant. */
export type ThemeFamily = 'core' | 'slate' | 'ocean' | 'plum' | 'grove' | 'amoled' | 'contrast';

/** Whether a pack paints a light or a dark page. */
export type Appearance = 'light' | 'dark';

export interface ThemeMeta {
  readonly id: ThemeName;
  /** User-visible name. Descriptive, not promotional. */
  readonly label: string;
  /** One honest line describing what the pack looks like. */
  readonly description: string;
  readonly family: ThemeFamily;
  readonly appearance: Appearance;
}

/**
 * Every built-in theme in gallery order: the neutral pair first, then the
 * coloured packs, then the special-purpose ones (AMOLED, high contrast).
 */
export const THEME_META: readonly ThemeMeta[] = [
  {
    id: 'light',
    label: 'Light',
    description: 'Neutral light theme on white with a blue accent.',
    family: 'core',
    appearance: 'light',
  },
  {
    id: 'dark',
    label: 'Dark',
    description: 'Neutral dark theme on charcoal with a blue accent.',
    family: 'core',
    appearance: 'dark',
  },
  {
    id: 'slate-light',
    label: 'Slate Light',
    description: 'Cool blue-grey surfaces on a tinted page, muted navy accent.',
    family: 'slate',
    appearance: 'light',
  },
  {
    id: 'slate-dark',
    label: 'Slate Dark',
    description: 'Cool blue-grey dark theme with a muted blue accent.',
    family: 'slate',
    appearance: 'dark',
  },
  {
    id: 'ocean-light',
    label: 'Ocean Light',
    description: 'Blue-green light theme with a deep teal accent.',
    family: 'ocean',
    appearance: 'light',
  },
  {
    id: 'ocean-dark',
    label: 'Ocean Dark',
    description: 'Blue-green dark theme with a cyan accent.',
    family: 'ocean',
    appearance: 'dark',
  },
  {
    id: 'plum-light',
    label: 'Plum Light',
    description: 'Muted purple light theme with a violet accent.',
    family: 'plum',
    appearance: 'light',
  },
  {
    id: 'plum-dark',
    label: 'Plum Dark',
    description: 'Muted purple dark theme with a violet accent.',
    family: 'plum',
    appearance: 'dark',
  },
  {
    id: 'grove-light',
    label: 'Grove Light',
    description: 'Warm paper tones with wood-grain texture and a moss accent.',
    family: 'grove',
    appearance: 'light',
  },
  {
    id: 'grove-dark',
    label: 'Grove Dark',
    description: 'Warm walnut tones with wood-grain texture and a moss accent.',
    family: 'grove',
    appearance: 'dark',
  },
  {
    id: 'amoled',
    label: 'AMOLED',
    description: 'Pure-black dark theme; unlit pixels on OLED screens.',
    family: 'amoled',
    appearance: 'dark',
  },
  {
    id: 'hc-light',
    label: 'High contrast (light)',
    description: 'Black on white with heavy borders and a thicker focus ring.',
    family: 'contrast',
    appearance: 'light',
  },
  {
    id: 'hc-dark',
    label: 'High contrast (dark)',
    description: 'White on black with heavy borders and a thicker focus ring.',
    family: 'contrast',
    appearance: 'dark',
  },
];

/**
 * Ordered theme list with human labels, for the Settings/ribbon picker.
 * Derived from `THEME_META` so ordering and labels have one source.
 */
export const THEME_OPTIONS: ReadonlyArray<{ value: ThemeName; label: string }> = THEME_META.map(
  (m) => ({ value: m.id, label: m.label }),
);

/** Accent presets offered in Settings (empty string = the theme default). */
export const ACCENT_PRESETS: ReadonlyArray<{ value: string; label: string }> = [
  { value: '', label: 'Theme default' },
  { value: '#2563eb', label: 'Blue' },
  { value: '#7c3aed', label: 'Violet' },
  { value: '#0d9488', label: 'Teal' },
  { value: '#6d8a4e', label: 'Moss' },
  { value: '#c2410c', label: 'Amber' },
  { value: '#be123c', label: 'Rose' },
];
