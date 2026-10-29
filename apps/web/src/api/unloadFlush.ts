// A debounced write must not be lost when the page goes away (t39-e1, d9cd19eb
// + b39db5ce; shared with account preferences in t67).
//
// `onPageHide(flush)`: calls `flush` on `pagehide` and when the document turns
// hidden, which is the last moment a page can start a request (`keepalive`
// lets it outlive the page). `sendBeacon` is not usable for these writes: it
// cannot carry the storage-authority and CSRF headers and it only POSTs.
//
// `pendingWrite(key, parse)`: the browser may still drop that keepalive request
// (Chromium did in a live run: 2 of 6 appearance picks were lost on reload), so
// the write is also kept in localStorage under `<key>:pending` and the next
// start sends it again, unless the server has taken a newer value since. Every
// storage error (private mode, quota) is swallowed: then only the keepalive
// write is left.

/** Listen for the page going away; returns the unsubscribe. */
export function onPageHide(flush: () => void): () => void {
  if (typeof window === 'undefined') return () => undefined;
  const onVisibility = (): void => {
    if (typeof document !== 'undefined' && document.visibilityState === 'hidden') flush();
  };
  window.addEventListener('pagehide', flush);
  document.addEventListener('visibilitychange', onVisibility);
  return () => {
    window.removeEventListener('pagehide', flush);
    document.removeEventListener('visibilitychange', onVisibility);
  };
}

export interface PendingWriteStore<T> {
  read(): T | null;
  /** `null` forgets the pending write. */
  write(value: T | null): void;
}

/** The pending write kept for `key` (an empty key stores nothing). `parse`
 *  validates what was read back; return null to ignore it. */
export function pendingWrite<T>(key: string, parse: (raw: unknown) => T | null): PendingWriteStore<T> {
  const slot = `${key}:pending`;
  const usable = (): boolean => key !== '' && typeof localStorage !== 'undefined';
  return {
    read() {
      if (!usable()) return null;
      try {
        const raw = localStorage.getItem(slot);
        return raw === null ? null : parse(JSON.parse(raw));
      } catch {
        return null;
      }
    },
    write(value) {
      if (!usable()) return;
      try {
        if (value === null) localStorage.removeItem(slot);
        else localStorage.setItem(slot, JSON.stringify(value));
      } catch {
        /* private mode / quota: only the keepalive write is left */
      }
    },
  };
}
