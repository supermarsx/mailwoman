import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  createPushClient,
  type EventSourceCtor,
  type EventSourceLike,
  type PushStatus,
  type WebSocketCtor,
  type WebSocketLike,
} from './pushClient.ts';
import type { PushTransport, StateChange } from '../contracts/push.ts';

class FakeWs implements WebSocketLike {
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  sent: string[] = [];
  closed = false;
  constructor(public url: string) {}
  send(data: string): void {
    this.sent.push(data);
  }
  close(): void {
    this.closed = true;
  }
  open(): void {
    this.onopen?.({});
  }
  message(data: unknown): void {
    this.onmessage?.({ data });
  }
  drop(): void {
    this.onclose?.({});
  }
}

class FakeEs implements EventSourceLike {
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  closed = false;
  constructor(
    public url: string,
    public init?: { withCredentials?: boolean },
  ) {}
  close(): void {
    this.closed = true;
  }
  open(): void {
    this.onopen?.({});
  }
  message(data: unknown): void {
    this.onmessage?.({ data });
  }
  fail(): void {
    this.onerror?.({});
  }
}

function stateChange(): string {
  return JSON.stringify({ '@type': 'StateChange', changed: { a1: { Email: 'e1' } } });
}

/** Records status transitions with their transport for assertions. */
function recorder() {
  const calls: Array<[PushStatus, PushTransport]> = [];
  return {
    calls,
    onStatus: (s: PushStatus, t: PushTransport) => calls.push([s, t]),
    statuses: () => calls.map((c) => c[0]),
    last: () => calls[calls.length - 1],
  };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('createPushClient — WebSocket happy path', () => {
  it('opens a WS, reports open on ws transport, and decodes StateChange', () => {
    const ws: FakeWs[] = [];
    const rec = recorder();
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
      onStatus: rec.onStatus,
      heartbeatMs: 1000,
    });
    const seen: StateChange[] = [];
    client.onStateChange((c) => seen.push(c));

    client.connect();
    expect(ws).toHaveLength(1);
    expect(rec.statuses()).toContain('connecting');

    ws[0]!.open();
    expect(client.transport()).toBe('ws');
    expect(rec.last()).toEqual(['open', 'ws']);

    ws[0]!.message(stateChange());
    expect(seen).toHaveLength(1);
    expect(seen[0]!.changed['a1']!.Email).toBe('e1');

    client.close();
  });

  it('sends a heartbeat ping on the WS each interval while open', () => {
    const ws: FakeWs[] = [];
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
      heartbeatMs: 1000,
    });
    client.connect();
    ws[0]!.open();
    expect(ws[0]!.sent).toHaveLength(0);
    vi.advanceTimersByTime(1000);
    expect(ws[0]!.sent).toHaveLength(1);
    expect(JSON.parse(ws[0]!.sent[0]!)['@type']).toBe('ping');
    client.close();
  });

  it('treats a silent socket (two missed intervals) as dead and reconnects', () => {
    const ws: FakeWs[] = [];
    const rec = recorder();
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
      onStatus: rec.onStatus,
      heartbeatMs: 1000,
      backoff: [10],
    });
    client.connect();
    ws[0]!.open();
    // No inbound activity for > 2 intervals → drop + reconnect.
    vi.advanceTimersByTime(3000);
    expect(rec.statuses()).toContain('reconnecting');
    client.close();
  });
});

describe('createPushClient — transport ladder', () => {
  it('falls WS → SSE after exhausting the WS rung, with backoff between attempts', () => {
    const ws: FakeWs[] = [];
    const es: FakeEs[] = [];
    const rec = recorder();
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
      EventSourceImpl: class extends FakeEs {
        constructor(url: string, init?: { withCredentials?: boolean }) {
          super(url, init);
          es.push(this);
        }
      },
      onStatus: rec.onStatus,
      maxAttemptsPerRung: 2,
      backoff: [10, 10],
      rungDelayMs: 5,
    });

    client.connect();
    expect(ws).toHaveLength(1);
    ws[0]!.drop(); // attempt 1 fails → backoff reconnect on same rung
    vi.advanceTimersByTime(10);
    expect(ws).toHaveLength(2);
    ws[1]!.drop(); // rung exhausted → drop to SSE
    vi.advanceTimersByTime(5);
    expect(es).toHaveLength(1);
    es[0]!.open();
    expect(client.transport()).toBe('sse');
    expect(es[0]!.init?.withCredentials).toBe(true);
    client.close();
  });

  it('degrades to poll when both WS and SSE are unavailable, then re-attempts the ladder', () => {
    // Constructors that throw on construction stand in for a runtime where the
    // transport is blocked (jsdom's own WebSocket is present but unusable), so
    // both rungs fail fast and the client drops to passive polling.
    const FailWs = class {
      constructor() {
        throw new Error('ws blocked');
      }
    } as unknown as WebSocketCtor;
    const FailEs = class {
      constructor() {
        throw new Error('sse blocked');
      }
    } as unknown as EventSourceCtor;
    const rec = recorder();
    const client = createPushClient({
      WebSocketImpl: FailWs,
      EventSourceImpl: FailEs,
      onStatus: rec.onStatus,
      maxAttemptsPerRung: 1,
      rungDelayMs: 5,
      upgradeIntervalMs: 100,
    });
    client.connect();
    vi.advanceTimersByTime(5); // ws rung → sse rung
    vi.advanceTimersByTime(5); // sse rung → poll
    expect(client.transport()).toBe('poll');
    expect(rec.last()).toEqual(['degraded', 'poll']);
    // While polling it periodically re-attempts the whole ladder to recover.
    vi.advanceTimersByTime(100);
    expect(rec.statuses().filter((s) => s === 'connecting' || s === 'reconnecting').length).toBeGreaterThan(0);
    client.close();
  });
});

describe('createPushClient — lifecycle', () => {
  it('close reports closed on the offline transport and tears the socket down', () => {
    const ws: FakeWs[] = [];
    const rec = recorder();
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
      onStatus: rec.onStatus,
    });
    client.connect();
    ws[0]!.open();
    client.close();
    expect(client.transport()).toBe('offline');
    expect(rec.last()).toEqual(['closed', 'offline']);
    expect(ws[0]!.closed).toBe(true);
  });

  it('reconnect restarts from the top of the ladder', () => {
    const ws: FakeWs[] = [];
    const client = createPushClient({
      WebSocketImpl: class extends FakeWs {
        constructor(url: string) {
          super(url);
          ws.push(this);
        }
      },
    });
    client.connect();
    ws[0]!.open();
    client.reconnect();
    // A fresh WS was created from the top of the ladder.
    expect(ws.length).toBeGreaterThanOrEqual(2);
    client.close();
  });
});
