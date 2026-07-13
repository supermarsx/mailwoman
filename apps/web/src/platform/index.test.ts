import { afterEach, describe, expect, it } from 'vitest';
import { getPlatform, isTauri, setPlatform, type Platform } from './index.ts';
import { createBrowserPlatform } from './browser.ts';
import { createTauriPlatform } from './tauri.ts';

afterEach(() => {
  // Reset the injected singleton + any test globals between cases.
  setPlatform(createBrowserPlatform());
  delete (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  delete (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__;
  globalThis.localStorage?.clear();
});

describe('isTauri()', () => {
  it('is false in a plain browser (no shell internals global)', () => {
    expect(isTauri()).toBe(false);
  });

  it('is true when the Tauri v2 internals global is present', () => {
    (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    expect(isTauri()).toBe(true);
  });
});

describe('browser platform (the default, unchanged web path)', () => {
  it('reports the web platform kind', () => {
    expect(getPlatform().platform().kind).toBe('web');
  });

  it('is a single same-origin server', async () => {
    const p = getPlatform();
    expect(await p.getServerUrl()).toBeNull();
    expect(await p.listServers()).toEqual([{ url: '', label: 'This server' }]);
  });

  it('never issues a bearer token (cookie path)', async () => {
    expect(await getPlatform().getSessionToken()).toBeNull();
  });

  it('round-trips the secure-store fallback', async () => {
    const p = getPlatform();
    await p.secureSet('k', 'v');
    expect(await p.secureGet('k')).toBe('v');
    await p.secureDelete('k');
    expect(await p.secureGet('k')).toBeNull();
  });

  it('reports capture protection as unsupported (→ caller keeps the watermark)', async () => {
    expect(await getPlatform().setCaptureProtection(true)).toEqual({ supported: false });
  });

  it('has no biometric and self-contained is off', async () => {
    const p = getPlatform();
    expect(await p.biometricAvailable()).toBe(false);
    expect(await p.selfContainedStatus()).toBe('off');
  });

  it('registers/unsubscribes notification-action listeners without throwing', () => {
    const off = getPlatform().onNotificationAction(() => {});
    expect(typeof off).toBe('function');
    off();
  });
});

describe('setPlatform() injects a fake (test seam)', () => {
  it('replaces the active platform', () => {
    const fake = { ...createBrowserPlatform(), platform: () => ({ kind: 'desktop', os: 'x', version: '1' }) } as Platform;
    setPlatform(fake);
    expect(getPlatform().platform().kind).toBe('desktop');
  });
});

describe('native (tauri) platform', () => {
  it('reports the shell-injected platform descriptor', async () => {
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = {
      platform: { kind: 'desktop', os: 'windows', version: '26.6.0' },
      native: true,
    };
    const p = await createTauriPlatform();
    expect(p.platform()).toEqual({ kind: 'desktop', os: 'windows', version: '26.6.0' });
  });

  it('derives the push transport from the platform kind (no runtime needed)', async () => {
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = {
      platform: { kind: 'android', os: 'android', version: '26.6.0' },
    };
    const android = await createTauriPlatform();
    expect(android.getPushTransport()).toBe('unifiedpush');

    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = {
      platform: { kind: 'desktop', os: 'windows', version: '26.6.0' },
    };
    const desktop = await createTauriPlatform();
    expect(desktop.getPushTransport()).toBe('webpush');
  });

  it('routes a native capability through a Tauri IPC command (absent in this env)', async () => {
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = {
      platform: { kind: 'desktop', os: 'windows', version: '26.6.0' },
    };
    const p = await createTauriPlatform();
    // The real impl invokes `mw_set_capture_protection` over Tauri IPC; with no
    // Tauri runtime (unit env) the dynamic `@tauri-apps` import rejects — proving
    // the native path calls the shell rather than the browser fallback.
    await expect(p.setCaptureProtection(true)).rejects.toThrow();
  });
});
