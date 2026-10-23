import { describe, it, expect } from 'vitest';
import { PSEUDO_EXPANSION, pseudoLocalize } from './pseudo.ts';
import { t } from './registry.ts';

describe('pseudoLocalize', () => {
  it('accents letters, brackets the text and grows it by at least 40 %', () => {
    for (const s of ['Save', 'Delete this message?', 'A', 'Move 3 messages to Archive']) {
      const p = pseudoLocalize(s);
      expect(p.startsWith('[')).toBe(true);
      expect(p.endsWith(']')).toBe(true);
      expect(p.length - 2).toBeGreaterThanOrEqual(Math.ceil(s.length * (1 + PSEUDO_EXPANSION)));
      expect(p).not.toMatch(/[A-Za-z]/);
    }
    expect(pseudoLocalize('Save')).toBe('[Šáṽé ~]');
  });

  it('keeps digits, punctuation, bidi isolation marks and emoji', () => {
    const p = pseudoLocalize('⁨bob⁩: 42 % 🎙');
    expect(p).toContain('⁨ƀóƀ⁩: 42 % 🎙');
  });

  it('pads in short groups so the text can wrap', () => {
    const p = pseudoLocalize('x'.repeat(50));
    expect(p).toMatch(/^\[ẋ{50}( ~{1,5})+\]$/);
  });

  it('leaves the empty string alone', () => {
    expect(pseudoLocalize('')).toBe('');
  });

  it('is off in normal builds and tests: t() returns the catalog text', () => {
    expect(__MW_I18N_PSEUDO__).toBe(false);
    expect(t('common-ok')).toBe('OK');
  });
});
