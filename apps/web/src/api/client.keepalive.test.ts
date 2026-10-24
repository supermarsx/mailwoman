// JMAP saves that outlive the page (t32-h11, 26.20).
//
// A note or task save starts on the keystroke or the Save click; if the tab is
// closed or reloaded right after, an ordinary fetch is cancelled with the page
// and the write can be lost. `jmap(body, { keepalive: true })` asks the browser
// to finish the request after the page is gone. Browsers allow 64 KiB of
// keepalive bodies in flight at once and reject a fetch over that (a rejection
// the client would report as "offline"), so a body that does not fit in what is
// left goes as an ordinary request. The CSRF header and the cookie mode stay as
// they are.

import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { createClient } from './client.ts';
import type { JmapRequest } from './jmap-types.ts';

const realFetch = globalThis.fetch;

function setRequest(text: string): JmapRequest {
  return { using: [], methodCalls: [['Note/set', { accountId: 'a', update: { n1: { bodyHtml: text } } }, 's']] };
}

/** A fetch whose answers the test releases one by one. */
function heldFetch(): { fetch: ReturnType<typeof vi.fn>; release(): void } {
  const waiting: Array<() => void> = [];
  const fn = vi.fn(
    () =>
      new Promise<Response>((resolve) => {
        waiting.push(() => resolve(new Response(JSON.stringify({ methodResponses: [], sessionState: 's' }))));
      }),
  );
  return { fetch: fn, release: () => waiting.shift()?.() };
}

function initOf(fn: ReturnType<typeof vi.fn>, call: number): RequestInit {
  return fn.mock.calls[call]?.[1] as RequestInit;
}

beforeEach(() => {
  document.cookie = 'mw_csrf=tok-123; path=/';
});

afterEach(() => {
  globalThis.fetch = realFetch;
  document.cookie = 'mw_csrf=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
  vi.restoreAllMocks();
});

describe('jmap keepalive', () => {
  it('sends a small save with keepalive, the CSRF header and the same-origin cookie mode', async () => {
    const held = heldFetch();
    globalThis.fetch = held.fetch as unknown as typeof fetch;
    const client = createClient('');

    const pending = client.jmap(setRequest('short'), { keepalive: true });
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(1));
    const init = initOf(held.fetch, 0);
    expect(init.keepalive).toBe(true);
    expect(init.credentials).toBe('same-origin');
    expect(new Headers(init.headers).get('X-CSRF-Token')).toBe('tok-123');
    held.release();
    await pending;
  });

  it('does not ask for keepalive unless the caller does', async () => {
    const held = heldFetch();
    globalThis.fetch = held.fetch as unknown as typeof fetch;
    const client = createClient('');

    const pending = client.jmap(setRequest('short'));
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(1));
    expect(initOf(held.fetch, 0).keepalive).not.toBe(true);
    held.release();
    await pending;
  });

  it('sends a body over the 64 KiB keepalive limit as an ordinary request', async () => {
    const held = heldFetch();
    globalThis.fetch = held.fetch as unknown as typeof fetch;
    const client = createClient('');

    const pending = client.jmap(setRequest('x'.repeat(70 * 1024)), { keepalive: true });
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(1));
    expect(initOf(held.fetch, 0).keepalive).not.toBe(true);
    held.release();
    await pending;
  });

  it('keeps the keepalive bodies in flight under the limit, and frees the room when one answers', async () => {
    const held = heldFetch();
    globalThis.fetch = held.fetch as unknown as typeof fetch;
    const client = createClient('');
    const big = 'y'.repeat(40 * 1024);

    const first = client.jmap(setRequest(big), { keepalive: true });
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(1));
    const second = client.jmap(setRequest(big), { keepalive: true });
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(2));
    expect(initOf(held.fetch, 0).keepalive).toBe(true);
    // 40 KiB + 40 KiB would pass 64 KiB: the second goes as an ordinary request.
    expect(initOf(held.fetch, 1).keepalive).not.toBe(true);

    held.release();
    await first;
    const third = client.jmap(setRequest(big), { keepalive: true });
    await vi.waitFor(() => expect(held.fetch).toHaveBeenCalledTimes(3));
    expect(initOf(held.fetch, 2).keepalive).toBe(true);
    held.release();
    held.release();
    await Promise.all([second, third]);
  });
});
