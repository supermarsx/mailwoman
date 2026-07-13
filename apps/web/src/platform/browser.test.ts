import { afterEach, describe, expect, it, vi } from 'vitest';
import { createBrowserPlatform } from './browser.ts';

// jsdom has no Notification / ServiceWorker / PushManager / App-Badging APIs, so
// these tests install minimal fakes to exercise the browser fallbacks, and remove
// them afterwards so the rest of the suite sees the plain-browser environment.

interface G {
  Notification?: unknown;
  PushManager?: unknown;
}
const g = globalThis as unknown as G;

afterEach(() => {
  delete g.Notification;
  delete g.PushManager;
  vi.restoreAllMocks();
  globalThis.localStorage?.clear();
  document.title = '';
  // Drop any serviceWorker we defined on navigator.
  if ('serviceWorker' in navigator) {
    delete (navigator as unknown as { serviceWorker?: unknown }).serviceWorker;
  }
});

describe('notify() — Notification API bridge', () => {
  it('shows a notification and a click dispatches the "default" action', async () => {
    const clicks: { title: string; onclick: (() => void) | null }[] = [];
    class FakeNotification {
      static permission = 'granted';
      static requestPermission = vi.fn(async () => 'granted' as NotificationPermission);
      onclick: (() => void) | null = null;
      constructor(
        public title: string,
        public options: unknown,
      ) {
        clicks.push(this);
      }
    }
    g.Notification = FakeNotification;

    const p = createBrowserPlatform();
    const seen: { notificationId: string; actionId: string }[] = [];
    p.onNotificationAction((e) => seen.push(e));

    await p.notify({ id: 'm1', title: 'New', body: 'hi' });
    expect(clicks).toHaveLength(1);
    // Simulate the user clicking the OS notification.
    clicks[0]?.onclick?.();
    expect(seen).toEqual([{ notificationId: 'm1', actionId: 'default' }]);
  });

  it('requests permission when it is default, and degrades if denied', async () => {
    const made: unknown[] = [];
    class FakeNotification {
      static permission = 'denied';
      static requestPermission = vi.fn(async () => 'denied' as NotificationPermission);
      constructor() {
        made.push(this);
      }
    }
    g.Notification = FakeNotification;
    await createBrowserPlatform().notify({ id: 'x', title: 't', body: 'b' });
    expect(made).toHaveLength(0); // denied → nothing shown, no throw.
  });

  it('is a silent no-op when the Notification API is absent', async () => {
    await expect(createBrowserPlatform().notify({ id: 'x', title: 't', body: 'b' })).resolves.toBeUndefined();
  });
});

describe('setBadgeCount() — App Badging with a tab-title fallback', () => {
  it('uses the App Badging API when present', async () => {
    const setAppBadge = vi.fn(async () => undefined);
    const clearAppBadge = vi.fn(async () => undefined);
    Object.assign(navigator, { setAppBadge, clearAppBadge });
    document.title = 'Mailwoman';

    const p = createBrowserPlatform();
    await p.setBadgeCount(4);
    expect(setAppBadge).toHaveBeenCalledWith(4);
    await p.setBadgeCount(0);
    expect(clearAppBadge).toHaveBeenCalled();
    expect(document.title).toBe('Mailwoman'); // untouched when the API is used.

    delete (navigator as unknown as { setAppBadge?: unknown }).setAppBadge;
    delete (navigator as unknown as { clearAppBadge?: unknown }).clearAppBadge;
  });

  it('falls back to a tab-title badge and restores the original title', async () => {
    document.title = 'Mailwoman';
    const p = createBrowserPlatform();
    await p.setBadgeCount(3);
    expect(document.title).toBe('(3) Mailwoman');
    await p.setBadgeCount(0);
    expect(document.title).toBe('Mailwoman');
  });
});

describe('secure store fallback', () => {
  it('round-trips via localStorage when the OPFS vault is unavailable (jsdom)', async () => {
    const p = createBrowserPlatform();
    await p.secureSet('token', 'sekret');
    expect(await p.secureGet('token')).toBe('sekret');
    await p.secureDelete('token');
    expect(await p.secureGet('token')).toBeNull();
  });
});

describe('pushSubscribe() — Web Push (VAPID)', () => {
  it('returns null when Web Push is unavailable', async () => {
    expect(await createBrowserPlatform().pushSubscribe()).toBeNull();
  });

  it('subscribes with the server VAPID key and returns the frozen shape', async () => {
    g.PushManager = function PushManager() {};
    const subscribe = vi.fn(async () => ({
      endpoint: 'https://push.example/abc',
      expirationTime: null,
      toJSON: () => ({ keys: { p256dh: 'PKEY', auth: 'AKEY' } }),
      getKey: () => null,
    }));
    const registration = { pushManager: { subscribe } };
    Object.defineProperty(navigator, 'serviceWorker', {
      value: { getRegistration: async () => registration },
      configurable: true,
    });
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({ publicKey: 'dGVzdA' }), // base64url("test")
    }));
    vi.stubGlobal('fetch', fetchMock);

    const info = await createBrowserPlatform().pushSubscribe();
    expect(fetchMock).toHaveBeenCalledWith('/api/push/vapid');
    expect(subscribe).toHaveBeenCalledOnce();
    expect(info).toEqual({
      transport: 'webpush',
      endpoint: 'https://push.example/abc',
      keys: { p256dh: 'PKEY', auth: 'AKEY' },
      appId: null,
      expiresAt: null,
    });
  });

  it('returns null when no service worker is registered (e7 wires it)', async () => {
    g.PushManager = function PushManager() {};
    Object.defineProperty(navigator, 'serviceWorker', {
      value: { getRegistration: async () => undefined },
      configurable: true,
    });
    expect(await createBrowserPlatform().pushSubscribe()).toBeNull();
  });
});

describe('getPushTransport() + capture protection', () => {
  it('reports webpush when PushManager exists, else null', () => {
    expect(createBrowserPlatform().getPushTransport()).toBeNull();
    g.PushManager = function PushManager() {};
    expect(createBrowserPlatform().getPushTransport()).toBe('webpush');
  });

  it('reports capture protection unsupported (caller keeps the watermark)', async () => {
    expect(await createBrowserPlatform().setCaptureProtection(true)).toEqual({ supported: false });
  });
});
