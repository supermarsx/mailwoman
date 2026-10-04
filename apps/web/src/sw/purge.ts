// Page-side half of the logout purge (t28-e3). The service worker deletes its
// caches when it sees the logout request (`public/sw.js`, LOGOUT_PATHS) — but
// only for a page it controls, and a hard reload (Shift+Reload) leaves the page
// uncontrolled for the rest of its life. Cache Storage is shared by the page and
// the worker, so the page can delete the same caches itself.

/** Every cache `public/sw.js` creates starts with this (see its purgeCaches()). */
const CACHE_PREFIX = 'mw-';

/**
 * Delete every Mailwoman cache in Cache Storage. Best-effort + feature-detected:
 * a no-op where the Cache API is absent (jsdom, insecure origins), and a failure
 * never rejects, so it cannot break the logout that calls it.
 */
export async function purgeShellCaches(): Promise<void> {
  if (typeof caches === 'undefined') return;
  try {
    const names = await caches.keys();
    await Promise.all(
      names.filter((name) => name.startsWith(CACHE_PREFIX)).map((name) => caches.delete(name)),
    );
  } catch {
    // Storage unavailable (private mode, blocked site data): nothing was cached.
  }
}
