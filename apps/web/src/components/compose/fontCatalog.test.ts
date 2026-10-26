import { describe, expect, it } from 'vitest';
import { COMPOSER_FONTS, composerFontStack, normalizeComposerFont } from './fontCatalog';

describe('composer font catalogue', () => {
  it('keeps bundled families separate from system fallback choices', () => {
    expect(COMPOSER_FONTS.filter(font => font.source === 'bundled')).toHaveLength(12);
    expect(COMPOSER_FONTS.filter(font => font.source === 'system')).toHaveLength(7);
    expect(new Set(COMPOSER_FONTS.map(font => font.id)).size).toBe(COMPOSER_FONTS.length);
    expect(composerFontStack('inter')).toBe('"Inter Variable", Inter, Arial, sans-serif');
    expect(composerFontStack('unknown')).toBeUndefined();
  });
  it('preserves legacy names and permits only complete catalogue stacks', () => {
    for (const font of COMPOSER_FONTS) {
      expect(normalizeComposerFont(font.label.toUpperCase())).toBe(font.stack);
      expect(normalizeComposerFont(font.id)).toBe(font.stack);
      expect(normalizeComposerFont(font.stack)).toBe(font.stack);
    }
    for (const input of ['Arial; background:url(https://example.test)', 'url(font.woff2)', 'Arial, evil', 'inherit', '']) {
      expect(normalizeComposerFont(input)).toBeUndefined();
    }
  });
});
