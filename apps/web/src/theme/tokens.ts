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
    /** Control and panel boundaries; held to 3:1 on every surface. */
    borderStrong: string;
    /** Marker colour: unread square, today, active indicators. */
    signal: string;
    /** Ink on a `signal` fill. */
    signalText: string;
    /** Focus outline colour. */
    focus: string;
    /** Translucent field laid over a background image behind text. */
    scrim: string;
    /** Text on `scrim`, readable whatever the image underneath. */
    scrimText: string;
  };
  space: Record<0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8, string>;
  radius: { sm: string; md: string; lg: string; pill: string };
  elevation: Record<0 | 1 | 2 | 3, string>;
  border: { hair: string; rule: string; heavy: string };
  texture: { grain: string; paper: string };
  font: { ui: string; reading: string; mono: string };
  fontSize: { base: string };
  weight: { regular: string; medium: string; bold: string; heavy: string };
  tracking: { none: string; label: string };
  lineHeight: { ui: string };
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

/**
 * What each pack declares by hand. `signal`, `signalText`, `scrim` and
 * `scrimText` follow one rule for every pack and are filled in by `themeOf`.
 */
type PackPalette = Omit<Palette, 'signal' | 'signalText' | 'scrim' | 'scrimText'>;

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

// Square corners everywhere (t32). `pill` stays for true circles only:
// avatars, status dots, colour swatches.
const radius: ThemeTokens['radius'] = { sm: '0', md: '0', lg: '0', pill: '999px' };

// Rule widths: `hair` for dividers, `rule` for control and panel boundaries,
// `heavy` for section and top-bar rules. A component that thickens a border
// takes the same number of pixels off its padding, so no box grows.
const border: ThemeTokens['border'] = { hair: '1px', rule: '2px', heavy: '3px' };

/**
 * Hard offset shadows with no blur, in a near-black that depends on the pack's
 * appearance. A floating surface also carries a `border.rule` in
 * `borderStrong`, so it stays separated where shadows are not drawn (forced
 * colours) or barely show (a black offset on a near-black page).
 */
function elevationOf(appearance: Appearance): ThemeTokens['elevation'] {
  const ink = appearance === 'light' ? '#111111' : '#000000';
  return {
    0: 'none',
    1: `2px 2px 0 0 ${ink}`,
    2: `4px 4px 0 0 ${ink}`,
    3: `6px 6px 0 0 ${ink}`,
  };
}

// The UI face is Jost, bundled with the web build (styles/typeface.css). The
// reading and mono families are the operator's `mailwoman fonts pull` overlay
// (fonts/manifest.json + styles/fonts.css.ts) with system fallbacks, so nothing
// breaks when those files are absent. font-src 'self' — no remote URL ever.
const font: ThemeTokens['font'] = {
  ui: '"Jost", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
  reading: '"Newsreader", Georgia, "Times New Roman", serif',
  mono: '"JetBrains Mono", ui-monospace, "Cascadia Code", Consolas, monospace',
};

const fontSize: ThemeTokens['fontSize'] = { base: '14px' };

// Headings use `heavy`, small labels `bold`.
const weight: ThemeTokens['weight'] = { regular: '400', medium: '500', bold: '700', heavy: '800' };

// `label` tracking is for small uppercase labels only; themes.css.ts sets it to
// 0 for Arabic, Japanese and Chinese, where letter-spacing breaks the script.
const tracking: ThemeTokens['tracking'] = { none: '0', label: '0.06em' };

// Pinned because Jost's own `normal` line height (about 1.45) is taller than
// the system faces it replaces; 1.2 keeps every inherited line box at or under
// its previous height.
const lineHeight: ThemeTokens['lineHeight'] = { ui: '1.2' };

// Accent is wrapped so an inline `--mw-accent` on :root overrides any theme's
// default without touching the contract (plan §2.3 accent override).
function accent(base: string): string {
  return `var(--mw-accent, ${base})`;
}

/**
 * Brand colour slots: custom properties a deployment's branding can set on
 * `:root` at runtime to replace a pack's colour without a rebuild. Unset, every
 * pack paints its own value. The accent slot is `--mw-accent`, the property the
 * user's accent choice already uses, so code that applies a brand accent has to
 * leave a user's own choice in place. Brand values are not checked here: the
 * code that applies them has to measure them against the pack's surfaces and
 * `accentText` with `contrast.ts` first.
 */
export const BRAND_SLOTS = {
  accent: '--mw-accent',
  danger: '--mw-brand-danger',
  signal: '--mw-brand-signal',
  signalText: '--mw-brand-signal-text',
} as const;

function brand(slot: Exclude<keyof typeof BRAND_SLOTS, 'accent'>, base: string): string {
  return `var(${BRAND_SLOTS[slot]}, ${base})`;
}

// Bauhaus yellow, the same in every pack. It is below 3:1 on light pages, so a
// light pack draws a signal shape filled behind `signalText` or outlined in
// `borderStrong`, never as a bare yellow mark.
const SIGNAL = '#f5c400';
const SIGNAL_TEXT = '#111111';

// Opacity of the scrim over an image. At 0.8 the pack's text clears 4.5:1 on
// the scrim over a pure white and over a pure black image, which bounds every
// image in between (contrast.test.ts measures both ends for every pack).
const SCRIM_ALPHA = 0.8;

function scrimOf(bg: string): string {
  const h = bg.replace('#', '');
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
  return `rgba(${r}, ${g}, ${b}, ${SCRIM_ALPHA})`;
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
//
// `dark` is the default pack (t32): a near-black page, off-white text, Bauhaus
// blue for actions and selection, red for alerts, yellow for markers and focus.
// `light` follows the same rules on an off-white page. Every value is measured
// by contrast.test.ts; the minimum ratios are noted where they are close.
const dark: PackPalette = {
  bg: '#111111',
  bgAlt: '#1a1a1a',
  bgSink: '#0a0a0a',
  surface: '#1a1a1a',
  // Decorative dividers only (advisory tier); controls use borderStrong.
  border: '#555555',
  borderStrong: '#bdb8ad', // ≥ 8.8:1 on every surface
  text: '#f2efe6',
  textDim: '#a39e94', // ≥ 6.5:1 on every surface
  accent: accent('#4d7cff'), // ≥ 4.6:1 on every surface
  // Dark ink on filled controls: an accent bright enough to read on a
  // near-black page is too bright to carry white text. The same token labels
  // the success/danger fills.
  accentText: '#0b0b0b', // 5.3:1 on accent
  danger: brand('danger', '#ff5a4a'),
  success: '#4cd07d',
  warning: '#ffb000',
  link: '#8fb0ff',
  selection: '#1d3a8f',
  focus: '#f5c400',
};

const light: PackPalette = {
  bg: '#f4f1ea',
  bgAlt: '#ebe7dd',
  bgSink: '#e2ddd1',
  surface: '#fbf9f4',
  border: '#b8b2a6',
  borderStrong: '#111111',
  text: '#111111',
  textDim: '#4a4740', // ≥ 6.8:1 on every surface
  accent: accent('#1d3fbf'),
  accentText: '#ffffff',
  danger: brand('danger', '#b01a10'), // ≥ 5.1:1 on every surface
  success: '#1e6b35', // ≥ 4.8:1 on every surface
  warning: '#7a4f00',
  link: '#1d3fbf',
  selection: '#f5d64a',
  focus: '#111111',
};

// High-contrast: WCAG AAA-leaning, pure black/white frame, no textures. Control
// boundaries are the text colour; the focus outline is the accent.
const hcLight: PackPalette = {
  bg: '#ffffff',
  bgAlt: '#ffffff',
  bgSink: '#ffffff',
  surface: '#ffffff',
  border: '#000000',
  borderStrong: '#000000',
  text: '#000000',
  textDim: '#1a1a1a',
  accent: accent('#0b3d91'),
  accentText: '#ffffff',
  danger: brand('danger', '#8b0000'),
  success: '#0a5d00',
  warning: '#6b4200',
  link: '#0b3d91',
  selection: '#ffd54a',
  focus: accent('#0b3d91'),
};

const hcDark: PackPalette = {
  bg: '#000000',
  bgAlt: '#000000',
  bgSink: '#000000',
  surface: '#000000',
  border: '#ffffff',
  borderStrong: '#ffffff',
  text: '#ffffff',
  textDim: '#eaeaea',
  accent: accent('#7db3ff'),
  accentText: '#000000',
  danger: brand('danger', '#ff6b6b'),
  success: '#6dffa8',
  warning: '#ffd35c',
  link: '#9cc3ff',
  selection: '#004a8f',
  focus: accent('#7db3ff'),
};

// AMOLED keeps its black page and takes the Dark pack's control boundary and
// focus colours.
const amoled: PackPalette = {
  bg: '#000000',
  bgAlt: '#0a0a0c',
  bgSink: '#000000',
  surface: '#0d0f12',
  border: '#23262d',
  borderStrong: '#bdb8ad',
  text: '#e6e8eb',
  textDim: '#8b929c',
  accent: accent('#3b82f6'),
  // Same reasoning as the Dark pack: dark ink on filled controls.
  accentText: '#0b1220',
  danger: brand('danger', '#f87171'),
  success: '#4ade80',
  warning: '#fbbf24',
  link: '#60a5fa',
  selection: '#1e3a63',
  focus: '#f5c400',
};

// The coloured packs keep their palettes. Each control boundary
// (`borderStrong`) is a mix of the pack's own divider and muted-text colours
// that clears 3:1 on its four surfaces; the focus outline is the accent.

// Grove: warm, woody. Paper-cream light + walnut dark, mossy-green accent.
const groveLight: PackPalette = {
  bg: '#f5efe4',
  bgAlt: '#efe6d6',
  bgSink: '#e7dcc8',
  surface: '#fbf6ec',
  border: '#cbb894',
  borderStrong: '#827359',
  text: '#3a2f24',
  // Grove's light surfaces are tinted cream, so muted/status colours need to be
  // darker than they would on white to clear 4.5:1 on bgSink #e7dcc8.
  textDim: '#63563f',
  accent: accent('#55703a'),
  accentText: '#ffffff',
  danger: brand('danger', '#953823'),
  success: '#3c5c2c',
  warning: '#75500f',
  link: '#6b4f2a',
  selection: '#dcd0af',
  focus: accent('#55703a'),
};

const groveDark: PackPalette = {
  bg: '#211c16',
  bgAlt: '#2a2319',
  bgSink: '#191510',
  surface: '#2f2820',
  border: '#4a3f30',
  borderStrong: '#887861',
  text: '#ece2d0',
  textDim: '#a9977c',
  accent: accent('#9cb87a'),
  accentText: '#1a1509',
  danger: brand('danger', '#e08a6f'),
  success: '#9ccf7a',
  warning: '#e0b25f',
  link: '#cdae7e',
  selection: '#4a3d26',
  focus: accent('#9cb87a'),
};

// Slate: cool blue-grey neutrals. Light side sits on a tinted grey page.
const slateLight: PackPalette = {
  bg: '#eef1f5',
  bgAlt: '#e4e8ee',
  bgSink: '#d9dee6',
  surface: '#f8fafc',
  border: '#9aa5b4',
  borderStrong: '#6c7685',
  text: '#1b2230',
  textDim: '#4d5666',
  accent: accent('#33547c'),
  accentText: '#ffffff',
  danger: brand('danger', '#9e1c1c'),
  success: '#1a6135',
  warning: '#7d500c',
  link: '#284a76',
  selection: '#c3d4ea',
  focus: accent('#33547c'),
};

const slateDark: PackPalette = {
  bg: '#151a21',
  bgAlt: '#1c222b',
  bgSink: '#10141a',
  surface: '#1f2630',
  border: '#414c5b',
  borderStrong: '#6e7988',
  text: '#e3e8ef',
  textDim: '#9aa5b5',
  accent: accent('#7aa2d6'),
  accentText: '#0e1218',
  danger: brand('danger', '#f08a8a'),
  success: '#6fd08c',
  warning: '#e9b45c',
  link: '#8ab4e8',
  selection: '#2b3d55',
  focus: accent('#7aa2d6'),
};

// Ocean: blue-green ground, teal accent. Cooler and more saturated than Slate.
const oceanLight: PackPalette = {
  bg: '#f2f8f9',
  bgAlt: '#e6f0f2',
  bgSink: '#d8e7ea',
  surface: '#fbfdfd',
  border: '#8fabb1',
  borderStrong: '#637d84',
  text: '#0f2b30',
  textDim: '#3f585f',
  accent: accent('#0c5a66'),
  accentText: '#ffffff',
  danger: brand('danger', '#9c2323'),
  success: '#15603c',
  warning: '#7a4a0c',
  link: '#0a5177',
  selection: '#bfe0e6',
  focus: accent('#0c5a66'),
};

const oceanDark: PackPalette = {
  bg: '#0d171b',
  bgAlt: '#122127',
  bgSink: '#0a1215',
  surface: '#15252c',
  border: '#37525c',
  borderStrong: '#607a83',
  text: '#dceaee',
  textDim: '#93aab2',
  accent: accent('#4fc3d4'),
  accentText: '#04191d',
  danger: brand('danger', '#f38b8b'),
  success: '#68d3a0',
  warning: '#e8b45f',
  link: '#6fc9e0',
  selection: '#1e3d49',
  focus: accent('#4fc3d4'),
};

// Plum: muted purple ground, violet accent.
const plumLight: PackPalette = {
  bg: '#f7f4fa',
  bgAlt: '#efe9f4',
  bgSink: '#e5dcee',
  surface: '#fdfbfe',
  border: '#a698b6',
  borderStrong: '#7e708d',
  text: '#241a2e',
  textDim: '#554764',
  accent: accent('#5d3690'),
  accentText: '#ffffff',
  danger: brand('danger', '#9d2033'),
  success: '#1e6042',
  warning: '#7b4a0c',
  link: '#5d3690',
  selection: '#dccdec',
  focus: accent('#5d3690'),
};

const plumDark: PackPalette = {
  bg: '#17131d',
  bgAlt: '#1e1826',
  bgSink: '#110e16',
  surface: '#231c2c',
  border: '#4b3f5c',
  borderStrong: '#7a6d8b',
  text: '#e9e3f0',
  textDim: '#a89ab9',
  accent: accent('#b28ce0'),
  accentText: '#180f22',
  danger: brand('danger', '#f08a9b'),
  success: '#72d3a3',
  warning: '#e6b45f',
  link: '#c0a2e8',
  selection: '#3a2c4d',
  focus: accent('#b28ce0'),
};

// Shared a11y token values. Structural constants (touch target, motion
// durations) are identical across themes; the focus ring is derived per theme
// from its `focus` colour. `focusRingColor` + `focusRingWidth` drive the global
// `:focus-visible` outline (themes.css.ts), which forced-colours mode keeps;
// `focusRing` is the box-shadow form for components that draw their own ring:
// a bg-coloured spacer, then the focus colour. Motion durations are switched to
// ~0 under prefers-reduced-motion by themes.css.ts (not here).
function a11yOf(color: Palette): ThemeTokens['a11y'] {
  return {
    focusRing: `0 0 0 2px ${color.bg}, 0 0 0 5px ${color.focus}`,
    focusRingWidth: '3px',
    focusRingColor: color.focus,
    touchTarget: '24px',
    motionDuration: '150ms',
    motionDurationSlow: '240ms',
  };
}

/** Assemble a full token set from a palette, its appearance and a texture pack. */
function themeOf(
  pack: PackPalette,
  appearance: Appearance,
  texture: ThemeTokens['texture'],
): ThemeTokens {
  const color: Palette = {
    ...pack,
    signal: brand('signal', SIGNAL),
    signalText: brand('signalText', SIGNAL_TEXT),
    scrim: scrimOf(pack.bg),
    scrimText: pack.text,
  };
  return {
    color,
    space,
    radius,
    elevation: elevationOf(appearance),
    border,
    texture,
    font,
    fontSize,
    weight,
    tracking,
    lineHeight,
    density: { rowH: '56px' },
    a11y: a11yOf(color),
  };
}

const GROVE_TEXTURE = { grain: GROVE_GRAIN, paper: GROVE_PAPER } as const;

/** Every built-in theme, keyed by its frozen `data-theme` value. */
export const THEMES: Record<ThemeName, ThemeTokens> = {
  light: themeOf(light, 'light', NO_TEXTURE),
  dark: themeOf(dark, 'dark', NO_TEXTURE),
  'hc-light': themeOf(hcLight, 'light', NO_TEXTURE),
  'hc-dark': themeOf(hcDark, 'dark', NO_TEXTURE),
  amoled: themeOf(amoled, 'dark', NO_TEXTURE),
  'grove-light': themeOf(groveLight, 'light', GROVE_TEXTURE),
  'grove-dark': themeOf(groveDark, 'dark', GROVE_TEXTURE),
  'slate-light': themeOf(slateLight, 'light', NO_TEXTURE),
  'slate-dark': themeOf(slateDark, 'dark', NO_TEXTURE),
  'ocean-light': themeOf(oceanLight, 'light', NO_TEXTURE),
  'ocean-dark': themeOf(oceanDark, 'dark', NO_TEXTURE),
  'plum-light': themeOf(plumLight, 'light', NO_TEXTURE),
  'plum-dark': themeOf(plumDark, 'dark', NO_TEXTURE),
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
 * Every built-in theme in gallery order: the default pair first (Dark, the
 * default, before Light), then the coloured packs, then the special-purpose
 * ones (AMOLED, high contrast).
 */
export const THEME_META: readonly ThemeMeta[] = [
  {
    id: 'dark',
    label: 'Dark',
    description: 'Near-black page, off-white text, blue actions, red alerts, yellow markers.',
    family: 'core',
    appearance: 'dark',
  },
  {
    id: 'light',
    label: 'Light',
    description: 'Off-white page, black text, blue actions, red alerts, yellow markers.',
    family: 'core',
    appearance: 'light',
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
    description: 'Black on white with black borders and no tinted surfaces.',
    family: 'contrast',
    appearance: 'light',
  },
  {
    id: 'hc-dark',
    label: 'High contrast (dark)',
    description: 'White on black with white borders and no tinted surfaces.',
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
