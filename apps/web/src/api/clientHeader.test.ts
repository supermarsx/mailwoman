// t32-f13 (e82): every client names itself in `X-Mailwoman-Client` — a browser
// with `browser/<version>` on sign-in only, a shell with `desktop|mobile/<version>;
// updater=on|off` on sign-in and on every request to its server.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { clientHeader, createConfiguredClient } from './transport.ts';
import { apiFetch } from './http.ts';
import { setPlatform, type Platform } from '../platform/index.ts';
import { createBrowserPlatform } from '../platform/browser.ts';
import pkg from '../../package.json' with { type: 'json' };

interface G {
  __TAURI_INTERNALS__?: unknown;
  __MW_CONFIG__?: unknown;
  __MW_BASE__?: unknown;
}
const g = globalThis as unknown as G;

afterEach(() => {
  delete g.__TAURI_INTERNALS__;
  delete g.__MW_CONFIG__;
  delete g.__MW_BASE__;
  setPlatform(createBrowserPlatform());
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function stubFetch(body: unknown = { accountId: 'a', username: 'u' }): ReturnType<typeof vi.fn> {
  const fetchMock = vi.fn(async () => new Response(JSON.stringify(body), { status: 200 }));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

/** The `X-Mailwoman-Client` value of call `n`, or null. */
function sent(fetchMock: ReturnType<typeof vi.fn>, n = 0): string | null {
  const init = fetchMock.mock.calls[n]?.[1] as RequestInit | undefined;
  return new Headers(init?.headers).get('x-mailwoman-client');
}

function shell(platform: Record<string, unknown>): void {
  g.__TAURI_INTERNALS__ = {};
  g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org', native: true, platform };
  setPlatform({ ...createBrowserPlatform(), getSessionToken: async () => 'tok' } as Platform);
}

describe('X-Mailwoman-Client in a browser', () => {
  it('is browser/<the web package version>, on sign-in only', () => {
    expect(clientHeader()).toEqual({ value: `browser/${pkg.version}`, everyRequest: false });
  });

  it('goes with the sign-in and the second-factor step', async () => {
    const fetchMock = stubFetch();
    await createConfiguredClient().login({ username: 'u', password: 'p' });
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/api/login');
    expect(sent(fetchMock)).toBe(`browser/${pkg.version}`);

    await apiFetch('/api/login/2fa', { method: 'POST', body: '{}' });
    expect(sent(fetchMock, 1)).toBe(`browser/${pkg.version}`);
  });

  it('goes with a sign-in under a deploy prefix', async () => {
    g.__MW_BASE__ = '/mail';
    const fetchMock = stubFetch();
    await createConfiguredClient().login({ username: 'u', password: 'p' });
    expect(fetchMock.mock.calls[0]?.[0]).toBe('/mail/api/login');
    expect(sent(fetchMock)).toBe(`browser/${pkg.version}`);
  });

  it('leaves every other request as the caller built it', async () => {
    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    expect((fetchMock.mock.calls[0]?.[1] as RequestInit).headers).toBeUndefined();

    await apiFetch('/api/me');
    expect(fetchMock.mock.calls[1]).toEqual(['/api/me']); // no init stays no init

    await apiFetch('/api/x', { method: 'PUT', headers: { 'content-type': 'application/json' } });
    expect(sent(fetchMock, 2)).toBeNull();
    // A path that only starts like a sign-in route is not one.
    await apiFetch('/api/loginx', { method: 'POST' });
    expect(sent(fetchMock, 3)).toBeNull();
  });

  it('keeps the caller\'s headers, as a plain object or a Headers', async () => {
    const fetchMock = stubFetch();
    await apiFetch('/api/login', { method: 'POST', headers: { 'content-type': 'application/json' } });
    const plain = (fetchMock.mock.calls[0]?.[1] as RequestInit).headers as Record<string, string>;
    expect(plain['content-type']).toBe('application/json');
    expect(plain['X-Mailwoman-Client']).toBe(`browser/${pkg.version}`);

    await apiFetch('/api/login', { method: 'POST', headers: new Headers({ 'content-type': 'application/json' }) });
    const init = fetchMock.mock.calls[1]?.[1] as RequestInit;
    expect(new Headers(init.headers).get('content-type')).toBe('application/json');
    expect(sent(fetchMock, 1)).toBe(`browser/${pkg.version}`);
  });
});

describe('X-Mailwoman-Client in a shell', () => {
  it('desktop: kind, shell version and updater state, on every request', async () => {
    shell({ kind: 'desktop', os: 'windows', version: '26.20.0', updater: true });
    expect(clientHeader()).toEqual({ value: 'desktop/26.20.0; updater=on', everyRequest: true });

    const fetchMock = stubFetch();
    const client = createConfiguredClient();
    await client.login({ username: 'u', password: 'p' });
    await client.me();
    expect(fetchMock.mock.calls[0]?.[0]).toBe('https://mail.example.org/api/login');
    expect(sent(fetchMock, 0)).toBe('desktop/26.20.0; updater=on');
    expect(fetchMock.mock.calls[1]?.[0]).toBe('https://mail.example.org/api/me');
    expect(sent(fetchMock, 1)).toBe('desktop/26.20.0; updater=on');
    // The bearer header is still there next to it.
    const me = (fetchMock.mock.calls[1]?.[1] as RequestInit).headers as Record<string, string>;
    expect(me.Authorization).toBe('Bearer tok');
  });

  it('desktop without an update feed says updater=off', () => {
    shell({ kind: 'desktop', os: 'linux', version: '26.20.0', updater: false });
    expect(clientHeader().value).toBe('desktop/26.20.0; updater=off');
    shell({ kind: 'desktop', os: 'linux', version: '26.20.0' });
    expect(clientHeader().value).toBe('desktop/26.20.0; updater=off');
  });

  it('android and ios are the mobile shell, never with an updater', () => {
    for (const kind of ['android', 'ios']) {
      shell({ kind, os: kind, version: '26.20.0', updater: true });
      expect(clientHeader()).toEqual({ value: 'mobile/26.20.0; updater=off', everyRequest: true });
    }
  });

  it('a version that is not a release number is sent as unknown', () => {
    shell({ kind: 'desktop', version: '26.20.0\r\nX-Evil: 1', updater: false });
    expect(clientHeader().value).toBe('desktop/unknown; updater=off');
    shell({ kind: 'desktop', version: '', updater: false });
    expect(clientHeader().value).toBe('desktop/unknown; updater=off');
  });

  it('a native flag without a known shell kind counts as a browser', () => {
    shell({ os: 'windows', version: '26.20.0' });
    expect(clientHeader()).toEqual({ value: `browser/${pkg.version}`, everyRequest: false });
  });
});
