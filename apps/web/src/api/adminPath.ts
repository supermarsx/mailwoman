// The admin console's location (t32-h15) — the ONE resolver for it.
//
// The operator chooses where the console lives (`MW_ADMIN_PATH`, default `/admin`),
// and the server must never send that choice to anyone outside it: not in the
// bundle, not in `index.html` for the mail client, not in the service worker. So it
// is not compiled in and not published. The console page instead carries a marker
// the server writes only into the page it serves at the admin path:
//
//   <meta name="mw-console" content="admin">
//
// and the console's own address IS its location. Every admin API URL and every
// console link is built from `location.pathname` of the page that carries the
// marker, never from a literal.
//
// Without the marker (the Vite dev server, a server older than t32-h15) the console
// is where it always was: `/admin` under the deploy prefix.
//
// Like `basePath.ts`, this module imports only that file, so it can be used from
// anywhere without an import cycle.

import { basePath, stripBase } from './basePath.ts';

/** Where the console is when the page does not say otherwise. */
export const DEFAULT_CONSOLE_PATH = '/admin';

/** Does the served page carry the console marker? */
function hasConsoleMarker(): boolean {
  if (typeof document === 'undefined') return false;
  const meta = document.querySelector('meta[name="mw-console"]');
  return meta?.getAttribute('content') === 'admin';
}

/** `pathname` with the deploy prefix and any trailing slashes removed. */
function appPath(pathname: string): string {
  return stripBase(pathname).replace(/\/+$/, '') || '/';
}

/**
 * The console page relative to the deploy prefix: `/admin` by default, the
 * operator's path (e.g. `/ops-7f3k`) when the server marked this page as the
 * console. Admin API routes hang off it (`<base><console>/users`), and console
 * sections off its fragment (`<console>#/users`).
 */
export function consolePath(): string {
  if (hasConsoleMarker() && typeof location !== 'undefined') {
    const p = appPath(location.pathname);
    if (p !== '/') return p;
  }
  return DEFAULT_CONSOLE_PATH;
}

/** Is this page the admin console (rather than the mail app or the consent screen)? */
export function isConsolePage(): boolean {
  if (hasConsoleMarker()) return true;
  if (typeof location === 'undefined') return false;
  const p = appPath(location.pathname);
  return p === DEFAULT_CONSOLE_PATH || p.startsWith(`${DEFAULT_CONSOLE_PATH}/`);
}

/**
 * An admin API URL: `path` (starting with `/`, e.g. `/users`) under the console,
 * under `base` (the deploy prefix by default; a native shell passes its server).
 */
export function adminUrl(path: string, base: string = basePath()): string {
  return `${base}${consolePath()}${path}`;
}

/** Is `appLocation` (deploy prefix removed) the console page or one of its sections? */
export function isConsoleLocation(appLocation: string): boolean {
  const root = consolePath();
  return appLocation === root || appLocation.startsWith(`${root}#`) || appLocation.startsWith(`${root}/`);
}
