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
//
// The fragment may end in a query (t32-e71a): `#/<section>[/<rest>]?field=<id>`
// names a field to focus (`fieldAnchors.ts`, UX10), `#/audit?target=<kind>:<id>`
// filters a list. A move that changes only the query stays on the same page, so
// it does not ask the dirty-state guard.

import { createSignal, type Accessor } from 'solid-js';
import { basePath, stripBase } from '../../../api/basePath.ts';
import { consolePath } from '../../../api/adminPath.ts';

export interface AdminRoute {
  /** Section id, `dashboard` for the plain console page. */
  section: string;
  /** Segments after the section (decoded). */
  rest: string[];
  /** The fragment query (`#/audit?target=setting:x`), when there is one. */
  query?: Readonly<Record<string, string>>;
  /** The field to focus (`?field=<id>`), when the location names one. */
  field?: string;
}

/** Query parameters of a location's fragment. */
export type RouteQuery = Readonly<Record<string, string>>;

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
  const queryAt = fragment.indexOf('?');
  const pathPart = queryAt === -1 ? fragment : fragment.slice(0, queryAt);
  const parts = pathPart
    .replace(/^\/+/, '')
    .split('/')
    .filter((p) => p !== '')
    .map(decode);
  const [section = 'dashboard', ...rest] = parts;
  const route: AdminRoute = { section, rest };
  if (queryAt !== -1) {
    const query: Record<string, string> = {};
    for (const [k, v] of new URLSearchParams(fragment.slice(queryAt + 1))) query[k] = v;
    if (Object.keys(query).length > 0) route.query = query;
    const field = query['field'];
    if (field !== undefined && field !== '') route.field = field;
  }
  return route;
}

/** The app location of a section (`dashboard` is the plain console page), with an
 *  optional fragment query (`{field: 'smtp.port'}` → `#/<section>?field=smtp.port`). */
export function sectionPath(section: string, rest: readonly string[] = [], query?: RouteQuery): string {
  const root = consolePath();
  const qs = query === undefined ? '' : new URLSearchParams(Object.entries(query)).toString();
  if (section === 'dashboard' && rest.length === 0 && qs === '') return root;
  return `${root}#/${[section, ...rest].map(encodeURIComponent).join('/')}${qs === '' ? '' : `?${qs}`}`;
}

/** The location of a field: its section and sub-page with `?field=<id>`. */
export function fieldPath(section: string, field: string, rest: readonly string[] = []): string {
  return sectionPath(section, rest, { field });
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
  /** A location reduced to the page it shows (the query left out). */
  const page = (loc: string): string => {
    const r = parseAdminPath(loc);
    return sectionPath(r.section, r.rest);
  };
  /** A location reduced to what identifies the route, query included. */
  const key = (loc: string): string => {
    const r = parseAdminPath(loc);
    return sectionPath(r.section, r.rest, r.query);
  };
  const [route, setRoute] = createSignal<AdminRoute>(parseAdminPath(current()));
  let shown = key(current());

  async function navigate(path: string, opts: { replace?: boolean } = {}): Promise<boolean> {
    if (key(path) === shown) return true;
    // Only the query changes (a field deep link on the open page): no leave question.
    if (page(path) !== page(shown) && !(await leaveCheck())) return false;
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
    if (page(target) === page(shown)) {
      shown = target;
      setRoute(parseAdminPath(current()));
      return;
    }
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
