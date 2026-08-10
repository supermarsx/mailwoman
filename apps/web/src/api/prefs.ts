// Per-user appearance sync (t19 e13, SPEC §17.3 — "all synced server-side per
// user"). Until now theme/density/accent lived in `localStorage` only, and the
// theme slice said so in a comment; this module is the transport that closes it.
//
// The seam is the one t19-e6 left behind in `state/slices/theme.ts`:
// `appearancePrefs()` (one serializable object), `setAppearancePrefs(partial)`
// (re-validates through `parseAppearancePrefs`, so a bad field from the wire
// degrades instead of corrupting), and `subscribePrefs(fn)` (fires after every
// change). Nothing else in the theme layer changes, and this file is the only
// place that knows the appearance preferences ever leave the device.
//
// Server contract — `crates/mw-server/src/prefs_routes.rs`, the same
// session-authed `/api/account/*` surface as signatures/identities/notifications
// (t16 e18), so the account is the SESSION's and is never in the body:
//
//   GET    /api/account/appearance → { appearance, updatedAt, deploymentDefault }
//   PUT    /api/account/appearance ← { appearance }        → { ok, updatedAt }
//   DELETE /api/account/appearance                          → { ok, updatedAt }
//
// The stored object is opaque to the server: it stamps `updatedAt` from its own
// clock, caps the size, and requires a JSON object — the field names are this
// side's business.
//
// ## Which value wins
//
// Prefs are edited on more than one device, so "sync" has to answer a conflict
// question. This module keeps a MARK in `localStorage` — the last state it knows
// the server has (`{ updatedAt, prefs }`) — and reconciles once, at `start()`:
//
//   * server has nothing            → push this device's set (first-time seed);
//   * this device never synced      → adopt the server's (a new device joining an
//                                     account takes that account's appearance);
//   * local differs from the mark
//     AND the server has not moved
//     since that mark               → push (offline edits are NOT clobbered by a
//                                     stale server value — the case that makes
//                                     naive "server always wins" lose work);
//   * otherwise                     → adopt the server's.
//
// After that one reconcile it is last-write-wins, which is the honest description
// of a preference blob with no field-level merge: two devices editing appearance
// within the same session, the later PUT stands. Nothing is silently reverted on
// the losing device's screen — it keeps rendering what it has until it next
// starts up. `updatedAt` is a server-clock ordering signal and a display value;
// a client clock never decides anything here.
//
// A failed push never touches local state: `localStorage` remains the source of
// truth, the status goes to `local-only`/`error`, and the next change retries.
// Signed out (401) is a normal state, not an error — the app is fully usable with
// device-local preferences.

import { withBase } from './basePath.ts';
import type { AppearancePrefs } from '../theme/appearance.ts';

/** The subset of the theme slice this module drives (t19-e6's sync seam). */
export interface AppearancePrefsSeam {
  appearancePrefs(): AppearancePrefs;
  setAppearancePrefs(next: Partial<AppearancePrefs>): void;
  subscribePrefs(fn: (prefs: AppearancePrefs) => void): () => void;
}

/** The deployment-wide default (admin › Appearance). A DEFAULT, not a policy. */
export interface DeploymentAppearance {
  theme: string;
  brandName: string;
  accent: string | null;
}

/** `GET /api/account/appearance`. */
export interface StoredAppearance {
  /** The account's stored object, or `null` when it has never saved one. */
  appearance: unknown | null;
  /** Server-clock ms when it was stored, or `null`. */
  updatedAt: number | null;
  deploymentDefault: DeploymentAppearance;
}

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) =>
  fetch(input, { credentials: 'same-origin', ...init });

/** Raised on a non-2xx appearance request; `status` separates 401 from failure. */
export class AppearanceSyncError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'AppearanceSyncError';
    this.status = status;
  }
}

// Sub-path hosting (t20 B4): root-absolute in source, prefixed at call time.
// `withBase` is the identity function at the origin root.
const ENDPOINT = '/api/account/appearance';

/** The three appearance calls. Injectable so the sync unit-tests without a server. */
export interface AppearancePrefsApi {
  load(): Promise<StoredAppearance>;
  save(prefs: AppearancePrefs): Promise<number | null>;
  /** Forget the account's stored appearance (falls back to the deployment default). */
  reset(): Promise<void>;
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let detail = `request failed with ${res.status}`;
    try {
      const body = (await res.json()) as { error?: string };
      if (typeof body.error === 'string' && body.error !== '') detail = body.error;
    } catch {
      /* non-JSON body — keep the generic message */
    }
    throw new AppearanceSyncError(res.status, detail);
  }
  return (await res.json()) as T;
}

const DEFAULT_DEPLOYMENT: DeploymentAppearance = {
  theme: '',
  brandName: '',
  accent: null,
};

/** Narrow an untrusted `GET` body. A malformed field reads as "nothing stored"
 *  rather than throwing — an unusable response must not break the app's theme. */
function parseStored(body: unknown): StoredAppearance {
  const b = (body ?? {}) as Partial<StoredAppearance>;
  const raw = b.appearance;
  const d = (b.deploymentDefault ?? {}) as Partial<DeploymentAppearance>;
  return {
    // Only an object can round-trip into a preference set; anything else is
    // treated as absent so the client falls back to its own defaults.
    appearance: typeof raw === 'object' && raw !== null && !Array.isArray(raw) ? raw : null,
    updatedAt: typeof b.updatedAt === 'number' && Number.isFinite(b.updatedAt) ? b.updatedAt : null,
    deploymentDefault: {
      theme: typeof d.theme === 'string' ? d.theme : DEFAULT_DEPLOYMENT.theme,
      brandName: typeof d.brandName === 'string' ? d.brandName : DEFAULT_DEPLOYMENT.brandName,
      accent: typeof d.accent === 'string' && d.accent !== '' ? d.accent : null,
    },
  };
}

export function createAppearancePrefsApi(fetcher: Fetcher = defaultFetcher): AppearancePrefsApi {
  return {
    async load() {
      return parseStored(await jsonOrThrow<unknown>(await fetcher(withBase(ENDPOINT))));
    },
    async save(prefs) {
      const res = await fetcher(withBase(ENDPOINT), {
        method: 'PUT',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ appearance: prefs }),
      });
      const body = await jsonOrThrow<{ updatedAt?: number }>(res);
      return typeof body.updatedAt === 'number' ? body.updatedAt : null;
    },
    async reset() {
      await jsonOrThrow<unknown>(await fetcher(withBase(ENDPOINT), { method: 'DELETE' }));
    },
  };
}

// ── The sync ─────────────────────────────────────────────────────────────────

/**
 * `idle` before `start()`; `loading` during the reconcile; `synced` once the
 * server holds this device's set; `local-only` when there is no session (401/403)
 * — a normal state, preferences simply stay on the device; `error` when a request
 * failed for any other reason.
 */
export type SyncStatus = 'idle' | 'loading' | 'synced' | 'local-only' | 'error';

/** The last state this device knows the server has. */
interface SyncMark {
  updatedAt: number | null;
  /** Serialized prefs, compared as a string so field order cannot matter. */
  prefs: string;
}

const MARK_KEY = 'mw.theme.sync';

function readMark(): SyncMark | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const raw = localStorage.getItem(MARK_KEY);
    if (raw === null) return null;
    const m = JSON.parse(raw) as Partial<SyncMark>;
    if (typeof m.prefs !== 'string') return null;
    return { updatedAt: typeof m.updatedAt === 'number' ? m.updatedAt : null, prefs: m.prefs };
  } catch {
    return null;
  }
}

function writeMark(mark: SyncMark): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(MARK_KEY, JSON.stringify(mark));
  } catch {
    /* private mode / quota — the mark is an optimization, not correctness */
  }
}

/** Drop the mark, so the next `start()` reconciles as a device that never synced. */
export function clearSyncMark(): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.removeItem(MARK_KEY);
  } catch {
    /* ignore */
  }
}

export interface AppearanceSyncOptions {
  api?: AppearancePrefsApi;
  /** Coalescing window for pushes, ms. A theme gallery produces bursts of
   *  changes as the user browses; only the settled value needs to travel. */
  debounceMs?: number;
}

export interface AppearanceSync {
  /** Reconcile with the server, then keep pushing changes. Safe to await. */
  start(): Promise<void>;
  /** Unsubscribe and send any pending push (fire-and-forget). */
  stop(): void;
  status(): SyncStatus;
  /** Server-clock ms of the last value we know the server has, or `null`. */
  lastSyncedAt(): number | null;
  /** The deployment default from the last successful load, or `null`. */
  deploymentDefault(): DeploymentAppearance | null;
  /** Send a pending push now and wait for it. */
  flush(): Promise<void>;
  /** Forget the account's stored appearance; local preferences are untouched. */
  reset(): Promise<void>;
  /** Observe status changes. Returns an unsubscribe. */
  subscribeStatus(fn: (status: SyncStatus) => void): () => void;
}

/** No session — preferences legitimately stay device-local. */
function isSignedOut(e: unknown): boolean {
  return e instanceof AppearanceSyncError && (e.status === 401 || e.status === 403);
}

export function createAppearanceSync(
  prefs: AppearancePrefsSeam,
  options: AppearanceSyncOptions = {},
): AppearanceSync {
  const api = options.api ?? createAppearancePrefsApi();
  const debounceMs = options.debounceMs ?? 600;

  let status: SyncStatus = 'idle';
  let mark: SyncMark | null = readMark();
  let deployment: DeploymentAppearance | null = null;
  let unsubscribe: (() => void) | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: Promise<void> = Promise.resolve();
  // Counts local edits so an edit made WHILE the initial GET is in flight is not
  // overwritten by the response it raced.
  let localEdits = 0;

  const statusListeners = new Set<(s: SyncStatus) => void>();

  function setStatus(next: SyncStatus): void {
    if (next === status) return;
    status = next;
    for (const fn of statusListeners) fn(next);
  }

  const serialize = (p: AppearancePrefs): string => JSON.stringify(p);

  /** Record what the server now holds, so we neither re-push it nor mistake it
   *  for an unsynced local edit at the next start. */
  function markSynced(p: AppearancePrefs, updatedAt: number | null): void {
    mark = { updatedAt, prefs: serialize(p) };
    writeMark(mark);
  }

  async function push(): Promise<void> {
    const current = prefs.appearancePrefs();
    const payload = serialize(current);
    // Already up there (including the value we just adopted FROM the server —
    // this is what stops an adopt from echoing straight back as a write).
    if (mark !== null && mark.prefs === payload) {
      setStatus('synced');
      return;
    }
    try {
      const updatedAt = await api.save(current);
      markSynced(current, updatedAt);
      setStatus('synced');
    } catch (e) {
      // The mark is deliberately NOT advanced: the next change retries, and the
      // user's on-screen preferences are never rolled back by a failed write.
      setStatus(isSignedOut(e) ? 'local-only' : 'error');
    }
  }

  function schedulePush(): void {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
      pending = push();
    }, debounceMs);
  }

  async function start(): Promise<void> {
    if (unsubscribe !== null) return; // idempotent
    setStatus('loading');
    localEdits = 0;
    unsubscribe = prefs.subscribePrefs(() => {
      localEdits += 1;
      schedulePush();
    });

    let stored: StoredAppearance;
    try {
      stored = await api.load();
    } catch (e) {
      setStatus(isSignedOut(e) ? 'local-only' : 'error');
      return;
    }
    deployment = stored.deploymentDefault;

    const local = prefs.appearancePrefs();
    const editedDuringLoad = localEdits > 0;
    const neverSynced = mark === null;
    const localDiffersFromMark = mark !== null && mark.prefs !== serialize(local);
    const serverMovedSinceMark =
      mark !== null &&
      stored.updatedAt !== null &&
      (mark.updatedAt === null || stored.updatedAt > mark.updatedAt);

    const adopt =
      stored.appearance !== null &&
      !editedDuringLoad &&
      (neverSynced || !localDiffersFromMark || serverMovedSinceMark);

    if (adopt) {
      // Re-validated by the slice, so a field the server holds but this build no
      // longer understands falls back rather than corrupting the set.
      prefs.setAppearancePrefs(stored.appearance as Partial<AppearancePrefs>);
      markSynced(prefs.appearancePrefs(), stored.updatedAt);
      setStatus('synced');
      return;
    }
    // Nothing stored, unsynced local edits the server has not superseded, or an
    // edit that raced the load: this device's set is the newer one.
    pending = push();
    await pending;
  }

  return {
    start,
    stop() {
      unsubscribe?.();
      unsubscribe = null;
      if (timer !== undefined) {
        clearTimeout(timer);
        timer = undefined;
        // Do not drop a change the user already made — send it on the way out.
        pending = push();
      }
    },
    status: () => status,
    lastSyncedAt: () => mark?.updatedAt ?? null,
    deploymentDefault: () => deployment,
    async flush() {
      if (timer !== undefined) {
        clearTimeout(timer);
        timer = undefined;
        pending = push();
      }
      await pending;
    },
    async reset() {
      try {
        await api.reset();
        clearSyncMark();
        mark = null;
        setStatus('synced');
      } catch (e) {
        setStatus(isSignedOut(e) ? 'local-only' : 'error');
      }
    },
    subscribeStatus(fn) {
      statusListeners.add(fn);
      return () => statusListeners.delete(fn);
    },
  };
}

// ── Process-wide singleton ───────────────────────────────────────────────────
//
// The sync belongs to the app, not to the Settings dialog: a second device
// should adopt the account's appearance at BOOT, not the first time someone
// opens Settings. `startAppearanceSync` is therefore idempotent and callable
// from wherever the app starts — the earliest caller wins and every later call
// (including Settings mounting) just returns the running instance.

let singleton: AppearanceSync | null = null;

/**
 * Start the app-wide appearance sync, or return the one already running.
 * Safe to call any number of times and from any number of places.
 */
export function startAppearanceSync(
  prefs: AppearancePrefsSeam,
  options: AppearanceSyncOptions = {},
): AppearanceSync {
  if (singleton === null) {
    singleton = createAppearanceSync(prefs, options);
    void singleton.start();
  }
  return singleton;
}

/** The running sync, or `null` when none has been started. */
export function appearanceSync(): AppearanceSync | null {
  return singleton;
}

/** Tear the singleton down (tests, and a full sign-out). */
export function stopAppearanceSync(): void {
  singleton?.stop();
  singleton = null;
}
