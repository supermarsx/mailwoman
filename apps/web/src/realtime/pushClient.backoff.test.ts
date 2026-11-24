// Reconnect timing of the push client (t35-e4): full-jitter backoff counted
// across rungs, close 1012 / 1013, a bounded and resettable failure count, the
// rate-limited session check, the catch-up stagger flag and poll jitter. The
// random source is injected and time is fake, so every bound is exact.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createPushClient,
  type EventSourceLike,
  type PushStatus,
  type PushStatusDetail,
  type SessionCheck,
  type WebSocketLike,
} from './pushClient.ts';
import { closeKind, jittered, reconnectDelay } from './reconnectPolicy.ts';
import {
  CATCHUP_SPREAD_MS,
  RECONNECT_BASE_MS,
  RECONNECT_CAP_MS,
  RECONNECT_MAX_FAILURES,
  RESTART_SPREAD_MS,
} from '../contracts/push.ts';

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

function harness(
  random: () => number,
  extra: Partial<Parameters<typeof createPushClient>[0]> = {},
  check: () => Promise<SessionCheck> = async () => 'ok',
) {
  const ws: FakeWs[] = [];
  const es: FakeEs[] = [];
  const events: Array<{ status: PushStatus; detail?: PushStatusDetail }> = [];
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
    onStatus: (status, _transport, detail) => events.push({ status, ...(detail ? { detail } : {}) }),
    checkSession,
    random,
    ...extra,
  });
  return { ws, es, events, client, checkSession };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

/** Small deterministic generator (mulberry32) for the distribution check. */
function seeded(seed: number): () => number {
  let a = seed;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

describe('reconnectDelay', () => {
  it('drop: uniform in [0, min(cap, base * 2^(n-1))] for every failure count', () => {
    for (let n = 1; n <= 30; n++) {
      const ceiling = Math.min(RECONNECT_CAP_MS, RECONNECT_BASE_MS * 2 ** (Math.min(n, RECONNECT_MAX_FAILURES) - 1));
      expect(reconnectDelay('drop', n, () => 0)).toBe(0);
      const high = reconnectDelay('drop', n, () => 0.999999);
      expect(high).toBeLessThanOrEqual(ceiling);
      expect(high).toBeGreaterThan(ceiling - 2);
    }
    expect(reconnectDelay('drop', 1, () => 0.999999)).toBeLessThanOrEqual(1000);
    expect(reconnectDelay('drop', 3, () => 0.999999)).toBeLessThanOrEqual(4000);
    expect(reconnectDelay('drop', 99, () => 0.999999)).toBeLessThanOrEqual(60_000);
  });

  it('spreads the delays: sampled mean is about half the ceiling and no sample exceeds it', () => {
    const random = seeded(7);
    for (const n of [1, 3, 5]) {
      const ceiling = RECONNECT_BASE_MS * 2 ** (n - 1);
      const samples = Array.from({ length: 4000 }, () => reconnectDelay('drop', n, random));
      const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
      expect(Math.max(...samples)).toBeLessThanOrEqual(ceiling);
      expect(Math.min(...samples)).toBeGreaterThanOrEqual(0);
      expect(mean).toBeGreaterThan(ceiling * 0.45);
      expect(mean).toBeLessThan(ceiling * 0.55);
      // Ten equal buckets are each between 8 % and 12 % of the samples.
      const buckets = new Array<number>(10).fill(0);
      for (const v of samples) buckets[Math.min(9, Math.floor((v / ceiling) * 10))]! += 1;
      for (const count of buckets) {
        expect(count / samples.length).toBeGreaterThan(0.08);
        expect(count / samples.length).toBeLessThan(0.12);
      }
    }
  });

  it('restart (1012): the first retry is uniform in [0, restartSpread]', () => {
    expect(reconnectDelay('restart', 1, () => 0)).toBe(0);
    expect(reconnectDelay('restart', 1, () => 0.999999)).toBeLessThanOrEqual(RESTART_SPREAD_MS);
    expect(reconnectDelay('restart', 1, () => 0.999999)).toBeGreaterThan(RESTART_SPREAD_MS - 2);
    // Later failures never shorten the window below the ordinary backoff ceiling.
    expect(reconnectDelay('restart', 6, () => 0.999999)).toBeGreaterThan(RESTART_SPREAD_MS);
  });

  it('busy (1013): at least the restart spread, at most twice it, never above the cap', () => {
    expect(reconnectDelay('busy', 1, () => 0)).toBe(RESTART_SPREAD_MS);
    expect(reconnectDelay('busy', 1, () => 0.999999)).toBeLessThanOrEqual(2 * RESTART_SPREAD_MS);
    expect(reconnectDelay('busy', 1, () => 0.999999)).toBeGreaterThan(2 * RESTART_SPREAD_MS - 2);
    expect(reconnectDelay('busy', 20, () => 0.999999)).toBeLessThanOrEqual(RECONNECT_CAP_MS);
    expect(reconnectDelay('busy', 20, () => 0)).toBe(RESTART_SPREAD_MS);
  });

  it('honours configured base, cap and spread', () => {
    expect(reconnectDelay('drop', 4, () => 0.999999, { baseMs: 10, capMs: 50 })).toBeLessThanOrEqual(50);
    expect(reconnectDelay('restart', 1, () => 0.999999, { restartSpreadMs: 100 })).toBeLessThanOrEqual(1000);
  });

  it('classifies close codes', () => {
    expect(closeKind(1008)).toBe('ended');
    expect(closeKind(1012)).toBe('restart');
    expect(closeKind(1013)).toBe('busy');
    expect(closeKind(1006)).toBe('drop');
    expect(closeKind(undefined)).toBe('drop');
  });

  it('jittered stays within the fraction on both sides', () => {
    expect(jittered(30_000, 0.2, () => 0)).toBe(24_000);
    expect(jittered(30_000, 0.2, () => 0.999999)).toBeLessThanOrEqual(36_000);
    expect(jittered(30_000, 0.2, () => 0.5)).toBe(30_000);
  });
});

describe('push client — backoff', () => {
  it('counts failures across rungs, so falling to SSE does not reset the window', () => {
    const h = harness(() => 0.999999);
    h.client.connect();
    h.ws[0]!.closeWith(1006); // failure 1: ceiling 1 s
    vi.advanceTimersByTime(997);
    expect(h.ws).toHaveLength(1);
    vi.advanceTimersByTime(3);
    expect(h.ws).toHaveLength(2);
    h.ws[1]!.closeWith(1006); // failure 2: ws rung used up, next rung after a 2 s ceiling
    vi.advanceTimersByTime(1997);
    expect(h.es).toHaveLength(0);
    vi.advanceTimersByTime(3);
    expect(h.es).toHaveLength(1);
    h.es[0]!.fail(); // failure 3: ceiling 4 s on the same rung
    vi.advanceTimersByTime(3997);
    expect(h.es).toHaveLength(1);
    vi.advanceTimersByTime(3);
    expect(h.es).toHaveLength(2);
    h.client.close();
  });

  it('with the lowest random value every retry is immediate', () => {
    const h = harness(() => 0);
    h.client.connect();
    h.ws[0]!.closeWith(1006);
    vi.advanceTimersByTime(0);
    expect(h.ws).toHaveLength(2);
    h.client.close();
  });

  it('close 1012: the first retry lands in [0, spread] and not earlier than the draw', () => {
    const h = harness(() => 0.999999, { restartSpreadMs: 10_000 });
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1012);
    vi.advanceTimersByTime(9_990);
    expect(h.ws).toHaveLength(1);
    vi.advanceTimersByTime(10);
    expect(h.ws).toHaveLength(2);
    expect(h.events.at(-1)?.status).toBe('reconnecting');
    h.client.close();
  });

  it('close 1012 with a low draw reconnects at once', () => {
    const h = harness(() => 0.01, { restartSpreadMs: 10_000 });
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1012);
    vi.advanceTimersByTime(100);
    expect(h.ws).toHaveLength(2);
    h.client.close();
  });

  it('close 1013: waits at least the spread, at most twice it', () => {
    let draw = 0;
    const h = harness(() => draw, { restartSpreadMs: 10_000 });
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1013);
    vi.advanceTimersByTime(9_999);
    expect(h.ws).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(h.ws).toHaveLength(2);
    // A refused handshake after 1013 still waits like 1013, at the top of the window.
    draw = 0.999999;
    h.ws[1]!.closeWith(1006);
    vi.advanceTimersByTime(19_990);
    expect(h.es).toHaveLength(0);
    vi.advanceTimersByTime(10);
    expect(h.es).toHaveLength(1);
    h.client.close();
  });

  it('close 1008 still ends as unauthorized and opens nothing more', () => {
    const h = harness(() => 0);
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1008);
    expect(h.events.at(-1)?.status).toBe('unauthorized');
    vi.advanceTimersByTime(600_000);
    expect(h.ws).toHaveLength(1);
    expect(h.es).toHaveLength(0);
  });

  it('keeps every delay at or below the cap however long the outage lasts', () => {
    const h = harness(() => 0.999999, { maxAttemptsPerRung: 1_000, maxAutomaticUpgrades: 0 });
    h.client.connect();
    let last = 0;
    for (let i = 0; i < 60; i++) {
      const before = h.ws.length;
      h.ws.at(-1)!.closeWith(1006);
      let waited = 0;
      while (h.ws.length === before && waited <= RECONNECT_CAP_MS + 1) {
        vi.advanceTimersByTime(1);
        waited += 1;
      }
      expect(h.ws.length).toBe(before + 1);
      last = waited;
    }
    expect(last).toBeLessThanOrEqual(RECONNECT_CAP_MS);
    expect(last).toBeGreaterThan(RECONNECT_CAP_MS - 5);
    h.client.close();
  });

  it('a stable connection resets the count; a short one does not', () => {
    const h = harness(() => 0.999999, { maxAttemptsPerRung: 1_000, stableAfterMs: 30_000 });
    h.client.connect();
    h.ws[0]!.closeWith(1006);
    vi.advanceTimersByTime(1_000);
    h.ws[1]!.closeWith(1006);
    vi.advanceTimersByTime(2_000);
    h.ws[2]!.open();
    vi.advanceTimersByTime(5_000); // open for less than the stable window
    h.ws[2]!.closeWith(1006); // failure 3: ceiling 4 s
    vi.advanceTimersByTime(3_997);
    expect(h.ws).toHaveLength(3);
    vi.advanceTimersByTime(3);
    expect(h.ws).toHaveLength(4);
    h.ws[3]!.open();
    vi.advanceTimersByTime(31_000); // stable
    h.ws[3]!.closeWith(1006); // counts as the first failure again: ceiling 1 s
    vi.advanceTimersByTime(1_000);
    expect(h.ws).toHaveLength(5);
    h.client.close();
  });

  it('an explicit reconnect starts again from the first rung with a fresh count', () => {
    const h = harness(() => 0.999999, { maxAttemptsPerRung: 1_000 });
    h.client.connect();
    for (let i = 0; i < 4; i++) {
      h.ws.at(-1)!.closeWith(1006);
      vi.advanceTimersByTime(60_000);
    }
    const before = h.ws.length;
    h.client.reconnect();
    expect(h.ws).toHaveLength(before + 1);
    h.ws.at(-1)!.closeWith(1006);
    vi.advanceTimersByTime(1_000);
    expect(h.ws).toHaveLength(before + 2);
    h.client.close();
  });

  it('no timers remain after close()', () => {
    const h = harness(() => 0.5);
    h.client.connect();
    h.ws[0]!.closeWith(1012);
    h.client.close();
    expect(vi.getTimerCount()).toBe(0);
  });
});

describe('push client — session check', () => {
  it('does not run on the first failure and runs once the second follows', async () => {
    const h = harness(() => 0);
    h.client.connect();
    h.ws[0]!.open();
    h.ws[0]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(0);
    expect(h.checkSession).not.toHaveBeenCalled();
    h.ws[1]!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(0);
    expect(h.checkSession).toHaveBeenCalledTimes(1);
    h.client.close();
  });

  it('is single-flight while an answer is pending', async () => {
    let answer: (v: SessionCheck) => void = () => undefined;
    const h = harness(() => 0, { maxAttemptsPerRung: 1_000 }, () => new Promise<SessionCheck>((r) => (answer = r)));
    h.client.connect();
    for (let i = 0; i < 5; i++) {
      h.ws.at(-1)!.closeWith(1006);
      await vi.advanceTimersByTimeAsync(0);
    }
    expect(h.checkSession).toHaveBeenCalledTimes(1);
    answer('ok');
    await vi.advanceTimersByTimeAsync(0);
    h.client.close();
  });

  it('runs at most once per 30 s', async () => {
    const h = harness(() => 0, { maxAttemptsPerRung: 1_000 });
    h.client.connect();
    for (let i = 0; i < 6; i++) {
      h.ws.at(-1)!.closeWith(1006);
      await vi.advanceTimersByTimeAsync(1_000);
    }
    expect(h.checkSession).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(30_000);
    h.ws.at(-1)!.closeWith(1006);
    await vi.advanceTimersByTimeAsync(0);
    expect(h.checkSession).toHaveBeenCalledTimes(2);
    h.client.close();
  });

  it('a thousand clients losing the stream at once send no session check on the first drop', async () => {
    const clients = Array.from({ length: 1_000 }, () => harness(() => 0.5));
    for (const c of clients) {
      c.client.connect();
      c.ws[0]!.open();
      c.ws[0]!.closeWith(1012);
    }
    await vi.advanceTimersByTimeAsync(0);
    expect(clients.reduce((n, c) => n + c.checkSession.mock.calls.length, 0)).toBe(0);
    for (const c of clients) c.client.close();
  });
});

describe('push client — catch-up stagger flag', () => {
  it('is false after one ordinary drop, true after 1012 or two failures, and clears afterwards', () => {
    const h = harness(() => 0, { maxAttemptsPerRung: 1_000 });
    h.client.connect();
    h.ws[0]!.open();
    expect(h.events.at(-1)).toEqual({ status: 'open', detail: { spreadCatchUp: false } });
    h.ws[0]!.closeWith(1006);
    vi.advanceTimersByTime(10);
    h.ws[1]!.open();
    expect(h.events.at(-1)).toEqual({ status: 'open', detail: { spreadCatchUp: false } });
    h.ws[1]!.closeWith(1012);
    vi.advanceTimersByTime(10);
    h.ws[2]!.open();
    expect(h.events.at(-1)).toEqual({ status: 'open', detail: { spreadCatchUp: true } });
    h.ws[2]!.closeWith(1006);
    vi.advanceTimersByTime(10);
    h.ws[3]!.closeWith(1006);
    vi.advanceTimersByTime(10);
    h.ws[4]!.open();
    expect(h.events.at(-1)).toEqual({ status: 'open', detail: { spreadCatchUp: true } });
    h.client.close();
  });
});

describe('push client — poll rung jitter', () => {
  it('re-attempts the ladder 30 s +/- 20 % after entering poll', () => {
    for (const [draw, wait] of [
      [0, 24_000],
      [0.5, 30_000],
      [0.999999, 36_000],
    ] as const) {
      const attempts = { ws: 0 };
      const client = createPushClient({
        WebSocketImpl: class {
          constructor() {
            attempts.ws += 1;
            throw new Error('blocked');
          }
        } as unknown as new (url: string) => WebSocketLike,
        EventSourceImpl: class {
          constructor() {
            throw new Error('blocked');
          }
        } as unknown as new (url: string) => EventSourceLike,
        maxAttemptsPerRung: 1,
        rungDelayMs: 0,
        maxAutomaticUpgrades: 1,
        checkSession: async () => 'ok',
        random: () => draw,
      });
      client.connect();
      vi.advanceTimersByTime(10);
      expect(client.transport()).toBe('poll');
      const before = attempts.ws;
      vi.advanceTimersByTime(wait - 12); // 10 ms already elapsed
      expect(attempts.ws).toBe(before);
      vi.advanceTimersByTime(3);
      expect(attempts.ws).toBeGreaterThan(before);
      client.close();
    }
  });
});

describe('storm shape', () => {
  it('2,000 clients dropped with 1012 reconnect spread across the window, none in the first draw of the window beyond its share', () => {
    const random = seeded(42);
    const clients = Array.from({ length: 2_000 }, () => harness(random));
    for (const c of clients) {
      c.client.connect();
      c.ws[0]!.open();
      c.ws[0]!.closeWith(1012);
    }
    const perSecond = new Array<number>(16).fill(0);
    let seen = 0;
    for (let s = 0; s < 16; s++) {
      vi.advanceTimersByTime(1_000);
      const total = clients.filter((c) => c.ws.length > 1).length;
      perSecond[s] = total - seen;
      seen = total;
    }
    expect(seen).toBe(2_000);
    // 15 s window: about 133 per second; the busiest second stays under 200.
    expect(Math.max(...perSecond.slice(0, 15))).toBeLessThan(200);
    for (const c of clients) c.client.close();
  });
});

// Keep the constant referenced so a change of the catch-up window is a test edit.
describe('constants', () => {
  it('catch-up window is two seconds', () => {
    expect(CATCHUP_SPREAD_MS).toBe(2_000);
  });
});
