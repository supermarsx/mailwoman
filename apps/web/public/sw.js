// Mailwoman app-shell Service Worker (plan §2.5, owned by e5). Hand-rolled — no
// Workbox. Ships copied verbatim into dist/ (no bundling), so it cannot import
// from src/. This file is the only implementation of its routing: the unit tests
// (src/sw/sw.test.ts) load and run THIS file, there is no second copy to keep in
// sync.
//
// What may be stored (t28-e3). Cache Storage is plaintext on disk and the Cache
// API ignores HTTP cache directives, so a response header cannot opt out — the
// decision is made here, in chooseStrategy(), and nowhere else:
//   - cache-first  for hashed build assets + fonts (immutable, not user data)
//   - network-first for the two PUBLIC, pre-auth config endpoints in
//     CACHEABLE_API (cache is the offline fallback)
//   - offline navigation → the stored app shell
//   - everything else → not handled by this worker at all (plain network).
//     That includes every other /jmap/* and /api/* path: /jmap/download/*
//     (attachment bytes, whole .eml source), /api/export/*, /api/me,
//     /jmap/session, /jmap/eventsource. None of it is ever written to a cache.
// Only a complete 200 is stored (never a 206 partial), entries are capped in
// number and age (LIMITS), and every `mw-*` cache is deleted when the page sends
// a logout request (LOGOUT_PATHS).

// Must equal shellCacheName(SHELL_CACHE_VERSION) from src/contracts/offline.ts.
const CACHE_NAME = 'mw-shell-v1';

// ── Sub-path hosting (t20 B4) ──────────────────────────────────────────────
// The deploy prefix is DERIVED, not templated. This file ships copied verbatim
// into dist/ and is served from wherever the app's base is — `/sw.js` at the
// origin root, `/mail/sw.js` under `MW_BASE_PATH=/mail` — so its own location IS
// the prefix. `new URL('./', self.location).pathname` is `/` or `/mail/` exactly.
//
// Deriving beats server-side templating here: the SW cannot read the
// `__MW_BASE__` injected into index.html (different global scope), and a derived
// value can never disagree with the scope the worker was actually registered
// under. It also means the server serves this file byte-for-byte.
const SHELL_URL = new URL('./', self.location).pathname;
const SHELL_URLS = [SHELL_URL];
// `SHELL_URL` always ends in '/', so BASE is '' at the root and '/mail' under a
// prefix — matching stripBase() in src/api/basePath.ts.
const BASE = SHELL_URL.slice(0, -1);

/** Drop the deploy prefix so the matchers below stay prefix-blind. */
function stripBase(pathname) {
  if (BASE === '' || !pathname.startsWith(BASE)) return pathname;
  const rest = pathname.slice(BASE.length);
  if (rest === '') return '/';
  return rest.startsWith('/') ? rest : pathname;
}

function isApiPath(pathname) {
  return pathname.startsWith('/jmap/') || pathname.startsWith('/api/');
}

// The ONLY API responses that may be stored. Both are served before login and
// are the same for every visitor: the IdP buttons the login screen shows
// (id, kind, display name) and the VAPID public key. Anything that needs a
// session stays off this list.
const CACHEABLE_API = ['/api/sso/providers', '/api/push/vapid'];

// A query string disqualifies the request: `/api/sso/providers?domain=…` would
// record the domain the user typed as part of the cache key.
function isCacheableApi(pathname, search) {
  return search === '' && CACHEABLE_API.includes(pathname);
}

// Requests that end the mailbox session (`client.logout()` in src/api/client.ts
// → POST /api/logout; the SSO variant is POST /api/sso/logout in mw-server's
// sso.rs). Seeing one deletes every `mw-*` cache.
const LOGOUT_PATHS = ['/api/logout', '/api/sso/logout'];

function isLogout(request) {
  if (request.method !== 'POST') return false;
  const url = new URL(request.url);
  return url.origin === self.location.origin && LOGOUT_PATHS.includes(stripBase(url.pathname));
}

function isFont(pathname) {
  return /\.(?:woff2?|ttf|otf)$/i.test(pathname);
}

function isHashedAsset(pathname) {
  return /-[A-Za-z0-9_]{8,}\.[a-z0-9]+$/i.test(pathname) || pathname.startsWith('/assets/');
}

// `index.html` returned for a SUBRESOURCE request must never be cached (t24-e13).
// mw-server answers any unmatched path with the SPA shell under a 200, so a miss on
// /fonts/*.woff2 comes back 200 text/html; the status cannot tell that from a real
// hit, and cache-first never revalidates — so the HTML would be served as the font
// until the entry aged out, outliving the operator running `mailwoman fonts pull`.
function isSpaFallbackForSubresource(request, res) {
  if (request.mode === 'navigate') return false;
  return (res.headers.get('content-type') || '').toLowerCase().includes('text/html');
}

// Reads only `method`, `url` and `mode`, so it classifies a stored cache key as
// well as a live request (sweep() relies on that).
function chooseStrategy(request) {
  if (request.method !== 'GET') return 'passthrough';
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return 'passthrough';
  const pathname = stripBase(url.pathname);
  // Checked BEFORE the navigation and asset rules: an attachment opened in a new
  // tab is a navigation, and `/jmap/download/a/b/report-2026final.pdf` has the
  // shape of a hashed asset. Neither may fall through to a caching strategy.
  if (isApiPath(pathname)) {
    return isCacheableApi(pathname, url.search) ? 'network-first' : 'passthrough';
  }
  if (request.mode === 'navigate') return 'shell-fallback';
  if (isHashedAsset(pathname) || isFont(pathname)) return 'cache-first';
  return 'passthrough';
}

// ── Bounds (t28-e3) ────────────────────────────────────────────────────────
// Each stored response carries the time it was stored in this header; an entry
// without it (anything a pre-t28 worker wrote) counts as infinitely old.
const STAMP_HEADER = 'x-mw-cached-at';
const DAY_MS = 24 * 60 * 60 * 1000;
// Per strategy: how many entries may stay and for how long. A full build emits
// ~250 hashed assets, so 512 holds the current build plus the previous one.
const LIMITS = {
  'network-first': { maxEntries: 8, maxAgeMs: DAY_MS },
  'cache-first': { maxEntries: 512, maxAgeMs: 30 * DAY_MS },
};
// sweep() reads every entry, so it runs at most this often after a store.
const SWEEP_INTERVAL_MS = 60 * 1000;
let lastSweep = 0;

function isShellKey(request) {
  const url = new URL(request.url);
  return url.origin === self.location.origin && url.pathname === SHELL_URL && url.search === '';
}

function storedAt(res) {
  return Number(res.headers.get(STAMP_HEADER)) || 0;
}

function isFresh(res, strategy) {
  return Date.now() - storedAt(res) <= LIMITS[strategy].maxAgeMs;
}

// Only a complete 200 is stored. `res.ok` would also admit a 206 partial, which
// `cache.put` rejects with a TypeError.
function isStorable(request, res) {
  return res.status === 200 && !isSpaFallbackForSubresource(request, res);
}

/**
 * Store a copy of `res` under `key`, stamped with the current time. Best-effort:
 * a failed write (quota, `Vary: *`) must not fail the request being answered.
 */
async function store(cache, key, res) {
  try {
    const copy = res.clone();
    const stamped = new Response(copy.body, {
      status: copy.status,
      statusText: copy.statusText,
      headers: copy.headers,
    });
    stamped.headers.set(STAMP_HEADER, String(Date.now()));
    await cache.put(key, stamped);
    if (Date.now() - lastSweep >= SWEEP_INTERVAL_MS) await sweep(cache);
  } catch {
    // Not stored; the caller still returns the network response.
  }
}

/**
 * Enforce what may stay in the cache: delete every entry that chooseStrategy()
 * would not store today (this is what removes the attachment and .eml responses
 * a pre-t28 worker wrote into the same cache), every entry past its age limit,
 * and the oldest entries beyond the per-strategy cap. The app shell entry is
 * exempt — it is replaced on every online navigation to the shell URL.
 */
async function sweep(cache) {
  lastSweep = Date.now();
  const kept = { 'network-first': [], 'cache-first': [] };
  for (const request of await cache.keys()) {
    if (isShellKey(request)) continue;
    const strategy = chooseStrategy({ method: 'GET', url: request.url, mode: '' });
    const res = strategy in LIMITS ? await cache.match(request) : undefined;
    if (res === undefined || !isFresh(res, strategy)) {
      await cache.delete(request);
      continue;
    }
    kept[strategy].push({ request, at: storedAt(res) });
  }
  for (const strategy of Object.keys(kept)) {
    const entries = kept[strategy].sort((a, b) => a.at - b.at);
    const excess = entries.length - LIMITS[strategy].maxEntries;
    for (const entry of entries.slice(0, Math.max(0, excess))) {
      await cache.delete(entry.request);
    }
  }
}

/** Delete every Mailwoman cache, shell and assets included. */
async function purgeCaches() {
  const names = await caches.keys();
  await Promise.all(
    names.filter((name) => name.startsWith('mw-')).map((name) => caches.delete(name)),
  );
}

async function networkFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  try {
    const res = await fetch(request);
    if (isStorable(request, res)) await store(cache, request, res);
    return res;
  } catch (err) {
    const cached = await cache.match(request);
    if (cached && isFresh(cached, 'network-first')) return cached;
    throw err;
  }
}

// A hit is served without an age check: these URLs are content-hashed, and the
// age limit is applied by sweep(), which only runs after a successful store —
// that is, while online, when a deleted asset can be fetched again.
async function cacheFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);
  if (cached) return cached;
  const res = await fetch(request);
  if (isStorable(request, res)) await store(cache, request, res);
  return res;
}

function isShellDocument(request, res) {
  return (
    res.status === 200 &&
    isShellKey(request) &&
    (res.headers.get('content-type') || '').toLowerCase().includes('text/html')
  );
}

async function shellFallback(request) {
  try {
    const res = await fetch(request);
    // Keep the stored shell current, and put it back after a logout purge.
    if (isShellDocument(request, res)) {
      await store(await caches.open(CACHE_NAME), SHELL_URL, res);
    }
    return res;
  } catch (err) {
    const cache = await caches.open(CACHE_NAME);
    const shell = await cache.match(SHELL_URL);
    if (shell) return shell;
    throw err;
  }
}

function respond(request, strategy) {
  switch (strategy) {
    case 'network-first':
      return networkFirst(request);
    case 'cache-first':
      return cacheFirst(request);
    default:
      return shellFallback(request);
  }
}

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      // Precache the app shell entries individually so one 404 can't fail install.
      await Promise.all(
        SHELL_URLS.map(async (url) => {
          try {
            const res = await fetch(url);
            if (res.status === 200) await store(cache, url, res);
          } catch {
            // Offline at install: the shell is stored on the next navigation.
          }
        }),
      );
      await self.skipWaiting();
    })(),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      // Drop superseded shell caches (mw-shell-v{older}).
      const names = await caches.keys();
      await Promise.all(
        names
          .filter((name) => name.startsWith('mw-shell-v') && name !== CACHE_NAME)
          .map((name) => caches.delete(name)),
      );
      // The current cache keeps its name across worker versions, so an upgrade
      // must also drop whatever the previous worker stored that this one would
      // not (see sweep()).
      await sweep(await caches.open(CACHE_NAME));
      await self.clients.claim();
    })(),
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  // Purge on the logout REQUEST, not on its response: the client drops its
  // session state whether or not the call succeeds (logout() in
  // src/state/slices/mail.ts clears in a `finally`).
  if (isLogout(request)) event.waitUntil(purgeCaches());
  // Not calling respondWith() leaves the request to the browser: writes
  // (POST /jmap/api etc.), downloads, the event stream and everything else
  // chooseStrategy() does not store never pass through this worker's fetch().
  const strategy = chooseStrategy(request);
  if (strategy === 'passthrough') return;
  event.respondWith(respond(request, strategy));
});

// ── Web Push wake (V5, plan §2.3) ──────────────────────────────────────────
// The server sends an OPAQUE wake — it carries NO message content (§2.3). Its only
// job is to nudge the client to foreground-fetch `/changes` (the same refetch the
// WS/SSE realtime path does). So on `push` we: (1) message any open clients so the
// SPA refetches + renders its own native notification via the capability layer, and
// (2) if no client is visible, show a generic, content-free notification so the wake
// is not silently dropped. The wake is never parsed as mail.
self.addEventListener('push', (event) => {
  event.waitUntil(
    (async () => {
      const clientList = await self.clients.matchAll({
        type: 'window',
        includeUncontrolled: true,
      });
      // Nudge every open tab to refetch — content is fetched over JMAP, not push.
      for (const client of clientList) {
        client.postMessage({ type: 'mw-push-wake' });
      }
      const anyVisible = clientList.some((c) => c.visibilityState === 'visible');
      // Only surface an OS notification when the app is not already in front; a
      // visible tab refetches and renders its own richer, in-app notification.
      if (!anyVisible && self.registration.showNotification) {
        await self.registration.showNotification('Mailwoman', {
          body: 'You have new activity.',
          tag: 'mw-wake',
          renotify: false,
        });
      }
    })(),
  );
});

// Focus (or open) the app when the generic wake notification is clicked.
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(
    (async () => {
      const clientList = await self.clients.matchAll({
        type: 'window',
        includeUncontrolled: true,
      });
      const existing = clientList.find((c) => 'focus' in c);
      if (existing) {
        await existing.focus();
      } else if (self.clients.openWindow) {
        await self.clients.openWindow(SHELL_URL);
      }
    })(),
  );
});
