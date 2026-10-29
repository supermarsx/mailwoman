// The settings model C, user side (t32-admin §2.4 + §2.8, worker f01).
//
// Server contract — `crates/mw-server/src/settings_routes.rs`, session-authed:
//
//   GET    /api/settings        → { settings: { <key>: SettingEntry } }
//   PUT    /api/settings/<key>  ← { value }  → SettingEntry
//                                 403 { code: "setting_locked", lockedBy } when an
//                                 administrator holds the key
//                                 422 { code, error, field } for a refused value
//   DELETE /api/settings/<key>              → SettingEntry (the user's value removed)
//
// An entry is the EFFECTIVE value: the administrator's global, domain, group or
// per-user value, or the user's own one when the policy lets the user override,
// or the built-in default. `locked` says the user cannot change it ("Set by your
// administrator"); `userValue` is the user's stored choice even while it does
// not apply.
//
// This module keeps one shared, reactive copy of the map (loaded once per
// sign-in, refreshed after every write), so every `useSetting` reads the same
// state. Signed out (401) is a normal state: the map is empty and every control
// falls back to its device-local behaviour.

import { createSignal } from 'solid-js';
import { withBase } from './basePath.ts';
import { apiFetch } from './http.ts';
import { idempotentWrite } from './recovery.ts';
import { preferenceAccount, preferenceAccountIsCurrent } from './accountPreferenceLifecycle.ts';

/** Where an effective value came from (`SettingSource::as_str` on the server). */
export type SettingSource =
  | 'default'
  | 'envFallback'
  | 'envPin'
  | 'global'
  | 'domain'
  | 'profile'
  | 'group'
  | 'admin_user'
  | 'user';

/** One key's effective value for the signed-in user. */
export interface SettingEntry {
  value: unknown;
  source: SettingSource;
  /** The user cannot change it. */
  locked: boolean;
  /** The scope whose lock decides the value, when a lock does. */
  lockedBy?: SettingSource;
  /** The user's own stored value, even when it does not apply. */
  userValue?: unknown;
}

export type SettingsMap = Readonly<Record<string, SettingEntry>>;

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

// t82: a setting write is a PUT of the whole value or a DELETE back to the
// default, both idempotent: retried on a transient failure (`recovery.ts`).
const defaultFetcher: Fetcher = (input, init) => apiFetch(input, { credentials: 'same-origin', ...init, ...idempotentWrite(init, ['PUT', 'DELETE']) });

/**
 * A refused settings request. `code` is the server's machine code
 * (`setting_locked`, `invalidValue`, `unknownSetting`, …) and `field` the JSON
 * pointer of a refused value; the UI maps both to localised text and never
 * shows the server's English.
 */
export class SettingsApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly field: string | null;
  constructor(status: number, code: string, field: string | null) {
    super(`settings request failed: ${status} ${code}`);
    this.name = 'SettingsApiError';
    this.status = status;
    this.code = code;
    this.field = field;
  }
}

const SOURCES: readonly SettingSource[] = [
  'default',
  'envFallback',
  'envPin',
  'global',
  'domain',
  'profile',
  'group',
  'admin_user',
  'user',
];

function isSource(v: unknown): v is SettingSource {
  return typeof v === 'string' && (SOURCES as readonly string[]).includes(v);
}

/** Validate one wire entry; a malformed one is dropped, never trusted. */
export function parseEntry(raw: unknown): SettingEntry | null {
  if (raw === null || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (!('value' in r) || !isSource(r['source']) || typeof r['locked'] !== 'boolean') return null;
  const entry: SettingEntry = { value: r['value'], source: r['source'], locked: r['locked'] };
  if (isSource(r['lockedBy'])) entry.lockedBy = r['lockedBy'];
  if ('userValue' in r) entry.userValue = r['userValue'];
  return entry;
}

async function failure(res: Response): Promise<SettingsApiError> {
  let code = `http${res.status}`;
  let field: string | null = null;
  try {
    const body = (await res.json()) as { code?: unknown; field?: unknown };
    if (typeof body.code === 'string' && body.code !== '') code = body.code;
    if (typeof body.field === 'string') field = body.field;
  } catch {
    /* not JSON: keep the status code */
  }
  return new SettingsApiError(res.status, code, field);
}

const url = (key?: string): string =>
  withBase(key === undefined ? '/api/settings' : `/api/settings/${encodeURIComponent(key)}`);

function mutationHeaders(): Record<string, string> {
  const authority = preferenceAccount()?.storageAuthority;
  return authority ? {'x-mailwoman-storage-authority': authority} : {};
}

/** `GET /api/settings`. */
export async function fetchSettings(fetcher: Fetcher = defaultFetcher): Promise<SettingsMap> {
  const res = await fetcher(url());
  if (!res.ok) throw await failure(res);
  const body = (await res.json()) as { settings?: unknown };
  const out: Record<string, SettingEntry> = {};
  if (body.settings !== null && typeof body.settings === 'object') {
    for (const [key, raw] of Object.entries(body.settings as Record<string, unknown>)) {
      const entry = parseEntry(raw);
      if (entry !== null) out[key] = entry;
    }
  }
  return out;
}

async function entryOrThrow(res: Response): Promise<SettingEntry> {
  if (!res.ok) throw await failure(res);
  const entry = parseEntry(await res.json());
  if (entry === null) throw new SettingsApiError(res.status, 'invalidResponse', null);
  return entry;
}

/** `PUT /api/settings/<key>`. */
export async function putSetting(
  key: string,
  value: unknown,
  fetcher: Fetcher = defaultFetcher,
): Promise<SettingEntry> {
  return entryOrThrow(
    await fetcher(url(key), {
      method: 'PUT',
      headers: { ...mutationHeaders(), 'content-type': 'application/json' },
      body: JSON.stringify({ value }),
    }),
  );
}

/** `DELETE /api/settings/<key>`: back to what the administrator or the default says. */
export async function deleteSetting(key: string, fetcher: Fetcher = defaultFetcher): Promise<SettingEntry> {
  return entryOrThrow(await fetcher(url(key), { method: 'DELETE', headers: mutationHeaders() }));
}

// ── shared state ─────────────────────────────────────────────────────────────

const [settingsMap, setSettingsMap] = createSignal<SettingsMap>({});
const [settingsStatus, setSettingsStatus] = createSignal<'idle' | 'loading' | 'ready' | 'signed-out' | 'error'>(
  'idle',
);

let inflight: Promise<void> | null = null;
let activeFetcher: Fetcher = defaultFetcher;
let generation = 0;
let loadSequence = 0;
const mutations = new Map<string, Promise<SettingEntry>>();
const revisions = new Map<string, number>();

/** The shared map (reactive). Empty until `loadSettings` settles. */
export function settings(): SettingsMap {
  return settingsMap();
}

/** Where the shared map stands (reactive). */
export function settingsLoadStatus(): 'idle' | 'loading' | 'ready' | 'signed-out' | 'error' {
  return settingsStatus();
}

/**
 * Load the shared map once (later calls reuse the first request until
 * `refresh` is set). A 401 leaves it empty with status `signed-out`.
 */
export function loadSettings(opts: { refresh?: boolean; fetcher?: Fetcher } = {}): Promise<void> {
  if (opts.fetcher !== undefined) activeFetcher = opts.fetcher;
  if (inflight !== null && opts.refresh !== true) return inflight;
  if (settingsStatus() === 'ready' && opts.refresh !== true) return Promise.resolve();
  const current = generation;
  const sequence = ++loadSequence;
  const loadedRevisions = new Map(revisions);
  const pendingKeys = new Set(mutations.keys());
  if (settingsStatus() !== 'ready') setSettingsStatus('loading');
  const run = fetchSettings(activeFetcher)
    .then((map) => {
      if (current !== generation || sequence !== loadSequence) return;
      const merged = { ...map };
      for (const [key, revision] of revisions) {
        if (pendingKeys.has(key) || loadedRevisions.get(key) !== revision || mutations.has(key)) {
          const entry = settingsMap()[key];
          if (entry) merged[key] = entry;
          else delete merged[key];
        }
      }
      setSettingsMap(merged);
      setSettingsStatus('ready');
    })
    .catch((e: unknown) => {
      if (current !== generation || sequence !== loadSequence) return;
      if (e instanceof SettingsApiError && e.status === 401) setSettingsMap({});
      setSettingsStatus(e instanceof SettingsApiError && e.status === 401 ? 'signed-out' : 'error');
    })
    .finally(() => {
      if (inflight === run) inflight = null;
    });
  inflight = run;
  return run;
}

/** Replace one key's entry in the shared map. */
function store(key: string, entry: SettingEntry): void {
  setSettingsMap({ ...settingsMap(), [key]: entry });
}

/**
 * Write the user's value for `key` and update the shared map. A 403
 * `setting_locked` refreshes the map (the administrator changed something since
 * it loaded) before rethrowing.
 */
function mutateSetting(key: string, operation: (fetcher: Fetcher) => Promise<SettingEntry>): Promise<SettingEntry> {
  const current = generation;
  const owner = preferenceAccount();
  const currentOwner = (): boolean => current === generation && (owner ? preferenceAccountIsCurrent(owner) : preferenceAccount() === null);
  const fetcher = activeFetcher;
  const revision = (revisions.get(key) ?? 0) + 1;
  revisions.set(key, revision);
  const previous = mutations.get(key);
  const execute = async (): Promise<SettingEntry> => {
    // A queued request must never run with another account's session cookie.
    if (!currentOwner()) throw new SettingsApiError(401, 'account_changed', null);
    try {
      const entry = await operation(fetcher);
      if (currentOwner() && revisions.get(key) === revision) store(key, entry);
      return entry;
    } catch (e) {
      if (currentOwner() && revisions.get(key) === revision && e instanceof SettingsApiError && e.code === 'setting_locked') {
        mutations.delete(key);
        void loadSettings({ refresh: true });
      }
      throw e;
    }
  };
  // Preserve user intent on the server too: same-key writes/resets run in order.
  const run = (previous ? previous.catch(() => undefined).then(execute) : execute()).finally(() => {
    if (mutations.get(key) === run) mutations.delete(key);
  });
  mutations.set(key, run);
  return run;
}

export function writeSetting(key: string, value: unknown): Promise<SettingEntry> {
  return mutateSetting(key, fetcher => putSetting(key, value, fetcher));
}

/** Remove the user's value for `key` and update the shared map. */
export function resetSetting(key: string): Promise<SettingEntry> {
  return mutateSetting(key, fetcher => deleteSetting(key, fetcher));
}

/** Forget the shared map (sign-out, tests). */
export function clearSettings(): void {
  generation++;
  mutations.clear();
  revisions.clear();
  inflight = null;
  activeFetcher = defaultFetcher;
  setSettingsMap({});
  setSettingsStatus('idle');
}
