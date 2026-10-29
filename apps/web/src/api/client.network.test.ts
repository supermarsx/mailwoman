// Network up/down reporting from `guarded` (t39 C1).
//
// The "connection lost" toast is cleared by the next "online" report. A retry
// that reached the server but was refused (401 after the session lapsed while
// offline) used to report nothing, so the toast stayed on screen although the
// server had answered.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { ApiError, NetworkError, createClient } from './client.ts';

const realFetch = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = realFetch;
  vi.restoreAllMocks();
});

const unauthorized = (): Response =>
  new Response(JSON.stringify({ error: 'unauthenticated' }), {
    status: 401,
    headers: { 'content-type': 'application/json' },
  });

describe('network reports', () => {
  it('reports offline on a network failure, then online when a retry ends in 401', async () => {
    globalThis.fetch = vi
      .fn()
      .mockRejectedValueOnce(new TypeError('Failed to fetch'))
      .mockResolvedValueOnce(unauthorized());
    const client = createClient('');
    const seen: boolean[] = [];
    client.onNetwork((up) => seen.push(up));

    await expect(client.me()).rejects.toBeInstanceOf(NetworkError);
    expect(seen).toEqual([false]);

    await expect(client.me()).rejects.toBeInstanceOf(ApiError);
    expect(seen).toEqual([false, true]);
  });

  it('reports online for a 500 answer as well: the server was reached', async () => {
    globalThis.fetch = vi.fn(async () => new Response('boom', { status: 500 }));
    const client = createClient('');
    const seen: boolean[] = [];
    client.onNetwork((up) => seen.push(up));
    await expect(client.me()).rejects.toBeInstanceOf(ApiError);
    expect(seen).toEqual([true]);
  });
});
