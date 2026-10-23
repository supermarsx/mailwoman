// The native shells' server address (t31). A desktop or mobile shell is not
// served by the Mailwoman server it talks to, so the user names it on the Login
// screen. It lives in `__MW_CONFIG__.serverUrl`, which the shell injects as `null`
// and the native platform fills from its server store at start
// (`platform/tauri.ts`); `transport.ts` reads it at every request.
//
// Kept apart from `transport.ts` so the Login screen can use it in tests that
// replace the transport module with a fake client.

import { isTauri, platformReady } from '../platform/index.ts';

interface MwConfig {
  serverUrl?: unknown;
  native?: unknown;
}

function mwConfig(): MwConfig {
  return (globalThis as { __MW_CONFIG__?: MwConfig }).__MW_CONFIG__ ?? {};
}

/** True only in a Tauri shell configured for native bearer auth (plan §2.2). */
export function isNativeAuth(): boolean {
  return isTauri() && mwConfig().native === true;
}

/**
 * The native shell's server base URL without a trailing slash, or `null` while
 * no server address has been entered (the Login screen asks for one).
 */
export function nativeServerBase(): string | null {
  const url = mwConfig().serverUrl;
  if (typeof url !== 'string' || url.length === 0) return null;
  return url.endsWith('/') ? url.slice(0, -1) : url;
}

/**
 * Canonicalise a server address the user typed: `https://` is assumed when no
 * scheme is given, only `http:` and `https:` are accepted, and the trailing slash
 * is dropped (a path is kept for a server under a sub-path). `null` when it is not
 * an address.
 */
export function normalizeServerUrl(raw: string): string | null {
  const typed = raw.trim();
  if (typed === '') return null;
  const withScheme = /^[a-z][a-z0-9+.-]*:\/\//i.test(typed) ? typed : `https://${typed}`;
  let url: URL;
  try {
    url = new URL(withScheme);
  } catch {
    return null;
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return null;
  if (url.hostname === '' || url.username !== '' || url.password !== '') return null;
  if (url.search !== '' || url.hash !== '') return null;
  const path = url.pathname.replace(/\/+$/, '');
  return `${url.origin}${path}`;
}

/**
 * Point the native shell at `url` (already normalised): the client's next
 * request goes there, and the platform keeps it for the next start. A different
 * server than before drops the stored token, which belongs to the old one.
 */
export async function chooseServerUrl(url: string): Promise<void> {
  const g = globalThis as { __MW_CONFIG__?: MwConfig };
  const previous = nativeServerBase();
  // In place: the native platform holds the same object.
  (g.__MW_CONFIG__ ??= {}).serverUrl = url;
  const platform = await platformReady();
  if (previous !== null && previous !== url) await platform.clearSessionToken();
  await platform.setServerUrl(url);
}
