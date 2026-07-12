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
// by media queries in themes.css.ts.
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
  textDim: '#6b7280',
  accent: accent('#2563eb'),
  accentText: '#ffffff',
  danger: '#b91c1c',
  success: '#15803d',
  warning: '#b45309',
  link: '#2563eb',
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
  accentText: '#ffffff',
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
  accentText: '#ffffff',
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
  textDim: '#7a6a53',
  accent: accent('#6d8a4e'),
  accentText: '#ffffff',
  danger: '#a3402c',
  success: '#4f7a3a',
  warning: '#9a6a1f',
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

/** Assemble a full token set from a palette + texture pack. */
function themeOf(color: Palette, texture: ThemeTokens['texture']): ThemeTokens {
  return { color, space, radius, elevation, texture, font, fontSize, density: { rowH: '56px' } };
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
};

/** Ordered theme list with human labels, for the Settings picker. */
export const THEME_OPTIONS: ReadonlyArray<{ value: ThemeName; label: string }> = [
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
  { value: 'hc-light', label: 'High contrast (light)' },
  { value: 'hc-dark', label: 'High contrast (dark)' },
  { value: 'amoled', label: 'AMOLED' },
  { value: 'grove-light', label: 'Grove Light' },
  { value: 'grove-dark', label: 'Grove Dark' },
];

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
