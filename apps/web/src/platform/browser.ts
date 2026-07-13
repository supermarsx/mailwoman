// Browser implementation of the `Platform` capability layer (plan §2.1, §3 e6).
//
// This is the DEFAULT everywhere and the honest web fallback: it never touches a
// Tauri API, degrades every optional capability gracefully, and keeps the browser
// code path byte-identical to pre-V5 behaviour. Every richer fallback here is
// guarded so that under jsdom / an insecure context / a browser missing the API
// it falls back to the same inert behaviour the e0 stub had:
//   * Web Push (VAPID) subscribe        → null when no ServiceWorker/PushManager;
//   * OPFS-AES-GCM secure store          → localStorage when no OPFS/IndexedDB;
//   * tab-title badge                    → no-op when there is no document;
//   * notification action bridge         → the click dispatches a 'default' action.

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
import {
  decryptJson,
  encryptJson,
  getOrCreateProfileKey,
  idbAvailable,
  idbKeyStore,
  opfsAvailable,
  opfsBackend,
  type BlobBackend,
} from '../offline/index.ts';

/** Prefix for the localStorage-backed secure-store fallback (below OPFS). */
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

/** The transport base for the VAPID fetch (browser: '' same-origin). Kept local
 *  to avoid a cycle with `api/transport.ts` (which imports this module). */
function serverBase(): string {
  const url = (globalThis as { __MW_CONFIG__?: { serverUrl?: unknown } }).__MW_CONFIG__?.serverUrl;
  if (typeof url !== 'string' || url.length === 0) return '';
  return url.endsWith('/') ? url.slice(0, -1) : url;
}

/** Decode a URL-safe base64 VAPID key to the `applicationServerKey` byte array. */
function urlBase64ToUint8Array(base64: string): Uint8Array {
  const padding = '='.repeat((4 - (base64.length % 4)) % 4);
  const normalized = (base64 + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(normalized);
  const out = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i += 1) out[i] = raw.charCodeAt(i);
  return out;
}

/** Base64-encode an ArrayBuffer (the p256dh/auth push keys → wire strings). */
function bufToBase64(buf: ArrayBuffer | null): string {
  if (buf === null) return '';
  const bytes = new Uint8Array(buf);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

/**
 * The OPFS-AES-GCM secure vault (reuses the V4 device-at-rest crypto). Available
 * only when both OPFS and IndexedDB exist (real browser, secure context); under
 * jsdom / private windows it is absent and callers fall back to localStorage.
 * Each secret is a `[12-byte IV | ciphertext]` blob at `secure/<enc-key>.enc`.
 */
function makeVault(): { get(k: string): Promise<string | null>; set(k: string, v: string): Promise<void>; del(k: string): Promise<void> } | null {
  if (!opfsAvailable() || !idbAvailable()) return null;
  let backend: BlobBackend | null = null;
  let keyP: Promise<CryptoKey> | null = null;
  function ensure(): { backend: BlobBackend; keyP: Promise<CryptoKey> } {
    backend ??= opfsBackend();
    keyP ??= getOrCreateProfileKey(idbKeyStore());
    return { backend, keyP };
  }
  const path = (k: string): string => `secure/${encodeURIComponent(k)}.enc`;
  return {
    async get(k) {
      const { backend, keyP } = ensure();
      const blob = await backend.read(path(k));
      if (blob === null) return null;
      try {
        return await decryptJson<string>(await keyP, blob);
      } catch {
        return null; // corrupt / wrong-key blob reads back as absent.
      }
    },
    async set(k, v) {
      const { backend, keyP } = ensure();
      await backend.write(path(k), await encryptJson(await keyP, v));
    },
    async del(k) {
      const { backend } = ensure();
      await backend.remove(path(k));
    },
  };
}

export function createBrowserPlatform(): Platform {
  // Notification-action listeners. A browser has no OS action buttons, but a
  // clicked notification dispatches a synthetic 'default' action so the deep-link
  // consumer (open the referenced thread) still works.
  const actionListeners = new Set<(e: NotificationActionEvent) => void>();
  function dispatchAction(e: NotificationActionEvent): void {
    for (const cb of actionListeners) cb(e);
  }

  // Lazily built; null when the OPFS vault is unavailable (→ localStorage).
  const vault = makeVault();

  // Tab-title badge state: remember the untouched title to restore on clear.
  let baseTitle: string | null = null;

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

    // ── Secure store: OPFS-AES-GCM vault where available, else localStorage. ──
    async secureGet(key: string) {
      if (vault) return vault.get(key);
      return localStore()?.getItem(SECURE_PREFIX + key) ?? null;
    },
    async secureSet(key: string, value: string) {
      if (vault) return vault.set(key, value);
      localStore()?.setItem(SECURE_PREFIX + key, value);
    },
    async secureDelete(key: string) {
      if (vault) return vault.del(key);
      localStore()?.removeItem(SECURE_PREFIX + key);
    },

    // ── Notifications: Notification API where permitted; degrade silently. ──
    async notify(input: NotifyInput) {
      if (!hasNotification()) return;
      try {
        const Notif = globalThis.Notification;
        const show = (): void => {
          const n = new Notif(input.title, { body: input.body, tag: input.id });
          // No OS action buttons in a plain browser: a click is the 'default'
          // action, carrying the thread back so the consumer can deep-link.
          n.onclick = () => dispatchAction({ notificationId: input.id, actionId: 'default' });
        };
        if (Notif.permission === 'granted') {
          show();
        } else if (Notif.permission !== 'denied') {
          const perm = await Notif.requestPermission();
          if (perm === 'granted') show();
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
      // Preferred: the App Badging API (installed PWA). Falls back to a tab-title
      // count so an ordinary tab still surfaces the unread badge.
      try {
        if (navigator_?.setAppBadge && navigator_?.clearAppBadge) {
          if (n > 0) await navigator_.setAppBadge(n);
          else await navigator_.clearAppBadge();
          return;
        }
      } catch {
        /* Badging API present but refused: fall through to the title badge. */
      }
      if (typeof document === 'undefined') return;
      baseTitle ??= document.title.replace(/^\(\d+\)\s+/, '');
      document.title = n > 0 ? `(${n}) ${baseTitle}` : baseTitle;
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

    // ── Biometric: none in the browser default. ──
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

    // ── Push: Web Push (VAPID) against the server's public key. ──
    async pushSubscribe(): Promise<PushSubscriptionInfo | null> {
      const navigator_ = nav();
      if (
        navigator_ === undefined ||
        !('serviceWorker' in navigator_) ||
        typeof globalThis === 'undefined' ||
        !('PushManager' in globalThis)
      ) {
        return null; // no Web Push here (jsdom / unsupported): degrade to none.
      }
      try {
        const reg = await navigator_.serviceWorker.getRegistration();
        if (reg?.pushManager === undefined) return null; // e7 registers the SW.
        const res = await fetch(`${serverBase()}/api/push/vapid`);
        if (!res.ok) return null;
        const { publicKey } = (await res.json()) as { publicKey: string };
        const sub = await reg.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlBase64ToUint8Array(publicKey) as BufferSource,
        });
        const json = sub.toJSON() as { keys?: { p256dh?: string; auth?: string } };
        return {
          transport: 'webpush',
          endpoint: sub.endpoint,
          keys: {
            p256dh: json.keys?.p256dh ?? bufToBase64(sub.getKey('p256dh')),
            auth: json.keys?.auth ?? bufToBase64(sub.getKey('auth')),
          },
          appId: null,
          expiresAt: sub.expirationTime !== null ? new Date(sub.expirationTime).toISOString() : null,
        };
      } catch {
        return null; // any failure → no subscription (caller stays on foreground poll).
      }
    },
    async pushUnsubscribe() {
      const navigator_ = nav();
      if (navigator_ === undefined || !('serviceWorker' in navigator_)) return;
      try {
        const reg = await navigator_.serviceWorker.getRegistration();
        const sub = await reg?.pushManager?.getSubscription?.();
        await sub?.unsubscribe();
      } catch {
        /* nothing to unsubscribe. */
      }
    },
    getPushTransport(): PushTransport | null {
      return typeof globalThis !== 'undefined' && 'PushManager' in globalThis ? 'webpush' : null;
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
