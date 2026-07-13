// Transport configuration (plan §2.2, §3 e6) — the ONE place the SPA decides
// whether it talks to its own origin over cookies (browser) or to a configured
// Mailwoman server over a bearer token (the V5 native shell).
//
// The browser path is the hard regression gate: with no shell-injected config,
// `transportBase()` is '' and `createConfiguredClient()` is exactly
// `createClient()` — cookie, same-origin, no bearer, byte-identical to pre-V5.
//
// The shell injects `globalThis.__MW_CONFIG__` at boot (the same config-injection
// precedent `viewers/max-security.ts` reads): `serverUrl` retargets the base and
// `native: true` turns on the bearer path (the token comes from the platform's
// OS-keychain session store).

import { createClient, type ClientAuth, type Client } from './client.ts';
import { getPlatform, isTauri } from '../platform/index.ts';

interface MwConfig {
  serverUrl?: unknown;
  native?: unknown;
}

function mwConfig(): MwConfig {
  return (globalThis as { __MW_CONFIG__?: MwConfig }).__MW_CONFIG__ ?? {};
}

/**
 * The transport base URL. Browser: '' (same-origin). Native shell: the injected
 * `serverUrl`. A trailing slash is trimmed so `${base}/api/...` stays well-formed.
 */
export function transportBase(): string {
  const url = mwConfig().serverUrl;
  if (typeof url !== 'string' || url.length === 0) return '';
  return url.endsWith('/') ? url.slice(0, -1) : url;
}

/** True only in a Tauri shell configured for native bearer auth (plan §2.2). */
export function isNativeAuth(): boolean {
  return isTauri() && mwConfig().native === true;
}

/** The bearer-auth provider backed by the platform's OS-keychain token store. */
export function platformAuth(): ClientAuth {
  return { token: () => getPlatform().getSessionToken() };
}

/**
 * Build the API client for the current runtime. In a plain browser this returns
 * `createClient()` verbatim (cookie, same-origin) — the regression-critical path.
 * In a configured native shell it points at the injected server and attaches the
 * keychain bearer token.
 */
export function createConfiguredClient(): Client {
  if (isNativeAuth()) return createClient(transportBase(), platformAuth());
  const base = transportBase();
  return base === '' ? createClient() : createClient(base);
}
