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

  it('App reaches the Admin screen via lazy(() => import(...))', () => {
    expect(app).toMatch(/lazy\(\s*\(\)\s*=>\s*import\(['"]\.\/screens\/Admin\/index\.tsx['"]\)\s*\)/);
  });

  it('App does NOT statically import the Admin screen', () => {
    expect(app).not.toMatch(/^import[^\n]*['"]\.\/screens\/Admin[^\n]*$/m);
  });

  it('App gates the admin route on the /admin path', () => {
    expect(app).toMatch(/isAdminRoute/);
  });
});
