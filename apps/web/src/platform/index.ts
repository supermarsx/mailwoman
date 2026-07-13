// Platform capability layer (SPEC §16 / plan §2.1, §3 e0/e6) — the ONE additive
// change inside `apps/web`, and the frozen JS↔shell boundary for V5's thin
// desktop/mobile shells.
//
// The SPA is web-first: it must build, typecheck, test, and RUN UNCHANGED in a
// plain browser. Native OS integration (notifications with actions, OS keychain,
// screen-capture protection, biometric app-lock, deep links, share/drag-out, push,
// multi-server, self-contained local server) is reached ONLY through this
// interface, never by calling a Tauri API directly. Two implementations back it:
//   * `./browser.ts`  — web fallbacks (Notification API, Web Push, OPFS vault,
//                        watermark). Created SYNCHRONOUSLY; the default everywhere.
//   * `./tauri.ts`    — native impl; DYNAMICALLY imported only under `isTauri()`
//                        so `@tauri-apps/*` never enter the browser bundle.
//
// Every optional-capability method returns a `{ supported }`-aware result (or a
// nullable) so callers degrade gracefully in browser mode. This module is frozen
// at plan approval (§2); changes require a coordinator re-broadcast.

// browser.ts is a plain web module (no Tauri deps), so a static import keeps the
// default path fully synchronous and tree-shakeable. Only `./tauri.ts` is
// dynamically imported (in `initPlatform`) so `@tauri-apps/*` stay out of the
// browser bundle.
import { createBrowserPlatform } from './browser.ts';

export type PlatformKind = 'web' | 'desktop' | 'android' | 'ios';

export interface PlatformInfo {
  kind: PlatformKind;
  os: string;
  version: string;
}

/** A configured Mailwoman server (multi-server: work + personal). */
export interface ServerEntry {
  url: string;
  label: string;
}

/** One actionable button on a native notification (archive/delete/reply). */
export interface NotifyAction {
  id: string;
  label: string;
}

export interface NotifyInput {
  id: string;
  title: string;
  body: string;
  /** Optional action buttons; browsers without action support degrade to none. */
  actions?: NotifyAction[];
  /** Optional thread the notification refers to (deep-link back into the SPA). */
  threadId?: string;
}

/** Delivered when the user taps a notification action (native) or the body. */
export interface NotificationActionEvent {
  notificationId: string;
  actionId: string;
}

/** Result of a capability that may be unavailable on the current OS/browser. */
export interface CapabilityResult {
  supported: boolean;
}

export type PushTransport = 'webpush' | 'unifiedpush' | 'apns';

/**
 * The frozen push subscription shape (§2.3) exchanged with `/api/push/subscribe`.
 * `keys` is present for Web Push only; `appId` for UnifiedPush/APNs. Content NEVER
 * transits push — a subscription only lets the server send an opaque wake signal.
 */
export interface PushSubscriptionInfo {
  transport: PushTransport;
  endpoint: string;
  keys: { p256dh: string; auth: string } | null;
  appId: string | null;
  /** ISO-8601 UTC, or null when the transport does not expire subscriptions. */
  expiresAt: string | null;
}

export type SelfContainedStatus = 'off' | 'starting' | 'ready' | 'error';

/** A payload handed in by the OS share sheet / file handler (loose by design). */
export interface ShareTargetPayload {
  title?: string;
  text?: string;
  url?: string;
  files?: { name: string; mime: string; bytes?: Uint8Array; blobId?: string }[];
}

/** A file the user drags OUT of the app (attachment → desktop/other app). */
export interface DragOutFile {
  name: string;
  mime: string;
  bytes?: Uint8Array;
  blobId?: string;
}

/** Unsubscribe/cleanup handle returned by the event-registration methods. */
export type Unsubscribe = () => void;

/**
 * The capability interface every consumer talks to. All methods are async (or
 * return a disposer). Native impls fulfil them via Tauri plugins/IPC; the browser
 * impl degrades each one honestly.
 */
export interface Platform {
  platform(): PlatformInfo;

  // ── Server config (multi-server: work + personal) ──
  getServerUrl(): Promise<string | null>;
  setServerUrl(url: string): Promise<void>;
  listServers(): Promise<ServerEntry[]>;
  selectServer(url: string): Promise<void>;

  // ── Auth token store (native: OS keychain; browser: no-op, cookie path) ──
  getSessionToken(): Promise<string | null>;
  setSessionToken(token: string): Promise<void>;
  clearSessionToken(): Promise<void>;

  // ── Secure store (key-vault wrap; native: keychain; browser: OPFS-AES-GCM) ──
  secureGet(key: string): Promise<string | null>;
  secureSet(key: string, value: string): Promise<void>;
  secureDelete(key: string): Promise<void>;

  // ── Notifications ──
  notify(input: NotifyInput): Promise<void>;
  onNotificationAction(cb: (event: NotificationActionEvent) => void): Unsubscribe;
  setBadgeCount(n: number): Promise<void>;

  // ── Deep links / mailto: ──
  onOpenUrl(cb: (url: string) => void): Unsubscribe;
  registerMailtoHandler(): Promise<void>;

  // ── Screen-capture protection (§7.6) ──
  setCaptureProtection(enabled: boolean): Promise<CapabilityResult>;

  // ── Biometric app-lock ──
  biometricAvailable(): Promise<boolean>;
  biometricAuthenticate(input: { reason: string }): Promise<boolean>;

  // ── Share / drag-out ──
  onShareTarget(cb: (payload: ShareTargetPayload) => void): Unsubscribe;
  startDragOut(files: DragOutFile[]): Promise<void>;

  // ── Push ──
  pushSubscribe(): Promise<PushSubscriptionInfo | null>;
  pushUnsubscribe(): Promise<void>;
  getPushTransport(): PushTransport | null;

  // ── Self-contained lifecycle (desktop only; browser/mobile: "off") ──
  selfContainedStatus(): Promise<SelfContainedStatus>;
  startLocalServer(): Promise<void>;
  stopLocalServer(): Promise<void>;
}

/**
 * Feature-detect the Tauri runtime. True only inside a shell webview; false in
 * every plain browser (so the browser path is never perturbed). Detects the v2
 * internals global rather than importing anything.
 */
export function isTauri(): boolean {
  return (
    typeof globalThis !== 'undefined' &&
    typeof (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined'
  );
}

// The active platform singleton. Defaults to the browser impl (synchronous, safe
// everywhere). `initPlatform()` swaps in the native impl at boot when running in a
// shell. Consumers read `getPlatform()` synchronously and always get a valid impl.
let current: Platform | null = null;

/** The active platform (browser impl by default; native after `initPlatform`). */
export function getPlatform(): Platform {
  if (current === null) {
    // Lazily create the browser default. Imported eagerly here is fine — it pulls
    // NO Tauri code; only `tauri.ts` is dynamically imported (below).
    current = createBrowserDefault();
  }
  return current;
}

/**
 * Resolve and install the correct platform for the current runtime. Idempotent.
 * In a browser this is a no-op returning the browser impl. In a Tauri shell it
 * DYNAMICALLY imports `./tauri.ts` (so `@tauri-apps/*` stay out of the browser
 * bundle) and installs the native impl. e7 calls this once during app boot.
 */
export async function initPlatform(): Promise<Platform> {
  if (isTauri()) {
    const mod = await import('./tauri.ts');
    current = await mod.createTauriPlatform();
  } else {
    current = createBrowserDefault();
  }
  return current;
}

/** Install a specific platform impl (tests inject a fake; e7 may pre-resolve). */
export function setPlatform(platform: Platform): void {
  current = platform;
}

function createBrowserDefault(): Platform {
  return createBrowserPlatform();
}
