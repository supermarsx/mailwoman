// Optimistic concurrency for admin writes (t32-cfg UX19).
//
// A screen keeps the tag of what it read (`set(etag)` after the load) and runs
// each write through `write(fn)`, which hands `fn` the tag to send as `If-Match`.
// When someone else changed the object in between, the server answers
// `412 {code: "stale", updatedBy, updatedAt}`; `write` then records the conflict
// (shown by `StaleNotice`: "Changed by <who> at <time>" with Reload and Keep mine)
// and rejects with the error, so the form keeps the admin's input.
//
//   • Reload — the screen reloads the object (its own `reload`), which clears the
//     conflict and drops the admin's edits.
//   • Keep mine — the same write again with the object's current tag (from the
//     412, or from `refresh()` when the server sent none), so the admin's values
//     replace the other change on purpose.

import { createSignal, type Accessor } from 'solid-js';
import { isStale } from './adminApi.ts';

export interface StaleConflict {
  updatedBy: string | null;
  /** ISO 8601, or null when the server did not say. */
  updatedAt: string | null;
}

/** What a write returns: at least the object's new tag, when the server sent one. */
export interface WriteResult {
  etag?: string | null;
}

export interface EtagOptions {
  /** Reads the object's current tag without touching the form (Keep mine without a tag in the 412). */
  refresh?: () => Promise<string | null>;
}

export interface EtagState {
  etag: Accessor<string | null>;
  set(tag: string | null): void;
  /** The open conflict, or null. */
  conflict: Accessor<StaleConflict | null>;
  /** Run a write with `If-Match`. A 412 records the conflict and rejects. */
  write<T extends WriteResult>(fn: (ifMatch: string | null) => Promise<T>): Promise<T>;
  /** Repeat the refused write with the current tag (Keep mine). */
  keepMine<T extends WriteResult = WriteResult>(): Promise<T>;
  /** Forget the conflict (after a reload). */
  dismiss(): void;
}

export function useEtag(opts: EtagOptions = {}): EtagState {
  const [etag, setEtag] = createSignal<string | null>(null);
  const [conflict, setConflict] = createSignal<StaleConflict | null>(null);
  let refused: ((ifMatch: string | null) => Promise<WriteResult>) | null = null;
  let latest: string | null = null;

  async function write<T extends WriteResult>(fn: (ifMatch: string | null) => Promise<T>): Promise<T> {
    try {
      const out = await fn(etag());
      if (out && out.etag !== undefined) setEtag(out.etag);
      setConflict(null);
      refused = null;
      return out;
    } catch (err) {
      if (isStale(err)) {
        refused = fn;
        latest = err.currentEtag ?? null;
        setConflict({ updatedBy: err.updatedBy ?? null, updatedAt: err.updatedAt ?? null });
      }
      throw err;
    }
  }

  return {
    etag,
    set(tag) {
      setEtag(tag);
      setConflict(null);
      refused = null;
    },
    conflict,
    write,
    async keepMine<T extends WriteResult = WriteResult>(): Promise<T> {
      const fn = refused;
      if (fn === null) throw new Error('no refused write to repeat');
      const tag = latest ?? (opts.refresh ? await opts.refresh() : null);
      setEtag(tag);
      return (await write(fn)) as T;
    },
    dismiss() {
      setConflict(null);
      refused = null;
    },
  };
}
