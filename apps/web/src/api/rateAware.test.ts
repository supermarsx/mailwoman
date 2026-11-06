import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  MAX_BACKGROUND, MAX_RETRIES, backoffActive, isReadOnlyJmap, onRateEvent, parseRetryAfter, rateAwareSend,
  rateRuntime, rateTotals, resetRateState, retryDelay, safeToRepeat, takeRateReport, type RateEvent,
} from './rateAware.ts';

const real = { ...rateRuntime };
let clock = 1_000_000;
let slept: number[] = [];

beforeEach(() => {
  resetRateState();
  clock = 1_000_000;
  slept = [];
  rateRuntime.now = () => clock;
  rateRuntime.random = () => 0.5;
  rateRuntime.sleep = (ms) => { slept.push(ms); clock += ms; return Promise.resolve(); };
});
afterEach(() => { Object.assign(rateRuntime, real); resetRateState(); });

const limited = (retryAfter?: string): Response =>
  new Response('busy', { status: 429, ...(retryAfter !== undefined ? { headers: { 'retry-after': retryAfter } } : {}) });
const ok = (): Response => new Response('{"ok":true}', { status: 200 });
const get = { method: 'GET', path: '/api/me' } as const;

/** A sender that answers with each response in turn (the last one repeats). */
function sender(...answers: Array<() => Response>): { send: () => Promise<Response>; calls: () => number } {
  let n = 0;
  return { send: () => Promise.resolve(answers[Math.min(n++, answers.length - 1)]!()), calls: () => n };
}

describe('retry of a safe request', () => {
  it('waits Retry-After and succeeds without any event but waiting/clear', async () => {
    const events: RateEvent[] = [];
    onRateEvent((e) => events.push(e));
    const s = sender(() => limited('2'), ok);
    const res = await rateAwareSend(get, s.send);
    expect(res.status).toBe(200);
    expect(s.calls()).toBe(2);
    expect(slept).toEqual([2000 + 125]); // Retry-After plus jitter (0.5 * 250)
    expect(events.map((e) => e.kind)).toEqual(['waiting', 'clear']);
    expect(rateTotals()).toEqual({ seen: 1, retries: 1, exhausted: 0 });
  });

  it('uses jittered exponential backoff when the server names no Retry-After', async () => {
    const s = sender(() => limited(), () => limited(), ok);
    await rateAwareSend(get, s.send);
    // random 0.5: attempt 0 cap 500 -> 375; attempt 1 cap 1000 -> 750.
    expect(slept).toEqual([375, 750]);
    expect(retryDelay(2, null)).toBe(1500);
    expect(retryDelay(0, 3000)).toBe(3125);
  });

  it('gives up after MAX_RETRIES and reports exhaustion once', async () => {
    const events: RateEvent[] = [];
    onRateEvent((e) => events.push(e));
    const s = sender(() => limited('1'));
    const res = await rateAwareSend(get, s.send);
    expect(res.status).toBe(429);
    expect(s.calls()).toBe(MAX_RETRIES + 1);
    expect(events.filter((e) => e.kind === 'exhausted')).toHaveLength(1);
    expect(rateTotals()).toEqual({ seen: MAX_RETRIES + 1, retries: MAX_RETRIES, exhausted: 1 });
  });

  it('never waits more than 10 s in all', async () => {
    const s = sender(() => limited('4'));
    await rateAwareSend(get, s.send);
    expect(slept.reduce((a, b) => a + b, 0)).toBeLessThanOrEqual(10_000);
    expect(s.calls()).toBe(3); // 4.125 s + 4.125 s, a third wait would pass the budget
  });

  it('does not start a wait the budget cannot cover', async () => {
    const s = sender(() => limited('30'));
    const res = await rateAwareSend(get, s.send);
    expect(res.status).toBe(429);
    expect(s.calls()).toBe(1);
    expect(slept).toEqual([]);
  });

  it('retries a 503 only when it names a Retry-After', async () => {
    const plain = sender(() => new Response('x', { status: 503 }));
    expect((await rateAwareSend(get, plain.send)).status).toBe(503);
    expect(plain.calls()).toBe(1);
    const advised = sender(() => new Response('x', { status: 503, headers: { 'retry-after': '1' } }), ok);
    expect((await rateAwareSend(get, advised.send)).status).toBe(200);
  });

  it('stops waiting when the request is aborted', async () => {
    const controller = new AbortController();
    controller.abort();
    const s = sender(() => limited('1'), ok);
    const res = await rateAwareSend({ ...get, signal: controller.signal }, s.send);
    expect(res.status).toBe(429);
    expect(s.calls()).toBe(1);
  });
});

describe('what is safe to send again', () => {
  const read = JSON.stringify({ using: [], methodCalls: [['Email/get', {}, 'a'], ['Email/query', {}, 'b']] });
  const write = JSON.stringify({ using: [], methodCalls: [['Email/get', {}, 'a'], ['Email/set', {}, 'b']] });

  it('classifies requests', () => {
    expect(safeToRepeat('GET', '/api/x', null)).toBe(true);
    expect(safeToRepeat('POST', '/api/sanitize', '{}')).toBe(true);
    expect(safeToRepeat('POST', '/jmap/api', read)).toBe(true);
    expect(safeToRepeat('POST', '/jmap/api', write)).toBe(false);
    expect(safeToRepeat('POST', '/api/login', '{}')).toBe(false);
    expect(safeToRepeat('PUT', '/api/x', '{}')).toBe(false);
    expect(isReadOnlyJmap('not json')).toBe(false);
    expect(isReadOnlyJmap(JSON.stringify({ methodCalls: [] }))).toBe(false);
  });

  it('retries a read-only JMAP batch and never a write', async () => {
    const r = sender(() => limited('1'), ok);
    expect((await rateAwareSend({ method: 'POST', path: '/jmap/api', body: read }, r.send)).status).toBe(200);
    const w = sender(() => limited('1'), ok);
    expect((await rateAwareSend({ method: 'POST', path: '/jmap/api', body: write }, w.send)).status).toBe(429);
    expect(w.calls()).toBe(1);
  });

  it('leaves a JMAP limit refusal to the client, which repeats it for any batch', async () => {
    const problem = (): Response => new Response(JSON.stringify({ type: 'urn:ietf:params:jmap:error:limit', limit: 'maxConcurrentRequests' }), { status: 429 });
    const s = sender(problem, ok);
    const res = await rateAwareSend({ method: 'POST', path: '/jmap/api', body: read }, s.send);
    expect(res.status).toBe(429);
    expect(s.calls()).toBe(1);
  });
});

describe('Retry-After parsing', () => {
  it('reads seconds and dates, and ignores junk', () => {
    expect(parseRetryAfter('3')).toBe(3000);
    expect(parseRetryAfter('99999')).toBe(3_600_000);
    expect(parseRetryAfter(new Date(clock + 5000).toUTCString(), clock)).toBeGreaterThan(3000);
    expect(parseRetryAfter('soon')).toBeNull();
    expect(parseRetryAfter(null)).toBeNull();
  });
});

describe('background work', () => {
  it('runs at most MAX_BACKGROUND at once and never holds back the foreground', async () => {
    let active = 0;
    let peak = 0;
    const releases: Array<() => void> = [];
    const slow = (): Promise<Response> => {
      active += 1; peak = Math.max(peak, active);
      return new Promise((resolve) => releases.push(() => { active -= 1; resolve(ok()); }));
    };
    const jobs = Array.from({ length: MAX_BACKGROUND + 3 }, () => rateAwareSend({ ...get, pace: 'background' }, slow));
    await Promise.resolve(); await Promise.resolve();
    expect(active).toBe(MAX_BACKGROUND);
    const foreground = rateAwareSend({ ...get, pace: 'foreground' }, () => Promise.resolve(ok()));
    expect((await foreground).status).toBe(200);
    while (releases.length > 0) { releases.shift()!(); await Promise.resolve(); await Promise.resolve(); }
    await Promise.all(jobs);
    expect(peak).toBe(MAX_BACKGROUND);
  });

  it('pauses while a Retry-After window is open, and the foreground does not', async () => {
    const fresh = sender(() => limited('5'));
    await rateAwareSend(get, fresh.send);
    expect(backoffActive()).toBe(true);
    slept = [];
    const s = sender(ok);
    const res = await rateAwareSend({ ...get, pace: 'background' }, s.send);
    expect(res.status).toBe(200);
    expect(slept.length).toBeGreaterThan(0); // it waited for the window instead of sending
    const before = slept.length;
    expect(backoffActive()).toBe(false);
    await rateAwareSend({ ...get, pace: 'background' }, s.send);
    expect(slept.length).toBe(before);
    // and a foreground request is sent while a window is open
    await rateAwareSend(get, sender(() => limited('5')).send);
    expect(backoffActive()).toBe(true);
    slept = [];
    expect((await rateAwareSend({ ...get, pace: 'foreground' }, sender(ok).send)).status).toBe(200);
    expect(slept).toEqual([]);
  });
});

describe('identical GETs in flight', () => {
  it('are sent once and every caller can read the answer', async () => {
    let calls = 0;
    let release!: () => void;
    const gate = new Promise<void>((r) => { release = r; });
    const send = async (): Promise<Response> => { calls += 1; await gate; return ok(); };
    const a = rateAwareSend(get, send, 'k');
    const b = rateAwareSend(get, send, 'k');
    release();
    const [ra, rb] = await Promise.all([a, b]);
    expect(calls).toBe(1);
    expect(await ra.json()).toEqual({ ok: true });
    expect(await rb.json()).toEqual({ ok: true });
    // finished: the next one is a new request
    await rateAwareSend(get, send, 'k');
    expect(calls).toBe(2);
  });

  it('are not shared when the caller can abort', async () => {
    let calls = 0;
    const send = (): Promise<Response> => { calls += 1; return Promise.resolve(ok()); };
    const signal = new AbortController().signal;
    await Promise.all([rateAwareSend({ ...get, signal }, send, 'k'), rateAwareSend({ ...get, signal }, send, 'k')]);
    expect(calls).toBe(2);
  });
});

describe('the timing report', () => {
  it('hands out counts since the last report, then nothing', async () => {
    expect(takeRateReport()).toBeNull();
    await rateAwareSend(get, sender(() => limited('1'), ok).send);
    expect(takeRateReport()).toEqual({ seen: 1, retries: 1, exhausted: 0 });
    expect(takeRateReport()).toBeNull();
  });
});
