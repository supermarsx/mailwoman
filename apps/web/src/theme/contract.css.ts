// FROZEN theme token contract (plan §2.3, SPEC §17.1).
//
// `createThemeContract` fixes the CSS-custom-property NAMES here; each built-in
// theme (e4) is a `createTheme` class that assigns these vars, so runtime
// switching is just a `data-theme` attribute on `:root` (+ inline `--mw-accent`
// and `data-density` overrides). Component authors (e7/e8) and theme authors
// (e4) both reference `vars.*`; the SAME variables are injected into the
// sanitized message iframe and the print stylesheet so mail + chrome theme
// together. These names are FROZEN — changing one is a coordinator re-broadcast.
//
// zero-runtime: vanilla-extract compiles this to static CSS at build (via the
// vite plugin, wired in vite.config.ts). No arbitrary CSS injection (§7.4).

import { createThemeContract } from '@vanilla-extract/css';

export const vars = createThemeContract({
  color: {
    bg: null,
    bgAlt: null,
    bgSink: null,
    surface: null,
    border: null,
    text: null,
    textDim: null,
    accent: null,
    accentText: null,
    danger: null,
    success: null,
    warning: null,
    link: null,
    selection: null,
  },
  space: {
    0: null,
    1: null,
    2: null,
    3: null,
    4: null,
    5: null,
    6: null,
    7: null,
    8: null,
  },
  radius: {
    sm: null,
    md: null,
    lg: null,
    pill: null,
  },
  elevation: {
    0: null,
    1: null,
    2: null,
    3: null,
  },
  texture: {
    // `url(...)` for the Grove themes, `none` elsewhere; gated under
    // reduced-transparency / high-contrast by e4.
    grain: null,
    paper: null,
  },
  font: {
    ui: null,
    reading: null,
    mono: null,
  },
  fontSize: {
    base: null,
  },
  density: {
    // Row height; driven by the `data-density: compact|cozy|relaxed` attribute.
    rowH: null,
  },
});

/** The frozen `data-theme` values (plan §2.3). */
export type ThemeName =
  | 'light'
  | 'dark'
  | 'hc-light'
  | 'hc-dark'
  | 'amoled'
  | 'grove-light'
  | 'grove-dark';

/** The frozen `data-density` values (plan §2.3). */
export type Density = 'compact' | 'cozy' | 'relaxed';
