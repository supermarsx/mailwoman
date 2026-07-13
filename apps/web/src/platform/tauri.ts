// Native (Tauri) implementation of the `Platform` capability layer (plan §2.1,
// §2.5, §3 e6).
//
// IMPORTANT — why the imports look the way they do: `@tauri-apps/*` is NOT a
// dependency of `apps/web` (it lives only in the `apps/desktop`/`apps/mobile`
// build graphs). To keep this module compiling AND to guarantee those packages
// NEVER enter the plain-browser bundle, every Tauri import here is a RUNTIME
// dynamic import through a NON-LITERAL specifier tagged `/* @vite-ignore */`:
//   * the non-literal specifier makes `tsc` type the result `any` and skip module
//     resolution, so `apps/web` typechecks without the packages installed;
//   * `@vite-ignore` makes Vite leave it as a native runtime `import()` instead of
//     trying to resolve/bundle it — so no `@tauri-apps` code lands in any chunk.
// This module itself is reached ONLY via `index.ts`'s dynamic `import('./tauri.ts')`
// under `isTauri()`, so in a browser it is never even fetched.
//
// The capability surface is intentionally THIN: nearly every method is one
// `invoke('mw_*')` call whose Rust command (e1 desktop / e2 mobile / e4 capture)
// does the real OS work, plus a few `listen('mw://…')` event bridges. The command
// + event names below are the frozen JS↔shell contract for e1/e2/e4.

import { createBrowserPlatform } from './browser.ts';
import type {
  CapabilityResult,
  NotificationActionEvent,
  NotifyInput,
  Platform,
  PlatformInfo,
  PlatformKind,
  PushSubscriptionInfo,
  PushTransport,
  SelfContainedStatus,
  ServerEntry,
  ShareTargetPayload,
  Unsubscribe,
} from './index.ts';

/** Structural view of `@tauri-apps/api/core` (only `invoke`). */
interface TauriCore {
  invoke<T = void>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}
/** Structural view of `@tauri-apps/api/event` (only `listen`). */
interface TauriEvent {
  listen<T>(event: string, handler: (e: { payload: T }) => void): Promise<() => void>;
}

/** Load a Tauri module at runtime without letting Vite/tsc resolve it (see top). */
async function loadTauri<T>(spec: string): Promise<T> {
  return (await import(/* @vite-ignore */ spec)) as T;
}

const core = (): Promise<TauriCore> => loadTauri<TauriCore>('@tauri-apps/api/core');
const events = (): Promise<TauriEvent> => loadTauri<TauriEvent>('@tauri-apps/api/event');

/** `invoke` a shell command, returning the typed result. */
async function invoke<T = void>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: inv } = await core();
  return inv<T>(cmd, args);
}

/**
 * Bridge a Tauri event to the interface's synchronous `Unsubscribe` shape: start
 * `listen()` (async), and return a disposer that awaits the unlisten handle.
 */
function bridge<T>(event: string, handler: (payload: T) => void): Unsubscribe {
  let unlisten: (() => void) | null = null;
  let disposed = false;
  void events()
    .then(({ listen }) => listen<T>(event, (e) => handler(e.payload)))
    .then((off) => {
      if (disposed) off();
      else unlisten = off;
    })
    .catch(() => {
      /* the shell has no such event source: nothing to bridge. */
    });
  return () => {
    disposed = true;
    unlisten?.();
  };
}

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

/** The committed push transport per platform kind (plan §2.3). */
function pushTransportFor(kind: PlatformKind): PushTransport {
  if (kind === 'android') return 'unifiedpush';
  if (kind === 'ios') return 'apns';
  return 'webpush'; // desktop
}

// OS-keychain namespaces (e1's `mw_keychain_*` contract, t7-e1.md): the session
// bearer token and the key-vault-wrap secure store are distinct services in the
// native credential store. The session token is a single fixed-key entry.
const KC_SESSION = 'mailwoman.session';
const KC_SECURE = 'mailwoman.secure';
const KC_SESSION_KEY = 'token';

/**
 * Create the native platform. Composes over the browser fallback (so anything the
 * shell does not back yet still degrades honestly) and overrides each capability
 * with its `mw_*` command / `mw://*` event bridge.
 */
export async function createTauriPlatform(): Promise<Platform> {
  const base = createBrowserPlatform();
  const info = injectedPlatform();
  const kind: PlatformKind = info?.kind ?? 'desktop';

  return {
    ...base,

    platform(): PlatformInfo {
      return info ?? base.platform();
    },

    // ── Server config (multi-server, persisted natively — e1 `mw_server_*`). ──
    getServerUrl: () => invoke<string | null>('mw_server_get_selected'),
    // The Platform interface's single-arg `setServerUrl` maps to add-then-select
    // over e1's multi-server store (label defaults to the URL).
    setServerUrl: async (url) => {
      await invoke('mw_server_add', { url, label: url });
      await invoke('mw_server_select', { url });
    },
    listServers: () => invoke<ServerEntry[]>('mw_server_list'),
    selectServer: (url) => invoke('mw_server_select', { url }),

    // ── Auth token store (OS keychain, `mw_keychain_*` service=session). ──
    getSessionToken: () =>
      invoke<string | null>('mw_keychain_get', { service: KC_SESSION, key: KC_SESSION_KEY }),
    setSessionToken: (token) =>
      invoke('mw_keychain_set', { service: KC_SESSION, key: KC_SESSION_KEY, value: token }),
    clearSessionToken: () =>
      invoke('mw_keychain_delete', { service: KC_SESSION, key: KC_SESSION_KEY }),

    // ── Secure store (OS keychain, `mw_keychain_*` service=secure). ──
    secureGet: (key) => invoke<string | null>('mw_keychain_get', { service: KC_SECURE, key }),
    secureSet: (key, value) => invoke('mw_keychain_set', { service: KC_SECURE, key, value }),
    secureDelete: (key) => invoke('mw_keychain_delete', { service: KC_SECURE, key }),

    // ── Notifications (native, with action buttons) + badge. ──
    notify: (input: NotifyInput) => invoke('mw_notify', { input }),
    onNotificationAction: (cb: (e: NotificationActionEvent) => void): Unsubscribe =>
      bridge<NotificationActionEvent>('mw://notification-action', cb),
    setBadgeCount: (count: number) => invoke('mw_set_badge_count', { count }),

    // ── Deep links / mailto:. ──
    onOpenUrl: (cb: (url: string) => void): Unsubscribe =>
      bridge<string>('mw://open-url', cb),
    registerMailtoHandler: () => invoke('mw_register_mailto_handler'),

    // ── Screen-capture protection (e4: Win/macOS content-protection, FLAG_SECURE). ──
    setCaptureProtection: (enabled) =>
      invoke<CapabilityResult>('mw_set_capture_protection', { enabled }),

    // ── Biometric app-lock. ──
    biometricAvailable: () => invoke<boolean>('mw_biometric_available'),
    biometricAuthenticate: ({ reason }) =>
      invoke<boolean>('mw_biometric_authenticate', { reason }),

    // ── Share / drag-out (e1 `mw_dragout_materialize`, bytes as number[]). ──
    onShareTarget: (cb: (payload: ShareTargetPayload) => void): Unsubscribe =>
      bridge<ShareTargetPayload>('mw://share-target', cb),
    // e1 materializes temp files from raw bytes; blobId → bytes must be resolved
    // by the caller first. Uint8Array → number[] for IPC serialization.
    startDragOut: (files) =>
      invoke('mw_dragout_materialize', {
        files: files.map((f) => ({
          name: f.name,
          mime: f.mime,
          bytes: f.bytes !== undefined ? Array.from(f.bytes) : [],
        })),
      }),

    // ── Push (desktop WebPush / Android UnifiedPush / iOS APNs). ──
    pushSubscribe: () => invoke<PushSubscriptionInfo | null>('mw_push_subscribe'),
    pushUnsubscribe: () => invoke('mw_push_unsubscribe'),
    getPushTransport: (): PushTransport | null => pushTransportFor(kind),

    // ── Self-contained lifecycle (desktop only). ──
    selfContainedStatus: () => invoke<SelfContainedStatus>('mw_self_contained_status'),
    startLocalServer: () => invoke('mw_start_local_server'),
    stopLocalServer: () => invoke('mw_stop_local_server'),
  };
}
