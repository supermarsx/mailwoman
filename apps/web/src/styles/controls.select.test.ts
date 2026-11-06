// The native <select> popup is painted by the browser, so the base stylesheet has to hand it
// the theme: color-scheme per appearance, option/optgroup colours from tokens, forced colours.
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const src = readFileSync(resolve('src/styles/controls.css.ts'), 'utf8');
const block = (selector: string): string => {
  const at = src.indexOf(`globalStyle('${selector}'`);
  expect(at, selector).toBeGreaterThan(-1);
  return src.slice(at, src.indexOf('\n});', at) + 4);
};

describe('base select styling', () => {
  it('sets color-scheme from the active theme appearance', () => {
    expect(block(':root[data-appearance="light"]')).toContain("colorScheme: 'light'");
    expect(block(':root[data-appearance="dark"]')).toContain("colorScheme: 'dark'");
  });
  it('colours options and optgroups with tokens, never literals', () => {
    const rule = block(':where(select option, select optgroup)');
    expect(rule).toContain('vars.color.bgSink');
    expect(rule).toContain('vars.color.text');
    expect(block(':where(select option:checked)')).toContain('vars.color.accent');
    expect(block(':where(select option:hover)')).toContain('vars.color.bgAlt');
    expect(block(':where(select optgroup)')).toContain('vars.color.textDim');
    expect(block(':where(select option:disabled, select optgroup:disabled)')).toContain('vars.color.textDim');
    expect(src).not.toMatch(/select[\s\S]{0,200}#[0-9a-fA-F]{3,8}\b/);
  });
  it('has a focus ring and forced-colors support for select and options', () => {
    expect(block(':where(select:focus-visible)')).toContain('focusRingColor');
    expect(src).toMatch(/forced-colors: active[\s\S]{0,120}CanvasText/);
    expect(src).toContain("'Highlight'");
  });
  it('the combobox keeps highlighted rows on token colours', () => {
    const css = readFileSync(resolve('src/components/combobox.css.ts'), 'utf8');
    expect(css).toMatch(/\[data-active="true"\]`, \{ background: vars\.color\.bgAlt, color: vars\.color\.text/);
    expect(css).toContain("'Highlight'");
    expect(css).toContain("'Canvas'");
  });
});
