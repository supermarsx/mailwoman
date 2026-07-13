import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// The hard rule (plan §1.4, risk R2): `@tauri-apps/*` must NEVER enter the plain
// browser bundle. These are SOURCE-level guards backing the built-output check:
//   * only `tauri.ts` may reference `@tauri-apps`;
//   * it may reference it ONLY through the non-literal, `@vite-ignore`d dynamic
//     import (so Vite never resolves/bundles it, and it loads lazily and only
//     under isTauri()) — never a static `import ... from '@tauri-apps/...'`.

function read(rel: string): string {
  return readFileSync(fileURLToPath(new URL(rel, import.meta.url)), 'utf8');
}

describe('tauri packages stay out of the browser bundle', () => {
  it('index.ts and browser.ts never import an @tauri-apps package', () => {
    // Match import specifiers (quoted), not prose in doc comments.
    for (const file of ['./index.ts', './browser.ts']) {
      const src = read(file);
      expect(src).not.toMatch(/from\s+['"]@tauri-apps/);
      expect(src).not.toMatch(/import\(\s*['"]@tauri-apps/);
    }
  });

  it('index.ts reaches tauri.ts only through a dynamic import', () => {
    const src = read('./index.ts');
    expect(src).not.toMatch(/import\s+[^;]*from\s+['"]\.\/tauri\.ts['"]/);
    expect(src).toContain("import('./tauri.ts')");
  });

  it('tauri.ts uses only the @vite-ignore dynamic-import escape hatch', () => {
    const src = read('./tauri.ts');
    // No static import of a @tauri-apps package.
    expect(src).not.toMatch(/import\s+[^;]*from\s+['"]@tauri-apps/);
    // Every @tauri-apps specifier is carried by the guarded loader.
    expect(src).toContain('/* @vite-ignore */');
    expect(src).toContain("loadTauri<TauriCore>('@tauri-apps/api/core')");
  });
});
