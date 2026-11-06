// Pacing for requests the server may answer with 429 or 503 (t106).
//
// `apiFetch` (`http.ts`) sends every request through `rateAwareSend`. It adds:
//
//   - Retry of a request that is safe to send twice (a GET, a read-only JMAP
//     batch, `/api/sanitize`), after a 429, or a 503 that names a Retry-After:
//     at most MAX_RETRIES times, waiting what Retry-After says (plus jitter) or an
//     exponential, jittered delay, and never more than MAX_TOTAL_WAIT_MS in all. A
//     wait the budget cannot cover is not started: the response is returned.
//     A write is never repeated here (a JMAP "limit" refusal, which ran nothing,
//     is repeated by `client.ts`).
//   - Two classes of request. A `foreground` request (the user asked for it) is
//     sent at once. A `background` one (prefetch of neighbouring messages and
//     folders) shares MAX_BACKGROUND slots and is held back while any 429 is in
//     its Retry-After window, so prefetch never competes with what the user
//     is waiting for and never keeps hammering a server that said wait.
//   - One in-flight GET per identical request (same URL and headers, no abort
//     signal): the second caller receives a copy of the first answer.
//   - Counters (429s seen, retries, requests that gave up) that the client timing
//     report sends to the server, and events for the one toast shown when a
//     request gave up.
//
// Nothing here logs a URL, header or body.

/** Who is waiting for the answer. */
export type PaceClass = 'foreground' | 'background';

export const MAX_RETRIES = 3;
/** The most time one request spends waiting to be sent again. */
export const MAX_TOTAL_WAIT_MS = 10_000;
export const BASE_DELAY_MS = 500;
/** Background requests in flight at once. */
export const MAX_BACKGROUND = 6;
/** Pause applied to background work after a 429 that names no Retry-After. */
const DEFAULT_BACKOFF_MS = 1_000;

/** Timer and random sources; tests replace them. */
export const rateRuntime = {
  now: (): number => Date.now(),
  random: (): number => Math.random(),
  sleep: (ms: number, signal?: AbortSignal): Promise<void> =>
    new Promise((resolve, reject) => {
      if (signal?.aborted === true) { reject(signal.reason); return; }
      const onAbort = (): void => { clearTimeout(timer); reject(signal!.reason); };
      const timer = setTimeout(() => { signal?.removeEventListener('abort', onAbort); resolve(); }, ms);
      signal?.addEventListener('abort', onAbort, { once: true });
    }),
};

export type RateEvent =
  /** A request is waiting to be sent again; `untilMs` is when. */
  | { kind: 'waiting'; untilMs: number }
  /** A request gave up; `untilMs` is when the server said to try again. */
  | { kind: 'exhausted'; untilMs: number }
  /** A request that had been waiting succeeded. */
  | { kind: 'clear' };

type Listener = (event: RateEvent) => void;
const listeners = new Set<Listener>();

/** Subscribe to {@link RateEvent}s. Returns the unsubscribe function. */
export function onRateEvent(listener: Listener): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
function emit(event: RateEvent): void {
  for (const l of listeners) {
    try { l(event); } catch { /* a listener never breaks a request */ }
  }
}

/** What the timing report sends: counts since the last report. */
export interface RateCounts { seen: number; retries: number; exhausted: number }
const totals: RateCounts = { seen: 0, retries: 0, exhausted: 0 };
let reported: RateCounts = { seen: 0, retries: 0, exhausted: 0 };

/** All counts since the page loaded. */
export function rateTotals(): Readonly<RateCounts> {
  return { ...totals };
}

/** The counts since the last call (the next report's `rate` field), or `null` when all are zero. */
export function takeRateReport(): RateCounts | null {
  const delta = {
    seen: totals.seen - reported.seen,
    retries: totals.retries - reported.retries,
    exhausted: totals.exhausted - reported.exhausted,
  };
  reported = { ...totals };
  return delta.seen === 0 && delta.retries === 0 && delta.exhausted === 0 ? null : delta;
}

// ─── backoff and background slots ──────────────────────────────────────────────

let backoffUntil = 0;
let activeBackground = 0;
const slotWaiters: Array<() => void> = [];

/** Whether a Retry-After window is open. */
export function backoffActive(): boolean {
  return rateRuntime.now() < backoffUntil;
}

function noteBackoff(ms: number): void {
  backoffUntil = Math.max(backoffUntil, rateRuntime.now() + ms);
}

async function acquireBackground(signal?: AbortSignal): Promise<void> {
  for (;;) {
    if (signal?.aborted === true) throw signal.reason;
    const remaining = backoffUntil - rateRuntime.now();
    if (remaining > 0) { await rateRuntime.sleep(remaining, signal); continue; }
    if (activeBackground < MAX_BACKGROUND) { activeBackground += 1; return; }
    await new Promise<void>((resolve, reject) => {
      const wake = (): void => { signal?.removeEventListener('abort', onAbort); resolve(); };
      const onAbort = (): void => {
        const at = slotWaiters.indexOf(wake);
        if (at >= 0) slotWaiters.splice(at, 1);
        reject(signal!.reason);
      };
      slotWaiters.push(wake);
      signal?.addEventListener('abort', onAbort, { once: true });
    });
  }
}

function releaseBackground(): void {
  activeBackground -= 1;
  slotWaiters.shift()?.();
}

// ─── what may be sent again ───────────────────────────────────────────────────

const READ_METHOD = /\/(get|query|changes|queryChanges)$/;

/** True when every call of a JMAP batch only reads. */
export function isReadOnlyJmap(body: BodyInit | null | undefined): boolean {
  if (typeof body !== 'string') return false;
  try {
    const calls = (JSON.parse(body) as { methodCalls?: unknown }).methodCalls;
    if (!Array.isArray(calls) || calls.length === 0) return false;
    return calls.every((call) => Array.isArray(call) && typeof call[0] === 'string' && (call[0] === 'Core/echo' || READ_METHOD.test(call[0])));
  } catch {
    return false;
  }
}

/** Whether sending this request again leaves the same state as sending it once. */
export function safeToRepeat(method: string, path: string, body: BodyInit | null | undefined): boolean {
  if (method === 'GET' || method === 'HEAD') return true;
  if (method !== 'POST') return false;
  if (path === '/api/sanitize') return true;
  return path === '/jmap/api' && isReadOnlyJmap(body);
}

/** A JMAP "limit" refusal ran none of its calls; `client.ts` waits and sends it again. */
async function isJmapLimit(res: Response, path: string): Promise<boolean> {
  if (path !== '/jmap/api' || res.status !== 429) return false;
  const body = (await res.clone().json().catch(() => null)) as { type?: unknown } | null;
  return body?.type === 'urn:ietf:params:jmap:error:limit';
}

/** Retry-After as milliseconds: delta-seconds or an HTTP date; `null` when absent or unreadable. */
export function parseRetryAfter(value: string | null, now: number = rateRuntime.now()): number | null {
  if (value === null) return null;
  const text = value.trim();
  if (/^\d+$/.test(text)) return Math.min(Number(text), 3600) * 1000;
  const at = Date.parse(text);
  return Number.isNaN(at) ? null : Math.max(0, Math.min(at - now, 3600_000));
}

/** Delay before retry `attempt` (0-based): Retry-After plus a little jitter, else full jitter over an exponential cap. */
export function retryDelay(attempt: number, retryAfterMs: number | null): number {
  if (retryAfterMs !== null) return retryAfterMs + Math.floor(rateRuntime.random() * 250);
  const cap = BASE_DELAY_MS * 2 ** attempt;
  return Math.floor(cap / 2 + rateRuntime.random() * (cap / 2));
}

// ─── the wrapper ───────────────────────────────────────────────────────────────

export interface RateRequest {
  method: string;
  /** The path without the deploy prefix. */
  path: string;
  body?: BodyInit | null | undefined;
  pace?: PaceClass | undefined;
  signal?: AbortSignal | null | undefined;
}

async function withRetries(req: RateRequest, send: () => Promise<Response>): Promise<Response> {
  const repeatable = safeToRepeat(req.method, req.path, req.body);
  const signal = req.signal ?? undefined;
  let waited = 0;
  let waiting = false;
  for (let attempt = 0; ; attempt += 1) {
    const res = await send();
    const limited = res.status === 429;
    const unavailable = res.status === 503 && res.headers.get('retry-after') !== null;
    if (!limited && !unavailable) {
      if (waiting) emit({ kind: 'clear' });
      return res;
    }
    if (limited) totals.seen += 1;
    const advised = parseRetryAfter(res.headers.get('retry-after'));
    noteBackoff(advised ?? DEFAULT_BACKOFF_MS);
    const delay = retryDelay(attempt, advised);
    const retry = repeatable && attempt < MAX_RETRIES && waited + delay <= MAX_TOTAL_WAIT_MS
      && !(await isJmapLimit(res, req.path)) && signal?.aborted !== true;
    if (!retry) {
      if (repeatable) {
        totals.exhausted += 1;
        emit({ kind: 'exhausted', untilMs: Math.max(backoffUntil, rateRuntime.now() + 1000) });
      }
      return res;
    }
    waiting = true;
    emit({ kind: 'waiting', untilMs: rateRuntime.now() + delay });
    await rateRuntime.sleep(delay, signal);
    waited += delay;
    totals.retries += 1;
  }
}

const sharing = new Map<string, Promise<Response>>();

/** A readable copy for one more caller (a stand-in without `clone` is passed on as is). */
function copy(res: Response): Response {
  return typeof res.clone === 'function' ? res.clone() : res;
}

/**
 * Send one request with the pacing above. `send` performs the actual fetch and
 * may be called again for a retry; it must not consume a one-shot body.
 */
export async function rateAwareSend(req: RateRequest, send: () => Promise<Response>, share?: string): Promise<Response> {
  const signal = req.signal ?? undefined;
  if (share !== undefined && signal === undefined) {
    const running = sharing.get(share);
    if (running !== undefined) return copy(await running);
    const work = runOne(req, send).finally(() => { sharing.delete(share); });
    sharing.set(share, work);
    return copy(await work);
  }
  return runOne(req, send);
}

async function runOne(req: RateRequest, send: () => Promise<Response>): Promise<Response> {
  const background = req.pace === 'background';
  if (background) await acquireBackground(req.signal ?? undefined);
  try {
    return await withRetries(req, send);
  } finally {
    if (background) releaseBackground();
  }
}

/** Forget all state (tests). */
export function resetRateState(): void {
  backoffUntil = 0;
  activeBackground = 0;
  slotWaiters.length = 0;
  sharing.clear();
  totals.seen = 0; totals.retries = 0; totals.exhausted = 0;
  reported = { seen: 0, retries: 0, exhausted: 0 };
  listeners.clear();
}
