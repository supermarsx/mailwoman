import { describe, expect, it } from 'vitest';
import { customAccentShade, customAccentValue } from './customAccent.ts';
import { THEME_REGISTRY } from './registry.ts';
import { accentContextOf } from './accentMath.ts';
import { contrastRatio } from './contrast.ts';

describe('personal custom accent', () => {
  it('accepts only bounded opaque hex colours', () => {
    expect(customAccentValue('#Ab12Ef')).toBe('#ab12ef');
    for (const invalid of ['red', '#fff', '#ffffff00', 'url(https://example.test)', null]) expect(customAccentValue(invalid)).toBe('');
  });
  it('keeps readable ink and visible accents on every theme', () => {
    for (const theme of Object.values(THEME_REGISTRY)) {
      for (const color of ['#ffffff', '#000000', '#ff00ff', '#12c0ae']) {
        const shade = customAccentShade(color, theme.id)!;
        const context = accentContextOf(theme.palette);
        for (const surface of context.surfaces) expect(contrastRatio(shade.accent, surface)).toBeGreaterThanOrEqual(3);
        expect(contrastRatio(shade.accent, shade.accentText)).toBeGreaterThanOrEqual(4.5);
      }
    }
  });
});
