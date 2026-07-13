// Native (Tauri) implementation of the `Platform` capability layer (plan §2.1,
// §2.5, §3 e0/e1/e6).
//
// IMPORTANT: this module is DYNAMICALLY imported by `./index.ts` only under
// `isTauri()`, so it (and, once e6 wires them, the `@tauri-apps/*` packages it
// pulls in) NEVER enter the plain-browser bundle — they load as a lazy chunk that
// executes only inside a shell webview.
//
// e0 ships a delegating stub: it reports the true native platform kind (from the
// shell-injected `__MW_CONFIG__`) and otherwise reuses the browser fallbacks so
// the SPA is fully functional in a shell today. The real native wiring lands next:
//   * e1 — desktop notifications-with-actions, OS keychain (session token + vault
//          wrap via `keyring`), deep-link/mailto, badge, biometric, drag-out,
//          multi-server, invoked over Tauri IPC commands.
//   * e2 — mobile UnifiedPush subscribe, share targets, file handlers, badge.
//   * e4 — `setCaptureProtection` → `set_content_protection` / FLAG_SECURE.
//   * e6 — pulls `@tauri-apps/api` + the plugin JS packages (declared in the shell
//          package.jsons), replaces these bodies with real plugin/IPC calls, and
//          threads the configurable server base URL + bearer auth through the
//          transport. Each method keeps the frozen signature below.

import { createBrowserPlatform } from './browser.ts';
import type { Platform, PlatformInfo, PlatformKind } from './index.ts';

/** Read the shell-injected platform descriptor (`__MW_CONFIG__.platform`). */
function injectedPlatform(): PlatformInfo | null {
  const cfg = (
    globalThis as {
      __MW_CONFIG__?: { platform?: { kind?: unknown; os?: unknown; version?: unknown } };
    }
  ).__MW_CONFIG__;
  const p = cfg?.platform;
  if (!p) return null;
  const kind = p.kind;
  if (kind !== 'desktop' && kind !== 'android' && kind !== 'ios') return null;
  return {
    kind: kind as PlatformKind,
    os: typeof p.os === 'string' ? p.os : 'unknown',
    version: typeof p.version === 'string' ? p.version : '',
  };
}

/**
 * Create the native platform. Async because the real impl (e6) awaits dynamic
 * imports of `@tauri-apps/*`. The e0 stub composes over the browser fallback and
 * overrides only what it can know natively today (the platform descriptor).
 */
export async function createTauriPlatform(): Promise<Platform> {
  const base = createBrowserPlatform();
  const info = injectedPlatform();

  return {
    ...base,
    platform(): PlatformInfo {
      return info ?? base.platform();
    },
    // Every other capability delegates to the browser fallback until its owning
    // executor (e1/e2/e4/e6) replaces it with the real Tauri plugin/IPC call. The
    // spread above wires the frozen signatures; overrides land here.
  };
}
