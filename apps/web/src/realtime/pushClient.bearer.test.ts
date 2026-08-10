import { afterEach, describe, expect, it, vi } from 'vitest';
import { createPushClient, type WebSocketLike } from './pushClient.ts';

// The native shell cannot set an Authorization header on a WebSocket/EventSource,
// so a supplied bearer is appended as an `access_token` query param. A browser
// passes no bearer → the URL is unchanged (the regression-critical default).

class FakeWS implements WebSocketLike {
  static urls: string[] = [];
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor(url: string) {
    FakeWS.urls.push(url);
  }
  send(): void {}
  close(): void {}
}

describe('pushClient bearer threading', () => {
  it('appends access_token to the WS URL when a bearer is set', () => {
    FakeWS.urls = [];
    createPushClient({ wsUrl: 'ws://host/jmap/ws', bearer: 'TK N', WebSocketImpl: FakeWS }).connect();
    expect(FakeWS.urls[0]).toBe('ws://host/jmap/ws?access_token=TK%20N');
  });

  it('leaves the WS URL untouched with no bearer (browser default)', () => {
    FakeWS.urls = [];
    createPushClient({ wsUrl: 'ws://host/jmap/ws', WebSocketImpl: FakeWS }).connect();
    expect(FakeWS.urls[0]).toBe('ws://host/jmap/ws');
  });

  it('respects an existing query string on the URL', () => {
    FakeWS.urls = [];
    createPushClient({ wsUrl: 'ws://host/jmap/ws?x=1', bearer: 'A', WebSocketImpl: FakeWS }).connect();
    expect(FakeWS.urls[0]).toBe('ws://host/jmap/ws?x=1&access_token=A');
  });
});

// ── Sub-path hosting (t20 B4) ───────────────────────────────────────────────
// With no explicit `wsUrl`/`sseUrl`, both realtime endpoints are derived from the
// server-injected deploy prefix. At the origin root they are byte-identical to
// the pre-sub-path defaults; under `/mail` they carry the prefix.

class FakeES {
  static urls: string[] = [];
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  constructor(url: string) {
    FakeES.urls.push(url);
  }
  close(): void {}
}

/** A WS that fails on construction, so the ladder drops straight to SSE. */
class DeadWS implements WebSocketLike {
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor() {
    throw new Error('ws blocked');
  }
  send(): void {}
  close(): void {}
}

describe('pushClient default endpoints under a sub-path', () => {
  const g = globalThis as unknown as { __MW_BASE__?: unknown };
  afterEach(() => {
    delete g.__MW_BASE__;
  });

  function wsUrlFor(base?: string): string {
    if (base !== undefined) g.__MW_BASE__ = base;
    FakeWS.urls = [];
    createPushClient({ WebSocketImpl: FakeWS }).connect();
    return FakeWS.urls[0] as string;
  }

  it('defaults to the root WS path with no prefix (unchanged)', () => {
    expect(wsUrlFor()).toBe(`ws://${location.host}/jmap/ws`);
  });

  it('prefixes the WS path under a sub-path, keeping the absolute form', () => {
    expect(wsUrlFor('/mail')).toBe(`ws://${location.host}/mail/jmap/ws`);
  });

  it('prefixes the SSE path, which stays same-origin relative', () => {
    vi.useFakeTimers();
    try {
      g.__MW_BASE__ = '/mail';
      FakeES.urls = [];
      const client = createPushClient({
        WebSocketImpl: DeadWS as unknown as typeof FakeWS,
        EventSourceImpl: FakeES as never,
        maxAttemptsPerRung: 1,
        rungDelayMs: 5,
      });
      client.connect();
      vi.advanceTimersByTime(5); // ws rung exhausted → sse rung
      expect(FakeES.urls[0]).toBe('/mail/jmap/eventsource');
      client.close();
    } finally {
      vi.useRealTimers();
    }
  });

  it('an explicitly supplied url is never re-prefixed', () => {
    g.__MW_BASE__ = '/mail';
    FakeWS.urls = [];
    createPushClient({ wsUrl: 'wss://other/jmap/ws', WebSocketImpl: FakeWS }).connect();
    expect(FakeWS.urls[0]).toBe('wss://other/jmap/ws');
  });
});
