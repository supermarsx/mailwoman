// Merging a reloaded PIM list with this client's own writes (t32-h11, 26.20).
//
// A change pushed from another device or session makes the open Tasks or Notes
// module reload its list while the user may be writing to it. The reload can be
// answered from data read before one of this client's writes reached the
// server, so taking it as-is would drop a row just created, put back an old
// value, or show a row just deleted. `LocalWrites` remembers each create /
// update / destroy until a load that was sent after the write was answered has
// come back: that load already holds the write, so it decides the row. Until
// then the local version of the row is kept.
//
// `keepUnchanged` then reuses the previous object for every row whose content
// did not change, so a view keyed by object identity does not re-create it.

import type { Id } from '../../api/jmap-types.ts';

type WriteKind = 'create' | 'update' | 'destroy';

interface Entry<T> {
  kind: WriteKind;
  /** The fields this client wrote (updates only), laid over the loaded row. */
  fields: Partial<T>;
  /** Writes to the row sent and not yet answered. */
  inFlight: number;
  /** The clock value when the last write to the row was answered. */
  settledAt: number;
}

/** The rows to show after a load, and which of them carry local writes. */
export interface Merged<T> {
  rows: T[];
  /** Ids whose shown row is (or includes) this client's version, not the load's. */
  local: Set<Id>;
}

export class LocalWrites<T extends { id: Id }> {
  private clock = 0;
  private readonly entries = new Map<Id, Entry<T>>();

  /** Stamp a load as it is sent; pass the stamp to `merge` with its answer. */
  loadSent(): number {
    this.clock += 1;
    return this.clock;
  }

  /**
   * Record a write to `id` as it is sent. Call the returned function once the
   * server has answered it, whether it was applied or refused (the next load
   * then shows what the server holds).
   */
  begin(id: Id, kind: WriteKind, fields: Partial<T> = {}): () => void {
    const prev = this.entries.get(id);
    const entry: Entry<T> = prev ?? { kind, fields: {}, inFlight: 0, settledAt: 0 };
    // A destroy outranks earlier writes; an update after a create stays a create.
    if (prev === undefined || kind === 'destroy') entry.kind = kind;
    entry.fields = { ...entry.fields, ...fields };
    entry.inFlight += 1;
    this.entries.set(id, entry);
    let answered = false;
    return () => {
      if (answered) return;
      answered = true;
      entry.inFlight -= 1;
      this.clock += 1;
      entry.settledAt = this.clock;
    };
  }

  /**
   * Merge the answer (`loaded`) of the load stamped `sentAt` with the rows shown
   * now (`shown`). A row this client wrote after that load was sent keeps its
   * local version: its written fields are laid over the loaded row, a row the
   * load does not have yet is kept (when `inScope` allows it), and a destroyed
   * row stays out. Entries the load has caught up with are forgotten.
   */
  merge(shown: readonly T[], loaded: readonly T[], sentAt: number, inScope: (row: T) => boolean = () => true): Merged<T> {
    const rows: T[] = [];
    const local = new Set<Id>();
    const seen = new Set<Id>();
    for (const row of loaded) {
      seen.add(row.id);
      const e = this.newerThan(row.id, sentAt);
      if (e === undefined) {
        rows.push(row);
      } else if (e.kind !== 'destroy') {
        rows.push({ ...row, ...e.fields });
        local.add(row.id);
      }
    }
    for (const row of shown) {
      if (seen.has(row.id)) continue;
      const e = this.newerThan(row.id, sentAt);
      if (e !== undefined && e.kind !== 'destroy' && inScope(row)) {
        rows.push(row);
        local.add(row.id);
      }
    }
    for (const id of [...this.entries.keys()]) this.newerThan(id, sentAt);
    return { rows, local };
  }

  /** The entry for `id` when the load stamped `sentAt` may not hold it yet. */
  private newerThan(id: Id, sentAt: number): Entry<T> | undefined {
    const e = this.entries.get(id);
    if (e === undefined) return undefined;
    if (e.inFlight === 0 && e.settledAt < sentAt) {
      this.entries.delete(id);
      return undefined;
    }
    return e;
  }
}

/** Structural equality for loaded rows (plain JSON values; key order ignored). */
export function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a) && Array.isArray(b)) {
    return a.length === b.length && a.every((v, i) => sameValue(v, b[i]));
  }
  const ra = a as Record<string, unknown>;
  const rb = b as Record<string, unknown>;
  const ka = Object.keys(ra).filter((k) => ra[k] !== undefined);
  const kb = Object.keys(rb).filter((k) => rb[k] !== undefined);
  return ka.length === kb.length && ka.every((k) => sameValue(ra[k], rb[k]));
}

/** `next`, with the object from `prev` reused for every row whose content is unchanged. */
export function keepUnchanged<T extends { id: Id }>(prev: readonly T[], next: readonly T[]): T[] {
  const byId = new Map(prev.map((row) => [row.id, row]));
  return next.map((row) => {
    const old = byId.get(row.id);
    return old !== undefined && sameValue(old, row) ? old : row;
  });
}
