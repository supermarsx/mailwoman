// The stable token names a custom theme is stored under (t32-ext §V2).

import { describe, it, expect } from 'vitest';
import { vars } from './contract.css.ts';
import { THEMES } from './tokens.ts';
import {
  CONTRACT_PROPERTIES,
  LEGACY_COLOR_PROPERTIES,
  PUBLIC_TOKEN_KEYS,
  flattenTokens,
  tokenProperties,
} from './publicTokens.ts';

describe('public token names', () => {
  it('resolves every dotted key to the custom property the build uses', () => {
    expect(CONTRACT_PROPERTIES['color.bg']).toBe(vars.color.bg.slice(4, -1));
    expect(CONTRACT_PROPERTIES['border.rule']).toBe(vars.border.rule.slice(4, -1));
    expect(CONTRACT_PROPERTIES['elevation.2']).toBe(vars.elevation[2].slice(4, -1));
    expect(CONTRACT_PROPERTIES['a11y.focusRingColor']).toBe(vars.a11y.focusRingColor.slice(4, -1));
    for (const key of PUBLIC_TOKEN_KEYS) expect(CONTRACT_PROPERTIES[key], key).toMatch(/^--\S+$/);
  });

  it('covers every token of a built-in pack, and nothing else', () => {
    const flat = flattenTokens(THEMES.paper);
    expect(Object.keys(flat).sort()).toEqual([...PUBLIC_TOKEN_KEYS].sort());
  });

  it('applies a full pack as contract names plus the legacy colour names', () => {
    const pairs = tokenProperties(flattenTokens(THEMES.blueprint));
    const map = new Map(pairs);
    expect(map.get(CONTRACT_PROPERTIES['color.bg'] as string)).toBe('#0b2a5b');
    expect(map.get('--bg')).toBe('#0b2a5b');
    expect(map.get('--border-strong')).toBe(THEMES.blueprint.color.borderStrong);
    // One contract name per token, one legacy name per colour.
    const colours = Object.keys(LEGACY_COLOR_PROPERTIES).length;
    expect(pairs).toHaveLength(PUBLIC_TOKEN_KEYS.length + colours);
  });

  it('skips unknown keys and non-string values from an untrusted stored theme', () => {
    const pairs = tokenProperties({
      'color.bg': '#123456',
      'color.nope': '#ffffff',
      'border.rule': 3,
      __proto__: { 'color.text': '#000000' },
    });
    expect(pairs).toEqual([
      [CONTRACT_PROPERTIES['color.bg'], '#123456'],
      ['--bg', '#123456'],
    ]);
  });
});
