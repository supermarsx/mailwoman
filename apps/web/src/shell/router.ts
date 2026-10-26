// Minimal hash-based shell router (plan §2.5 routes). The V2 shell switched its
// mail surfaces with a plain signal and never grew a router dependency; e10 keeps
// that lightness. This makes the four PIM modules deep-linkable and reachable
// (`#/calendar/:view?`, `#/tasks`, `#/notes/:id?`, `#/contacts/:id?`) while
// leaving Mail/Outbox/Attachments working exactly as before. No new dependency —
// pure `location.hash` + `hashchange`, inert under jsdom (which never fires
// hashchange unless the hash is set).

import { confirmShellNavigation } from './navigationGuard.ts';
import { createSignal, onCleanup, type Accessor } from 'solid-js';

/** Every top-level surface the shell can show (mail-family, the four PIM modules,
 *  and the V4 `keys` key-management module — plan §2.5). */
export type ShellSurface =
  | 'mail'
  | 'outbox'
  | 'attachments'
  | 'calendar'
  | 'tasks'
  | 'notes'
  | 'contacts'
  | 'keys';

/** The four V3 PIM module surfaces (plan §2.5) — the ones e10 mounts + wires. */
export const PIM_SURFACES: readonly ShellSurface[] = ['calendar', 'tasks', 'notes', 'contacts'];

const ALL_SURFACES = new Set<ShellSurface>([
  'mail',
  'outbox',
  'attachments',
  'calendar',
  'tasks',
  'notes',
  'contacts',
  // V4 (plan §2.5): the key-management module is a reachable, deep-linkable
  // `#/keys` surface — NOT a PIM surface (so `isPimSurface` stays false for it).
  'keys',
]);

/** Is `s` a PIM module surface (vs a mail-family surface)? */
export function isPimSurface(s: ShellSurface): boolean {
  return (PIM_SURFACES as readonly string[]).includes(s);
}

/** A parsed route: the surface plus its optional param (calendar view / notes|contacts id). */
export interface ShellRoute {
  surface: ShellSurface;
  param: string | null;
}

/** Decode a route parameter; a malformed escape is no parameter, not a throw. */
function decodeParam(raw: string): string | null {
  try {
    return decodeURIComponent(raw);
  } catch {
    return null;
  }
}

/** Parse a location hash into a route. Unknown/empty hashes resolve to Mail. */
export function parseHash(hash: string): ShellRoute {
  const raw = hash.replace(/^#/, '').replace(/^\//, '');
  const parts = raw.split('/');
  const head = parts[0] ?? '';
  const known = ALL_SURFACES.has(head as ShellSurface);
  const surface = (known ? head : 'mail') as ShellSurface;
  const rawParam = parts[1];
  const param = known && rawParam !== undefined && rawParam !== '' ? decodeParam(rawParam) : null;
  return { surface, param };
}

/**
 * Does `hash` address a surface that does not exist (`#/bogus`)? Only a
 * route-shaped fragment (`#/<head>`) with a non-empty head counts: the empty
 * hash and `#/` are the inbox, and a plain fragment such as `#main` is not a
 * route at all.
 */
export function isUnknownRoute(hash: string): boolean {
  if (!hash.startsWith('#/')) return false;
  const head = hash.slice(2).split('/')[0] ?? '';
  return head !== '' && !ALL_SURFACES.has(head as ShellSurface);
}

/** Build the canonical hash for a route (`#/calendar/week`, `#/tasks`). */
export function routeHash(surface: ShellSurface, param?: string | null): string {
  return param !== undefined && param !== null && param !== ''
    ? `#/${surface}/${encodeURIComponent(param)}`
    : `#/${surface}`;
}

export interface ShellRouter {
  /** The current route (reactive). */
  route: Accessor<ShellRoute>;
  /** Navigate to a surface (updates the hash; the mail surfaces keep their own state). */
  navigate(surface: ShellSurface, param?: string | null): void;
}

/**
 * A reactive hash router bound to `window.location.hash`. Constructed inside the
 * shell component so its `hashchange` listener is torn down with the component.
 *
 * An unknown route (`#/bogus`, at load or typed later) is replaced in place by
 * the inbox's `#/mail`, so Back does not return to it, and `onUnknown` is told
 * so the shell can say why the page changed (t32-h10).
 */
export function createShellRouter(onUnknown?: () => void): ShellRouter {
  /** The current hash, with an unknown route already replaced by the inbox. */
  function settledHash(): string {
    if (typeof location === 'undefined') return '';
    if (!isUnknownRoute(location.hash)) return location.hash;
    const inbox = routeHash('mail');
    history.replaceState(history.state, '', `${location.pathname}${location.search}${inbox}`);
    onUnknown?.();
    return inbox;
  }

  const [route, setRoute] = createSignal<ShellRoute>(parseHash(settledHash()));

  let request = 0;
  let deciding = false;
  let alive = true;
  onCleanup(() => { alive = false; request += 1; });
  function commit(h: string): void {
    if (typeof location !== 'undefined' && location.hash !== h) location.hash = h;
    setRoute(parseHash(h));
  }
  function propose(h: string): void {
    if (deciding) return;
    const ticket = ++request;
    const decision = confirmShellNavigation();
    if (typeof decision === 'boolean') { if (decision) commit(h); return; }
    deciding = true;
    void decision.then(allowed => { if (allowed && alive && ticket === request) commit(h); }).catch(() => undefined).finally(() => { deciding = false; });
  }
  if (typeof window !== 'undefined') {
    const onHash = (): void => {
      const h = settledHash();
      const current = routeHash(route().surface, route().param);
      if (h === current) return;
      // Keep the mounted editor and URL together until it permits leaving.
      history.replaceState(history.state, '', `${location.pathname}${location.search}${current}`);
      propose(h);
    };
    window.addEventListener('hashchange', onHash);
    onCleanup(() => window.removeEventListener('hashchange', onHash));
  }
  function navigate(surface: ShellSurface, param?: string | null): void {
    const h = routeHash(surface, param ?? null);
    if (h === routeHash(route().surface, route().param)) return;
    propose(h);
  }

  return { route, navigate };
}
