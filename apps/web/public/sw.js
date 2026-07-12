// Service Worker stub (plan §3 e0; owned + filled by e5).
//
// e5 replaces this with the app-shell precache + fetch strategy (§2.5:
// network-first for /jmap/*, cache-first for hashed assets/fonts, cache name
// `mw-shell-v{N}`). Until then this is a harmless pass-through: it installs and
// activates but adds no fetch handler, so the browser uses the network as usual.

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()));
