// Minimal hash-based shell router (plan §2.5 routes). The V2 shell switched its
// mail surfaces with a plain signal and never grew a router dependency; e10 keeps
// that lightness. This makes the four PIM modules deep-linkable and reachable
// (`#/calendar/:view?`, `#/tasks`, `#/notes/:id?`, `#/contacts/:id?`) while
// leaving Mail/Outbox/Attachments working exactly as before. No new dependency —
// pure `location.hash` + `hashchange`, inert under jsdom (which never fires
// hashchange unless the hash is set).

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

/** Parse a location hash into a route. Unknown/empty hashes resolve to Mail. */
export function parseHash(hash: string): ShellRoute {
  const raw = hash.replace(/^#/, '').replace(/^\//, '');
  const parts = raw.split('/');
  const head = parts[0] ?? '';
  const known = ALL_SURFACES.has(head as ShellSurface);
  const surface = (known ? head : 'mail') as ShellSurface;
  const rawParam = parts[1];
  const param = known && rawParam !== undefined && rawParam !== '' ? decodeURIComponent(rawParam) : null;
  return { surface, param };
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
 */
export function createShellRouter(): ShellRouter {
  const initial = typeof location !== 'undefined' ? location.hash : '';
  const [route, setRoute] = createSignal<ShellRoute>(parseHash(initial));

  if (typeof window !== 'undefined') {
    const onHash = (): void => {
      setRoute(parseHash(location.hash));
    };
    window.addEventListener('hashchange', onHash);
    onCleanup(() => window.removeEventListener('hashchange', onHash));
  }

  function navigate(surface: ShellSurface, param?: string | null): void {
    const h = routeHash(surface, param ?? null);
    if (typeof location !== 'undefined') {
      // Setting the hash fires `hashchange` → setRoute; if it's already the
      // target hash the event won't fire, so update the signal directly.
      if (location.hash !== h) location.hash = h;
      else setRoute(parseHash(h));
    } else {
      setRoute(parseHash(h));
    }
  }

  return { route, navigate };
}
