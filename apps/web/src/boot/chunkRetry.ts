// t52w: one place where a lazy chunk download is retried (the build's preload
// helper; vite.config.ts `chunkRetry()` puts this module in place of Vite's own
// `\0vite/preload-helper.js`).
//
// Why the download is retried BEFORE the module loader sees the file: Chromium
// 149, Firefox 151 and WebKit 26.5 all keep a failed module fetch in the
// document's module map. A second `import()` of a URL that answered 503 once,
// or of a chunk whose static dependency did, fails again without a request
// (checked 2026-11-24 with a server that answers 503 to the first request for
// each file and 200 after). Retrying the `import()` therefore cannot work; only
// a reload clears the map. So every file of a dynamic import (the chunk, the
// chunks it imports, their stylesheets: Vite's dependency list) is first
// fetched with `fetch()`, which can be repeated, and handed to the module
// loader only once it is in the HTTP cache (hashed files are served with a
// one-year `immutable` lifetime, crates/mw-server/src/lib.rs asset_response).
//
// What is retried: a 429/502/503/504 answer (the asset cache's 503
// `assetLoadBusy` carries `Retry-After: 1`) and a network error, up to
// RETRY_DELAYS_MS.length more times, waiting at least the listed delay and at
// most MAX_WAIT_MS, a delta-seconds `Retry-After` honoured inside those bounds.
// About 2 s in all without `Retry-After`.
//
// What is not: any other status, a 404 above all. A chunk that is missing after
// a deploy goes on to the module loader at once, fails there, and reaches the
// existing handling unchanged (`vite:preloadError` and the rejected import ->
// sw/bundleRecovery.ts, which reloads into a newer build or leaves the error
// page with its reload button). The retry never reloads and never shows
// anything itself.
//
// Stylesheets are not kept failed, so a stylesheet link is tried first and the
// file is fetched with the retry only after it errors.
//
// Otherwise this behaves like Vite's helper: each dependency is requested once
// per page, one already in the document as a link is skipped, scripts get a
// `modulepreload` link, stylesheets a `stylesheet` link that is waited for, and
// a failure is dispatched as `vite:preloadError` before it is thrown.

/** Waits before the 2nd, 3rd and 4th attempt (the minimum when `Retry-After` is shorter). */
export const RETRY_DELAYS_MS: readonly number[] = [250, 600, 1200];
/** Upper bound for one wait, whatever `Retry-After` asks for. */
export const MAX_WAIT_MS = 2500;

/** Statuses worth another attempt: overload and gateway answers. */
export const isTransientStatus = (status: number): boolean => status === 429 || (status >= 502 && status <= 504);

export interface RetryEnv {
  fetch: (url: string, init: RequestInit) => Promise<Response>;
  sleep: (ms: number) => Promise<void>;
}

let env: RetryEnv = {
  // eslint-disable-next-line mw/no-global-fetch -- Public hashed build files; no account API request.
  fetch: (url, init) => fetch(url, init),
  sleep: (ms) => new Promise((resolve) => { setTimeout(resolve, ms); }),
};
/** Tests only: replace fetch / sleep. */
export function setChunkRetryEnv(next: RetryEnv): RetryEnv { const previous = env; env = next; return previous; }

/**
 * Fetch `url` until it answers 200 (body read, so the HTTP cache holds all of
 * it), a non-transient status, or the attempts run out. Resolves `true` only
 * for a complete 200; never rejects.
 */
export async function fetchWithRetry(url: string, io: RetryEnv = env): Promise<boolean> {
  for (let attempt = 0; ; attempt++) {
    let asked = 0;
    try {
      const response = await io.fetch(url, { credentials: 'same-origin' });
      if (response.ok) return await response.arrayBuffer(), true;
      if (!isTransientStatus(response.status)) return false;
      // Delta-seconds only (what the server sends); anything else reads as 0.
      asked = (Number(response.headers.get('retry-after')) || 0) * 1000;
    } catch {
      // Network error: same schedule as a 503 without Retry-After.
    }
    const delay = RETRY_DELAYS_MS[attempt];
    if (delay === undefined) return false;
    await io.sleep(Math.min(MAX_WAIT_MS, Math.max(delay, asked)));
  }
}

const seen = new Set<string>();
/** Tests only: forget which dependencies this page already requested. */
export const resetChunkRetry = (): void => { seen.clear(); for (const url of Object.keys(early)) delete early[url]; };

/**
 * Downloads started before the import asks for them, by absolute URL: the boot
 * script's (src/boot/boot.js, for the first screen) and `prefetchFile`'s. An
 * import waits for one of these instead of requesting the file a second time.
 */
const early: Record<string, Promise<boolean> | undefined> = ((globalThis as { __mwPreloaded?: Record<string, Promise<boolean> | undefined> }).__mwPreloaded ??= {});

/** Download a hashed file ahead of its import, with the same retry; once per URL. */
export const prefetchFile = (url: string): Promise<boolean> => (early[url] ??= fetchWithRetry(url));

/** A stylesheet link; resolves false (and removes it) when the sheet fails to load. */
const sheet = (url: string): Promise<boolean> => new Promise((resolve) => {
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.crossOrigin = '';
  link.href = url;
  link.onload = () => { resolve(true); };
  link.onerror = () => { link.remove(); resolve(false); };
  document.head.appendChild(link);
});

async function preloadOne(dep: string, importerUrl: string | undefined): Promise<void> {
  const url = new URL(dep, importerUrl ?? document.baseURI).href;
  const css = url.endsWith('.css');
  if (seen.has(url) || [...document.getElementsByTagName('link')].some((l) => l.href === url && l.rel === (css ? 'stylesheet' : 'modulepreload'))) return;
  seen.add(url);
  // A download started earlier that did not complete is tried again here.
  const first = early[url];
  if (!css) {
    if (!(first && await first)) await fetchWithRetry(url);
    const link = document.createElement('link');
    link.rel = 'modulepreload';
    link.crossOrigin = '';
    link.href = url;
    link.setAttribute('as', 'script');
    document.head.appendChild(link);
    return;
  }
  // A stylesheet does not stay failed: load it, and only after an error fetch
  // it with the retry and load it again.
  if (first && !(await first)) await fetchWithRetry(url);
  if (await sheet(url)) return;
  if (!(await fetchWithRetry(url)) || !(await sheet(url))) throw new Error(`Unable to preload CSS for ${url}`);
}

function preloadError(error: unknown): void {
  const event = new Event('vite:preloadError', { cancelable: true }) as Event & { payload?: unknown };
  event.payload = error;
  window.dispatchEvent(event);
  if (!event.defaultPrevented) throw error;
}

/**
 * The build's `__vitePreload(baseModule, deps, importerUrl)`: `deps` lists the
 * files of the import relative to `importerUrl` (the importing chunk's
 * `import.meta.url`, the build uses a relative base). Signature and failure
 * events as Vite's own helper.
 */
export async function vitePreload<T>(baseModule: () => Promise<T>, deps?: readonly string[], importerUrl?: string): Promise<T> {
  if (deps?.length) {
    // preloadOne resolves to nothing and rejects only with an Error.
    for (const failure of await Promise.all(deps.map((dep) => preloadOne(dep, importerUrl).catch((error: unknown) => error)))) if (failure) preloadError(failure);
  }
  return baseModule().catch((error: unknown) => { preloadError(error); return undefined as T; });
}
