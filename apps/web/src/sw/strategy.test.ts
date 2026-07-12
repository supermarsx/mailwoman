import { describe, it, expect, vi } from 'vitest';
import {
  chooseStrategy,
  isApiPath,
  isFont,
  isHashedAsset,
  respondTo,
  SHELL_URL,
  type FetchDeps,
  type ReqLike,
} from './strategy.ts';

const ORIGIN = 'https://mail.example.com';
const get = (path: string, mode?: string): ReqLike => ({
  url: `${ORIGIN}${path}`,
  method: 'GET',
  ...(mode === undefined ? {} : { mode }),
});

describe('path classifiers', () => {
  it('recognises the JMAP + API surface', () => {
    expect(isApiPath('/jmap/api')).toBe(true);
    expect(isApiPath('/jmap/ws')).toBe(true);
    expect(isApiPath('/api/sanitize')).toBe(true);
    expect(isApiPath('/assets/index-a1b2c3d4.js')).toBe(false);
  });

  it('recognises self-hosted fonts', () => {
    expect(isFont('/fonts/inter.woff2')).toBe(true);
    expect(isFont('/fonts/serif.ttf')).toBe(true);
    expect(isFont('/assets/index-a1b2c3d4.js')).toBe(false);
  });

  it('recognises content-hashed build assets', () => {
    expect(isHashedAsset('/assets/index-a1b2c3d4.js')).toBe(true);
    expect(isHashedAsset('/logo-deadbeef99.svg')).toBe(true);
    expect(isHashedAsset('/index.html')).toBe(false);
  });
});

describe('chooseStrategy', () => {
  it('routes /jmap and /api to network-first', () => {
    expect(chooseStrategy(get('/jmap/api'))).toBe('network-first');
    expect(chooseStrategy(get('/api/sanitize'))).toBe('network-first');
  });

  it('routes top-level navigations to the shell fallback', () => {
    expect(chooseStrategy(get('/inbox', 'navigate'))).toBe('shell-fallback');
  });

  it('routes hashed assets + fonts to cache-first', () => {
    expect(chooseStrategy(get('/assets/index-a1b2c3d4.js'))).toBe('cache-first');
    expect(chooseStrategy(get('/fonts/inter.woff2'))).toBe('cache-first');
  });

  it('passes everything else through', () => {
    expect(chooseStrategy(get('/favicon.ico'))).toBe('passthrough');
  });

  it('never caches non-GET (writes hit the network)', () => {
    expect(chooseStrategy({ url: `${ORIGIN}/jmap/api`, method: 'POST' })).toBe('passthrough');
  });
});

function res(body = 'ok', ok = true): Response {
  return { ok, clone: () => res(body, ok), body } as unknown as Response;
}

function deps(
  over: Partial<FetchDeps> & { fetch: FetchDeps['fetch'] },
): { deps: FetchDeps; puts: string[] } {
  const puts: string[] = [];
  return {
    puts,
    deps: {
      cacheMatch: vi.fn(async () => undefined),
      cachePut: vi.fn(async (url: string) => {
        puts.push(url);
      }),
      shellUrl: SHELL_URL,
      ...over,
    },
  };
}

describe('respondTo', () => {
  it('network-first serves + caches the network response when online', async () => {
    const network = res('fresh');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => network) });
    const out = await respondTo(get('/jmap/api'), d);
    expect(out).toBe(network);
    expect(puts).toContain(`${ORIGIN}/jmap/api`);
  });

  it('network-first falls back to cache when the network throws', async () => {
    const cached = res('stale');
    const { deps: d } = deps({
      fetch: vi.fn(async () => {
        throw new Error('offline');
      }),
      cacheMatch: vi.fn(async () => cached),
    });
    expect(await respondTo(get('/jmap/api'), d)).toBe(cached);
  });

  it('network-first rethrows when offline and nothing is cached', async () => {
    const { deps: d } = deps({
      fetch: vi.fn(async () => {
        throw new Error('offline');
      }),
    });
    await expect(respondTo(get('/jmap/api'), d)).rejects.toThrow('offline');
  });

  it('cache-first serves the cache without touching the network', async () => {
    const cached = res('immutable');
    const fetchSpy = vi.fn(async () => res('network'));
    const { deps: d } = deps({ fetch: fetchSpy, cacheMatch: vi.fn(async () => cached) });
    expect(await respondTo(get('/assets/app-a1b2c3d4.js'), d)).toBe(cached);
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('cache-first fetches + populates the cache on a miss', async () => {
    const network = res('firstload');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => network) });
    expect(await respondTo(get('/assets/app-a1b2c3d4.js'), d)).toBe(network);
    expect(puts).toContain(`${ORIGIN}/assets/app-a1b2c3d4.js`);
  });

  it('offline navigation falls back to the precached app shell', async () => {
    const shell = res('<!doctype html>');
    const { deps: d } = deps({
      fetch: vi.fn(async () => {
        throw new Error('offline');
      }),
      cacheMatch: vi.fn(async (url: string) => (url === SHELL_URL ? shell : undefined)),
    });
    expect(await respondTo(get('/inbox', 'navigate'), d)).toBe(shell);
  });
});
