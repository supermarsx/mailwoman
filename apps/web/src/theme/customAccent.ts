import { accentContextOf, shadeAccent, toOklch } from './accentMath.ts';
import { themeEntry } from './registry.ts';
import { vars, type ThemeName } from './contract.css.ts';
import { LEGACY_COLOR_PROPERTIES } from './publicTokens.ts';

export function customAccentValue(value: unknown): string {
  return typeof value === 'string' && /^#[0-9a-f]{6}$/i.test(value) ? value.toLowerCase() : '';
}
/** Reuses the same surface/ink contrast constraints as the named accents. */
export function customAccentShade(value: string, theme: ThemeName) {
  const color = customAccentValue(value);
  const lch = color ? toOklch(color) : null;
  if (!lch) return null;
  return shadeAccent({id: 'blue', labelId: '', hue: lch.h, chroma: lch.c, lightness: lch.l}, accentContextOf(themeEntry(theme).palette));
}
export function applyCustomAccent(root: HTMLElement, value: string, theme: ThemeName): void {
  const shade = customAccentShade(value, theme);
  for (const [key, color] of [[vars.color.accent.slice(4, -1), shade?.accent], [vars.color.accentText.slice(4, -1), shade?.accentText], [LEGACY_COLOR_PROPERTIES.accent, shade?.accent], [LEGACY_COLOR_PROPERTIES.accentText, shade?.accentText]] as const) {
    if (color) root.style.setProperty(key, color);
    else root.style.removeProperty(key);
  }
}
