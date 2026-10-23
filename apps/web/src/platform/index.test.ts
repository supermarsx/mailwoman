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

  it('has no server configuration to change (t32)', () => {
    const p = getPlatform() as unknown as Record<string, unknown>;
    for (const name of ['getServerUrl', 'setServerUrl', 'listServers', 'selectServer']) {
      expect(p[name]).toBeUndefined();
    }
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
    // Tauri runtime (unit env) there is no `__TAURI_INTERNALS__` and the call
    // rejects — proving the native path calls the shell rather than the browser
    // fallback.
    await expect(p.setCaptureProtection(true)).rejects.toThrow();
  });
});

// t31: the native platform reaches the shell through the injected
// `__TAURI_INTERNALS__` (no `@tauri-apps` import), puts the stored server address
// where the transport reads it, and holds the keychain token.
describe('native (tauri) platform over __TAURI_INTERNALS__', () => {
  interface Call {
    cmd: string;
    args?: Record<string, unknown>;
  }
  function fakeShell(answers: Record<string, unknown>): { calls: Call[]; callbacks: ((e: unknown) => void)[] } {
    const calls: Call[] = [];
    const callbacks: ((e: unknown) => void)[] = [];
    (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args?: Record<string, unknown>) => {
        calls.push(args === undefined ? { cmd } : { cmd, args });
        if (cmd in answers) return answers[cmd];
        return null;
      },
      transformCallback: (cb: (e: unknown) => void) => {
        callbacks.push(cb);
        return callbacks.length;
      },
    };
    return { calls, callbacks };
  }

  it('leaves the injected server address as it is and asks the shell for none (t32)', async () => {
    const shell = fakeShell({ mw_server_get_selected: 'https://typed.example.org' });
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = { serverUrl: null, native: true };
    const p = (await createTauriPlatform()) as unknown as Record<string, unknown>;
    expect((globalThis as { __MW_CONFIG__?: { serverUrl?: unknown } }).__MW_CONFIG__?.serverUrl).toBeNull();
    expect(shell.calls.filter((c) => c.cmd.startsWith('mw_server_'))).toEqual([]);
    for (const name of ['getServerUrl', 'setServerUrl', 'listServers', 'selectServer']) {
      expect(p[name]).toBeUndefined();
    }
  });

  it('reads the keychain token once, and set/clear go to the keychain', async () => {
    const shell = fakeShell({ mw_keychain_get: 'tok-k' });
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = { serverUrl: 'https://m.example', native: true };
    const p = await createTauriPlatform();
    expect(await p.getSessionToken()).toBe('tok-k');
    expect(await p.getSessionToken()).toBe('tok-k');
    expect(shell.calls.filter((c) => c.cmd === 'mw_keychain_get')).toHaveLength(1);
    await p.setSessionToken('tok-2');
    expect(await p.getSessionToken()).toBe('tok-2');
    expect(shell.calls).toContainEqual({
      cmd: 'mw_keychain_set',
      args: { service: 'mailwoman.session', key: 'token', value: 'tok-2' },
    });
    await p.clearSessionToken();
    expect(await p.getSessionToken()).toBeNull();
    expect(shell.calls).toContainEqual({ cmd: 'mw_keychain_delete', args: { service: 'mailwoman.session', key: 'token' } });
  });

  it('bridges a shell event through the event plugin and stops on dispose', async () => {
    const shell = fakeShell({ 'plugin:event|listen': 7 });
    (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ = { serverUrl: 'https://m.example', native: true };
    const p = await createTauriPlatform();
    const seen: string[] = [];
    const off = p.onOpenUrl((url) => seen.push(url));
    await new Promise((r) => setTimeout(r, 0));
    const listen = shell.calls.find((c) => c.cmd === 'plugin:event|listen');
    expect(listen?.args).toMatchObject({ event: 'mw://open-url', target: { kind: 'Any' } });
    shell.callbacks[0]?.({ event: 'mw://open-url', id: 7, payload: 'mailto:a@example.org' });
    expect(seen).toEqual(['mailto:a@example.org']);
    off();
    await new Promise((r) => setTimeout(r, 0));
    expect(shell.calls).toContainEqual({ cmd: 'plugin:event|unlisten', args: { event: 'mw://open-url', eventId: 7 } });
  });
});
