// Registry + appearance-resolution invariants (SPEC §17.1).
//
// Scope split: this file pins the registry SHAPE (the contract downstream lanes
// consume) and the pure mode/schedule resolution, plus a contrast SMOKE check
// that every pack is viable. The exhaustive per-theme × per-pair WCAG matrix
// lives in `contrast.test.ts`.

import { describe, it, expect } from 'vitest';
import {
  THEME_GROUPS,
  THEME_LIST,
  THEME_REGISTRY,
  findTheme,
  isThemeName,
  themeEntry,
  themesByAppearance,
  variantFor,
  DEFAULT_DARK_THEME,
  DEFAULT_LIGHT_THEME,
} from './registry.ts';
import { THEMES, THEME_META, THEME_OPTIONS } from './tokens.ts';
import {
  CONTRAST_PAIRS,
  advisoryFailures,
  checkPalette,
  contrastRatio,
  failingPairs,
  parseColor,
  relativeLuminance,
} from './contrast.ts';
import {
  defaultAppearance,
  inDarkWindow,
  msUntilNextFlip,
  parseAppearancePrefs,
  parseClock,
  resolveAppearance,
  resolveThemeName,
  serializeAppearancePrefs,
  DEFAULT_SCHEDULE,
} from './appearance.ts';

const AT = (h: number, m = 0): Date => new Date(2026, 7, 10, h, m, 0, 0);

describe('theme registry', () => {
  it('covers every theme in the token table, in gallery order', () => {
    expect(THEME_LIST.map((e) => e.id)).toEqual(THEME_META.map((m) => m.id));
    expect(Object.keys(THEME_REGISTRY).sort()).toEqual(Object.keys(THEMES).sort());
  });

  it('ships the three new packs with both appearances', () => {
    for (const family of ['slate', 'ocean', 'plum'] as const) {
      const packs = THEME_LIST.filter((e) => e.family === family);
      expect(packs.map((p) => p.appearance).sort()).toEqual(['dark', 'light']);
    }
  });

  it('resolves the accent past its --mw-accent override wrapper', () => {
    for (const entry of THEME_LIST) {
      expect(entry.tokens.color.accent).toContain('var(--mw-accent');
      expect(entry.palette.accent).not.toContain('var(');
      expect(parseColor(entry.palette.accent)).not.toBeNull();
    }
  });

  it('pairs every theme with an opposite-appearance counterpart', () => {
    for (const entry of THEME_LIST) {
      const counterpart = THEME_REGISTRY[entry.counterpart];
      expect(counterpart.appearance).not.toBe(entry.appearance);
    }
  });

  it('keeps same-family counterparts inside the family', () => {
    for (const entry of THEME_LIST.filter((e) => e.family !== 'amoled')) {
      expect(THEME_REGISTRY[entry.counterpart].family).toBe(entry.family);
    }
    // AMOLED is dark-only, so its light counterpart leaves the family.
    expect(THEME_REGISTRY.amoled.counterpart).toBe('light');
  });

  it('exposes preview swatches and honest metadata for the gallery', () => {
    for (const entry of THEME_LIST) {
      expect(entry.swatches).toHaveLength(5);
      for (const swatch of entry.swatches) expect(parseColor(swatch)).not.toBeNull();
      expect(entry.label.length).toBeGreaterThan(0);
      expect(entry.description.length).toBeGreaterThan(0);
    }
  });

  it('groups themes by family with no theme lost or duplicated', () => {
    const grouped = THEME_GROUPS.flatMap((g) => g.themes.map((t) => t.id));
    expect(grouped.sort()).toEqual(THEME_LIST.map((e) => e.id).sort());
  });

  it('validates untrusted theme ids', () => {
    expect(isThemeName('grove-dark')).toBe(true);
    expect(isThemeName('ocean-light')).toBe(true);
    expect(isThemeName('nope')).toBe(false);
    expect(isThemeName('toString')).toBe(false); // prototype key, not a theme
    expect(isThemeName(null)).toBe(false);
    expect(findTheme('plum-dark')?.family).toBe('plum');
    expect(findTheme('nope')).toBeUndefined();
  });

  it('selects the requested appearance variant of a pack', () => {
    expect(variantFor('grove-dark', 'light')).toBe('grove-light');
    expect(variantFor('grove-light', 'light')).toBe('grove-light');
    expect(variantFor('ocean-light', 'dark')).toBe('ocean-dark');
    // Dark-only pack asked for dark stays put.
    expect(variantFor('amoled', 'dark')).toBe('amoled');
    expect(variantFor('amoled', 'light')).toBe('light');
  });

  it('keeps THEME_OPTIONS in step with the registry', () => {
    expect(THEME_OPTIONS.map((o) => o.value)).toEqual(THEME_LIST.map((e) => e.id));
    expect(themeEntry('slate-dark').label).toBe('Slate Dark');
    expect(themesByAppearance('light').every((e) => e.appearance === 'light')).toBe(true);
    expect(themesByAppearance('dark').length + themesByAppearance('light').length).toBe(
      THEME_LIST.length,
    );
    expect(THEME_REGISTRY[DEFAULT_LIGHT_THEME].appearance).toBe('light');
    expect(THEME_REGISTRY[DEFAULT_DARK_THEME].appearance).toBe('dark');
  });
});

describe('contrast primitives', () => {
  it('matches the WCAG reference extremes', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21, 5);
    expect(contrastRatio('#777777', '#ffffff')).toBeCloseTo(4.478, 2);
    expect(contrastRatio('#ffffff', '#ffffff')).toBeCloseTo(1, 5);
    expect(relativeLuminance([255, 255, 255])).toBeCloseTo(1, 5);
    expect(relativeLuminance([0, 0, 0])).toBeCloseTo(0, 5);
  });

  it('parses the colour forms the token layer uses', () => {
    expect(parseColor('#abc')).toEqual([170, 187, 204]);
    expect(parseColor('#2563eb')).toEqual([37, 99, 235]);
    expect(parseColor('rgb(1, 2, 3)')).toEqual([1, 2, 3]);
    expect(parseColor('var(--mw-accent, #2563eb)')).toEqual([37, 99, 235]);
  });

  it('treats unmeasurable or translucent colours as failures, never passes', () => {
    expect(parseColor('#12345678')).toBeNull(); // alpha < ff
    expect(parseColor('rgba(0,0,0,0.5)')).toBeNull();
    expect(parseColor('currentColor')).toBeNull();
    // A zero ratio clears no threshold.
    expect(contrastRatio('currentColor', '#ffffff')).toBe(0);
  });

  it('declares every WCAG tier over real palette keys', () => {
    expect(CONTRAST_PAIRS.length).toBeGreaterThan(0);
    const levels = new Set(CONTRAST_PAIRS.map((p) => p.level));
    expect(levels).toEqual(new Set(['text', 'non-text', 'advisory']));
    for (const pair of CONTRAST_PAIRS) {
      expect(THEMES.light.color).toHaveProperty(pair.fg);
      expect(THEMES.light.color).toHaveProperty(pair.bg);
      expect(pair.min).toBeGreaterThanOrEqual(3);
      expect(pair.why.length).toBeGreaterThan(0);
    }
    // Text pairs are the AA floor and must all be 4.5:1.
    expect(CONTRAST_PAIRS.filter((p) => p.level === 'text').every((p) => p.min === 4.5)).toBe(true);
    // The advisory tier is exactly the shared `border` token — nothing else may
    // opt out of enforcement (see contrast.ts for why it is not enforced).
    expect(
      new Set(CONTRAST_PAIRS.filter((p) => p.level === 'advisory').map((p) => p.fg)),
    ).toEqual(new Set(['border']));
  });

  it('separates enforced failures from advisory ones', () => {
    // A palette whose border is invisible still passes the enforced gate, and
    // the shortfall still shows up in the advisory report rather than vanishing.
    const weakBorder = { ...THEMES.light.color, border: '#fefefe' };
    expect(failingPairs(weakBorder)).toEqual([]);
    expect(advisoryFailures(weakBorder).length).toBeGreaterThan(0);
    // An unmeasurable colour is a hard failure, not a skip.
    expect(failingPairs({ ...THEMES.light.color, text: 'currentColor' }).length).toBeGreaterThan(0);
  });
});

describe('theme pack contrast', () => {
  // Smoke gate: no pack ships failing its own requirement table. The exhaustive
  // per-pair report is `contrast.test.ts`.
  for (const entry of THEME_LIST) {
    it(`${entry.id} satisfies every declared contrast pair`, () => {
      const failures = failingPairs(entry.palette).map(
        (r) => `${r.pair.fg} on ${r.pair.bg}: ${r.ratio} < ${r.pair.min} (${r.pair.level})`,
      );
      expect(failures).toEqual([]);
      expect(checkPalette(entry.palette)).toHaveLength(CONTRAST_PAIRS.length);
    });
  }
});

describe('appearance preferences', () => {
  it('defaults a fresh user to following the OS', () => {
    const p = defaultAppearance();
    expect(p.mode).toBe('system');
    expect(p.schedule).toEqual(DEFAULT_SCHEDULE);
  });

  it('migrates a pre-tri-state payload to a fixed choice and seeds the pair', () => {
    const p = parseAppearancePrefs({ theme: 'grove-dark', density: 'compact' });
    expect(p.mode).toBe('fixed');
    expect(p.theme).toBe('grove-dark');
    expect(p.darkTheme).toBe('grove-dark');
    expect(p.lightTheme).toBe('grove-light');
    expect(p.density).toBe('compact');
  });

  it('falls back field-by-field on a partly corrupt payload', () => {
    const p = parseAppearancePrefs({
      mode: 'sideways',
      theme: 'not-a-theme',
      lightTheme: 'ocean-light',
      density: 'enormous',
      schedule: { darkStart: '25:00', darkEnd: '06:30' },
      accent: 42,
      font: 'comic',
      layout: 'ribbon',
      ribbonCollapsed: 'yes',
    });
    expect(p.mode).toBe('system');
    expect(p.theme).toBe('light');
    expect(p.lightTheme).toBe('ocean-light');
    expect(p.density).toBe('cozy');
    expect(p.schedule).toEqual({ darkStart: '20:00', darkEnd: '06:30' });
    expect(p.accent).toBe('');
    expect(p.font).toBe('default');
    expect(p.layout).toBe('ribbon');
    expect(p.ribbonCollapsed).toBe(false);
  });

  it('round-trips through its serialized form', () => {
    const p = { ...defaultAppearance(), mode: 'schedule' as const, darkTheme: 'plum-dark' as const };
    expect(parseAppearancePrefs(JSON.parse(serializeAppearancePrefs(p)))).toEqual(p);
  });

  it('rejects a non-object payload', () => {
    expect(parseAppearancePrefs(null)).toEqual(defaultAppearance());
    expect(parseAppearancePrefs('nope')).toEqual(defaultAppearance());
  });
});

describe('mode resolution', () => {
  const base = { ...defaultAppearance(), lightTheme: 'ocean-light' as const, darkTheme: 'plum-dark' as const };

  it('fixed mode ignores the OS and the clock', () => {
    const prefs = { ...base, mode: 'fixed' as const, theme: 'grove-light' as const };
    expect(resolveThemeName(prefs, { systemDark: true, now: AT(23) })).toBe('grove-light');
    expect(resolveAppearance(prefs, { systemDark: true, now: AT(23) })).toBe('light');
  });

  it('system mode picks the pair member for the OS scheme', () => {
    const prefs = { ...base, mode: 'system' as const };
    expect(resolveThemeName(prefs, { systemDark: false, now: AT(12) })).toBe('ocean-light');
    expect(resolveThemeName(prefs, { systemDark: true, now: AT(12) })).toBe('plum-dark');
  });

  it('system mode repairs a pair member stored with the wrong appearance', () => {
    // Hand-edited prefs: a light pack sitting in the dark slot.
    const prefs = { ...base, mode: 'system' as const, darkTheme: 'grove-light' as const };
    expect(resolveThemeName(prefs, { systemDark: true, now: AT(12) })).toBe('grove-dark');
  });

  it('schedule mode follows a wrapping local window', () => {
    const prefs = { ...base, mode: 'schedule' as const };
    expect(resolveThemeName(prefs, { systemDark: false, now: AT(12) })).toBe('ocean-light');
    expect(resolveThemeName(prefs, { systemDark: false, now: AT(21) })).toBe('plum-dark');
    expect(resolveThemeName(prefs, { systemDark: false, now: AT(2) })).toBe('plum-dark');
    expect(resolveThemeName(prefs, { systemDark: true, now: AT(12) })).toBe('ocean-light');
  });

  it('handles clock parsing and window edges', () => {
    expect(parseClock('00:00')).toBe(0);
    expect(parseClock('07:30')).toBe(450);
    expect(parseClock('24:00')).toBeNull();
    expect(parseClock('7:30')).toBeNull();
    const sched = { darkStart: '20:00', darkEnd: '07:00' };
    expect(inDarkWindow(20 * 60, sched)).toBe(true); // inclusive start
    expect(inDarkWindow(7 * 60, sched)).toBe(false); // exclusive end
    expect(inDarkWindow(19 * 60 + 59, sched)).toBe(false);
    // Non-wrapping window (dark during the day) works too.
    expect(inDarkWindow(12 * 60, { darkStart: '09:00', darkEnd: '17:00' })).toBe(true);
    expect(inDarkWindow(20 * 60, { darkStart: '09:00', darkEnd: '17:00' })).toBe(false);
  });

  it('degrades an empty or malformed window to light rather than pinning dark', () => {
    expect(inDarkWindow(3 * 60, { darkStart: '08:00', darkEnd: '08:00' })).toBe(false);
    expect(inDarkWindow(3 * 60, { darkStart: 'x', darkEnd: '07:00' })).toBe(false);
    expect(msUntilNextFlip(AT(12), { darkStart: '08:00', darkEnd: '08:00' })).toBeNull();
    expect(msUntilNextFlip(AT(12), { darkStart: 'x', darkEnd: '07:00' })).toBeNull();
  });

  it('arms the next flip at the nearer window boundary', () => {
    const sched = { darkStart: '20:00', darkEnd: '07:00' };
    expect(msUntilNextFlip(AT(19), sched)).toBe(60 * 60 * 1000); // 1h to 20:00
    expect(msUntilNextFlip(AT(21), sched)).toBe(10 * 60 * 60 * 1000); // 10h to 07:00
    expect(msUntilNextFlip(AT(6, 30), sched)).toBe(30 * 60 * 1000);
    // Exactly on a boundary waits a full day for that edge, not zero.
    expect(msUntilNextFlip(AT(20), sched)).toBe(11 * 60 * 60 * 1000); // next is 07:00
  });
});
