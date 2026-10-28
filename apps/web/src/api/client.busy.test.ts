// A JMAP request the server refused as over its concurrency ceiling (t33).
//
// mw-server admits a bounded number of JMAP requests at once and answers the
// rest 429 with the RFC 8620 limit problem `maxConcurrentRequests`, before any
// of their calls run. A send racing the open readers' loads hit it and the
// composer showed "request failed with 429" for a send that was never tried.
// The client sends such a request again after a short wait; any other 429 is
// the caller's to handle.

import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { ApiError, createClient } from './client.ts';
import type { JmapRequest } from './jmap-types.ts';

const realFetch = globalThis.fetch;
const BODY: JmapRequest = { using: [], methodCalls: [['Email/get', { accountId: 'a', ids: ['m1'] }, 'g']] };
const OK = { methodResponses: [['Email/get', { list: [] }, 'g']], sessionState: 's' };

function busy(limit = 'maxConcurrentRequests'): Response {
  return new Response(JSON.stringify({ type: 'urn:ietf:params:jmap:error:limit', status: 429, limit }), {
    status: 429,
    headers: { 'content-type': 'application/problem+json' },
  });
}
const ok = (): Response => new Response(JSON.stringify(OK));

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  vi.useRealTimers();
  globalThis.fetch = realFetch;
  vi.restoreAllMocks();
});

describe('jmap over the concurrency ceiling', () => {
  it('sends the same request again after a wait and returns its answer', async () => {
    const fetch = vi.fn().mockResolvedValueOnce(busy()).mockResolvedValueOnce(busy()).mockResolvedValueOnce(ok());
    globalThis.fetch = fetch;
    const answer = createClient('').jmap(BODY);
    await vi.runAllTimersAsync();
    await expect(answer).resolves.toEqual(OK);
    expect(fetch).toHaveBeenCalledTimes(3);
    const bodies = fetch.mock.calls.map((call) => (call[1] as RequestInit).body);
    expect(new Set(bodies)).toEqual(new Set([JSON.stringify(BODY)]));
  });

  it('gives up after the last retry with the 429', async () => {
    const fetch = vi.fn(async () => busy());
    globalThis.fetch = fetch;
    const answer = createClient('').jmap(BODY);
    const settled = answer.catch((error: unknown) => error);
    await vi.runAllTimersAsync();
    const error = await settled;
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(429);
    expect(fetch).toHaveBeenCalledTimes(6);
  });

  it('does not retry a 429 for another limit', async () => {
    const fetch = vi.fn(async () => busy('maxCallsInRequest'));
    globalThis.fetch = fetch;
    const settled = createClient('').jmap(BODY).catch((error: unknown) => error);
    await vi.runAllTimersAsync();
    expect(((await settled) as ApiError).status).toBe(429);
    expect(fetch).toHaveBeenCalledTimes(1);
  });

  it('stops waiting when the caller aborts', async () => {
    const fetch = vi.fn(async () => busy());
    globalThis.fetch = fetch;
    const controller = new AbortController();
    const settled = createClient('').jmap(BODY, { signal: controller.signal }).catch((error: unknown) => error);
    await vi.advanceTimersByTimeAsync(0);
    controller.abort(new DOMException('superseded', 'AbortError'));
    await vi.runAllTimersAsync();
    expect(((await settled) as DOMException).name).toBe('AbortError');
    expect(fetch).toHaveBeenCalledTimes(1);
  });
});
