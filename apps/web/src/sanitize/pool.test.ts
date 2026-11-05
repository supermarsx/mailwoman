import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import {
  initSync, sanitizeEmailHtml, sanitizeEmailHtmlReader, sanitizeEmailHtmlReaderSafe,
} from '../wasm/mw-sanitize/mw_sanitize.js';
import { SanitizeCache, cacheKey } from './cache.ts';
import { SanitizerPool, QueueFullError, poolSize, type WorkerLike } from './pool.ts';
import type { FromWorker, SanitizeRequest, ToWorker } from './protocol.ts';
import { clearSanitizeCache, sanitizeBody, sanitizeCache, setSanitizeFallbacks, setSanitizePool } from './index.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../api/accountPreferenceLifecycle.ts';

let wasm: WebAssembly.Module;
beforeAll(() => {
  const bytes = readFileSync(resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'));
  initSync({ module: bytes });
  wasm = new WebAssembly.Module(bytes);
});

function direct(req: SanitizeRequest): string {
  if (req.op === 'strip') return sanitizeEmailHtml(req.html);
  const fn = req.op === 'reader' ? sanitizeEmailHtmlReader : sanitizeEmailHtmlReaderSafe;
  return fn(req.html, req.blockTrackers !== false, req.cleanLinks !== false);
}

/** A worker that answers like worker.entry.ts, with hooks to hold or crash jobs. */
class FakeWorker implements WorkerLike {
  static all: FakeWorker[] = [];
  onmessage: ((ev: MessageEvent<FromWorker>) => void) | null = null;
  onerror: ((ev: ErrorEvent) => void) | null = null;
  terminated = false;
  inits = 0;
  jobs: number[] = [];
  held: Array<() => void> = [];
  static behaviour: 'reply' | 'hold' | 'crash' | 'crashOnce' = 'reply';
  constructor() { FakeWorker.all.push(this); }
  postMessage(m: ToWorker): void {
    if (m.type === 'init') { this.inits += 1; expect(m.module).toBe(wasm); return; }
    this.jobs.push(m.id);
    const answer = (): void => {
      if (this.terminated) return;
      queueMicrotask(() => this.onmessage?.({ data: { type: 'result', id: m.id, ok: true, html: direct(m.req) } } as MessageEvent<FromWorker>));
    };
    if (FakeWorker.behaviour === 'crashOnce') {
      FakeWorker.behaviour = 'reply';
      queueMicrotask(() => this.onerror?.({ message: 'boom' } as ErrorEvent));
    } else if (FakeWorker.behaviour === 'crash') queueMicrotask(() => this.onerror?.({ message: 'boom' } as ErrorEvent));
    else if (FakeWorker.behaviour === 'hold') this.held.push(answer);
    else answer();
  }
  terminate(): void { this.terminated = true; }
}

function makePool(over: Partial<ConstructorParameters<typeof SanitizerPool>[0]> = {}): { pool: SanitizerPool; load: ReturnType<typeof vi.fn> } {
  const load = vi.fn(async () => wasm);
  const pool = new SanitizerPool({
    size: 2, createWorker: () => new FakeWorker(), loadModule: load,
    fallback: async (req) => `fallback:${req.html}`, ...over,
  });
  return { pool, load };
}
const req = (html: string, op: SanitizeRequest['op'] = 'strip'): SanitizeRequest => ({ op, html });
const tick = (): Promise<void> => new Promise((r) => setTimeout(r, 0));

afterEach(() => { FakeWorker.all = []; FakeWorker.behaviour = 'reply'; setSanitizePool(null); clearSanitizeCache(); });

describe('poolSize', () => {
  it('is clamp(hardwareConcurrency - 1, 1, 4), with an explicit override', () => {
    expect([1, 2, 4, 8, 64, undefined].map((n) => poolSize(n))).toEqual([1, 1, 3, 4, 4, 1]);
    expect(poolSize(8, 6)).toBe(6);
    expect(poolSize(8, 99)).toBe(8);
  });
});

describe('SanitizerPool', () => {
  it('compiles the module once and spawns lazily up to its size', async () => {
    FakeWorker.behaviour = 'hold';
    const { pool, load } = makePool();
    expect(FakeWorker.all).toHaveLength(0);
    const jobs = [1, 2, 3].map((n) => pool.run(req(`<p>${n}</p>`)));
    await tick();
    expect(FakeWorker.all).toHaveLength(2); // size 2, third job queued
    expect(load).toHaveBeenCalledTimes(1);
    expect(FakeWorker.all.every((w) => w.inits === 1)).toBe(true);
    expect(pool.stats()).toMatchObject({ workers: 2, busy: 2, queued: 1 });
    for (const w of FakeWorker.all) w.held.splice(0).forEach((f) => f());
    await tick();
    FakeWorker.all.forEach((w) => w.held.splice(0).forEach((f) => f()));
    expect(await Promise.all(jobs)).toEqual(['<p>1</p>', '<p>2</p>', '<p>3</p>']);
  });

  it('runs the open message before prefetch and drops the oldest prefetch to admit it when full', async () => {
    FakeWorker.behaviour = 'hold';
    const { pool } = makePool({ size: 1, maxQueue: 2 });
    const first = pool.run(req('<p>first</p>'));
    await tick();
    const low1 = pool.run(req('<p>low1</p>'), { priority: 'low' });
    const low2 = pool.run(req('<p>low2</p>'), { priority: 'low' });
    await expect(pool.run(req('<p>low3</p>'), { priority: 'low' })).rejects.toBeInstanceOf(QueueFullError);
    const high = pool.run(req('<p>open</p>'), { priority: 'high' }); // evicts low1
    await expect(low1).rejects.toBeInstanceOf(QueueFullError);
    const w = FakeWorker.all[0]!;
    const order: string[] = [];
    void first.then(() => order.push('first'));
    void high.then(() => order.push('open'));
    void low2.then(() => order.push('low2'));
    for (let i = 0; i < 3; i += 1) { await tick(); w.held.shift()?.(); }
    await Promise.all([first, high, low2]);
    expect(order).toEqual(['first', 'open', 'low2']);
  });

  it('drops a queued job on abort and ignores the result of an in-flight one', async () => {
    FakeWorker.behaviour = 'hold';
    const { pool } = makePool({ size: 1 });
    const inflight = new AbortController();
    const queued = new AbortController();
    const a = pool.run(req('<p>a</p>'), { signal: inflight.signal });
    const b = pool.run(req('<p>b</p>'), { signal: queued.signal });
    await tick();
    queued.abort();
    inflight.abort();
    await expect(b).rejects.toMatchObject({ name: 'AbortError' });
    await expect(a).rejects.toMatchObject({ name: 'AbortError' });
    expect(pool.stats().queued).toBe(0);
    FakeWorker.all[0]!.held.shift()!(); // the stale reply is discarded without effect
    await tick();
    expect(FakeWorker.all[0]!.jobs).toHaveLength(1); // the aborted queued job never ran
    await expect(pool.run(req('x'), { signal: inflight.signal })).rejects.toMatchObject({ name: 'AbortError' });
  });

  it('retries once on a fresh worker after a crash, then answers from the fallback', async () => {
    FakeWorker.behaviour = 'crash';
    const { pool } = makePool();
    expect(await pool.run(req('<p>x</p>'))).toBe('fallback:<p>x</p>');
    expect(pool.stats().crashes).toBe(2);
    expect(FakeWorker.all).toHaveLength(2);
    expect(FakeWorker.all.every((w) => w.terminated)).toBe(true);
    // The pool recovers: a healthy worker serves the next job.
    FakeWorker.behaviour = 'reply';
    expect(await pool.run(req('<p>y</p>'))).toBe('<p>y</p>');
  });

  it('succeeds when only the first attempt crashes', async () => {
    FakeWorker.behaviour = 'crashOnce';
    const { pool } = makePool({ size: 1 });
    expect(await pool.run(req('<p>z</p>'))).toBe('<p>z</p>');
    expect(pool.stats().crashes).toBe(1);
    expect(FakeWorker.all).toHaveLength(2);
  });

  it('falls back when no worker can be created or the wasm cannot be compiled', async () => {
    const { pool } = makePool({ createWorker: () => { throw new Error('no Worker'); } });
    expect(await pool.run(req('<p>a</p>'))).toBe('fallback:<p>a</p>');
    const { pool: bad } = makePool({ loadModule: async () => { throw new Error('csp'); } });
    expect(await bad.run(req('<p>b</p>'))).toBe('fallback:<p>b</p>');
  });

  it('gives the same output as the direct wasm call for every op', async () => {
    const { pool } = makePool();
    const hostile = '<p onclick="x()">Hi <a href="https://a.example/?utm_source=z">l</a><script>1</script><img src="https://t.example/p.gif" width="1" height="1"><span></span></p>';
    for (const op of ['strip', 'reader', 'readerSafe'] as const) {
      const r: SanitizeRequest = { op, html: hostile, blockTrackers: false, cleanLinks: true };
      expect(await pool.run(r)).toBe(direct(r));
    }
  });
});

describe('SanitizeCache', () => {
  it('evicts least recently used entries by bytes', () => {
    const c = new SanitizeCache(40); // 20 UTF-16 units
    c.set('a', '12345'); c.set('b', '12345'); // 20 bytes
    c.get('a');
    c.set('c', '12345'); // 30 > 40? no; add d to force
    c.set('d', '12345');
    expect(c.byteSize).toBeLessThanOrEqual(40);
    c.set('e', '12345');
    expect(c.get('b')).toBeUndefined(); // oldest untouched entry went first
    expect(c.get('e')).toBe('12345');
    c.set('huge', 'x'.repeat(100)); // bigger than half the budget: not cached
    expect(c.get('huge')).toBeUndefined();
    c.clear();
    expect(c.size).toBe(0);
    expect(c.byteSize).toBe(0);
  });

  it('keys differ by options and content but not by identical content', () => {
    expect(cacheKey('strip:11', 'a')).toBe(cacheKey('strip:11', 'a'));
    expect(cacheKey('strip:11', 'a')).not.toBe(cacheKey('reader:11', 'a'));
    expect(cacheKey('strip:11', 'a')).not.toBe(cacheKey('strip:11', 'b'));
  });
});

describe('sanitizeBody', () => {
  it('sanitises once for repeated and concurrent requests, and empties on account change', async () => {
    FakeWorker.behaviour = 'reply';
    const { pool } = makePool();
    const run = vi.spyOn(pool, 'run');
    setSanitizePool(pool);
    const r: SanitizeRequest = { op: 'reader', html: '<p>cache me</p>', blockTrackers: false, cleanLinks: true };
    const [a, b] = await Promise.all([sanitizeBody(r), sanitizeBody(r)]);
    expect(a).toBe(b);
    expect(await sanitizeBody(r)).toBe(a);
    expect(run).toHaveBeenCalledTimes(1);
    expect(sanitizeCache.size).toBe(1);
    activateAccountPreferences('acct-a');
    activateAccountPreferences('acct-b');
    expect(sanitizeCache.size).toBe(0);
    await sanitizeBody(r);
    expect(sanitizeCache.size).toBe(1);
    suspendAccountPreferences(); // sign-out
    expect(sanitizeCache.size).toBe(0);
  });

  it('uses the per-op fallback when the pool fails, and never caches an abort', async () => {
    const server = vi.fn(async (html: string) => `server:${html}`);
    setSanitizeFallbacks({ server });
    setSanitizePool(makePool({ createWorker: () => { throw new Error('none'); }, fallback: async () => { throw new Error('none'); } }).pool);
    expect(await sanitizeBody({ op: 'strip', html: '<p>s</p>' })).toBe('server:<p>s</p>');
    const ac = new AbortController();
    ac.abort();
    await expect(sanitizeBody({ op: 'strip', html: '<p>t</p>' }, { signal: ac.signal })).rejects.toMatchObject({ name: 'AbortError' });
    expect(sanitizeCache.size).toBe(1);
  });
});
