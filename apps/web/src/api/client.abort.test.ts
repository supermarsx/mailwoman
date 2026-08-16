// Cancellation on the JMAP transport (t22-e4).
//
// The list layer supersedes its own in-flight queries, and with append-on-scroll
// every scroll issues one, so aborts are ORDINARY here rather than exceptional.
// That is what makes the second describe block load-bearing: `req` maps a failed
// `fetch` to a `NetworkError`, and `guarded` turns a `NetworkError` into
// `notify(false)` — which is what `state/slices/offline.ts` replays its outbound
// queue on and what flips the app into offline mode. Threading the signal without
// exempting deliberate cancellation would announce "offline" during scrolling.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { createClient, NetworkError } from './client.ts';
import type { JmapRequest } from './jmap-types.ts';

const REQUEST: JmapRequest = { using: [], methodCalls: [] };

const realFetch = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = realFetch;
  vi.restoreAllMocks();
});

/** A fetch that never resolves until the caller's signal aborts. */
function abortableFetch(): ReturnType<typeof vi.fn> {
  return vi.fn(
    (_input: string, init?: RequestInit) =>
      new Promise<Response>((_resolve, reject) => {
        const signal = init?.signal;
        if (signal == null) return; // hangs: proves the signal was NOT passed
        if (signal.aborted) {
          reject(new DOMException('The operation was aborted.', 'AbortError'));
          return;
        }
        signal.addEventListener('abort', () =>
          reject(new DOMException('The operation was aborted.', 'AbortError')),
        );
      }),
  );
}

describe('jmap cancellation', () => {
  it('passes the signal to fetch, so an abort actually rejects the request', async () => {
    const fetchMock = abortableFetch();
    globalThis.fetch = fetchMock as unknown as typeof fetch;
    const client = createClient('');
    const controller = new AbortController();

    const pending = client.jmap(REQUEST, { signal: controller.signal });
    // If the signal were dropped, the fake never settles and this times out —
    // which is the point: a passing assertion cannot be faked by a hang.
    controller.abort();
    await expect(pending).rejects.toThrow(/abort/i);

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit | undefined;
    expect(init?.signal).toBe(controller.signal);
  });

  it('rejects with the abort reason, NOT a NetworkError', async () => {
    globalThis.fetch = abortableFetch() as unknown as typeof fetch;
    const client = createClient('');
    const controller = new AbortController();
    const pending = client.jmap(REQUEST, { signal: controller.signal });
    controller.abort();
    await expect(pending).rejects.not.toBeInstanceOf(NetworkError);
  });

  it('an abort does not announce an offline transition', async () => {
    // The regression the naive two-line version ships: every superseded scroll
    // fetch would drive `offline.ts` and the app-wide online flag to false.
    globalThis.fetch = abortableFetch() as unknown as typeof fetch;
    const client = createClient('');
    const seen: boolean[] = [];
    client.onNetwork((up) => seen.push(up));

    const controller = new AbortController();
    const pending = client.jmap(REQUEST, { signal: controller.signal });
    controller.abort();
    await expect(pending).rejects.toThrow();

    expect(seen).toEqual([]);
  });

  it('a REAL network failure is still a NetworkError and still reports offline', async () => {
    // The control for the assertion above: without it, "no offline transition"
    // would also hold for a client that never reports one at all.
    globalThis.fetch = vi.fn(() => Promise.reject(new TypeError('failed to fetch'))) as unknown as typeof fetch;
    const client = createClient('');
    const seen: boolean[] = [];
    client.onNetwork((up) => seen.push(up));

    await expect(client.jmap(REQUEST)).rejects.toBeInstanceOf(NetworkError);
    expect(seen).toEqual([false]);
  });

  it('omits the signal entirely when none is given, leaving the default path unchanged', async () => {
    const fetchMock = vi.fn((_input: string, _init?: RequestInit) =>
      Promise.resolve(new Response(JSON.stringify({ methodResponses: [], sessionState: 's' }), { status: 200 })),
    );
    globalThis.fetch = fetchMock as unknown as typeof fetch;
    const client = createClient('');

    await client.jmap(REQUEST);

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit | undefined;
    expect(init).toBeDefined();
    expect('signal' in init!).toBe(false);
    expect(init?.credentials).toBe('same-origin');
  });
});
