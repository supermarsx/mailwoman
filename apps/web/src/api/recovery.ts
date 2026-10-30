// Automatic recovery for user-facing writes (t82).
//
// A request opts in by passing `recovery` to `apiFetch` (`http.ts`), which hands
// the send to `recoverRequest` below. Nothing else changes behaviour: requests
// without the option are sent once, exactly as before. `apiFetch` itself already
// repeats a write once after a CSRF refusal (fresh token) and once after a
// step-up confirmation, inside each attempt made here.
//
// What is retried, and when:
//   - transient failures: a network error, a timeout of our own, 502/503/504,
//     and 429 with a Retry-After the server sends. Retried with exponential
//     backoff plus jitter, at most `attempts` sends in all, and ONLY for a
//     request the caller marks `idempotent` (sending it twice leaves the same
//     state as sending it once: a PUT of a whole value, a grant of a scope).
//     A request that is not idempotent is sent once; its failure is returned.
//     t91: a create or a credential submission is made idempotent by an
//     `idempotencyKey` (sent as `Idempotency-Key`, the same on every attempt):
//     the server replays its first successful answer to a repeat. A refusal
//     (401, 403, 4xx) is never retried, whatever the request.
//   - offline: when the browser says it is offline the send waits for the
//     `online` event (bounded by `offlineWaitMs`) and is then made. Nothing was
//     sent while offline, so this holds for every request. A write that must
//     survive the page closing keeps its own pending copy (`unloadFlush.ts`).
//   - 428 `storageAuthorityRequired`: the request was sent without the
//     account-binding header. When the UI holds a verified authority it is
//     added and the request is sent once more. A 409 `accountChanged` is the
//     account fence working and is never retried.
//   - 409/412 (another device or tab saved first): when the caller gives
//     `rebase`, it re-reads the current state and returns the request to send
//     instead (the user's change re-applied on top), or `null` when the change
//     does not merge; one rebase per call.
//
// Never forever: the attempt count, the backoff cap, the Retry-After cap and
// the offline wait are all bounded, and an abort signal stops the wait at once.
// Retries are logged with `console.debug` only, with the method and the path:
// never a query string, header or body.

export interface RecoveryOptions {
  /** Sending twice leaves the same state as sending once (or it carries an idempotency key). */
  idempotent: boolean;
  /**
   * t91: sent as `Idempotency-Key` on every attempt. The server keeps the first
   * successful answer for this key and replays it to a repeat (`idempotency.rs`),
   * so a create or a credential submission is safe to send again. A request
   * with a key is retried like an idempotent one.
   */
  idempotencyKey?: string;
  /** Most sends in all, the first included. Default 4. */
  attempts?: number;
  /** First backoff delay in ms (doubled each time, with jitter). Default 400. */
  baseDelayMs?: number;
  /** Largest backoff delay. Default 5000. */
  maxDelayMs?: number;
  /** A Retry-After longer than this is not waited for. Default 30000. */
  maxRetryAfterMs?: number;
  /** Per-attempt timeout in ms; 0 for none. Default 20000. */
  timeoutMs?: number;
  /** Longest wait for the browser to come back online. Default 600000. */
  offlineWaitMs?: number;
  /** Re-read and re-apply after a 409/412: the request to send instead, or null. */
  rebase?: (response: Response) => Promise<RequestInit | null>;
  /** Told when recovery has taken more than `slowAfterMs` (the "Retrying…" state). */
  onSlow?: (state: 'retrying' | 'offline' | null) => void;
  /** Default 1000. */
  slowAfterMs?: number;
  /** An authority to add after a 428 storageAuthorityRequired (`http.ts` supplies it). */
  authority?: () => string | null;
}

/** Why a recovered request finally failed, in terms the UI can explain. */
export type FailureReason = 'offline' | 'network' | 'server' | 'rate' | 'conflict' | 'account' | 'linked' | 'refused';

const REASONS: ReadonlySet<FailureReason> = new Set<FailureReason>(['offline', 'network', 'server', 'rate', 'conflict', 'account', 'linked', 'refused']);

/** A recovery that gave up on a thrown error (not a response). */
export class RecoveryError extends Error {
  readonly reason: FailureReason;
  constructor(reason: FailureReason, cause?: unknown) {
    super(`request failed: ${reason}`, cause === undefined ? undefined : { cause });
    this.name = 'RecoveryError';
    this.reason = reason;
  }
}

const TRANSIENT = new Set([502, 503, 504]);
const AUTHORITY_HEADER = 'x-mailwoman-storage-authority';
/** t91: the header the server deduplicates retries by (`crates/mw-server/src/idempotency.rs`). */
export const IDEMPOTENCY_HEADER = 'Idempotency-Key';

/** A fresh key for one user action (kept across its retries and its Retry button). */
export function newIdempotencyKey(): string {
  const random = globalThis.crypto;
  if (typeof random?.randomUUID === 'function') return random.randomUUID();
  const bytes = new Uint8Array(16);
  if (typeof random?.getRandomValues === 'function') random.getRandomValues(bytes);
  else for (let i = 0; i < bytes.length; i += 1) bytes[i] = Math.floor(Math.random() * 256);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

/**
 * `{ recovery }` for a write that is not idempotent on its own (a create, a
 * link or a renew with a password): it carries `key` as `Idempotency-Key` and
 * is retried on a transient failure, never on a refusal such as a 401.
 */
export function keyedWrite(key: string, extra: Omit<RecoveryOptions, 'idempotent' | 'idempotencyKey'> = {}): { recovery: RecoveryOptions } {
  return { recovery: { ...extra, idempotent: true, idempotencyKey: key } };
}

function withKey(init: RequestInit | undefined, key: string): RequestInit {
  const headers = new Headers(init?.headers);
  headers.set(IDEMPOTENCY_HEADER, key);
  return { ...init, headers };
}

/** The jitter source; tests replace it. */
export const recoveryRandom = { next: (): number => Math.random() };

/** Delay before attempt `n + 1` (n >= 1): full jitter over an exponential cap. */
export function backoffDelay(n: number, base: number, cap: number): number {
  const ceiling = Math.min(cap, base * 2 ** (n - 1));
  return Math.round(ceiling / 2 + recoveryRandom.next() * (ceiling / 2));
}

/** Retry-After as milliseconds (delta-seconds or an HTTP date), or null. */
export function retryAfterMs(value: string | null, now = Date.now()): number | null {
  if (value === null || value.trim() === '') return null;
  const trimmed = value.trim();
  if (/^\d+$/.test(trimmed)) return Number(trimmed) * 1000;
  const at = Date.parse(trimmed);
  return Number.isNaN(at) ? null : Math.max(0, at - now);
}

/** The method and path of a request, for the debug log (no query, no body). */
function describe(input: string, init: RequestInit | undefined): string {
  let path = input;
  try { path = new URL(input, globalThis.location?.href ?? 'http://localhost').pathname; } catch { path = input.split('?')[0] ?? input; }
  return `${(init?.method ?? 'GET').toUpperCase()} ${path}`;
}

function sleep(ms: number, signal: AbortSignal | null | undefined): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) { reject(new DOMException('Aborted', 'AbortError')); return; }
    const timer = setTimeout(() => { signal?.removeEventListener('abort', onAbort); resolve(); }, ms);
    const onAbort = (): void => { clearTimeout(timer); reject(new DOMException('Aborted', 'AbortError')); };
    signal?.addEventListener('abort', onAbort, { once: true });
  });
}

const isOffline = (): boolean => typeof navigator !== 'undefined' && navigator.onLine === false;

/** Resolve when the browser is online again; false after `limit` ms. */
function waitOnline(limit: number, signal: AbortSignal | null | undefined): Promise<boolean> {
  if (!isOffline() || typeof window === 'undefined') return Promise.resolve(true);
  return new Promise((resolve, reject) => {
    const done = (value: boolean): void => { clearTimeout(timer); window.removeEventListener('online', onOnline); signal?.removeEventListener('abort', onAbort); resolve(value); };
    const onOnline = (): void => done(true);
    const onAbort = (): void => { clearTimeout(timer); window.removeEventListener('online', onOnline); reject(new DOMException('Aborted', 'AbortError')); };
    const timer = setTimeout(() => done(false), limit);
    window.addEventListener('online', onOnline);
    signal?.addEventListener('abort', onAbort, { once: true });
  });
}

/** The `code` of a JSON error body, read from a clone. */
async function errorCode(response: Response): Promise<string | null> {
  try {
    const body = (await response.clone().json()) as { code?: unknown } | null;
    return typeof body?.code === 'string' ? body.code : null;
  } catch {
    return null;
  }
}

/** Why `response` (a final, not-ok answer) failed. */
export async function responseReason(response: Response): Promise<FailureReason> {
  const code = await errorCode(response);
  if (code === 'linkedContextUnsupported') return 'linked';
  if (code === 'accountChanged' || code === 'storageAuthorityRequired' || code === 'storageAuthorityUnavailable') return 'account';
  if (response.status === 429) return 'rate';
  if (response.status === 409 || response.status === 412) return 'conflict';
  if (response.status >= 500) return 'server';
  return 'refused';
}

/** Why a thrown error failed. */
export function errorReason(error: unknown): FailureReason {
  if (error instanceof RecoveryError) return error.reason;
  const status = (error as { status?: unknown } | null)?.status;
  if (typeof status === 'number') {
    if (status === 429) return 'rate';
    if (status === 409 || status === 412) return 'conflict';
    if (status >= 500) return 'server';
    return 'refused';
  }
  const name = (error as { name?: unknown } | null)?.name;
  if (name === 'LinkedContextUnsupportedError') return 'linked';
  // t91: the JMAP client wraps a failed send in its own NetworkError and keeps
  // the recovery's reason on it.
  const carried = (error as { reason?: unknown } | null)?.reason;
  if (name === 'NetworkError' && typeof carried === 'string' && REASONS.has(carried as FailureReason)) return carried as FailureReason;
  if (isOffline()) return 'offline';
  return error instanceof TypeError || name === 'NetworkError' ? 'network' : 'refused';
}

/**
 * Send with recovery. `send` makes one attempt with the given init (a timeout
 * signal is merged in when `timeoutMs` is set). Returns the last response; throws
 * a `RecoveryError` when every attempt failed without one, or an `AbortError`
 * when the caller's signal aborted.
 */
export async function recoverRequest(
  input: string,
  init: RequestInit | undefined,
  send: (init: RequestInit | undefined) => Promise<Response>,
  options: RecoveryOptions,
): Promise<Response> {
  const attempts = Math.max(1, options.attempts ?? 4);
  const base = options.baseDelayMs ?? 400;
  const cap = options.maxDelayMs ?? 5000;
  const retryAfterCap = options.maxRetryAfterMs ?? 30_000;
  const timeoutMs = options.timeoutMs ?? 20_000;
  const offlineWait = options.offlineWaitMs ?? 600_000;
  const signal = init?.signal ?? null;
  const label = describe(input, init);
  const key = options.idempotencyKey;
  const idempotent = options.idempotent || key !== undefined;
  let current = key === undefined ? init : withKey(init, key);
  let rebased = false;
  let authorised = false;
  // "Retrying…" is shown only once a retry or an offline wait is under way AND
  // the whole call has run past the slow threshold; a slow first answer alone
  // shows nothing.
  let phase: 'retrying' | 'offline' | null = null;
  let late = false;
  let shown = false;
  const show = (): void => { if (late && phase !== null) { shown = true; options.onSlow?.(phase); } };
  const slowTimer = options.onSlow ? setTimeout(() => { late = true; show(); }, options.slowAfterMs ?? 1000) : undefined;
  const mark = (state: 'retrying' | 'offline'): void => { if (phase !== state) { phase = state; show(); } };
  let offlineRounds = 0;
  try {
    for (let n = 1; ; n += 1) {
      if (isOffline()) {
        mark('offline');
        console.debug(`[recovery] ${label}: offline, waiting to send`);
        if (!(await waitOnline(offlineWait, signal))) throw new RecoveryError('offline');
      }
      let response: Response;
      try {
        response = await attempt(current, send, timeoutMs, signal);
      } catch (error) {
        if (signal?.aborted || (error as { name?: unknown })?.name === 'AbortError') throw error;
        if ((error as { name?: unknown })?.name === 'LinkedContextUnsupportedError') throw error;
        const offline = isOffline();
        // Offline before anything reached the server: wait and send, for any request.
        if (offline && offlineRounds < 5) { offlineRounds += 1; n -= 1; continue; }
        if (!idempotent || n >= attempts) throw new RecoveryError(error instanceof TimeoutSignal ? 'server' : 'network', error);
        const delay = backoffDelay(n, base, cap);
        console.debug(`[recovery] ${label}: attempt ${n} failed (${error instanceof TimeoutSignal ? 'timeout' : 'network'}), retrying in ${delay} ms`);
        mark('retrying');
        await sleep(delay, signal);
        continue;
      }
      if (response.ok) return response;
      if (response.status === 428 && !authorised && (await errorCode(response)) === 'storageAuthorityRequired') {
        const authority = options.authority?.() ?? null;
        const headers = new Headers(current?.headers);
        if (authority !== null && !headers.has(AUTHORITY_HEADER)) {
          authorised = true;
          headers.set(AUTHORITY_HEADER, authority);
          current = { ...current, headers };
          console.debug(`[recovery] ${label}: 428, sending again with the account authority`);
          n -= 1;
          continue;
        }
        return response;
      }
      if ((response.status === 409 || response.status === 412) && options.rebase && !rebased) {
        if ((await errorCode(response)) === 'accountChanged') return response;
        rebased = true;
        const next = await options.rebase(response);
        if (next === null) return response;
        console.debug(`[recovery] ${label}: ${response.status}, re-applied on the current state`);
        // The re-applied request is another body: under the same key the server
        // would refuse it, so it gets a key of its own.
        current = { ...(key === undefined ? next : withKey(next, `${key}.r`)), ...(signal ? { signal } : {}) };
        n -= 1;
        continue;
      }
      const transient = TRANSIENT.has(response.status) || response.status === 429;
      if (!transient || !idempotent || n >= attempts) return response;
      let delay = backoffDelay(n, base, cap);
      if (response.status === 429) {
        const after = retryAfterMs(response.headers.get('retry-after'));
        if (after === null || after > retryAfterCap) return response;
        delay = Math.max(after, 0);
      }
      console.debug(`[recovery] ${label}: attempt ${n} answered ${response.status}, retrying in ${delay} ms`);
      mark('retrying');
      await sleep(delay, signal);
    }
  } finally {
    clearTimeout(slowTimer);
    if (shown) options.onSlow?.(null);
  }
}

/** Our own per-attempt timeout (told apart from the caller's abort). */
class TimeoutSignal extends Error {
  constructor() { super('request timed out'); this.name = 'TimeoutError'; }
}

async function attempt(init: RequestInit | undefined, send: (init: RequestInit | undefined) => Promise<Response>, timeoutMs: number, signal: AbortSignal | null): Promise<Response> {
  if (timeoutMs <= 0 || typeof AbortController === 'undefined') return send(init);
  const controller = new AbortController();
  let timedOut = false;
  const timer = setTimeout(() => { timedOut = true; controller.abort(); }, timeoutMs);
  const forward = (): void => controller.abort();
  signal?.addEventListener('abort', forward, { once: true });
  try {
    return await send({ ...init, signal: controller.signal });
  } catch (error) {
    if (timedOut && !signal?.aborted) throw new TimeoutSignal();
    throw error;
  } finally {
    clearTimeout(timer);
    signal?.removeEventListener('abort', forward);
  }
}

/** The error to throw for a final, not-ok response, carrying its status and reason. */
export async function failureOf(response: Response): Promise<RecoveryError & { status: number }> {
  const error = new RecoveryError(await responseReason(response)) as RecoveryError & { status: number };
  Object.defineProperty(error, 'status', { value: response.status, enumerable: true });
  error.message = `request failed: ${error.reason} (${response.status})`;
  return error;
}

/**
 * `{ recovery }` for a write that sets a whole value (PUT, or a DELETE the
 * caller names), so a `defaultFetcher` can opt its writes in; `{}` for
 * anything else (a GET is not a user-facing write; a POST may create twice;
 * a keepalive send outlives the page and is not retried).
 */
export function idempotentWrite(init: RequestInit | undefined, methods: readonly string[] = ['PUT']): { recovery?: RecoveryOptions } {
  const method = (init?.method ?? 'GET').toUpperCase();
  if (!methods.includes(method) || init?.keepalive === true) return {};
  if (typeof ReadableStream !== 'undefined' && init?.body instanceof ReadableStream) return {};
  return { recovery: { idempotent: true } };
}

// ── t91: one key per action, kept across a manual retry ─────────────────────
// An editor's Save is its own Retry: after a failed create the user presses it
// again with the same content. If the first attempt did land (its answer was
// lost), a fresh key would create a second record. So a key whose request
// failed in transit is remembered by the request it was sent with, and the
// same request sent again within the server's window reuses it. A request that
// was answered (success or refusal) forgets its key: a later identical create
// is a new one.

const KEPT_KEY_MS = 9 * 60 * 1000;
const KEPT_KEY_LIMIT = 100;
const keptKeys = new Map<string, { key: string; at: number }>();

/** Whether `error` left it unknown if the request reached the server. */
function inTransit(error: unknown): boolean {
  if ((error as { name?: unknown } | null)?.name === 'AbortError') return true;
  const reason = errorReason(error);
  return reason === 'network' || reason === 'server' || reason === 'offline' || reason === 'rate';
}

/**
 * Run `send` with an idempotency key for the request `identity` (its scope and
 * exact body): the key of an earlier attempt of the same request that failed
 * in transit, or a fresh one.
 */
export async function keyedAttempt<T>(identity: string, send: (key: string) => Promise<T>, now: () => number = Date.now): Promise<T> {
  const at = now();
  for (const [id, kept] of keptKeys) if (at - kept.at > KEPT_KEY_MS) keptKeys.delete(id);
  const key = keptKeys.get(identity)?.key ?? newIdempotencyKey();
  try {
    const out = await send(key);
    keptKeys.delete(identity);
    return out;
  } catch (error) {
    if (inTransit(error)) {
      keptKeys.delete(identity);
      keptKeys.set(identity, { key, at });
      while (keptKeys.size > KEPT_KEY_LIMIT) keptKeys.delete(keptKeys.keys().next().value as string);
    } else {
      keptKeys.delete(identity);
    }
    throw error;
  }
}

/** Tests: forget every kept key. */
export function clearKeptKeys(): void { keptKeys.clear(); }
