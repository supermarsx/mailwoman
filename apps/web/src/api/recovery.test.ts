// t82: the recovery policy (`recovery.ts`), with fake timers. Each case drives a
// fake `send` and checks what was sent, how often and after which delays.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { backoffDelay, recoverRequest, recoveryRandom, retryAfterMs, RecoveryError, type RecoveryOptions } from './recovery.ts';

const json = (status: number, body: unknown = {}, headers: Record<string, string> = {}): Response =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json', ...headers } });

function sender(answers: (Response | Error)[]) {
  const calls: (RequestInit | undefined)[] = [];
  const send = vi.fn(async (init: RequestInit | undefined) => {
    calls.push(init);
    const next = answers.shift();
    if (next === undefined) throw new Error('no answer left');
    if (next instanceof Error) throw next;
    return next;
  });
  return { send, calls };
}

const PUT: RequestInit = { method: 'PUT', body: '{"value":1}', headers: { 'content-type': 'application/json' } };
const run = (s: ReturnType<typeof sender>, options: RecoveryOptions, init: RequestInit = PUT): Promise<Response> =>
  recoverRequest('/api/settings/x', init, s.send, { timeoutMs: 0, ...options });

describe('recoverRequest', () => {
  let debug: ReturnType<typeof vi.spyOn>;
  beforeEach(() => {
    vi.useFakeTimers();
    recoveryRandom.next = () => 1;
    debug = vi.spyOn(console, 'debug').mockImplementation(() => undefined);
  });
  afterEach(() => {
    vi.useRealTimers();
    recoveryRandom.next = () => Math.random();
    debug.mockRestore();
    vi.unstubAllGlobals();
  });

  it('retries 503 and network errors with exponential backoff for an idempotent request', async () => {
    const s = sender([json(503), new TypeError('Failed to fetch'), json(200, { ok: true })]);
    const done = run(s, { idempotent: true });
    await vi.advanceTimersByTimeAsync(0);
    expect(s.send).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(399);
    expect(s.send).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1); // 400 ms after the first
    expect(s.send).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(800); // doubled
    expect(s.send).toHaveBeenCalledTimes(3);
    expect((await done).status).toBe(200);
  });

  it('caps the attempts and returns the last answer', async () => {
    const s = sender([json(502), json(504), json(503)]);
    const done = run(s, { idempotent: true, attempts: 3 });
    await vi.runAllTimersAsync();
    expect((await done).status).toBe(503);
    expect(s.send).toHaveBeenCalledTimes(3);
  });

  it('never retries a request that is not idempotent', async () => {
    const s = sender([json(503)]);
    const done = run(s, { idempotent: false }, { method: 'POST', body: 'x' });
    await vi.runAllTimersAsync();
    expect((await done).status).toBe(503);
    expect(s.send).toHaveBeenCalledTimes(1);

    const failed = sender([new TypeError('Failed to fetch')]);
    const thrown = run(failed, { idempotent: false }, { method: 'POST', body: 'x' }).catch((e: unknown) => e);
    await vi.runAllTimersAsync();
    const error = await thrown;
    expect(error).toBeInstanceOf(RecoveryError);
    expect((error as RecoveryError).reason).toBe('network');
    expect(failed.send).toHaveBeenCalledTimes(1);
  });

  it('waits the Retry-After of a 429 before sending again', async () => {
    const s = sender([json(429, {}, { 'retry-after': '3' }), json(200)]);
    const done = run(s, { idempotent: true });
    await vi.advanceTimersByTimeAsync(2999);
    expect(s.send).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(s.send).toHaveBeenCalledTimes(2);
    expect((await done).status).toBe(200);
  });

  it('does not wait for a 429 without Retry-After or with one past the cap', async () => {
    const bare = sender([json(429)]);
    const a = run(bare, { idempotent: true });
    await vi.runAllTimersAsync();
    expect((await a).status).toBe(429);
    expect(bare.send).toHaveBeenCalledTimes(1);
    const long = sender([json(429, {}, { 'retry-after': '3600' })]);
    const b = run(long, { idempotent: true });
    await vi.runAllTimersAsync();
    expect((await b).status).toBe(429);
    expect(long.send).toHaveBeenCalledTimes(1);
  });

  it('re-applies the change on the current state after a 409, once', async () => {
    const s = sender([json(409, { code: 'conflict' }), json(200, { saved: true })]);
    const rebase = vi.fn(async () => ({ method: 'PUT', body: '{"value":1,"base":"v2"}', headers: { 'if-match': '"v2"' } }));
    const done = run(s, { idempotent: true, rebase });
    await vi.runAllTimersAsync();
    expect((await done).status).toBe(200);
    expect(rebase).toHaveBeenCalledTimes(1);
    expect(new Headers(s.calls[1]?.headers).get('if-match')).toBe('"v2"');
    expect(s.calls[1]?.body).toBe('{"value":1,"base":"v2"}');

    // A change that does not merge: the conflict is returned, nothing resent.
    const clash = sender([json(412)]);
    const c = run(clash, { idempotent: true, rebase: async () => null });
    await vi.runAllTimersAsync();
    expect((await c).status).toBe(412);
    expect(clash.send).toHaveBeenCalledTimes(1);

    // A second conflict after the rebase is returned (no loop).
    const twice = sender([json(409), json(409)]);
    const d = run(twice, { idempotent: true, rebase: async () => PUT });
    await vi.runAllTimersAsync();
    expect((await d).status).toBe(409);
    expect(twice.send).toHaveBeenCalledTimes(2);
  });

  it('never rebases over the account fence (409 accountChanged)', async () => {
    const s = sender([json(409, { code: 'accountChanged' })]);
    const rebase = vi.fn(async () => PUT);
    const done = run(s, { idempotent: true, rebase });
    await vi.runAllTimersAsync();
    expect((await done).status).toBe(409);
    expect(rebase).not.toHaveBeenCalled();
  });

  it('resumes after a 428 storageAuthorityRequired with the account authority, once', async () => {
    const s = sender([json(428, { code: 'storageAuthorityRequired' }), json(200)]);
    const done = run(s, { idempotent: false, authority: () => 'authority-0123456789' }, { method: 'POST', body: 'x' });
    await vi.runAllTimersAsync();
    expect((await done).status).toBe(200);
    expect(new Headers(s.calls[1]?.headers).get('x-mailwoman-storage-authority')).toBe('authority-0123456789');

    // No authority in hand: the 428 is returned as it is.
    const none = sender([json(428, { code: 'storageAuthorityRequired' })]);
    const n = run(none, { idempotent: true, authority: () => null });
    await vi.runAllTimersAsync();
    expect((await n).status).toBe(428);
    expect(none.send).toHaveBeenCalledTimes(1);
  });

  it('times out a hung attempt and retries it', async () => {
    let first = true;
    const send = vi.fn((init: RequestInit | undefined) => new Promise<Response>((resolve, reject) => {
      if (!first) { resolve(json(200)); return; }
      first = false;
      init?.signal?.addEventListener('abort', () => reject(new DOMException('Aborted', 'AbortError')));
    }));
    const done = recoverRequest('/api/x', PUT, send, { idempotent: true, timeoutMs: 1000 });
    await vi.advanceTimersByTimeAsync(1000 + 400);
    expect((await done).status).toBe(200);
    expect(send).toHaveBeenCalledTimes(2);
  });

  it('stops at once when the caller aborts', async () => {
    const controller = new AbortController();
    const s = sender([json(503), json(200)]);
    const done = run(s, { idempotent: true }, { ...PUT, signal: controller.signal }).catch((e: unknown) => e);
    await vi.advanceTimersByTimeAsync(10);
    controller.abort();
    const error = await done;
    expect((error as Error).name).toBe('AbortError');
    expect(s.send).toHaveBeenCalledTimes(1);
  });

  it('reports "Retrying…" only after the slow threshold, and clears it', async () => {
    const states: (string | null)[] = [];
    const fast = sender([json(503), json(200)]);
    const a = run(fast, { idempotent: true, onSlow: (s) => states.push(s) });
    await vi.runAllTimersAsync();
    await a;
    expect(states).toEqual([]); // recovered in 400 ms: nothing shown

    const slow = sender([json(503), json(503), json(200)]);
    const b = run(slow, { idempotent: true, onSlow: (s) => states.push(s) });
    await vi.advanceTimersByTimeAsync(1000);
    expect(states).toEqual(['retrying']);
    await vi.runAllTimersAsync();
    await b;
    expect(states).toEqual(['retrying', null]);
  });

  it('waits for the browser to come back online, then sends (any request)', async () => {
    let online = false;
    vi.stubGlobal('navigator', { get onLine() { return online; } });
    const s = sender([json(200)]);
    const done = run(s, { idempotent: false }, { method: 'POST', body: 'x' });
    await vi.advanceTimersByTimeAsync(5000);
    expect(s.send).not.toHaveBeenCalled();
    online = true;
    window.dispatchEvent(new Event('online'));
    await vi.advanceTimersByTimeAsync(0);
    expect((await done).status).toBe(200);
    expect(s.send).toHaveBeenCalledTimes(1);
  });

  it('gives up waiting offline after the bound', async () => {
    vi.stubGlobal('navigator', { onLine: false });
    const s = sender([json(200)]);
    const done = run(s, { idempotent: true, offlineWaitMs: 2000 }).catch((e: unknown) => e);
    await vi.advanceTimersByTimeAsync(2000);
    const error = await done;
    expect((error as RecoveryError).reason).toBe('offline');
    expect(s.send).not.toHaveBeenCalled();
  });

  it('logs at debug level only, with the method and path and no query or body', async () => {
    const s = sender([json(503), json(200)]);
    const done = recoverRequest('/api/settings/x?token=secret', PUT, s.send, { idempotent: true, timeoutMs: 0 });
    await vi.runAllTimersAsync();
    await done;
    expect(debug).toHaveBeenCalledTimes(1);
    const line = String(debug.mock.calls[0]?.[0]);
    expect(line).toContain('PUT /api/settings/x');
    expect(line).not.toContain('secret');
    expect(line).not.toContain('value');
  });
});

describe('helpers', () => {
  it('parses Retry-After seconds and dates', () => {
    expect(retryAfterMs('2')).toBe(2000);
    expect(retryAfterMs(new Date(10_000).toUTCString(), 4_000)).toBe(6000);
    expect(retryAfterMs('soon')).toBeNull();
    expect(retryAfterMs(null)).toBeNull();
  });
  it('keeps backoff within half and all of the capped ceiling', () => {
    recoveryRandom.next = () => 0;
    expect(backoffDelay(1, 400, 5000)).toBe(200);
    expect(backoffDelay(5, 400, 5000)).toBe(2500);
    recoveryRandom.next = () => 1;
    expect(backoffDelay(5, 400, 5000)).toBe(5000);
    recoveryRandom.next = () => Math.random();
  });
});
