import { describe, it, expect } from 'vitest';
import { t } from './registry.ts';
import {
  negotiateLocale,
  resolveDir,
  baseLanguage,
  isKnownLocale,
  fallbackChain,
  LOCALES,
} from './locales.ts';
import { isolate, stripIsolates } from './bidi.ts';

// The global test setup (src/test/setup.ts) seeds every `en/*.ftl` and activates
// `en`, so these assertions run against the real critical catalog.

describe('i18n registry / t()', () => {
  it('resolves a seeded en message', () => {
    expect(t('common-ok')).toBe('OK');
    expect(t('common-cancel')).toBe('Cancel');
  });

  it('returns the id for an unknown message (never throws)', () => {
    expect(t('does-not-exist')).toBe('does-not-exist');
  });

  it('formats a message with arguments without isolation marks', () => {
    // useIsolating is off, so no stray FSI/PDI leak into the output.
    const out = t('common-loading');
    expect(out).not.toMatch(/[⁦-⁩]/);
  });
});

describe('locale negotiation + direction', () => {
  it('ships exactly the 14 locales (the twelve gate locales + ar + pt-PT)', () => {
    expect(LOCALES).toHaveLength(14);
    expect(LOCALES).toContain('pt-BR');
    expect(LOCALES).toContain('pt-PT');
    expect(LOCALES).toContain('uk');
    expect(LOCALES).toContain('ar');
  });

  it('negotiates a requested tag down to a shipped locale', () => {
    expect(negotiateLocale(['de-AT', 'de'])).toBe('de');
    expect(negotiateLocale(['pt', 'en'])).toBe('pt-BR');
    expect(negotiateLocale(['ar'])).toBe('ar');
    expect(negotiateLocale(['ar-EG', 'ar'])).toBe('ar');
    expect(negotiateLocale(['xx-YY'])).toBe('en');
  });

  it('negotiates European and Brazilian Portuguese apart', () => {
    expect(negotiateLocale(['pt-PT'])).toBe('pt-PT');
    expect(negotiateLocale(['pt-PT', 'pt', 'en'])).toBe('pt-PT');
    expect(negotiateLocale(['pt_PT'])).toBe('pt-PT');
    expect(negotiateLocale(['pt-pt'])).toBe('pt-PT');
    // CLDR makes pt-PT the parent of every non-Brazilian region.
    for (const tag of ['pt-AO', 'pt-MZ', 'pt-CV', 'pt-GW', 'pt-ST', 'pt-TL', 'pt-MO', 'pt-CH', 'pt-LU', 'pt-Latn-AO']) {
      expect(negotiateLocale([tag, 'en'])).toBe('pt-PT');
    }
    expect(negotiateLocale(['pt-BR'])).toBe('pt-BR');
    expect(negotiateLocale(['pt-Latn-BR'])).toBe('pt-BR');
    // A bare `pt` keeps its old behaviour: CLDR likely subtags expand it to pt-Latn-BR.
    expect(negotiateLocale(['pt'])).toBe('pt-BR');
    expect(negotiateLocale(['en-GB', 'pt-PT'])).toBe('en');
  });

  it('resolves LTR for the LTR locales, RTL for ar + he plumbing', () => {
    for (const l of LOCALES) expect(resolveDir(l)).toBe(l === 'ar' ? 'rtl' : 'ltr');
    expect(resolveDir('ar')).toBe('rtl');
    expect(resolveDir('he-IL')).toBe('rtl');
  });

  it('derives base language + fallback chain', () => {
    expect(baseLanguage('pt-BR')).toBe('pt');
    expect(isKnownLocale('ja')).toBe(true);
    expect(isKnownLocale('ar')).toBe(true);
    expect(fallbackChain('de')).toEqual(['de', 'en']);
    expect(fallbackChain('en')).toEqual(['en']);
    expect(fallbackChain('ar')).toEqual(['ar', 'en']);
  });
});

describe('bidi isolation (SPEC §24)', () => {
  it('wraps untrusted text in FSI…PDI', () => {
    const out = isolate('hello');
    expect(out.charCodeAt(0)).toBe(0x2068); // FSI
    expect(out.charCodeAt(out.length - 1)).toBe(0x2069); // PDI
    expect(out).toContain('hello');
  });

  it('strips attacker-embedded direction overrides', () => {
    const spoof = 'gnp.‮exe'; // RLO in the middle
    expect(stripIsolates(spoof)).toBe('gnp.exe');
    expect(isolate(spoof)).not.toContain('‮');
  });

  it('collapses null/undefined to empty', () => {
    expect(isolate(null)).toBe('');
    expect(isolate(undefined)).toBe('');
  });
});
