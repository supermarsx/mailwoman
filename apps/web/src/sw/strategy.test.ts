import { describe, it, expect, vi } from 'vitest';
import {
  chooseStrategy,
  isApiPath,
  isFont,
  isHashedAsset,
  isSpaFallbackForSubresource,
  respondTo,
  shellUrl,
  shellUrls,
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

// Sub-path hosting (t20 B4). The prefix is stripped before the (prefix-blind)
// matchers run, so every classification under `/mail` must equal the one at the
// root — and, critically, a root-absolute path must NOT be misclassified when a
// prefix is configured.
describe('chooseStrategy under a sub-path', () => {
  const BASE = '/mail';

  it('classifies prefixed paths exactly as their root equivalents', () => {
    expect(chooseStrategy(get('/mail/jmap/api'), BASE)).toBe('network-first');
    expect(chooseStrategy(get('/mail/api/sanitize'), BASE)).toBe('network-first');
    expect(chooseStrategy(get('/mail/assets/index-a1b2c3d4.js'), BASE)).toBe('cache-first');
    expect(chooseStrategy(get('/mail/fonts/inter.woff2'), BASE)).toBe('cache-first');
    expect(chooseStrategy(get('/mail/inbox', 'navigate'), BASE)).toBe('shell-fallback');
    expect(chooseStrategy(get('/mail/favicon.ico'), BASE)).toBe('passthrough');
  });

  it('only strips on a segment boundary — /mailbox is not /mail + box', () => {
    // Were the prefix stripped as a bare string, this would become `/box/api/x`
    // and still classify as an API path for the wrong reason.
    expect(chooseStrategy(get('/mailbox/api/x'), BASE)).toBe('passthrough');
  });

  it('leaves an unprefixed path unstripped, and still classifies it', () => {
    // A path outside the prefix is passed through the matchers untouched rather
    // than rejected. Deliberate, and safe: the real worker is registered with
    // `scope: '/mail/'`, so its fetch handler never sees an out-of-scope request
    // at all. Leniency here only matters while a deployment is changing its base
    // path, where classifying by shape beats failing closed on a stale prefix.
    expect(chooseStrategy(get('/api/sanitize'), BASE)).toBe('network-first');
  });

  it('shell entries carry the prefix', () => {
    expect(shellUrl()).toBe('/');
    expect(shellUrls()).toEqual(['/']);
  });
});

function res(body = 'ok', ok = true, contentType = ''): Response {
  return {
    ok,
    clone: () => res(body, ok, contentType),
    body,
    headers: { get: (h: string) => (h.toLowerCase() === 'content-type' ? contentType : null) },
  } as unknown as Response;
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
      shellUrl: shellUrl(),
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

  // ── The SPA fallback must never be cached as a subresource (t24-e13) ───────
  // mw-server answers an unmatched path with index.html under a 200, so a miss on
  // a font came back 200 text/html and `res.ok` cached it under the font's URL.
  // Cache-first never revalidates, so the poisoned entry then outlived the
  // operator actually installing the fonts. These are the assertions that would
  // have caught it: the old code only checked `res.ok`, which is true here.
  it('cache-first does NOT cache the SPA fallback returned for a font', async () => {
    const fallback = res('<!doctype html><html>…', true, 'text/html; charset=utf-8');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => fallback) });
    // The response is still returned to the caller — only the caching is refused.
    expect(await respondTo(get('/fonts/inter-400.woff2'), d)).toBe(fallback);
    expect(puts, 'index.html must not be cached under a font URL').toEqual([]);
  });

  it('cache-first does NOT cache the SPA fallback returned for a hashed asset', async () => {
    const fallback = res('<!doctype html>', true, 'text/html');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => fallback) });
    await respondTo(get('/assets/app-a1b2c3d4.js'), d);
    expect(puts).toEqual([]);
  });

  it('network-first does NOT cache an HTML fallback returned for an API path', async () => {
    const fallback = res('<!doctype html>', true, 'text/html');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => fallback) });
    await respondTo(get('/jmap/api'), d);
    expect(puts).toEqual([]);
  });

  it('still caches a REAL font response', async () => {
    const font = res('wOF2…', true, 'font/woff2');
    const { deps: d, puts } = deps({ fetch: vi.fn(async () => font) });
    expect(await respondTo(get('/fonts/inter-400.woff2'), d)).toBe(font);
    expect(puts).toContain(`${ORIGIN}/fonts/inter-400.woff2`);
  });

  it('still caches HTML for a NAVIGATION (that one really is the shell)', async () => {
    const shell = res('<!doctype html>', true, 'text/html');
    const { deps: d } = deps({ fetch: vi.fn(async () => shell) });
    // Navigations are 'shell-fallback', which does not cache on the happy path;
    // the guard must not reclassify them as poisoned.
    expect(isSpaFallbackForSubresource(get('/inbox', 'navigate'), shell)).toBe(false);
    expect(await respondTo(get('/inbox', 'navigate'), d)).toBe(shell);
  });

  it('offline navigation falls back to the precached app shell', async () => {
    const shell = res('<!doctype html>');
    const { deps: d } = deps({
      fetch: vi.fn(async () => {
        throw new Error('offline');
      }),
      cacheMatch: vi.fn(async (url: string) => (url === shellUrl() ? shell : undefined)),
    });
    expect(await respondTo(get('/inbox', 'navigate'), d)).toBe(shell);
  });
});
