// The native shells' server address (t32). A desktop or mobile shell is not
// served by the Mailwoman server it talks to. The address is fixed when the app
// is built (`MW_SHELL_SERVER_URL`) or, on desktop, by a provisioning file an
// administrator installs; the shell injects it as `__MW_CONFIG__.serverUrl` before
// any SPA code runs, or `null` when it has neither. The user never enters or
// changes it. `transport.ts` reads it at every request.
//
// Kept apart from `transport.ts` so the Login screen can use it in tests that
// replace the transport module with a fake client.

import { isTauri } from '../platform/index.ts';

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
 * The native shell's server base URL without a trailing slash, or `null` when
 * the app was built without one and nothing provisioned one (the app then shows
 * that it has not been set up).
 */
export function nativeServerBase(): string | null {
  const url = mwConfig().serverUrl;
  if (typeof url !== 'string' || url.length === 0) return null;
  return url.endsWith('/') ? url.slice(0, -1) : url;
}
