// Mailwoman app-shell Service Worker (plan §2.5, owned by e5). Hand-rolled — no
// Workbox. Ships copied verbatim into dist/ (no bundling), so it cannot import
// from src/. Its routing MIRRORS src/sw/strategy.ts, the unit-tested spec — keep
// the two in sync.
//
// Strategy:
//   - network-first for /jmap/* and /api/*   (fresh mail wins; cache is fallback)
//   - cache-first  for hashed build assets + fonts (immutable)
//   - offline navigation → the precached app shell ('/')
//   - everything else → passthrough (plain network)

// Must equal shellCacheName(SHELL_CACHE_VERSION) from src/contracts/offline.ts.
const CACHE_NAME = 'mw-shell-v1';
const SHELL_URL = '/';
const SHELL_URLS = [SHELL_URL];

function isApiPath(pathname) {
  return pathname.startsWith('/jmap/') || pathname.startsWith('/api/');
}

function isFont(pathname) {
  return /\.(?:woff2?|ttf|otf)$/i.test(pathname);
}

function isHashedAsset(pathname) {
  return /-[A-Za-z0-9_]{8,}\.[a-z0-9]+$/i.test(pathname) || pathname.startsWith('/assets/');
}

function chooseStrategy(request) {
  if (request.method !== 'GET') return 'passthrough';
  const pathname = new URL(request.url).pathname;
  if (isApiPath(pathname)) return 'network-first';
  if (request.mode === 'navigate') return 'shell-fallback';
  if (isHashedAsset(pathname) || isFont(pathname)) return 'cache-first';
  return 'passthrough';
}

async function networkFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  try {
    const res = await fetch(request);
    if (res.ok) await cache.put(request, res.clone());
    return res;
  } catch (err) {
    const cached = await cache.match(request);
    if (cached) return cached;
    throw err;
  }
}

async function cacheFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);
  if (cached) return cached;
  const res = await fetch(request);
  if (res.ok) await cache.put(request, res.clone());
  return res;
}

async function shellFallback(request) {
  try {
    return await fetch(request);
  } catch (err) {
    const cache = await caches.open(CACHE_NAME);
    const shell = await cache.match(SHELL_URL);
    if (shell) return shell;
    throw err;
  }
}

async function respond(request) {
  switch (chooseStrategy(request)) {
    case 'network-first':
      return networkFirst(request);
    case 'cache-first':
      return cacheFirst(request);
    case 'shell-fallback':
      return shellFallback(request);
    default:
      return fetch(request);
  }
}

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      // Precache the app shell entries individually so one 404 can't fail install.
      await Promise.all(
        SHELL_URLS.map((url) => cache.add(url).catch(() => undefined)),
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
      await self.clients.claim();
    })(),
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  // Only GET is cacheable; let writes (POST /jmap/api etc.) hit the network.
  if (request.method !== 'GET') return;
  event.respondWith(respond(request));
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
        await self.clients.openWindow('/');
      }
    })(),
  );
});
