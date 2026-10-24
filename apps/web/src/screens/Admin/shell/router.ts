// Admin console routing (t32-admin.md §2.1).
//
// Sections are addressed in the URL fragment: `/admin#/<section>[/<rest>]`, the
// dashboard at plain `/admin`. Two things rule out real path segments such as
// `/admin/users`:
//   • the JSON API owns that space (`GET /admin/users`, `/admin/domains`,
//     `/admin/plugins`, ...), so a reload there gets the API's answer, not the app;
//   • `index.html` loads its chunks through relative URLs (`./assets/...`, the
//     relative Vite base that sub-path hosting needs), so a page at any depth below
//     `/admin` would ask for `/admin/<x>/assets/...` and never start.
// The fragment never reaches the server, so neither applies, and it works the same
// under `MW_BASE_PATH`.
//
// `/admin` is the default. The operator may move the console (`MW_ADMIN_PATH`,
// t32-h15); the console page is then wherever the server served it, and every
// location below is built from `consolePath()` rather than a literal.
//
// Navigation is `history.pushState`, so a section is linkable and Back works.
// Before an in-app navigation the router asks the dirty-state guard
// (`useDirtyGuard.ts`); a guarded navigation waits for the admin's answer.

import { createSignal, type Accessor } from 'solid-js';
import { basePath, stripBase } from '../../../api/basePath.ts';
import { consolePath } from '../../../api/adminPath.ts';

export interface AdminRoute {
  /** Section id, `dashboard` for the plain console page. */
  section: string;
  /** Segments after the section (decoded). */
  rest: string[];
}

function decode(p: string): string {
  try {
    return decodeURIComponent(p);
  } catch {
    return p;
  }
}

/** Parse an app location (deploy prefix removed, fragment included) into a route. */
export function parseAdminPath(location: string): AdminRoute {
  const hashAt = location.indexOf('#');
  const fragment = hashAt === -1 ? '' : location.slice(hashAt + 1);
  const parts = fragment
    .replace(/^\/+/, '')
    .split('/')
    .filter((p) => p !== '')
    .map(decode);
  const [section = 'dashboard', ...rest] = parts;
  return { section, rest };
}

/** The app location of a section (`dashboard` is the plain console page). */
export function sectionPath(section: string, rest: readonly string[] = []): string {
  const root = consolePath();
  if (section === 'dashboard' && rest.length === 0) return root;
  return `${root}#/${[section, ...rest].map(encodeURIComponent).join('/')}`;
}

/** The href for an app location, with the deploy prefix. */
export function hrefFor(path: string): string {
  return `${basePath()}${path}`;
}

export interface AdminRouter {
  route: Accessor<AdminRoute>;
  /** Navigate to an app location. Resolves false when the dirty guard kept the page. */
  navigate(path: string, opts?: { replace?: boolean }): Promise<boolean>;
  dispose(): void;
}

/** Asked before leaving the current page; resolves true to allow. */
export type LeaveCheck = () => Promise<boolean>;

export function createAdminRouter(leaveCheck: LeaveCheck = async () => true): AdminRouter {
  const current = (): string => `${stripBase(location.pathname)}${location.hash}`;
  /** A location reduced to what identifies the route. */
  const key = (loc: string): string => {
    const r = parseAdminPath(loc);
    return sectionPath(r.section, r.rest);
  };
  const [route, setRoute] = createSignal<AdminRoute>(parseAdminPath(current()));
  let shown = key(current());

  async function navigate(path: string, opts: { replace?: boolean } = {}): Promise<boolean> {
    if (key(path) === shown) return true;
    if (!(await leaveCheck())) return false;
    if (opts.replace) history.replaceState(null, '', hrefFor(path));
    else history.pushState(null, '', hrefFor(path));
    shown = key(path);
    setRoute(parseAdminPath(path));
    return true;
  }

  // Back/Forward, or a fragment typed into the address bar. The browser has already
  // moved; when the guard keeps the page, the shown entry is pushed back so the
  // address bar matches what is on screen.
  // A fragment move fires both `popstate` and `hashchange`; `asking` keeps the
  // second from opening a second question while the first is open.
  let asking = false;
  function onMove(): void {
    const target = key(current());
    if (target === shown || asking) return;
    asking = true;
    void leaveCheck().then((ok) => {
      asking = false;
      if (ok) {
        shown = key(current());
        setRoute(parseAdminPath(current()));
      } else {
        history.pushState(null, '', hrefFor(shown));
      }
    });
  }
  window.addEventListener('popstate', onMove);
  window.addEventListener('hashchange', onMove);

  return {
    route,
    navigate,
    dispose: () => {
      window.removeEventListener('popstate', onMove);
      window.removeEventListener('hashchange', onMove);
    },
  };
}
