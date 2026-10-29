// t85: badge colours derived from the live accent, 0.2 OKLCH L lighter/darker.
import { describe, expect, it } from 'vitest';
import {
  applyDerivedBadge,
  contrastingText,
  deriveBadge,
  derivedBadgeFor,
  effectiveAccent,
  DERIVED_BADGE_PROPERTY,
  DERIVED_BADGE_TEXT_PROPERTY,
} from './badgeDerived.ts';
import { toOklch, ACCENT_IDS } from './accentMath.ts';
import { contrastRatio } from './contrast.ts';
import { THEME_REGISTRY } from './registry.ts';
import { ACCENT_SHADES } from './accents.ts';
import { customAccentShade } from './customAccent.ts';
import { isBadgeColor, parseAppearancePrefs } from './appearance.ts';
import type { ThemeName } from './contract.css.ts';

const L = (css: string): number => toOklch(css)!.l;

describe('deriveBadge', () => {
  it('moves OKLCH lightness by 0.2 and keeps the hue', () => {
    const accent = '#3366cc';
    const base = toOklch(accent)!;
    const lighter = deriveBadge(accent, 'accent-lighter')!;
    const darker = deriveBadge(accent, 'accent-darker')!;
    expect(L(lighter.badge) - base.l).toBeCloseTo(0.2, 2);
    expect(base.l - L(darker.badge)).toBeCloseTo(0.2, 2);
    expect(Math.abs(toOklch(darker.badge)!.h - base.h)).toBeLessThan(3);
  });

  it('clamps at the ends of the range', () => {
    expect(deriveBadge('#f8f8f8', 'accent-lighter')!.badge).toBe('#ffffff');
    expect(deriveBadge('#101010', 'accent-darker')!.badge).toBe('#000000');
    expect(L(deriveBadge('#ffffff', 'accent-darker')!.badge)).toBeCloseTo(0.8, 2);
  });

  it('is not naive RGB scaling', () => {
    // 20% darker in RGB would be #2952a3; OKLCH L -0.2 is clearly darker.
    expect(deriveBadge('#3366cc', 'accent-darker')!.badge).not.toBe('#2952a3');
  });

  it('refuses an unreadable accent', () => {
    expect(deriveBadge('not-a-colour', 'accent-lighter')).toBeNull();
  });
});

describe('contrastingText', () => {
  it('picks black on light and white on dark, always >= 4.5:1', () => {
    expect(contrastingText('#ffff00')).toBe('#000000');
    expect(contrastingText('#000080')).toBe('#ffffff');
    for (let v = 0; v <= 255; v += 5) {
      const hex = `#${v.toString(16).padStart(2, '0').repeat(3)}`;
      expect(contrastRatio(contrastingText(hex), hex), hex).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('gives AA text on every derived badge in every theme and accent', () => {
    for (const theme of Object.keys(ACCENT_SHADES) as ThemeName[]) {
      for (const id of ['', ...ACCENT_IDS] as const) {
        for (const which of ['accent-lighter', 'accent-darker'] as const) {
          const d = derivedBadgeFor(theme, id, '', which)!;
          expect(contrastRatio(d.text, d.badge), `${theme} ${id} ${which}`).toBeGreaterThanOrEqual(4.5);
        }
      }
    }
  });
});

describe('effectiveAccent', () => {
  const light = (Object.keys(THEME_REGISTRY) as ThemeName[]).find((n) => THEME_REGISTRY[n].appearance === 'light')!;
  const dark = (Object.keys(THEME_REGISTRY) as ThemeName[]).find((n) => THEME_REGISTRY[n].appearance === 'dark')!;
  it('follows custom > named > theme accent, per theme', () => {
    expect(effectiveAccent(light, '', '')).toBe(THEME_REGISTRY[light].palette.accent);
    expect(effectiveAccent(dark, 'teal', '')).toBe(ACCENT_SHADES[dark].teal.accent);
    expect(effectiveAccent(dark, 'teal', '#cc3366')).toBe(customAccentShade('#cc3366', dark)!.accent);
    expect(effectiveAccent(light, 'teal', '')).not.toBe(effectiveAccent(dark, 'teal', ''));
  });
});

describe('setting wiring', () => {
  it('accepts the two new values in prefs', () => {
    expect(isBadgeColor('accent-lighter')).toBe(true);
    expect(isBadgeColor('accent-darker')).toBe(true);
    expect(isBadgeColor('accent-brighter')).toBe(false);
    expect(parseAppearancePrefs({ badgeColor: 'accent-darker' }).badgeColor).toBe('accent-darker');
  });

  it('writes the CSS properties through the CSSOM and clears them for other values', () => {
    const root = document.createElement('div');
    applyDerivedBadge(root, 'accent-lighter', 'light' as ThemeName, '', '#3366cc');
    const want = deriveBadge(customAccentShade('#3366cc', 'light' as ThemeName)!.accent, 'accent-lighter')!;
    expect(root.style.getPropertyValue(DERIVED_BADGE_PROPERTY)).toBe(want.badge);
    expect(root.style.getPropertyValue(DERIVED_BADGE_TEXT_PROPERTY)).toBe(want.text);
    applyDerivedBadge(root, 'teal', 'light' as ThemeName, '', '#3366cc');
    expect(root.style.getPropertyValue(DERIVED_BADGE_PROPERTY)).toBe('');
    expect(root.style.getPropertyValue(DERIVED_BADGE_TEXT_PROPERTY)).toBe('');
  });
});
