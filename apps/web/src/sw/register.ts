// Service-Worker registration, called from the app on startup (via the offline
// slice). Best-effort + feature-detected: absent under jsdom / older browsers,
// where the app simply runs online without a SW.

/** Register the hand-rolled app-shell SW (`public/sw.js`). No-op when unsupported. */
export async function registerServiceWorker(): Promise<void> {
  if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return;
  try {
    await navigator.serviceWorker.register('/sw.js', { type: 'classic', scope: '/' });
  } catch {
    // Registration is best-effort; the app is fully functional online without it.
  }
}
