import { afterEach, describe, expect, it, vi } from 'vitest';
import { createConfiguredClient, transportBase, isNativeAuth } from './transport.ts';
import { setPlatform, type Platform } from '../platform/index.ts';
import { createBrowserPlatform } from '../platform/browser.ts';

interface G {
  __TAURI_INTERNALS__?: unknown;
  __MW_CONFIG__?: unknown;
}
const g = globalThis as unknown as G;

afterEach(() => {
  delete g.__TAURI_INTERNALS__;
  delete g.__MW_CONFIG__;
  setPlatform(createBrowserPlatform());
  vi.restoreAllMocks();
});

/** Capture the single fetch call a client makes. */
function stubFetch(): ReturnType<typeof vi.fn> {
  const fetchMock = vi.fn(async () => ({
    ok: true,
    status: 200,
    json: async () => ({ username: 'u', accountId: 'a' }),
  }));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

describe('browser transport (the regression-critical path)', () => {
  it('base is empty and no native auth', () => {
    expect(transportBase()).toBe('');
    expect(isNativeAuth()).toBe(false);
  });

  it('createConfiguredClient hits same-origin with the cookie, no bearer', async () => {
    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('/api/me');
    expect(init.credentials).toBe('same-origin');
    expect(init.headers).toBeUndefined(); // byte-identical to pre-V5.
  });
});

describe('native transport (opt-in shell path)', () => {
  it('resolves the injected base + native flag', () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org/', native: true };
    expect(transportBase()).toBe('https://mail.example.org'); // trailing slash trimmed.
    expect(isNativeAuth()).toBe(true);
  });

  it('attaches the keychain bearer token and omits cookies', async () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org', native: true };
    const fake = { ...createBrowserPlatform(), getSessionToken: async () => 'TOKEN123' } as Platform;
    setPlatform(fake);

    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('https://mail.example.org/api/me');
    expect(init.credentials).toBe('omit');
    expect((init.headers as Record<string, string>).Authorization).toBe('Bearer TOKEN123');
  });

  it('falls back to the cookie path when the token store is empty', async () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org', native: true };
    const fake = { ...createBrowserPlatform(), getSessionToken: async () => null } as Platform;
    setPlatform(fake);

    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(init.credentials).toBe('same-origin'); // no token → no bearer, keep cookie.
  });
});
