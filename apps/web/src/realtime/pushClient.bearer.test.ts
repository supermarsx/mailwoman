import { describe, expect, it } from 'vitest';
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
