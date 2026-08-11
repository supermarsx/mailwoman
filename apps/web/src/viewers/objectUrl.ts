// Scope-bound ownership for `blob:` object URLs (t22 L6, media memory).
//
// `URL.createObjectURL(blob)` publishes a strong reference to the whole Blob in
// the document's blob registry. Until `URL.revokeObjectURL` is called the bytes
// are pinned for the lifetime of the tab, regardless of whether anything still
// references the URL. Before this module the ONLY revocation in the app was the
// export path (`state/slices/mail.ts`, `modules/{calendar,contacts}`): every
// attachment thumbnail and every opened attachment leaked its buffered Blob.
//
// The owner is created inside a component's reactive scope; `adopt()` registers
// a URL and `onCleanup` revokes everything still held when that scope disposes.
// Two details matter and are the reason this is a module rather than a one-liner:
//
//   * a fetch that resolves AFTER the scope disposed must not re-leak — `adopt()`
//     revokes immediately once disposed, so create/revoke stays balanced even
//     when a viewer is closed mid-download;
//   * a URL that is superseded (the reader opens a second attachment) is released
//     at that point rather than at unmount, via `release()`.
//
// `size()` is the instrument the tests count with: it is the number of live
// (created, not yet revoked) URLs this owner holds.

import { onCleanup } from 'solid-js';

/** Revoke a URL if the platform supports it (jsdom has no blob registry). */
function revoke(url: string): void {
  if (typeof URL !== 'undefined' && typeof URL.revokeObjectURL === 'function') {
    URL.revokeObjectURL(url);
  }
}

export interface ObjectUrlOwner {
  /** Take ownership of `url` and return it. Revoked when the owning scope
   *  disposes — or immediately, if it already has. Empty strings are ignored so
   *  a "no account yet" placeholder never enters the registry. */
  adopt: (url: string) => string;
  /** Revoke one owned URL now (no-op for a URL this owner does not hold). */
  release: (url: string | null | undefined) => void;
  /** Live URLs still held. The create/revoke balance is `size() === 0`. */
  size: () => number;
}

/** Create an object-URL owner bound to the current reactive scope. */
export function createObjectUrlOwner(): ObjectUrlOwner {
  const live = new Set<string>();
  let disposed = false;

  onCleanup(() => {
    disposed = true;
    for (const url of live) revoke(url);
    live.clear();
  });

  return {
    adopt(url: string): string {
      if (url === '') return url;
      if (disposed) {
        revoke(url);
        return url;
      }
      live.add(url);
      return url;
    },
    release(url: string | null | undefined): void {
      if (url === null || url === undefined || url === '') return;
      if (live.delete(url)) revoke(url);
    },
    size: (): number => live.size,
  };
}
