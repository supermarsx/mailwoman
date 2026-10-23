import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// The hard rule (plan §1.4, risk R2): `@tauri-apps/*` must NEVER enter the plain
// browser bundle. These are SOURCE-level guards backing the built-output check:
//   * no module imports a `@tauri-apps` package at all: `tauri.ts` reaches the
//     shell through the `__TAURI_INTERNALS__` object Tauri injects, and is itself
//     loaded lazily and only under isTauri().

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

  it('tauri.ts imports no @tauri-apps package and reaches the shell through __TAURI_INTERNALS__', () => {
    const src = read('./tauri.ts');
    // No import of a @tauri-apps package, static or dynamic: the webview has no
    // import map, so a bare specifier cannot resolve there (t31).
    expect(src).not.toMatch(/from\s+['"]@tauri-apps/);
    expect(src).not.toMatch(/import\(\s*(\/\*[^*]*\*\/\s*)?['"]@tauri-apps/);
    expect(src).not.toContain('@vite-ignore');
    expect(src).toContain('__TAURI_INTERNALS__');
  });
});
