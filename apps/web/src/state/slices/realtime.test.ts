import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRealtimeSlice, wireServiceWorkerWake, type RealtimeSlice } from './realtime.ts';
import { createClient, type Client } from '../../api/client.ts';
import type { WebSocketLike } from '../../realtime/pushClient.ts';

function wakeMessage(type: string): MessageEvent {
  return new MessageEvent('message', { data: { type } });
}

describe('wireServiceWorkerWake', () => {
  it('reconnects on a mw-push-wake message', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    wireServiceWorkerWake({ reconnect }, target);

    target.dispatchEvent(wakeMessage('mw-push-wake'));
    expect(reconnect).toHaveBeenCalledTimes(1);
  });

  it('ignores unrelated service-worker messages', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    wireServiceWorkerWake({ reconnect }, target);

    target.dispatchEvent(wakeMessage('other'));
    target.dispatchEvent(new MessageEvent('message'));
    expect(reconnect).not.toHaveBeenCalled();
  });

  it('is inert (no throw) when no service worker is available', () => {
    const reconnect = vi.fn();
    const cleanup = wireServiceWorkerWake({ reconnect }, undefined);
    expect(reconnect).not.toHaveBeenCalled();
    expect(() => cleanup()).not.toThrow();
  });

  it('cleanup removes the listener', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    const cleanup = wireServiceWorkerWake({ reconnect }, target);
    cleanup();

    target.dispatchEvent(wakeMessage('mw-push-wake'));
    expect(reconnect).not.toHaveBeenCalled();
  });
});

// The slice against the real API client and the real push client. Only `fetch`
// and the socket are fake, so what is under test is the wiring between a
// request's outcome and the connection state the banner renders.
class FakeWs implements WebSocketLike {
  static last: FakeWs | undefined;
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {
    FakeWs.last = this;
  }
  send(): void {}
  close(): void {}
}

const REQ = { using: [], methodCalls: [] };

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

const ok = (): Response => json(200, { methodResponses: [], sessionState: 's' });
const networkDown = (): Response => {
  throw new TypeError('Failed to fetch');
};
/** What the next `fetch` does: answer, or fail the way a dead network does. */
let next: () => Response = ok;

function signedInSlice(): { slice: RealtimeSlice; client: Client } {
  vi.stubGlobal('fetch', vi.fn(async () => next()));
  const client = createClient('');
  const slice = createRealtimeSlice(
    { client, showToast: () => undefined },
    // A long backoff: a dropped socket stays dropped for the length of a test.
    { push: { WebSocketImpl: FakeWs, backoff: [60_000] } },
  );
  slice.startRealtime();
  FakeWs.last?.onopen?.({});
  expect(slice.connectionState()).toBe('online');
  return { slice, client };
}

describe('realtime slice — request outcomes drive the connection state', () => {
  let live: RealtimeSlice | undefined;
  afterEach(() => {
    live?.stopRealtime();
    live = undefined;
    FakeWs.last = undefined;
    next = ok;
    vi.unstubAllGlobals();
  });

  it('a request that gets through clears "offline" when the socket stayed open', async () => {
    const { slice, client } = signedInSlice();
    live = slice;

    next = networkDown;
    await expect(client.jmap(REQ)).rejects.toThrow();
    expect(slice.connectionState()).toBe('offline');

    next = ok;
    await client.jmap(REQ);
    expect(slice.connectionState()).toBe('online');
    expect(slice.pushTransport()).toBe('ws');
  });

  it('a request that gets through does not hide a socket that really dropped', async () => {
    const { slice, client } = signedInSlice();
    live = slice;

    next = networkDown;
    await expect(client.jmap(REQ)).rejects.toThrow();
    FakeWs.last?.onclose?.({});
    expect(slice.connectionState()).toBe('connecting');

    next = ok;
    await client.jmap(REQ);
    expect(slice.connectionState()).toBe('connecting');
  });

  it('a 401 on an authenticated request marks the session ended', async () => {
    const { slice, client } = signedInSlice();
    live = slice;

    // Control: an answered request leaves the session alone.
    await client.jmap(REQ);
    expect(slice.connectionState()).toBe('online');

    next = () => json(401, { error: 'unauthorized' });
    await expect(client.jmap(REQ)).rejects.toMatchObject({ status: 401 });
    expect(slice.connectionState()).toBe('auth-expired');

    // Signing out ends it; the signed-out page shows nothing.
    slice.stopRealtime();
    expect(slice.connectionState()).toBe('idle');
  });

  it('a refused sign-in and the forced-password-change 403 are not an ended session', async () => {
    const { slice, client } = signedInSlice();
    live = slice;

    next = () => json(401, { error: 'invalid credentials' });
    await expect(
      client.login({ jmapUrl: 'https://mail.example.org', username: 'u', password: 'wrong' }),
    ).rejects.toMatchObject({ status: 401 });
    expect(slice.connectionState()).toBe('online');

    next = () => json(403, { error: 'password change required', passwordChangeRequired: true });
    await expect(client.jmap(REQ)).rejects.toMatchObject({ status: 403 });
    expect(slice.connectionState()).toBe('online');
  });

  it('a 401 with no session in use is the signed-out answer, not an expiry', async () => {
    const { slice, client } = signedInSlice();
    live = slice;
    slice.stopRealtime();

    next = () => json(401, { error: 'unauthorized' });
    await expect(client.me()).rejects.toMatchObject({ status: 401 });
    await expect(client.jmap(REQ)).rejects.toMatchObject({ status: 401 });
    expect(slice.connectionState()).toBe('idle');
  });
});
