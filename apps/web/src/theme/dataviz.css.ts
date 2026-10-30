// Chart series colours (t106), one validated categorical order for every
// admin chart.
//
// Slots 1 and 2 are the dataviz reference palette's blue and orange, which pass
// the colour-blind separation, lightness, chroma and 3:1 contrast checks on both
// a light and a dark chart surface (validate_palette.js: worst CVD ΔE 24.7 light,
// 26.8 dark). Dark packs get the dark steps, light packs the light steps; the
// steps are chosen per appearance, not flipped. Text never wears these colours:
// a series is named by its legend line-key and its label, in text tokens.
//
// The document root starts on the dark pack before JavaScript runs (themes.css.ts),
// so the dark steps are the default and every light pack overrides them.

import { createVar, globalStyle } from '@vanilla-extract/css';
import { THEME_REGISTRY } from './registry.ts';
import type { ThemeName } from './contract.css.ts';

export const series = { 1: createVar(), 2: createVar() } as const;

const DARK = { 1: '#3987e5', 2: '#d95926' } as const;
const LIGHT = { 1: '#2a78d6', 2: '#eb6834' } as const;

const bind = (steps: typeof DARK | typeof LIGHT) => ({ [series[1]]: steps[1], [series[2]]: steps[2] });

globalStyle(':root', { vars: bind(DARK) });
for (const name of Object.keys(THEME_REGISTRY) as ThemeName[]) {
  if (THEME_REGISTRY[name].appearance === 'light') globalStyle(`:root[data-theme="${name}"]`, { vars: bind(LIGHT) });
}
