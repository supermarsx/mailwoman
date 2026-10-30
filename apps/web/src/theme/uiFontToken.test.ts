// t101: "the font selection doesn't change anything" in the console's My
// preferences. The setting wrote `--mw-ui-font` on :root, but only the body rule
// read it; the console frame and sign-in gate restate `vars.font.ui`, which the
// packs bound to the bare pack face, so the console never changed font.
import { afterEach, describe, expect, it } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { THEMES } from './tokens.ts';
import { FONT_STACKS, type UiFont } from './appearance.ts';
import { UI_FONT_PROPERTY, emittedThemeTokens } from './uiFontToken.ts';
import { createAdminAppearance, type AdminAppearancePrefs } from '../screens/Admin/shell/appearanceStore.ts';

const SRC = resolve(__dirname, '..');
const source = (file: string): string => readFileSync(join(SRC, file), 'utf8').replace(/\/\*[\s\S]*?\*\/|\/\/.*$/gm, '');
function cssTsFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? cssTsFiles(path) : name.endsWith('.css.ts') ? [path] : [];
  });
}

afterEach(() => {
  document.documentElement.removeAttribute('style');
});

describe('interface font token', () => {
  it('every pack binds the UI face behind the font setting override', () => {
    for (const [name, tokens] of Object.entries(THEMES)) {
      const emitted = emittedThemeTokens(tokens);
      expect(emitted.font.ui, name).toBe(`var(--mw-ui-font, ${tokens.font.ui})`);
      expect(emitted.font.reading).toBe(tokens.font.reading);
      expect(emitted.font.mono).toBe(tokens.font.mono);
      expect(emitted.color).toBe(tokens.color);
    }
  });

  it('themes.css.ts registers every pack through the override', () => {
    const css = source('theme/themes.css.ts');
    const calls = [...css.matchAll(/createGlobalTheme\(([^;]*?)\);/g)].map((m) => m[1]!);
    expect(calls.length).toBeGreaterThanOrEqual(2);
    for (const args of calls) expect(args, args).toMatch(/,\s*vars,\s*emittedThemeTokens\(/);
  });

  it('the console frame and sign-in gate take the font from the token the setting reaches', () => {
    const restating = cssTsFiles(SRC)
      .filter((file) => /fontFamily:\s*vars\.font\.ui\b/.test(readFileSync(file, 'utf8')))
      .map((file) => relative(SRC, file).split(sep).join('/'));
    expect(restating).toEqual(expect.arrayContaining(['screens/Admin/admin.css.ts', 'screens/Admin/shell/shell.css.ts']));
    // No rule reads the raw override instead of the token (one source of truth).
    for (const file of cssTsFiles(SRC)) expect(source(relative(SRC, file)), relative(SRC, file)).not.toMatch(/var\(--mw-ui-font/);
  });

  it('the console preference writes the stack the token reads, live, for every option', () => {
    let saved: AdminAppearancePrefs | null = null;
    const c = createAdminAppearance({ store: { load: () => null, save: (p) => { saved = p; } }, webmail: () => null });
    try {
      for (const font of Object.keys(FONT_STACKS) as UiFont[]) {
        c.setFont(font);
        const stack = FONT_STACKS[font];
        expect(document.documentElement.style.getPropertyValue(UI_FONT_PROPERTY), font).toBe(stack ?? '');
        expect(saved).toMatchObject({ font });
      }
      c.hydrate({ theme: 'dark', accent: '', mode: 'dark', font: 'mono' });
      expect(document.documentElement.style.getPropertyValue(UI_FONT_PROPERTY)).toBe(FONT_STACKS.mono);
    } finally {
      c.dispose();
    }
  });
});
