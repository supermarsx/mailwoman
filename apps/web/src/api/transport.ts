// Transport configuration (plan §2.2, §3 e6) — the ONE place the SPA decides
// whether it talks to its own origin over cookies (browser) or to a configured
// Mailwoman server over a bearer token (the V5 native shell).
//
// The browser path is the hard regression gate: with no shell-injected config and
// no server-injected `__MW_BASE__`, `transportBase()` is '' and
// `createConfiguredClient()` is exactly `createClient()` — cookie, same-origin, no
// bearer, byte-identical to pre-V5.
//
// Sub-path hosting (t20 B4) rides on this same seam: under `MW_BASE_PATH=/mail`
// the browser base becomes '/mail', so every `${base}/api/...` in `client.ts`
// acquires the prefix with no further change.
//
// The shell injects `globalThis.__MW_CONFIG__` at boot (the same config-injection
// precedent `viewers/max-security.ts` reads): `serverUrl` retargets the base and
// `native: true` turns on the bearer path (the token comes from the platform's
// OS-keychain session store).
//
// The shells inject the server address they were built or provisioned for (t32),
// or `serverUrl: null` when they have none; nothing in the SPA changes it. Every
// request waits for the platform first (`platformReady`), so the first one at boot
// already sees the stored token.

import { createClient, type ClientAuth, type Client } from './client.ts';
import { basePath } from './basePath.ts';
import { setNativeTransport } from './http.ts';
import { isNativeAuth, nativeServerBase } from './serverAddress.ts';
import { platformReady } from '../platform/index.ts';

export { isNativeAuth, nativeServerBase } from './serverAddress.ts';

interface MwConfig {
  serverUrl?: unknown;
}

function mwConfig(): MwConfig {
  return (globalThis as { __MW_CONFIG__?: MwConfig }).__MW_CONFIG__ ?? {};
}

/**
 * The transport base URL. Browser: the sub-path prefix — `''` at the origin root
 * (unchanged from before sub-path support) or `/mail` when the server injected
 * `__MW_BASE__`. Native shell: the injected `serverUrl` verbatim, because the
 * operator sets that URL in full and it already carries any prefix; `__MW_BASE__`
 * is an `index.html` injection the native shell never receives.
 *
 * A trailing slash is trimmed either way so `${base}/api/...` stays well-formed.
 */
export function transportBase(): string {
  const url = mwConfig().serverUrl;
  if (typeof url !== 'string' || url.length === 0) return basePath();
  return url.endsWith('/') ? url.slice(0, -1) : url;
}

/**
 * The bearer-auth provider backed by the platform's OS-keychain token store. It
 * waits for the platform first, so a request made while `initPlatform` is still
 * running reads the native store, not the browser default's empty one.
 */
export function platformAuth(): ClientAuth {
  return {
    token: async () => (await platformReady()).getSessionToken(),
    setToken: async (token) => (await platformReady()).setSessionToken(token),
    clearToken: async () => (await platformReady()).clearSessionToken(),
  };
}

// A native shell sends root-relative `apiFetch` requests (Settings, prefs, …) to
// its server with the same token store. Registered once, at import, because the
// native flag is injected before any SPA code runs.
if (isNativeAuth()) {
  const auth = platformAuth();
  setNativeTransport({
    base: nativeServerBase,
    token: auth.token,
    setToken: (token) => auth.setToken?.(token) ?? Promise.resolve(),
  });
}

/**
 * Build the API client for the current runtime. In a plain browser this returns
 * `createClient()` verbatim (cookie, same-origin) — the regression-critical path.
 * In a configured native shell it points at the server the shell was built or
 * provisioned for and attaches the keychain bearer token.
 */
export function createConfiguredClient(): Client {
  if (isNativeAuth()) return createClient(nativeServerBase, platformAuth());
  const base = transportBase();
  return base === '' ? createClient() : createClient(base);
}
