import { afterEach, describe, expect, it, vi } from 'vitest';
import { createConfiguredClient, isNativeAuth, nativeServerBase, transportBase } from './transport.ts';
import { basePath, normalizeBase, shellBase, stripBase, withBase } from './basePath.ts';
import { setPlatform, type Platform } from '../platform/index.ts';
import { createBrowserPlatform } from '../platform/browser.ts';

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

  it('sends no cookie and no bearer before sign-in', async () => {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org', native: true };
    const fake = { ...createBrowserPlatform(), getSessionToken: async () => null } as Platform;
    setPlatform(fake);

    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    // The server is another origin: its cookies are not the shell's to send.
    expect(init.credentials).toBe('omit');
    expect(init.headers).toBeUndefined();
  });
});

// t31: the native sign-in, end to end through the client — the server address
// the shell was built or provisioned for (t32), `clientType: "native"`, the token
// kept through the platform, and the token on every later request.
describe('native sign-in (t31)', () => {
  /** A platform fake that records what the client asks of its token store. */
  function recordingPlatform(): { platform: Platform; calls: string[]; token: () => string | null } {
    let token: string | null = null;
    const calls: string[] = [];
    const platform = {
      ...createBrowserPlatform(),
      getSessionToken: async () => token,
      setSessionToken: async (t: string) => {
        calls.push(`set:${t}`);
        token = t;
      },
      clearSessionToken: async () => {
        calls.push('clear');
        token = null;
      },
    } as Platform;
    return { platform, calls, token: () => token };
  }

  function nativeShell(serverUrl: string | null): void {
    g.__TAURI_INTERNALS__ = {};
    g.__MW_CONFIG__ = { serverUrl, native: true, platform: { kind: 'desktop', os: 'windows', version: '' } };
  }

  it('signs in with clientType native, keeps the token and sends it afterwards', async () => {
    nativeShell('https://mail.example.org/');
    const rec = recordingPlatform();
    setPlatform(rec.platform);
    const fetchMock = vi.fn(async (input: string) => {
      if (input.endsWith('/api/login')) {
        return new Response(JSON.stringify({ ok: true, accountId: 'a', username: 'u', token: 'tok-1' }), { status: 200 });
      }
      return new Response(JSON.stringify({ username: 'u', accountId: 'a' }), { status: 200 });
    });
    vi.stubGlobal('fetch', fetchMock);

    const client = createConfiguredClient();

    const me = await client.login({ username: 'u', password: 'p' });
    expect(me).toEqual({ ok: true, accountId: 'a', username: 'u' }); // the token is not part of `Me`.
    const [loginUrl, loginInit] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(loginUrl).toBe('https://mail.example.org/api/login');
    expect(JSON.parse(loginInit.body as string)).toEqual({
      username: 'u',
      password: 'p',
      clientType: 'native',
    });
    expect(loginInit.credentials).toBe('omit');
    expect(rec.token()).toBe('tok-1');

    await client.me();
    const [meUrl, meInit] = fetchMock.mock.calls[1] as unknown as [string, RequestInit];
    expect(meUrl).toBe('https://mail.example.org/api/me');
    expect((meInit.headers as Record<string, string>).Authorization).toBe('Bearer tok-1');

    await client.logout();
    expect(rec.token()).toBeNull();
    expect(rec.calls.at(-1)).toBe('clear');
  });

  it('a browser sign-in never sends clientType', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ accountId: 'a', username: 'u' }), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    await createConfiguredClient().login({ username: 'u', password: 'p' });
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(JSON.parse(init.body as string)).not.toHaveProperty('clientType');
    expect(init.credentials).toBe('same-origin');
  });

  // t32-h9: the server refuses a sign-in that names a mail server, so the client
  // sends the two fields and nothing a caller adds.
  it('a sign-in sends only the address and the password', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ accountId: 'a', username: 'u' }), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    const input = { username: 'u', password: 'p', jmapUrl: 'https://jmap.example.org' };
    await createConfiguredClient().login(input);
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(JSON.parse(init.body as string)).toEqual({ username: 'u', password: 'p' });
  });

  it('with no configured server address, boot reads as signed out and sends nothing', async () => {
    nativeShell(null);
    setPlatform(recordingPlatform().platform);
    const fetchMock = stubFetch();
    expect(nativeServerBase()).toBeNull();
    await expect(createConfiguredClient().me()).rejects.toMatchObject({ status: 401 });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});

// ── Sub-path hosting (t20 B4) ───────────────────────────────────────────────
// `basePath.ts` has no test file of its own: it is a new module whose colocated
// test path is not in this lane's locks, so its unit tests live here beside its
// primary consumer.

describe('normalizeBase', () => {
  it('canonicalizes every shape an operator might configure', () => {
    expect(normalizeBase('/mail')).toBe('/mail');
    expect(normalizeBase('mail')).toBe('/mail'); // missing leading slash
    expect(normalizeBase('/mail/')).toBe('/mail'); // trailing slash
    expect(normalizeBase('  /mail/  ')).toBe('/mail'); // whitespace
    expect(normalizeBase('/a/b/')).toBe('/a/b'); // nested prefix
  });

  it('treats the origin root and every non-value as no prefix', () => {
    for (const raw of ['', '/', '   ', undefined, null, 42, {}]) {
      expect(normalizeBase(raw)).toBe('');
    }
  });
});

describe('basePath / withBase / shellBase', () => {
  it('is absent by default — every path byte-identical to pre-sub-path', () => {
    expect(basePath()).toBe('');
    expect(withBase('/api/me')).toBe('/api/me');
    expect(shellBase()).toBe('/');
  });

  it('reads the server-injected __MW_BASE__ and prefixes', () => {
    g.__MW_BASE__ = '/mail/';
    expect(basePath()).toBe('/mail');
    expect(withBase('/api/me')).toBe('/mail/api/me');
    expect(shellBase()).toBe('/mail/');
  });
});

describe('stripBase', () => {
  it('round-trips withBase', () => {
    expect(stripBase('/mail/api/me', '/mail')).toBe('/api/me');
    expect(stripBase('/mail/', '/mail')).toBe('/');
    expect(stripBase('/mail', '/mail')).toBe('/');
  });

  it('strips only on a segment boundary', () => {
    // The bug a naive `slice` would introduce: /mailbox is not /mail + box.
    expect(stripBase('/mailbox/api', '/mail')).toBe('/mailbox/api');
  });

  it('is the identity with no prefix, or for a foreign path', () => {
    expect(stripBase('/api/me', '')).toBe('/api/me');
    expect(stripBase('/other/api', '/mail')).toBe('/other/api');
  });
});

describe('transport under a sub-path', () => {
  it('the browser base becomes the prefix, so every client path carries it', async () => {
    g.__MW_BASE__ = '/mail';
    expect(transportBase()).toBe('/mail');
    const fetchMock = stubFetch();
    await createConfiguredClient().me();
    const [url] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('/mail/api/me');
  });

  it('a configured native serverUrl still wins verbatim', () => {
    // `__MW_BASE__` is an index.html injection; the native shell is pointed at a
    // full URL that already carries any prefix, so it must not be double-applied.
    g.__MW_BASE__ = '/mail';
    g.__MW_CONFIG__ = { serverUrl: 'https://mail.example.org/mail/', native: true };
    expect(transportBase()).toBe('https://mail.example.org/mail');
  });
});
