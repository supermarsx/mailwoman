// A session the server ended stops the push client (t32-h13 follow-up, f25):
// WS close 1008 stops it at once, and a drop whose session check answers 401
// stops it too. Before this the client reconnected every few seconds and every
// attempt was refused with 401 until the app's next request.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  createPushClient,
  type EventSourceLike,
  type PushStatus,
  type SessionCheck,
  type WebSocketLike,
} from './pushClient.ts';
import { createConnection } from './connection.ts';

class FakeWs implements WebSocketLike {
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {}
  send(): void {}
  close(): void {}
  open(): void {
    this.onopen?.({});
  }
  closeWith(code: number): void {
    this.onclose?.({ code });
  }
}

class FakeEs implements EventSourceLike {
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {}
  close(): void {}
  open(): void {
    this.onopen?.({});
  }
  fail(): void {
    this.onerror?.({});
  }
}

function harness(check: () => Promise<SessionCheck>) {
  const ws: FakeWs[] = [];
  const es: FakeEs[] = [];
  const statuses: PushStatus[] = [];
  const checkSession = vi.fn(check);
  const client = createPushClient({
    WebSocketImpl: class extends FakeWs {
      constructor(url: string) {
        super(url);
        ws.push(this);
      }
    },
    EventSourceImpl: class extends FakeEs {
      constructor(url: string) {
        super(url);
        es.push(this);
      }
    },
    onStatus: (s) => statuses.push(s),
    checkSession,
    backoff: [500, 1000, 2000, 4000, 8000],
    upgradeIntervalMs: 30_000,
  });
  return { ws, es, statuses, client, checkSession };
}

/** Attempts made over a minute of fake time, with every attempt refused. */
async function attemptsOverAMinute(h: ReturnType<typeof harness>): Promise<number> {
  for (let i = 0; i < 120; i++) {
    h.ws.at(-1)?.closeWith(1006);
    h.es.at(-1)?.fail();
    await vi.advanceTimersByTimeAsync(500);
  }
  return h.ws.length + h.es.length;
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('push client — session ended by the server', () => {
  it('stops on WS close 1008 and opens nothing more', async () => {
    const h = harness(async () => 'ok');
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1008);
    expect(h.statuses.at(-1)).toBe('unauthorized');
    expect(h.client.transport()).toBe('offline');
    await vi.advanceTimersByTimeAsync(120_000);
    expect(h.ws).toHaveLength(1);
    expect(h.es).toHaveLength(0);
    expect(h.statuses.at(-1)).toBe('unauthorized');
  });

  it('stops after a drop when the session check answers unauthorized', async () => {
    const h = harness(async () => 'unauthorized');
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(0);
    expect(h.checkSession).toHaveBeenCalledTimes(1);
    expect(h.statuses.at(-1)).toBe('unauthorized');
    expect(await attemptsOverAMinute(h)).toBe(1);
  });

  it('stops when the first connection attempt is refused and the session is gone', async () => {
    const h = harness(async () => 'unauthorized');
    h.client.connect();
    h.ws[0]!.closeWith(1006); // handshake refused (the browser hides the 401)
    await vi.advanceTimersByTimeAsync(0);
    expect(h.statuses.at(-1)).toBe('unauthorized');
    expect(await attemptsOverAMinute(h)).toBe(1);
  });

  it('keeps reconnecting after a plain drop while the session is valid', async () => {
    const h = harness(async () => 'ok');
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(500);
    expect(h.ws).toHaveLength(2);
    expect(h.statuses).not.toContain('unauthorized');
  });

  it('keeps reconnecting when the session check gets no answer', async () => {
    const h = harness(async () => 'unknown');
    h.client.connect();
    h.ws[0]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(500);
    expect(h.ws).toHaveLength(2);
    expect(h.statuses).not.toContain('unauthorized');
  });

  it('ignores a late unauthorized answer once a newer connection has opened', async () => {
    let answer: (v: SessionCheck) => void = () => undefined;
    const h = harness(() => new Promise<SessionCheck>((r) => (answer = r)));
    h.client.connect();
    h.ws[0]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(500);
    h.ws[1]!.open();
    answer('unauthorized');
    await vi.advanceTimersByTimeAsync(0);
    expect(h.statuses.at(-1)).toBe('open');
    expect(h.client.transport()).toBe('ws');
  });

  it('connects again after the next sign-in', async () => {
    const h = harness(async () => 'ok');
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1008);
    h.client.connect();
    expect(h.ws).toHaveLength(2);
    h.ws[1]!.open();
    expect(h.statuses.at(-1)).toBe('open');
  });

  it('the default check sends the bearer to the JMAP session resource', async () => {
    const fetchMock = vi.fn(async () => new Response('', { status: 401 }));
    vi.stubGlobal('fetch', fetchMock);
    try {
      const ws: FakeWs[] = [];
      const statuses: PushStatus[] = [];
      const client = createPushClient({
        bearer: 'tok-1',
        WebSocketImpl: class extends FakeWs {
          constructor(url: string) {
            super(url);
            ws.push(this);
          }
        },
        onStatus: (s) => statuses.push(s),
      });
      client.connect();
      ws[0]!.closeWith(1006);
      await vi.advanceTimersByTimeAsync(0);
      expect(fetchMock).toHaveBeenCalledTimes(1);
      const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
      expect(url).toMatch(/\/jmap\/session$/);
      expect((init.headers as Record<string, string>)['Authorization']).toBe('Bearer tok-1');
      expect(statuses.at(-1)).toBe('unauthorized');
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

describe('connection model — unauthorized push status', () => {
  it('shows auth-expired, and a sign-out (closed) clears it', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.report('unauthorized', 'offline');
    expect(c.state()).toBe('auth-expired');
    c.report('closed', 'offline');
    expect(c.state()).toBe('idle');
  });
});
