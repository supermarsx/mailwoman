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
//
// Every client names itself in `X-Mailwoman-Client` (t32-cfg §4.18): a browser
// sends `browser/<version>` on sign-in; a shell sends `desktop/<version>;
// updater=on|off` or `mobile/<version>; updater=off` on sign-in and on every
// request to its server, from the `platform` it injects. The server keeps it with
// the session (`sessions.client_version`); the console lists the versions in use.

import { createClient, type ClientAuth, type Client } from './client.ts';
import { basePath } from './basePath.ts';
import { setClientHeader, setNativeTransport, type ClientHeader } from './http.ts';
import { isNativeAuth, nativeServerBase } from './serverAddress.ts';
import { platformReady } from '../platform/index.ts';
import { withSessionCache } from './sessionCache.ts';

export { isNativeAuth, nativeServerBase } from './serverAddress.ts';

interface MwConfig {
  serverUrl?: unknown;
  platform?: { kind?: unknown; version?: unknown; updater?: unknown };
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

/** A version as the header carries it: release characters only, else `unknown`. */
function headerVersion(v: unknown): string {
  return typeof v === 'string' && /^[0-9A-Za-z][0-9A-Za-z.+-]{0,31}$/.test(v) ? v : 'unknown';
}

/**
 * What this client sends in `X-Mailwoman-Client`, and whether on every request.
 * A shell is told apart by the native flag, then by the `platform.kind` it
 * injects: `desktop`, or `android`/`ios` for the mobile shell. The web client's
 * version is the one it was built as (`apps/web/package.json`).
 */
export function clientHeader(): ClientHeader {
  if (isNativeAuth()) {
    const platform = mwConfig().platform ?? {};
    const kind = platform.kind === 'desktop' ? 'desktop' : platform.kind === 'android' || platform.kind === 'ios' ? 'mobile' : null;
    if (kind !== null) {
      const updater = kind === 'desktop' && platform.updater === true ? 'on' : 'off';
      return { value: `${kind}/${headerVersion(platform.version)}; updater=${updater}`, everyRequest: true };
    }
  }
  return { value: `browser/${headerVersion(__MW_VERSION__)}`, everyRequest: false };
}

setClientHeader(clientHeader);

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
  // t99: every client shares one kept JMAP session per mailbox (`sessionCache.ts`).
  if (isNativeAuth()) return withSessionCache(createClient(nativeServerBase, platformAuth()), `native:${nativeServerBase() ?? ''}`);
  const base = transportBase();
  return withSessionCache(base === '' ? createClient() : createClient(base), base);
}
