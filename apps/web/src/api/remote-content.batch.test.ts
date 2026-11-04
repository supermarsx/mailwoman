import { afterEach, describe, expect, it, vi } from 'vitest';
import { fetchImageBatch, imageToken, prewarmSignedImages, IMAGE_BATCH_MAX, IMAGE_BATCH_RETRIES } from './remote-content.ts';

const reply = (results: unknown[], status = 200): Response => new Response(JSON.stringify({ results }), { status, headers: { 'content-type': 'application/json' } });
const sent = (mock: ReturnType<typeof vi.fn>, call: number): { tokens: string[] } => JSON.parse(String((mock.mock.calls[call]?.[1] as RequestInit).body)) as { tokens: string[] };
afterEach(() => { vi.unstubAllGlobals(); });

describe('fetchImageBatch', () => {
  it('retries only the transiently refused images, quietly, honouring retryAfter with growing backoff', async () => {
    const mock = vi.fn()
      .mockResolvedValueOnce(reply([{ index: 0, status: 200 }, { index: 1, status: 503, retryAfter: 3 }, { index: 2, status: 403 }, { index: 3, status: 429 }]))
      .mockResolvedValueOnce(reply([{ index: 0, status: 503 }, { index: 1, status: 429 }]))
      .mockResolvedValueOnce(reply([{ index: 0, status: 200 }, { index: 1, status: 200 }]));
    vi.stubGlobal('fetch', mock);
    const waits: number[] = [];
    const out = await fetchImageBatch(['a', 'b', 'c', 'd'], { sleep: async (ms) => { waits.push(ms); } });
    expect(sent(mock, 1).tokens).toEqual(['b', 'd']);
    expect(sent(mock, 2).tokens).toEqual(['b', 'd']);
    expect(out.map((r) => r.status)).toEqual([200, 200, 403, 200]);
    expect(out.map((r) => r.index)).toEqual([0, 1, 2, 3]);
    expect(waits).toEqual([3000, 1000]);
  });

  it('returns the busy status only after the bounded retries are exhausted', async () => {
    const mock = vi.fn(async () => reply([{ index: 0, status: 503, retryAfter: 1 }]));
    vi.stubGlobal('fetch', mock);
    const sleep = vi.fn(async () => undefined);
    const out = await fetchImageBatch(['a'], { sleep });
    expect(out[0]?.status).toBe(503);
    expect(mock).toHaveBeenCalledTimes(IMAGE_BATCH_RETRIES + 1);
    expect(sleep).toHaveBeenCalledTimes(IMAGE_BATCH_RETRIES);
  });

  it('sends at most 100 tokens per request', async () => {
    const mock = vi.fn(async (_url: string, init: RequestInit) => reply((JSON.parse(String(init.body)) as { tokens: string[] }).tokens.map((_, index) => ({ index, status: 200 }))));
    vi.stubGlobal('fetch', mock);
    const tokens = Array.from({ length: 230 }, (_, i) => `t${i}`);
    const out = await fetchImageBatch(tokens);
    expect(mock).toHaveBeenCalledTimes(3);
    expect(sent(mock, 0).tokens).toHaveLength(IMAGE_BATCH_MAX);
    expect(out).toHaveLength(230);
  });

  it('rejects when the route is unavailable, and prewarm swallows that', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('no', { status: 404 })));
    await expect(fetchImageBatch(['a'])).rejects.toThrow();
    await expect(prewarmSignedImages([{ src: `${location.origin}/api/rc/i/abc` }])).resolves.toBeUndefined();
  });

  it('only batches same-server signed image sources', () => {
    expect(imageToken(`${location.origin}/api/rc/i/abc.def`)).toBe('abc.def');
    expect(imageToken('https://evil.example/api/rc/i/abc')).toBeNull();
    expect(imageToken(`${location.origin}/api/rc/p/abc`)).toBeNull();
  });
});
