import { describe, expect, it } from 'vitest';
import {
  activeSignal, choicesFor, classify, globMatch, localized, mayStore, parseStored, resolve, staleness,
  type ConsentConfig, type StoredConsent,
} from './policy.ts';

const cat = (id: 'functional' | 'analytics' | 'marketing', enabled: boolean, def = false) => ({ id, enabled, default: def, label: {}, description: {} });
const base = (over: Partial<ConsentConfig> = {}): ConsentConfig => ({
  enabled: true, mode: 'optIn', version: 2, lifetimeDays: 180, reprompt: true, honourGpc: true, honourDnt: false, unclassified: 'report',
  categories: [{ id: 'necessary', enabled: true, default: true, label: {}, description: {} }, cat('functional', true, true), cat('analytics', true), cat('marketing', false)],
  inventory: [
    { id: 'theme', kind: 'localStorage', pattern: 'mw.theme.*', category: 'functional', purpose: '', duration: '', party: 'first', provider: '' },
    { id: 'consent', kind: 'localStorage', pattern: 'mw.consent*', category: 'necessary', purpose: '', duration: '', party: 'first', provider: '' },
  ],
  rules: [{ kind: 'localStorage', pattern: '_ga*', category: 'analytics' }],
  ...over,
});
const NOW = 1_000_000_000_000;
const stored = (over: Partial<StoredConsent> = {}): StoredConsent => ({ version: 2, decision: 'custom', categories: { necessary: true, functional: true, analytics: false, marketing: false }, signal: 'none', decidedAt: NOW - 1000, ...over });
const quiet = { gpc: false, dnt: false };

describe('classification', () => {
  it('matches globs and prefers exact patterns and administrator rules', () => {
    expect(globMatch('mw.theme.*', 'mw.theme.prefs')).toBe(true);
    expect(globMatch('mw.theme.*', 'mw.theme')).toBe(false);
    expect(globMatch('a*b*c', 'a1b2c')).toBe(true);
    const c = base();
    expect(classify('localStorage', 'mw.theme.sync', c)).toEqual({ category: 'functional', source: 'inventory' });
    expect(classify('localStorage', '_ga_X', c)).toEqual({ category: 'analytics', source: 'rule' });
    expect(classify('localStorage', 'other', c)).toBeNull();
    expect(classify('cookie', 'mw.theme.sync', c)).toBeNull();
    const overridden = base({ rules: [{ kind: 'localStorage', pattern: 'mw.theme.prefs', category: 'marketing' }] });
    expect(classify('localStorage', 'mw.theme.prefs', overridden)?.category).toBe('marketing');
  });
});

describe('resolving a decision', () => {
  it('opt-in allows only necessary until the visitor decides', () => {
    const e = resolve(base(), null, quiet, NOW);
    expect(e.allowed).toEqual({ necessary: true, functional: false, analytics: false, marketing: false });
    expect(e.needsDecision).toBe(true);
  });

  it('opt-out applies the defaults until the visitor decides, and the stored choice after', () => {
    const c = base({ mode: 'optOut' });
    expect(resolve(c, null, quiet, NOW).allowed).toMatchObject({ functional: true, analytics: false });
    expect(resolve(c, stored({ categories: { necessary: true, functional: false, analytics: true, marketing: true } }), quiet, NOW).allowed)
      .toEqual({ necessary: true, functional: false, analytics: true, marketing: false });
  });

  it('never allows a category the administrator does not offer', () => {
    const e = resolve(base(), stored({ categories: { necessary: true, functional: true, analytics: true, marketing: true } }), quiet, NOW);
    expect(e.allowed.marketing).toBe(false);
    expect(e.adjustable.marketing).toBe(false);
  });

  it('notice-only shows no toggles and applies the defaults', () => {
    const e = resolve(base({ mode: 'noticeOnly' }), null, quiet, NOW);
    expect(e.allowed.functional).toBe(true);
    expect(e.adjustable).toEqual({ necessary: false, functional: false, analytics: false, marketing: false });
  });

  it('asks again after a version change or when the decision expired', () => {
    expect(staleness(base(), stored({ version: 1 }), NOW)).toBe('version');
    expect(staleness(base({ reprompt: false }), stored({ version: 1 }), NOW)).toBe('none');
    expect(staleness(base(), stored({ decidedAt: NOW - 181 * 86_400_000 }), NOW)).toBe('expired');
    expect(staleness(base(), null, NOW)).toBe('noDecision');
    const e = resolve(base(), stored({ version: 1 }), quiet, NOW);
    expect(e.needsDecision).toBe(true);
    expect(e.allowed.functional).toBe(false);
  });

  it('honours Global Privacy Control and Do Not Track as configured', () => {
    const accepted = stored({ categories: { necessary: true, functional: true, analytics: true, marketing: true } });
    const c = base({ categories: [...base().categories!.slice(0, 3), cat('marketing', true)] });
    const gpc = resolve(c, accepted, { gpc: true, dnt: false }, NOW);
    expect(gpc.allowed).toEqual({ necessary: true, functional: true, analytics: false, marketing: false });
    expect(gpc.signal).toBe('gpc');
    expect(gpc.adjustable.analytics).toBe(false);
    expect(resolve(c, accepted, { gpc: false, dnt: true }, NOW).allowed.analytics).toBe(true);
    expect(resolve({ ...c, honourDnt: true }, accepted, { gpc: false, dnt: true }, NOW).signal).toBe('dnt');
    expect(activeSignal({ ...c, honourGpc: false }, { gpc: true, dnt: false })).toBe('none');
  });

  it('allows everything and blocks nothing while the control is off', () => {
    const e = resolve({ enabled: false }, null, quiet, NOW);
    expect(Object.values(e.allowed).every(Boolean)).toBe(true);
    expect(mayStore('localStorage', 'anything', { enabled: false }, e)).toBe(true);
  });
});

describe('what may be stored', () => {
  it('follows the category of the name, and the unclassified policy for unknown names', () => {
    const c = base();
    const none = resolve(c, null, quiet, NOW);
    expect(mayStore('localStorage', 'mw.consent', c, none)).toBe(true);
    expect(mayStore('localStorage', 'mw.theme.prefs', c, none)).toBe(false);
    expect(mayStore('localStorage', '_ga', c, none)).toBe(false);
    expect(mayStore('localStorage', 'mystery', c, none)).toBe(true);
    const strict = base({ unclassified: 'analytics' });
    expect(mayStore('localStorage', 'mystery', strict, resolve(strict, null, quiet, NOW))).toBe(false);
  });

  it('bulk decisions cover only the offered categories', () => {
    expect(choicesFor('acceptAll', base())).toEqual({ necessary: true, functional: true, analytics: true, marketing: false });
    expect(choicesFor('rejectAll', base())).toEqual({ necessary: true, functional: false, analytics: false, marketing: false });
  });
});

describe('texts and stored records', () => {
  it('uses the exact locale, then its language, then English, then nothing', () => {
    const map = { en: 'Hello', pt: 'Olá', 'pt-BR': 'Oi' };
    expect(localized(map, 'pt-BR')).toBe('Oi');
    expect(localized(map, 'pt-PT')).toBe('Olá');
    expect(localized(map, 'de')).toBe('Hello');
    expect(localized({ de: ' ' }, 'de')).toBeUndefined();
    expect(localized(undefined, 'en')).toBeUndefined();
  });

  it('reads back only well-formed records', () => {
    expect(parseStored(JSON.stringify(stored()))?.decision).toBe('custom');
    expect(parseStored('{"version":1}')).toBeNull();
    expect(parseStored('nope')).toBeNull();
    expect(parseStored(null)).toBeNull();
    expect(parseStored(JSON.stringify({ ...stored(), categories: { analytics: 'yes' } }))?.categories.analytics).toBe(false);
  });
});
