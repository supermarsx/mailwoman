// Browser implementation of the `Platform` capability layer (plan §2.1, §3 e0).
//
// This is the DEFAULT everywhere and the honest web fallback: it never touches a
// Tauri API, degrades every optional capability gracefully, and keeps the browser
// code path byte-identical to pre-V5 behavior. e6 fleshes out the richer fallbacks
// (Web Push VAPID subscribe, OPFS-AES-GCM secure store, tab-title/favicon badge,
// the watermark toggle for capture); e0 ships working, side-effect-safe stubs so
// the SPA builds, typechecks, tests, and RUNS unchanged in a plain browser.

import type {
  CapabilityResult,
  DragOutFile,
  NotificationActionEvent,
  NotifyInput,
  Platform,
  PlatformInfo,
  PushSubscriptionInfo,
  PushTransport,
  SelfContainedStatus,
  ServerEntry,
  Unsubscribe,
} from './index.ts';

/** Prefix for the localStorage-backed secure-store fallback (e6 → OPFS vault). */
const SECURE_PREFIX = 'mw.secure.';

function hasNotification(): boolean {
  return typeof globalThis !== 'undefined' && 'Notification' in globalThis;
}

function nav(): Navigator | undefined {
  return typeof navigator !== 'undefined' ? navigator : undefined;
}

function localStore(): Storage | undefined {
  try {
    return globalThis.localStorage;
  } catch {
    return undefined;
  }
}

export function createBrowserPlatform(): Platform {
  // Notification-action listeners. In a browser we have no OS action buttons, but
  // the SPA may still dispatch synthetic actions (e.g. an in-app notification
  // center). e6 wires the real bridge; e0 keeps a working registry.
  const actionListeners = new Set<(e: NotificationActionEvent) => void>();

  return {
    platform(): PlatformInfo {
      const os = nav()?.platform ?? 'web';
      return { kind: 'web', os, version: '' };
    },

    // ── Server config: a browser is single, same-origin. ──
    async getServerUrl() {
      return null; // same-origin; the transport uses base ''.
    },
    async setServerUrl() {
      /* no-op: the browser cannot retarget its origin. */
    },
    async listServers(): Promise<ServerEntry[]> {
      return [{ url: '', label: 'This server' }];
    },
    async selectServer() {
      /* no-op in the browser. */
    },

    // ── Auth token store: browser uses the HttpOnly cookie, never a bearer. ──
    async getSessionToken() {
      return null;
    },
    async setSessionToken() {
      /* no-op: cookie path. */
    },
    async clearSessionToken() {
      /* no-op: cookie path. */
    },

    // ── Secure store: localStorage fallback (e6 upgrades to the OPFS vault). ──
    async secureGet(key: string) {
      return localStore()?.getItem(SECURE_PREFIX + key) ?? null;
    },
    async secureSet(key: string, value: string) {
      localStore()?.setItem(SECURE_PREFIX + key, value);
    },
    async secureDelete(key: string) {
      localStore()?.removeItem(SECURE_PREFIX + key);
    },

    // ── Notifications: Notification API where permitted; degrade silently. ──
    async notify(input: NotifyInput) {
      if (!hasNotification()) return;
      try {
        const Notif = globalThis.Notification;
        if (Notif.permission === 'granted') {
          new Notif(input.title, { body: input.body, tag: input.id });
        } else if (Notif.permission !== 'denied') {
          const perm = await Notif.requestPermission();
          if (perm === 'granted') {
            new Notif(input.title, { body: input.body, tag: input.id });
          }
        }
      } catch {
        /* Notifications unavailable (e.g. insecure context): degrade to nothing. */
      }
    },
    onNotificationAction(cb: (e: NotificationActionEvent) => void): Unsubscribe {
      actionListeners.add(cb);
      return () => actionListeners.delete(cb);
    },
    async setBadgeCount(n: number) {
      const navigator_ = nav() as
        | (Navigator & {
            setAppBadge?: (n?: number) => Promise<void>;
            clearAppBadge?: () => Promise<void>;
          })
        | undefined;
      try {
        if (n > 0) await navigator_?.setAppBadge?.(n);
        else await navigator_?.clearAppBadge?.();
      } catch {
        /* Badging API unsupported: e6 falls back to a tab-title/favicon badge. */
      }
    },

    // ── Deep links / mailto: no OS deep links in a browser. ──
    onOpenUrl(): Unsubscribe {
      return () => {};
    },
    async registerMailtoHandler() {
      const navigator_ = nav() as
        | (Navigator & {
            registerProtocolHandler?: (scheme: string, url: string) => void;
          })
        | undefined;
      try {
        navigator_?.registerProtocolHandler?.('mailto', `${location.origin}/?mailto=%s`);
      } catch {
        /* Not permitted in this context: no-op. */
      }
    },

    // ── Capture protection: a browser cannot prevent capture (§7.6 honesty). ──
    async setCaptureProtection(): Promise<CapabilityResult> {
      return { supported: false }; // caller keeps the V4 watermark.
    },

    // ── Biometric: none in the browser default (e6 may probe WebAuthn). ──
    async biometricAvailable() {
      return false;
    },
    async biometricAuthenticate() {
      return false;
    },

    // ── Share / drag-out: handled by the components' Web Share / HTML5 drag. ──
    onShareTarget(): Unsubscribe {
      return () => {};
    },
    async startDragOut(_files: DragOutFile[]) {
      /* The browser drag is initiated by the DOM element, not this layer. */
    },

    // ── Push: e6 wires Web Push (VAPID) subscribe; e0 reports availability. ──
    async pushSubscribe(): Promise<PushSubscriptionInfo | null> {
      return null; // e6 subscribes via the server VAPID key.
    },
    async pushUnsubscribe() {
      /* no-op until e6 wires the subscription. */
    },
    getPushTransport(): PushTransport | null {
      return typeof globalThis !== 'undefined' && 'PushManager' in globalThis
        ? 'webpush'
        : null;
    },

    // ── Self-contained: desktop-only; always "off" in the browser. ──
    async selfContainedStatus(): Promise<SelfContainedStatus> {
      return 'off';
    },
    async startLocalServer() {
      /* no-op in the browser. */
    },
    async stopLocalServer() {
      /* no-op in the browser. */
    },
  };
}
