import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// Source-level guard (mirrors the viewers' lazy-import gate, plan §1.7): the admin
// panel must be reached ONLY via `lazy(() => import('./screens/Admin/index.tsx'))`
// so the whole `screens/Admin/**` tree code-splits into its own chunk and stays
// OUT of the login→inbox mailbox entry bundle. If App.tsx ever statically imports
// the Admin screen, the bundler would fold it into the entry chunk — this fails.

function read(rel: string): string {
  return readFileSync(fileURLToPath(new URL(rel, import.meta.url)), 'utf8');
}

describe('the /admin route is lazily loaded (code-split off the mailbox bundle)', () => {
  const app = read('../../App.tsx');

  // t22-e10: this asserted the literal `lazy(() => import('...'))`. The property
  // it exists to protect is that the Admin tree is reached through a DYNAMIC
  // import, which is what makes the bundler split it out — and that is unchanged.
  // What changed is the shape: App now names the loader (`() => import(...)`) and
  // hands it to `LazyRoute`, which rebuilds a fresh `lazy()` per attempt so a
  // failed chunk load can actually be retried. Solid memoises a lazy's rejection,
  // so a module-level `lazy()` cannot be retried at all.
  //
  // Asserting on the dynamic import rather than on the wrapper keeps the guard
  // pointed at the property instead of at one spelling of it.
  it('App reaches the Admin screen via a dynamic import', () => {
    expect(app).toMatch(/\(\)\s*=>\s*import\(['"]\.\/screens\/Admin\/index\.tsx['"]\)/);
  });

  it('App does NOT statically import the Admin screen', () => {
    expect(app).not.toMatch(/^import[^\n]*['"]\.\/screens\/Admin[^\n]*$/m);
  });

  it('App gates the admin route on the /admin path', () => {
    expect(app).toMatch(/isAdminRoute/);
  });

  // t20-e12b: under `MW_BASE_PATH=/mail` the browser is at `/mail/admin`. Matching the
  // RAW pathname against `/admin` fails there, and because these are early returns the
  // failure is SILENT — the admin console and the OAuth consent screen would render the
  // mailbox instead of erroring. Both matchers must therefore strip the deploy prefix
  // first. Source-level guard, in this file's existing idiom (the matchers are module-
  // private, so there is nothing to call).
  it('both route matchers strip the deploy prefix before comparing', () => {
    const matchers = app.match(/function is(?:Admin|OAuthAuthorize)Route\(\)[^}]*}/g);
    expect(matchers).toHaveLength(2);
    for (const fn of matchers ?? []) {
      expect(fn).toMatch(/stripBase\(\s*location\.pathname\s*\)/);
      expect(fn).not.toMatch(/(?<!stripBase\()\blocation\.pathname\.replace/);
    }
  });
});
