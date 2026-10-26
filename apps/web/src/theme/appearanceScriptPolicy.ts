import { FONT_STACKS } from './appearance.ts';
import { ACCENTS, accentContextOf, isAccentId, shadeAccent } from './accentMath.ts';
import { customAccentShade } from './customAccent.ts';
import { themeEntry } from './registry.ts';
import { vars, type ThemeName } from './contract.css.ts';
import { flattenTokens, tokenProperties, LEGACY_COLOR_PROPERTIES } from './publicTokens.ts';
import type { AppearanceUpdate } from './appearanceSandbox.ts';

/** Emits only application-owned properties and validated values, never source text. */
export function appearanceScriptProperties(update: AppearanceUpdate, locked: readonly string[], currentTheme: ThemeName): Array<[string, string]> {
  const result: Array<[string, string]> = [];
  const theme = !locked.includes('mode') && (update.mode === 'light' || update.mode === 'dark') ? update.mode : currentTheme;
  if (update.mode && !locked.includes('mode')) {
    if (update.mode !== 'light' && update.mode !== 'dark') throw new Error('Unsupported appearance mode.');
    const colors = Object.fromEntries(Object.entries(flattenTokens(themeEntry(theme).tokens)).filter(([key]) => key.startsWith('color.') && !(locked.includes('accent') && ['color.accent', 'color.accentText', 'color.focus'].includes(key))));
    result.push(...tokenProperties(colors));
  }
  if (update.accent !== undefined && !locked.includes('accent')) {
    const def = isAccentId(update.accent) ? ACCENTS.find(item => item.id === update.accent) : undefined;
    const shade = def ? shadeAccent(def, accentContextOf(themeEntry(theme).palette)) : customAccentShade(update.accent, theme);
    if (!shade) throw new Error('Unsupported accent colour.');
    result.push([vars.color.accent.slice(4, -1), shade.accent], [vars.color.accentText.slice(4, -1), shade.accentText], [LEGACY_COLOR_PROPERTIES.accent, shade.accent], [LEGACY_COLOR_PROPERTIES.accentText, shade.accentText]);
  }
  if (update.font !== undefined && !locked.includes('font')) {
    if (!Object.hasOwn(FONT_STACKS, update.font)) throw new Error('Unsupported font.');
    const font = FONT_STACKS[update.font as keyof typeof FONT_STACKS];
    if (font) result.push(['--mw-ui-font', font], ['font-family', font]);
  }
  if (update.density !== undefined && !locked.includes('density')) {
    const sizes = {compact: ['40px', '13px'], cozy: ['56px', '14px'], relaxed: ['68px', '15px']} as const;
    if (!Object.hasOwn(sizes, update.density)) throw new Error('Unsupported density.');
    const [row, size] = sizes[update.density as keyof typeof sizes];
    result.push([vars.density.rowH.slice(4, -1), row], [vars.fontSize.base.slice(4, -1), size], ['font-size', size]);
  }
  return result;
}
