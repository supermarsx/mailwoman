// The named accents (t32-ext §V): every theme × accent cell meets the same
// contrast pairs as the packs themselves, the committed table is what the
// generator produces, and a colour accent stored by an older build is carried
// over to the nearest named accent.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { THEME_LIST, accentShade, type ThemeEntry } from './registry.ts';
import { THEMES } from './tokens.ts';
import { failingPairs, type ContrastResult } from './contrast.ts';
import {
  ACCENTS,
  ACCENT_IDS,
  computeAccentTable,
  nearestAccentId,
  oklchToHex,
  toOklch,
  type AccentId,
} from './accentMath.ts';
import { ACCENT_SHADES } from './accents.ts';
import {
  defaultAppearance,
  mergeAppearancePrefs,
  parseAppearancePrefs,
  serializeAppearancePrefs,
} from './appearance.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';

/**
 * The palette a page shows under `data-theme=<entry>` + `data-accent=<id>`:
 * what `themes.css.ts` emits for the cell, laid over the pack. The focus
 * outline moves with the accent only in packs whose focus IS the accent.
 */
function cellPalette(entry: ThemeEntry, shade: { accent: string; accentText: string }) {
  const focusIsAccent = entry.tokens.color.focus === entry.tokens.color.accent;
  return {
    ...entry.palette,
    accent: shade.accent,
    accentText: shade.accentText,
    focus: focusIsAccent ? shade.accent : entry.palette.focus,
  };
}

function describeFailures(results: ContrastResult[]): string[] {
  return results.map((r) => `${r.pair.fg} on ${r.pair.bg} = ${r.ratio} (min ${r.pair.min})`);
}

describe('the theme × accent matrix', () => {
  it('has a cell for every theme and every accent', () => {
    expect(Object.keys(ACCENT_SHADES).sort()).toEqual(THEME_LIST.map((e) => e.id).sort());
    expect(THEME_LIST).toHaveLength(18);
    expect(ACCENT_IDS).toHaveLength(10);
    for (const entry of THEME_LIST) {
      expect(Object.keys(ACCENT_SHADES[entry.id]), entry.id).toEqual([...ACCENT_IDS]);
      for (const id of ACCENT_IDS) {
        const cell = ACCENT_SHADES[entry.id][id];
        expect(cell.accent, `${entry.id}/${id}`).toMatch(/^#[0-9a-f]{6}$/);
        expect(cell.accentText, `${entry.id}/${id}`).toMatch(/^#[0-9a-f]{6}$/);
      }
    }
  });

  // 18 themes × 10 accents, each on every enforced pair of `CONTRAST_PAIRS`:
  // the accent at 3:1 on the four surfaces, its ink at 4.5:1 on the accent and
  // on the success and danger fills it also labels, the focus outline at 3:1
  // where it follows the accent, and every other pair of the pack unchanged.
  describe.each(THEME_LIST.map((e) => [e.id, e] as const))('%s', (_id, entry) => {
    it.each([...ACCENT_IDS])('%s meets every enforced pair', (id) => {
      const failures = failingPairs(cellPalette(entry, ACCENT_SHADES[entry.id][id]));
      expect(describeFailures(failures), `${entry.id}/${id}`).toEqual([]);
    });
  });

  it('fails a cell whose shade is too dark for a dark page or whose ink is unreadable', () => {
    // The check above is only worth something if it can fail.
    const dark = THEME_LIST.find((e) => e.id === 'dark') as ThemeEntry;
    const dim = failingPairs(cellPalette(dark, { accent: '#2a3a6a', accentText: '#0b0b0b' }));
    expect(describeFailures(dim)).toContain('accent on bg = 1.72 (min 3)');
    expect(dim.map((r) => `${r.pair.fg}/${r.pair.bg}`)).toContain('accentText/accent');

    // In a pack whose focus follows the accent, a weak shade fails the focus
    // outline too.
    const ocean = THEME_LIST.find((e) => e.id === 'ocean-light') as ThemeEntry;
    const pale = failingPairs(cellPalette(ocean, { accent: '#9fd8e0', accentText: '#ffffff' }));
    expect(pale.map((r) => `${r.pair.fg}/${r.pair.bg}`)).toEqual(
      expect.arrayContaining(['accent/bg', 'focus/bg', 'accentText/accent']),
    );
  });

  it('is the table the generator computes from the current palettes', () => {
    // `scripts/theme/accents.mjs` writes accents.ts from the same module; a
    // palette or accent edit without a regenerated table fails here.
    const palettes = Object.fromEntries(
      Object.entries(THEMES).map(([id, t]) => [id, t.color]),
    ) as Record<keyof typeof THEMES, (typeof THEMES)['dark']['color']>;
    expect(computeAccentTable(palettes)).toEqual(ACCENT_SHADES);
  });

  it('keeps each accent on its own hue', () => {
    for (const entry of THEME_LIST) {
      for (const def of ACCENTS) {
        const lch = toOklch(ACCENT_SHADES[entry.id][def.id].accent);
        expect(lch, `${entry.id}/${def.id}`).not.toBeNull();
        if (lch === null || def.chroma < 0.05) continue;
        const drift = Math.abs(((lch.h - def.hue + 540) % 360) - 180);
        expect(drift, `${entry.id}/${def.id} hue ${lch.h.toFixed(1)}`).toBeLessThan(6);
      }
    }
  });

  it("gives '' the pack's own accent", () => {
    for (const entry of THEME_LIST) {
      expect(accentShade(entry.id, '')).toEqual({
        accent: entry.palette.accent,
        accentText: entry.palette.accentText,
      });
      expect(accentShade(entry.id, 'teal')).toEqual(ACCENT_SHADES[entry.id].teal);
    }
  });
});

describe('OKLCH maths — published reference values', () => {
  // CSS Color 4 / Ottosson's reference conversions, independent of this module.
  it('reads sRGB primaries at their published OKLCH coordinates', () => {
    const red = toOklch('#ff0000');
    expect(red?.l).toBeCloseTo(0.628, 3);
    expect(red?.c).toBeCloseTo(0.2577, 3);
    expect(red?.h).toBeCloseTo(29.23, 1);
    const blue = toOklch('#0000ff');
    expect(blue?.l).toBeCloseTo(0.452, 3);
    expect(blue?.c).toBeCloseTo(0.3132, 3);
    expect(blue?.h).toBeCloseTo(264.05, 1);
    expect(toOklch('#ffffff')?.l).toBeCloseTo(1, 3);
  });

  it('writes them back, and pulls an out-of-gamut chroma into sRGB', () => {
    expect(oklchToHex(0.62796, 0.25768, 29.2339)).toBe('#ff0000');
    expect(oklchToHex(0.45201, 0.31321, 264.052)).toBe('#0000ff');
    // Chroma 0.4 at this lightness is outside sRGB: the hue is kept.
    const clipped = toOklch(oklchToHex(0.7, 0.4, 145));
    expect(clipped?.h).toBeCloseTo(145, 0);
    expect(clipped?.c).toBeLessThan(0.4);
  });
});

describe('a colour accent stored before t32 V', () => {
  it.each([
    ['#2563eb', 'blue'],
    ['#7c3aed', 'violet'],
    ['#0d9488', 'teal'],
    ['#6d8a4e', 'green'],
    ['#c2410c', 'orange'],
    ['#be123c', 'red'],
    ['#808080', 'graphite'],
    ['#e0218a', 'magenta'],
  ] as const)('%s maps to %s', (hex, id) => {
    expect(nearestAccentId(hex)).toBe(id);
    expect(parseAppearancePrefs({ accent: hex }).accentId).toBe(id);
  });

  it('keeps a valid accentId, and drops what it cannot read', () => {
    expect(parseAppearancePrefs({ accentId: 'teal' }).accentId).toBe('teal');
    expect(parseAppearancePrefs({ accentId: 'teal', accent: '#be123c' }).accentId).toBe('teal');
    expect(parseAppearancePrefs({ accentId: '' as const, accent: '#be123c' }).accentId).toBe('');
    expect(parseAppearancePrefs({ accentId: 'chartreuse' }).accentId).toBe('');
    expect(parseAppearancePrefs({ accent: 'not a colour' }).accentId).toBe('');
    expect(parseAppearancePrefs({ accent: '' }).accentId).toBe('');
    expect(nearestAccentId('not a colour')).toBeNull();
  });

  it('lets an old server blob bring its accent to a device that has one', () => {
    const current = { ...defaultAppearance(), accentId: 'teal' as AccentId };
    expect(mergeAppearancePrefs(current, { accent: '#7c3aed' }).accentId).toBe('violet');
    // A payload with neither keeps this device's accent.
    expect(mergeAppearancePrefs(current, { density: 'compact' }).accentId).toBe('teal');
  });

  it('stores the id, not a colour', () => {
    const stored = JSON.parse(
      serializeAppearancePrefs({ ...defaultAppearance(), accentId: 'indigo' }),
    ) as Record<string, unknown>;
    expect(stored.accentId).toBe('indigo');
    expect(stored).not.toHaveProperty('accent');
  });
});

describe('the slice applies the accent as an attribute', () => {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const root = (): HTMLElement => document.documentElement;

  beforeEach(() => {
    localStorage.clear();
    root().removeAttribute('data-accent');
    root().removeAttribute('data-theme');
  });

  it('sets data-accent, keeps it across a theme switch, and removes it for the default', () => {
    const s = createThemeSlice(ctx);
    s.setAccentId('magenta');
    expect(root().getAttribute('data-accent')).toBe('magenta');
    s.setTheme('paper');
    expect(root().getAttribute('data-theme')).toBe('paper');
    expect(root().getAttribute('data-accent')).toBe('magenta');
    // An inline property would beat the per-theme rules, so none is written.
    expect(root().style.getPropertyValue('--mw-accent')).toBe('');
    s.setAccentId('');
    expect(root().hasAttribute('data-accent')).toBe(false);
  });

  it('restores the accent from storage, including one stored as a colour', () => {
    localStorage.setItem('mw.theme.prefs', JSON.stringify({ accent: '#0d9488' }));
    const s = createThemeSlice(ctx);
    expect(s.accentId()).toBe('teal');
    expect(root().getAttribute('data-accent')).toBe('teal');
  });
});
