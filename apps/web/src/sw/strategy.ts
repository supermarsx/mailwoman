// Service-Worker fetch strategy (plan §2.5), authored as PURE functions so they
// are unit-tested here and MIRRORED by the hand-rolled runtime SW in
// public/sw.js. The runtime SW cannot import from src/ (it ships copied verbatim
// into dist/, no bundling — see vite.config.ts), so public/sw.js re-declares the
// same decisions inline. Keep the two in sync; these functions are the spec.

import { shellCacheName } from '../contracts/offline.ts';
import { basePath, shellBase, stripBase } from '../api/basePath.ts';

export type Strategy = 'network-first' | 'cache-first' | 'shell-fallback' | 'passthrough';

/** The runtime cache name (`mw-shell-v{N}`) the SW precaches + serves from. */
export const CACHE_NAME = shellCacheName();

/**
 * The app-shell entry precached for offline navigation — `/` at the origin root,
 * `/mail/` under a sub-path (t20 B4). It serves `index.html`; its (hashed) asset
 * references are then filled in cache-first on first load.
 *
 * A function rather than a const because the prefix is a RUNTIME value: the same
 * bundle serves from either place and only the server-injected `__MW_BASE__`
 * differs.
 */
export function shellUrl(): string {
  return shellBase();
}

/** The shell entries precached on install. */
export function shellUrls(): readonly string[] {
  return [shellUrl()];
}

export interface ReqLike {
  url: string;
  method: string;
  /** Request.mode — `'navigate'` marks a top-level navigation. */
  mode?: string;
}

/** JMAP + app API surface (network-first: fresh mail wins, cache is the fallback). */
export function isApiPath(pathname: string): boolean {
  return pathname.startsWith('/jmap/') || pathname.startsWith('/api/');
}

/** Self-hosted fonts (cache-first: immutable, `font-src 'self'`). */
export function isFont(pathname: string): boolean {
  return /\.(?:woff2?|ttf|otf)$/i.test(pathname);
}

/** Vite emits content-hashed, immutable build assets (cache-first). */
export function isHashedAsset(pathname: string): boolean {
  // e.g. /assets/index-a1b2c3d4.js  — a `-<hash>` segment before the extension.
  return /-[A-Za-z0-9_]{8,}\.[a-z0-9]+$/i.test(pathname) || pathname.startsWith('/assets/');
}

/**
 * Classify a request into a fetch strategy (§2.5).
 *
 * Sub-path hosting (t20 B4): the deploy prefix is stripped BEFORE matching, so the
 * matchers above stay prefix-blind and `/mail/api/x` classifies exactly as
 * `/api/x` does. `base` is injectable so the pure functions remain testable
 * without a global; it defaults to the runtime prefix.
 */
export function chooseStrategy(req: ReqLike, base: string = basePath()): Strategy {
  if (req.method !== 'GET') return 'passthrough';
  const pathname = stripBase(new URL(req.url).pathname, base);
  if (isApiPath(pathname)) return 'network-first';
  if (req.mode === 'navigate') return 'shell-fallback';
  if (isHashedAsset(pathname) || isFont(pathname)) return 'cache-first';
  return 'passthrough';
}

/** Cache + network operations, injected so `respondTo` is testable without the
 *  real SW `caches` / `fetch` globals. */
export interface FetchDeps {
  fetch(req: ReqLike): Promise<Response>;
  cacheMatch(url: string): Promise<Response | undefined>;
  cachePut(url: string, res: Response): Promise<void>;
  shellUrl: string;
}

async function networkFirst(req: ReqLike, deps: FetchDeps): Promise<Response> {
  try {
    const res = await deps.fetch(req);
    if (res.ok) await deps.cachePut(req.url, res.clone());
    return res;
  } catch (err) {
    const cached = await deps.cacheMatch(req.url);
    if (cached !== undefined) return cached;
    throw err;
  }
}

async function cacheFirst(req: ReqLike, deps: FetchDeps): Promise<Response> {
  const cached = await deps.cacheMatch(req.url);
  if (cached !== undefined) return cached;
  const res = await deps.fetch(req);
  if (res.ok) await deps.cachePut(req.url, res.clone());
  return res;
}

async function shellFallback(req: ReqLike, deps: FetchDeps): Promise<Response> {
  try {
    return await deps.fetch(req);
  } catch (err) {
    const shell = await deps.cacheMatch(deps.shellUrl);
    if (shell !== undefined) return shell;
    throw err;
  }
}

/** Run the chosen strategy for a request. Mirrors the runtime SW `fetch` handler. */
export function respondTo(req: ReqLike, deps: FetchDeps, base: string = basePath()): Promise<Response> {
  switch (chooseStrategy(req, base)) {
    case 'network-first':
      return networkFirst(req, deps);
    case 'cache-first':
      return cacheFirst(req, deps);
    case 'shell-fallback':
      return shellFallback(req, deps);
    case 'passthrough':
      return deps.fetch(req);
  }
}
