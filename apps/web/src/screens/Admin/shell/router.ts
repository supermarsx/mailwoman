// Admin console path routing (t32-admin.md §2.1).
//
// Sections live at `/admin/console/<section>[/<rest>]`, the dashboard at `/admin`.
// They cannot live at `/admin/<section>` as first drawn: the JSON API owns that
// space (`GET /admin/users`, `/admin/domains`, `/admin/plugins`, ...), and a
// reload of `/admin/users` would be answered by the API handler, not the app.
// No server route is mounted under `/admin/console`, so a reload there reaches
// the SPA fallback (`wants_spa_shell`, crates/mw-server/src/lib.rs) and gets the
// shell. Server lanes must keep that prefix free.
//
// Navigation is `history.pushState`, so a section is linkable and Back works.
// Before an in-app navigation the router asks the dirty-state guard
// (`useDirtyGuard.ts`); a guarded navigation waits for the admin's answer.

import { createSignal, type Accessor } from 'solid-js';
import { basePath, stripBase } from '../../../api/basePath.ts';

export const CONSOLE_PREFIX = '/admin/console';

export interface AdminRoute {
  /** Section id, `dashboard` for `/admin` and `/admin/console`. */
  section: string;
  /** Path segments after the section (decoded). */
  rest: string[];
}

/** Parse an app path (deploy prefix already removed) into a route. */
export function parseAdminPath(path: string): AdminRoute {
  const clean = path.replace(/\/+$/, '');
  if (!clean.startsWith(CONSOLE_PREFIX + '/')) return { section: 'dashboard', rest: [] };
  const parts = clean
    .slice(CONSOLE_PREFIX.length + 1)
    .split('/')
    .filter((p) => p !== '')
    .map((p) => {
      try {
        return decodeURIComponent(p);
      } catch {
        return p;
      }
    });
  const [section = 'dashboard', ...rest] = parts;
  return { section, rest };
}

/** The app path of a section (`dashboard` is `/admin`). */
export function sectionPath(section: string, rest: readonly string[] = []): string {
  if (section === 'dashboard' && rest.length === 0) return '/admin';
  return [CONSOLE_PREFIX, section, ...rest].map((p, i) => (i === 0 ? p : encodeURIComponent(p))).join('/');
}

/** The href for an app path, with the deploy prefix. */
export function hrefFor(path: string): string {
  return `${basePath()}${path}`;
}

export interface AdminRouter {
  route: Accessor<AdminRoute>;
  /** Navigate to an app path. Resolves false when the dirty guard kept the page. */
  navigate(path: string, opts?: { replace?: boolean }): Promise<boolean>;
  dispose(): void;
}

/** Asked before leaving the current page; resolves true to allow. */
export type LeaveCheck = () => Promise<boolean>;

export function createAdminRouter(leaveCheck: LeaveCheck = async () => true): AdminRouter {
  const current = (): string => stripBase(location.pathname);
  const [route, setRoute] = createSignal<AdminRoute>(parseAdminPath(current()));
  let shownPath = current();

  async function navigate(path: string, opts: { replace?: boolean } = {}): Promise<boolean> {
    if (path === shownPath) return true;
    if (!(await leaveCheck())) return false;
    if (opts.replace) history.replaceState(null, '', hrefFor(path));
    else history.pushState(null, '', hrefFor(path));
    shownPath = path;
    setRoute(parseAdminPath(path));
    return true;
  }

  // Back/Forward. The browser has already moved; when the guard keeps the page,
  // the previous entry is pushed back so the address bar matches what is shown.
  function onPop(): void {
    const target = current();
    if (target === shownPath) return;
    void leaveCheck().then((ok) => {
      if (ok) {
        shownPath = target;
        setRoute(parseAdminPath(target));
      } else {
        history.pushState(null, '', hrefFor(shownPath));
      }
    });
  }
  window.addEventListener('popstate', onPop);

  return {
    route,
    navigate,
    dispose: () => window.removeEventListener('popstate', onPop),
  };
}
