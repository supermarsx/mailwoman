// t40: the unread badge colour is stored in the appearance blob like the
// accent, and every option keeps the count text at 4.5:1 or better.
import { describe, expect, it } from 'vitest';
import { parseAppearancePrefs, isBadgeColor, defaultAppearance } from './appearance.ts';
import { ACCENT_SHADES } from './accents.ts';
import { ACCENT_IDS } from './accentMath.ts';
import { THEME_REGISTRY } from './registry.ts';
import { contrastRatio } from './contrast.ts';
import type { ThemeName } from './contract.css.ts';

describe('badge colour preference', () => {
  it('defaults to the theme colour and round-trips a named, accent or invalid value', () => {
    expect(defaultAppearance().badgeColor).toBe('default');
    expect(parseAppearancePrefs({ badgeColor: 'teal' }).badgeColor).toBe('teal');
    expect(parseAppearancePrefs({ badgeColor: 'accent' }).badgeColor).toBe('accent');
    expect(parseAppearancePrefs({ badgeColor: '#ff0000' }).badgeColor).toBe('default');
    expect(parseAppearancePrefs({}).badgeColor).toBe('default');
    expect(isBadgeColor('graphite')).toBe(true);
    expect(isBadgeColor('')).toBe(false);
  });
  it('every option has 4.5:1 ink in every theme, dark and light', () => {
    const themes = Object.keys(ACCENT_SHADES) as ThemeName[];
    expect(new Set(themes.map((name) => THEME_REGISTRY[name].appearance))).toEqual(new Set(['light', 'dark']));
    for (const name of themes) {
      const palette = THEME_REGISTRY[name].palette;
      // default: the pack's signal block; accent: the pack's own accent.
      expect(contrastRatio(palette.signal, palette.signalText), `${name} default`).toBeGreaterThanOrEqual(4.5);
      expect(contrastRatio(palette.accent, palette.accentText), `${name} accent`).toBeGreaterThanOrEqual(4.5);
      for (const id of ACCENT_IDS) {
        const { accent, accentText } = ACCENT_SHADES[name][id];
        expect(contrastRatio(accent, accentText), `${name} ${id}`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });
});
