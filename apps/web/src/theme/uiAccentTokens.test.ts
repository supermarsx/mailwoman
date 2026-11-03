import { afterEach, describe, expect, it } from 'vitest';
import { vars } from './contract.css.ts';
import { THEMES } from './tokens.ts';
import { emittedThemeTokens, UI_SELECTION_ACCENT_PERCENT } from './uiFontToken.ts';
import { applyCustomAccent, customAccentShade } from './customAccent.ts';

afterEach(() => document.documentElement.removeAttribute('style'));
describe('shared interface accent states', () => {
  it('binds every pack focus ring and selected surface to the live accent', () => {
    for (const [name, tokens] of Object.entries(THEMES)) {
      const emitted = emittedThemeTokens(tokens);
      expect(emitted.color.focus, name).toBe(vars.color.accent);
      expect(emitted.a11y.focusRingColor, name).toBe(vars.color.accent);
      expect(emitted.a11y.focusRing, name).toContain(vars.color.accent);
      expect(emitted.a11y.focusRing, name).toContain(vars.color.bg);
      expect(emitted.color.selection, name).toBe(`color-mix(in srgb, ${vars.color.accent} ${UI_SELECTION_ACCENT_PERCENT}%, ${vars.color.bg})`);
      expect(emitted.color.signal, name).toBe(tokens.color.signal);
    }
  });
  it('lets a custom accent switch all dependent states without stale inline focus overrides', () => {
    const root = document.documentElement, property = vars.color.accent.slice(4, -1);
    for (const color of ['#ff00ff', '#12c0ae']) {
      applyCustomAccent(root, color, 'dark');
      expect(root.style.getPropertyValue(property)).toBe(customAccentShade(color, 'dark')!.accent);
      expect(root.style.getPropertyValue(vars.color.focus.slice(4, -1))).toBe('');
      expect(root.style.getPropertyValue(vars.a11y.focusRingColor.slice(4, -1))).toBe('');
    }
    applyCustomAccent(root, '', 'dark');
    expect(root.style.getPropertyValue(property)).toBe('');
  });
});
