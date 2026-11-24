import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fetchWithRetry, isTransientStatus, MAX_WAIT_MS, prefetchFile, resetChunkRetry, RETRY_DELAYS_MS, setChunkRetryEnv, vitePreload, type RetryEnv } from './chunkRetry.ts';

type Answer = number | 'network' | { status: number; retryAfter: string };

/** A fake network: per URL, the answers in order (the last one repeats). */
function network(plan: Record<string, Answer[]>): RetryEnv & { calls: string[]; waits: number[] } {
  const calls: string[] = [];
  const waits: number[] = [];
  const seen: Record<string, number> = {};
  return {
    calls,
    waits,
    sleep: async (ms) => { waits.push(ms); },
    fetch: async (url) => {
      calls.push(url);
      const answers = plan[url] ?? [200];
      const answer = answers[Math.min(seen[url] = (seen[url] ?? -1) + 1, answers.length - 1)]!;
      if (answer === 'network') throw new TypeError('Failed to fetch');
      const status = typeof answer === 'number' ? answer : answer.status;
      const headers: Record<string, string> = typeof answer === 'number' ? {} : { 'retry-after': answer.retryAfter };
      return new Response(status === 200 ? 'x' : null, { status, headers });
    },
  };
}

const BASE = 'https://mail.example.test/assets/';
const importer = `${BASE}core-AAAA.js`;

let realEnv: RetryEnv | undefined;
beforeEach(() => {
  resetChunkRetry();
  document.head.querySelectorAll('link').forEach((l) => l.remove());
});
afterEach(() => {
  if (realEnv !== undefined) setChunkRetryEnv(realEnv);
  realEnv = undefined;
  vi.restoreAllMocks();
});

describe('retry schedule', () => {
  it('retries 502/503/504/429 and network errors, nothing else', () => {
    expect([429, 502, 503, 504].every(isTransientStatus)).toBe(true);
    expect([200, 304, 400, 403, 404, 410, 500].some(isTransientStatus)).toBe(false);
  });

  it('reads Retry-After as delta-seconds; anything else as no hint', async () => {
    const net = network({ a: [{ status: 503, retryAfter: 'soon' }, 200] });
    expect(await fetchWithRetry('a', net)).toBe(true);
    expect(net.waits).toEqual([RETRY_DELAYS_MS[0]]);
  });

  it('a 503 then 200 takes two requests and one wait of at least the first delay', async () => {
    const net = network({ a: [503, 200] });
    expect(await fetchWithRetry('a', net)).toBe(true);
    expect(net.calls).toEqual(['a', 'a']);
    expect(net.waits).toEqual([RETRY_DELAYS_MS[0]]);
  });

  it('honours Retry-After within the bounds', async () => {
    const net = network({ a: [{ status: 503, retryAfter: '1' }, { status: 503, retryAfter: '60' }, 200] });
    expect(await fetchWithRetry('a', net)).toBe(true);
    expect(net.waits).toEqual([1000, MAX_WAIT_MS]);
  });

  it('gives up after four attempts (about 2 s of waiting without Retry-After)', async () => {
    const net = network({ a: [503] });
    expect(await fetchWithRetry('a', net)).toBe(false);
    expect(net.calls).toHaveLength(RETRY_DELAYS_MS.length + 1);
    expect(net.waits.reduce((s, w) => s + w, 0)).toBe(2050);
  });

  it('retries a network error', async () => {
    const net = network({ a: ['network', 'network', 200] });
    expect(await fetchWithRetry('a', net)).toBe(true);
    expect(net.calls).toHaveLength(3);
  });

  it('does not retry a 404 (a chunk missing after a deploy goes to the existing recovery)', async () => {
    const net = network({ a: [404, 200] });
    expect(await fetchWithRetry('a', net)).toBe(false);
    expect(net.calls).toEqual(['a']);
    expect(net.waits).toEqual([]);
  });
});

describe('vitePreload (the build preload helper)', () => {
  it('fetches every dependency until it is complete, then adds the link, then imports', async () => {
    const net = network({ [`${BASE}mail-BBBB.js`]: [503, 503, 200], [`${BASE}shared-CCCC.js`]: [200] });
    realEnv = setChunkRetryEnv(net);
    const order: string[] = [];
    const mod = await vitePreload(async () => { order.push(`import after ${net.calls.length} requests`); return { ok: 1 }; }, ['./mail-BBBB.js', './shared-CCCC.js'], importer);
    expect(mod).toEqual({ ok: 1 });
    expect(net.calls.filter((u) => u.endsWith('mail-BBBB.js'))).toHaveLength(3);
    expect(order).toEqual(['import after 4 requests']);
    const links = [...document.head.querySelectorAll('link')].map((l) => `${l.rel} ${l.getAttribute('as')} ${l.crossOrigin} ${l.href}`);
    expect(links).toEqual([`modulepreload script  ${BASE}shared-CCCC.js`, `modulepreload script  ${BASE}mail-BBBB.js`]);
  });

  it('requests a dependency once per page and skips one the document already has', async () => {
    const net = network({});
    realEnv = setChunkRetryEnv(net);
    const existing = document.createElement('link');
    existing.rel = 'modulepreload';
    existing.href = `${BASE}core-AAAA.js`;
    document.head.append(existing);
    await vitePreload(async () => 1, ['./core-AAAA.js', './x-DDDD.js'], importer);
    await vitePreload(async () => 2, ['./x-DDDD.js'], importer);
    expect(net.calls).toEqual([`${BASE}x-DDDD.js`]);
  });

  const sheets = (): HTMLLinkElement[] => [...document.head.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')];

  it('a stylesheet that loads at once is not fetched', async () => {
    const net = network({});
    realEnv = setChunkRetryEnv(net);
    const result = vitePreload(async () => 'module', ['./mail-FFFF.css'], importer);
    await vi.waitFor(() => { expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('load'));
    await expect(result).resolves.toBe('module');
    expect(net.calls).toEqual([]);
  });

  it('a stylesheet that fails is fetched with the retry and loaded again', async () => {
    const net = network({ [`${BASE}mail-EEEE.css`]: [503, 200] });
    realEnv = setChunkRetryEnv(net);
    const result = vitePreload(async () => 'module', ['./mail-EEEE.css'], importer);
    await vi.waitFor(() => { expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('error'));
    await vi.waitFor(() => { expect(net.calls).toHaveLength(2); expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('load'));
    await expect(result).resolves.toBe('module');
  });

  it('a stylesheet that fails again is reported as a preload error', async () => {
    realEnv = setChunkRetryEnv(network({}));
    const seen: unknown[] = [];
    const listener = (event: Event): void => { seen.push((event as Event & { payload?: unknown }).payload); };
    window.addEventListener('vite:preloadError', listener);
    const result = vitePreload(async () => 'module', ['./mail-GGGG.css'], importer);
    await vi.waitFor(() => { expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('error'));
    await vi.waitFor(() => { expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('error'));
    await expect(result).rejects.toThrow(/Unable to preload CSS/);
    window.removeEventListener('vite:preloadError', listener);
    expect(String(seen[0])).toMatch(/mail-GGGG\.css/);
  });

  it('a 404 stylesheet is not retried', async () => {
    const net = network({ [`${BASE}gone-HHHH.css`]: [404] });
    realEnv = setChunkRetryEnv(net);
    const result = vitePreload(async () => 'module', ['./gone-HHHH.css'], importer);
    await vi.waitFor(() => { expect(sheets()).toHaveLength(1); });
    sheets()[0]!.dispatchEvent(new Event('error'));
    await expect(result).rejects.toThrow(/Unable to preload CSS/);
    expect(net.calls).toEqual([`${BASE}gone-HHHH.css`]);
  });

  it('waits for a download the boot script started instead of requesting the file again', async () => {
    const net = network({});
    realEnv = setChunkRetryEnv(net);
    const parked = (globalThis as { __mwPreloaded?: Record<string, Promise<boolean>> }).__mwPreloaded!;
    let finish: (ok: boolean) => void = () => undefined;
    parked[`${BASE}mail-IIII.js`] = new Promise((resolve) => { finish = resolve; });
    parked[`${BASE}mail-JJJJ.js`] = Promise.resolve(false);
    const load = vi.fn(async () => 'module');
    const result = vitePreload(load, ['./mail-IIII.js', './mail-JJJJ.js'], importer);
    await vi.waitFor(() => { expect(net.calls).toEqual([`${BASE}mail-JJJJ.js`]); });
    expect(load).not.toHaveBeenCalled();
    finish(true);
    await expect(result).resolves.toBe('module');
    // The boot download completed: no request of its own; the failed one was retried.
    expect(net.calls).toEqual([`${BASE}mail-JJJJ.js`]);
  });

  it('prefetchFile downloads a file once and an import then waits for it', async () => {
    const net = network({ [`${BASE}settings-KKKK.js`]: [503, 200] });
    realEnv = setChunkRetryEnv(net);
    const [a, b] = [prefetchFile(`${BASE}settings-KKKK.js`), prefetchFile(`${BASE}settings-KKKK.js`)];
    expect(a).toBe(b);
    expect(await a).toBe(true);
    await vitePreload(async () => 1, ['./settings-KKKK.js'], importer);
    expect(net.calls).toHaveLength(2);
  });

  it('a failed import is dispatched as vite:preloadError and thrown unless prevented', async () => {
    realEnv = setChunkRetryEnv(network({}));
    const failure = new TypeError('Failed to fetch dynamically imported module: x');
    await expect(vitePreload(async () => { throw failure; }, [], importer)).rejects.toBe(failure);
    const prevent = (event: Event): void => { event.preventDefault(); };
    window.addEventListener('vite:preloadError', prevent);
    await expect(vitePreload(async () => { throw failure; }, [], importer)).resolves.toBeUndefined();
    window.removeEventListener('vite:preloadError', prevent);
  });

  it('a dependency that stays 503 still reaches the module loader (its failure is the existing path)', async () => {
    const net = network({ [`${BASE}gone-GGGG.js`]: [503] });
    realEnv = setChunkRetryEnv(net);
    const load = vi.fn(async () => { throw new TypeError('Failed to fetch dynamically imported module'); });
    await expect(vitePreload(load, ['./gone-GGGG.js'], importer)).rejects.toThrow(/dynamically imported/);
    expect(load).toHaveBeenCalledTimes(1);
    expect(net.calls).toHaveLength(RETRY_DELAYS_MS.length + 1);
  });
});
