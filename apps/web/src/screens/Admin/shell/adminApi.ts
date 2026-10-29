// Typed admin request helpers for the UX seam (t32-cfg §3, UX5/UX6/UX19).
//
// The contract every admin write follows (t32-admin.md §13, §2.0 wire contracts):
//   • reads answer an `ETag` header (or an `etag` field); writes send `If-Match`;
//   • a write on a changed object answers `412 {code: "stale", updatedBy, updatedAt}`;
//   • a refused value answers `422 {code, detail, field}`, `field` being a JSON
//     pointer into the request body (`/smtp/port`);
//   • risky writes accept `"dryRun": true` and answer `{impact: {...}}` unwritten.
//
// The server halves land with e24a/e10b/e70a. Until a route exists the server
// answers 404 (or 405/501); `adminRead` reports that as `unavailable` so a screen
// can show "Not available" instead of an error, and never invents data.

import { basePath } from '../../../api/basePath.ts';
import { adminUrl } from '../../../api/adminPath.ts';
import { apiFetch } from '../../../api/http.ts';
import { AdminApiError } from '../../../state/slices/admin.ts';

/** A failed admin request with the server's problem body, when it sent one. */
export class AdminProblemError extends AdminApiError {
  readonly code: string | undefined;
  readonly resultUncertain: boolean;
  readonly detail: string | undefined;
  /** JSON pointer to the refused field (`/smtp/port`), 422 only. */
  readonly field: string | undefined;
  /** 412: who changed the object, and when (ISO 8601). */
  readonly updatedBy: string | undefined;
  readonly updatedAt: string | undefined;
  /** 412: the object's current tag, when the server sent one. */
  readonly currentEtag: string | undefined;

  constructor(status: number, body: Record<string, unknown>, etag: string | null) {
    const str = (k: string): string | undefined => (typeof body[k] === 'string' ? (body[k] as string) : undefined);
    super(status, str('detail') ?? str('error') ?? `admin request failed (${status})`);
    this.name = 'AdminProblemError';
    this.code = str('code');
    this.resultUncertain = body['resultUncertain'] === true;
    this.detail = str('detail');
    this.field = str('field');
    this.updatedBy = str('updatedBy');
    this.updatedAt = str('updatedAt');
    this.currentEtag = etag ?? str('etag');
  }

  /** True for the optimistic-concurrency refusal (UX19). */
  get stale(): boolean {
    return this.status === 412;
  }
}

/** Is `err` a 412 stale-write refusal? */
export function isStale(err: unknown): err is AdminProblemError {
  return err instanceof AdminProblemError && err.status === 412;
}

export interface AdminRequestInit {
  method?: 'GET' | 'PUT' | 'POST' | 'PATCH' | 'DELETE';
  /** JSON body; sent with `Content-Type: application/json`. */
  body?: unknown;
  /** The tag from the read; sent as `If-Match`. */
  ifMatch?: string | null;
  signal?: AbortSignal;
  /** Deploy prefix or a native shell's server. Defaults to `basePath()`. */
  base?: string;
}

export interface AdminResponse<T> {
  status: number;
  data: T;
  etag: string | null;
}

async function readBody(res: Response): Promise<unknown> {
  const text = await res.text();
  if (text === '') return null;
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return text;
  }
}

function tagOf(res: Response, data: unknown): string | null {
  const header = res.headers.get('ETag');
  if (header) return header;
  if (typeof data === 'object' && data !== null && typeof (data as { etag?: unknown }).etag === 'string') {
    return (data as { etag: string }).etag;
  }
  return null;
}

let consoleAnnounced = false;
function announceConsoleUse(): void {
  if (consoleAnnounced) return;
  consoleAnnounced = true;
  try {
    const channel = new BroadcastChannel('mw-analytics');
    channel.postMessage({ type: 'revoke' });
    channel.close();
  } catch { /* BroadcastChannel unavailable: the marker cookie still applies on reload */ }
}

/** One admin request. Resolves on 2xx; rejects with {@link AdminProblemError} otherwise. */
export async function adminRequest<T = unknown>(path: string, init: AdminRequestInit = {}): Promise<AdminResponse<T>> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (init.body !== undefined) headers['Content-Type'] = 'application/json';
  if (init.ifMatch) headers['If-Match'] = init.ifMatch;
  const res = await apiFetch(adminUrl(path, init.base ?? basePath()), {
    method: init.method ?? 'GET',
    credentials: 'same-origin',
    headers,
    body: init.body === undefined ? null : JSON.stringify(init.body),
    signal: init.signal ?? null,
  });
  const data = await readBody(res);
  const etag = tagOf(res, data);
  // t63 R2: this browser is being used for the console. Mail tabs that run an
  // analytics tag drop it (they reload; the server then refuses the tag here).
  if (res.ok) announceConsoleUse();
  if (!res.ok) {
    const body = typeof data === 'object' && data !== null ? (data as Record<string, unknown>) : {};
    throw new AdminProblemError(res.status, body, etag);
  }
  return { status: res.status, data: data as T, etag };
}

/** A read that may meet a server without the route yet. */
export type AdminReadResult<T> = { kind: 'ok'; data: T; etag: string | null } | { kind: 'unavailable' };

/** Statuses that mean "this server has no such route" rather than "it failed". */
const ABSENT = new Set([404, 405, 501]);

/** `GET` that turns a missing route into `unavailable`; other failures reject. */
export async function adminRead<T>(path: string, init: Omit<AdminRequestInit, 'method' | 'body' | 'ifMatch'> = {}): Promise<AdminReadResult<T>> {
  try {
    const r = await adminRequest<T>(path, init);
    return { kind: 'ok', data: r.data, etag: r.etag };
  } catch (err) {
    if (err instanceof AdminProblemError && ABSENT.has(err.status) && err.code === undefined) return { kind: 'unavailable' };
    throw err;
  }
}

// ── Impact preview (UX6) ───────────────────────────────────────────────────────

/** One consequence of a risky write: how many, and the first names. */
export interface ImpactItem {
  count: number;
  /** Up to the first 20 names (users refused, domains affected, ...). */
  names?: string[];
}

/** `{impact: {...}}` from a `dryRun` write. Keys name the consequence. */
export type Impact = Record<string, ImpactItem | number>;

/** Normalise a dry-run answer. Returns null when the server sent no impact. */
export function parseImpact(body: unknown): Record<string, ImpactItem> | null {
  if (typeof body !== 'object' || body === null) return null;
  const raw = (body as { impact?: unknown }).impact;
  if (typeof raw !== 'object' || raw === null) return null;
  const out: Record<string, ImpactItem> = {};
  for (const [key, v] of Object.entries(raw as Record<string, unknown>)) {
    if (typeof v === 'number') out[key] = { count: v };
    else if (typeof v === 'object' && v !== null && typeof (v as ImpactItem).count === 'number') {
      const names = (v as ImpactItem).names;
      out[key] = Array.isArray(names)
        ? { count: (v as ImpactItem).count, names: names.filter((n): n is string => typeof n === 'string') }
        : { count: (v as ImpactItem).count };
    } else if (Array.isArray(v)) {
      const names = v.filter((n): n is string => typeof n === 'string');
      out[key] = { count: names.length, names };
    }
  }
  return out;
}

// ── Configuration sources (UX9; `GET /admin/config/effective`, e70a) ───────────

/** Where a value comes from, as the effective-configuration catalogue reports it. */
export type EffectiveSource = 'default' | 'stored' | 'env-pin' | 'env-fallback' | 'boot' | 'build' | 'refused';

/** One row of `GET /admin/config/effective` (t32-cfg §4.1). */
export interface EffectiveEntry {
  id: string;
  group: string;
  class: 'ui' | 'readOnly' | 'refused' | 'cliOnly' | 'build' | 'testOnly' | 'Ui' | 'ReadOnly' | 'Refused' | 'CliOnly' | 'Build' | 'TestOnly';
  value: unknown;
  source: EffectiveSource;
  envVar?: string;
  restartNeeded?: boolean;
  hot?: boolean;
  /** The scope that locks the value, when one does. */
  locked?: string | boolean | null;
  updatedBy?: string;
  updatedAt?: string;
  etag?: string;
  docs?: string;
  reasonId?: string;
  valueStatus?: 'resolved' | 'resolvedPolicy' | 'build' | 'unavailable';
  configuredInput?: unknown;
  what?: string;
  configuredBy?: string;
  href?: string;
  kind?: string;
  searchText?: string;
}

/** `GET /admin/config/effective`; `unavailable` before e70a lands. */
export function loadEffectiveConfig(base?: string): Promise<AdminReadResult<EffectiveEntry[]>> {
  return adminRead<EffectiveEntry[]>('/config/effective', base === undefined ? {} : { base });
}

/** One registry entry of `GET /admin/settings` (t32-admin.md §2.4, e10b). */
export interface RegistryEntry {
  key: string;
  type: string;
  default?: unknown;
  globalValue?: unknown;
  min?: number;
  max?: number;
  pattern?: string;
  options?: string[];
  allowUserOverride?: boolean;
  locked?: boolean;
  envVar?: string;
  precedence?: 'pin' | 'fallback' | 'boot';
  updatedBy?: string;
  updatedAt?: string;
}

/** `GET /admin/settings`; `unavailable` before e10b lands. Accepts `[...]` or `{settings: [...]}`. */
export async function loadSettingsRegistry(base?: string): Promise<AdminReadResult<RegistryEntry[]>> {
  const r = await adminRead<RegistryEntry[] | { settings?: RegistryEntry[] }>('/settings', base === undefined ? {} : { base });
  if (r.kind === 'unavailable') return r;
  const list = Array.isArray(r.data) ? r.data : Array.isArray(r.data?.settings) ? r.data.settings : [];
  return { kind: 'ok', data: list, etag: r.etag };
}
