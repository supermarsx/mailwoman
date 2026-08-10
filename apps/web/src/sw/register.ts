// Service-Worker registration, called from the app on startup (via the offline
// slice). Best-effort + feature-detected: absent under jsdom / older browsers,
// where the app simply runs online without a SW.

import { withBase, shellBase } from '../api/basePath.ts';

/**
 * Register the hand-rolled app-shell SW (`public/sw.js`). No-op when unsupported.
 *
 * Sub-path hosting (t20 B4): the script is registered at the PREFIXED path and
 * scoped to the prefix — `/mail/sw.js` with `scope: '/mail/'`. Both matter:
 *
 *   • The old `'/sw.js'` 404s under a prefix, because the server only serves the
 *     bundle beneath its base path.
 *   • The old `scope: '/'` is *rejected by the browser* for a script served from
 *     `/mail/`: a worker may not claim a scope above its own directory unless the
 *     response carries `Service-Worker-Allowed`. Scoping to `/mail/` is the
 *     script's default maximum scope, so no such header is needed — and the app
 *     lives entirely under the prefix anyway, so nothing is lost.
 *
 * At the origin root `withBase`/`shellBase` yield `/sw.js` and `/`, i.e. exactly
 * the previous call.
 */
export async function registerServiceWorker(): Promise<void> {
  if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return;
  try {
    await navigator.serviceWorker.register(withBase('/sw.js'), {
      type: 'classic',
      scope: shellBase(),
    });
  } catch {
    // Registration is best-effort; the app is fully functional online without it.
  }
}
