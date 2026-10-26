import { describe, expect, it } from 'vitest';
import { appearanceScriptProperties } from './appearanceScriptPolicy.ts';
describe('isolated appearance API', () => {
  it('never emits held fields or arbitrary values', () => {
    expect(appearanceScriptProperties({accent: '#ff0000', mode: 'light', font: 'mono', density: 'compact'}, ['accent','mode','font','density'], 'dark')).toEqual([]);
    for (const update of [{accent: 'url(https://attacker.test)'}, {font: 'constructor'}, {density: 'inherit'}, {mode: 'system;display:none'}]) expect(() => appearanceScriptProperties(update, [], 'dark')).toThrow();
  });
  it('mode cannot indirectly replace locked accent, font, or density', () => {
    const output = appearanceScriptProperties({mode: 'light'}, ['accent','font','density'], 'dark');
    expect(output.some(([name]) => ['--accent','--accent-text','--mw-ui-font','font-size'].includes(name))).toBe(false);
  });
  it('maps supported choices only to trusted theme properties', () => {
    const output = appearanceScriptProperties({mode: 'light', accent: '#3366cc', density: 'compact', font: 'mono'}, [], 'dark');
    expect(output.length).toBeGreaterThan(10);
    expect(output).toContainEqual(['font-size','13px']);
    expect(output.some(([name]) => name === '--accent')).toBe(true);
    expect(output.every(([, value]) => !value.includes('url('))).toBe(true);
  });
});
