// `apiFetch` — the CSRF token on writes, and the one retry after a refusal.
//
// The cookie name, header name and refusal text asserted here are the server's
// (`crates/mw-server/src/hardening.rs`, `state_change_guard` in `lib.rs`),
// written out again rather than imported from `http.ts`, so a rename on either
// side fails here. That the two sides agree on a live server is what
// `crates/mw-server/tests/t30_csrf_strict.rs` and the strict e2e stacks show;
// this file covers the client's branches.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { apiFetch, csrfToken, setNativeTransport, type NativeTransport } from './http.ts';

type FetchArgs = [input: string, init?: RequestInit];

function clearCookie(): void {
  document.cookie = 'mw_csrf=; Max-Age=0; Path=/';
}

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

const refusal = (): Response => json(403, { error: 'missing or invalid CSRF token' });

/** The `X-CSRF-Token` a request was sent with, or `null`. */
function sentToken(init: RequestInit | undefined): string | null {
  return new Headers(init?.headers).get('x-csrf-token');
}

describe('apiFetch', () => {
  let fetchMock: ReturnType<typeof vi.fn<(...args: FetchArgs) => Promise<Response>>>;

  beforeEach(() => {
    clearCookie();
    fetchMock = vi.fn<(...args: FetchArgs) => Promise<Response>>();
    vi.stubGlobal('fetch', fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    clearCookie();
    delete (globalThis as { __MW_BASE__?: unknown }).__MW_BASE__;
  });

  it('reads the token from the mw_csrf cookie', () => {
    expect(csrfToken()).toBeNull();
    document.cookie = 'other=1; Path=/';
    document.cookie = 'mw_csrf=tok-1; Path=/';
    expect(csrfToken()).toBe('tok-1');
  });

  it('sends the cookie value as X-CSRF-Token on POST, PUT, PATCH and DELETE', async () => {
    document.cookie = 'mw_csrf=tok-1; Path=/';
    fetchMock.mockImplementation(async () => json(200, {}));
    for (const method of ['POST', 'PUT', 'PATCH', 'DELETE', 'post']) {
      await apiFetch('/api/x', { method, credentials: 'same-origin', headers: { 'content-type': 'application/json' } });
    }
    expect(fetchMock).toHaveBeenCalledTimes(5);
    for (const call of fetchMock.mock.calls) {
      expect(sentToken(call[1])).toBe('tok-1');
      // The caller's own init is kept.
      expect(call[1]?.credentials).toBe('same-origin');
      expect(new Headers(call[1]?.headers).get('content-type')).toBe('application/json');
    }
  });

  it('leaves GET, HEAD and a request without a method untouched', async () => {
    document.cookie = 'mw_csrf=tok-1; Path=/';
    fetchMock.mockImplementation(async () => json(200, {}));
    const init = { credentials: 'same-origin' } as const;
    await apiFetch('/api/me', init);
    await apiFetch('/api/me', { method: 'GET' });
    await apiFetch('/api/me', { method: 'HEAD' });
    await apiFetch('/api/me');
    expect(fetchMock.mock.calls[0]?.[1]).toBe(init);
    expect(fetchMock.mock.calls[3]?.[1]).toBeUndefined();
    for (const call of fetchMock.mock.calls) expect(sentToken(call[1])).toBeNull();
  });

  it('sends a write as given when there is no cookie', async () => {
    fetchMock.mockImplementation(async () => json(200, {}));
    const init = { method: 'POST', body: '{}' };
    await apiFetch('/api/x', init);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock.mock.calls[0]?.[1]).toBe(init);
  });

  it('sends the token on a request with an Authorization header too', async () => {
    // The server checks a bearer request that comes with a session cookie, so a
    // bearer header is no reason to leave the token out.
    document.cookie = 'mw_csrf=tok-1; Path=/';
    fetchMock.mockImplementation(async () => json(200, {}));
    await apiFetch('/jmap/api', { method: 'POST', headers: { Authorization: 'Bearer abc' } });
    const sent = new Headers(fetchMock.mock.calls[0]?.[1]?.headers);
    expect(sent.get('x-csrf-token')).toBe('tok-1');
    expect(sent.get('authorization')).toBe('Bearer abc');
  });

  it('sends a bearer request as built when no cookie can be read (a native shell)', async () => {
    fetchMock.mockImplementation(async () => json(200, {}));
    const init = { method: 'POST', credentials: 'omit', headers: { Authorization: 'Bearer abc' } } as const;
    await apiFetch('https://mail.example.org/jmap/api', init);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock.mock.calls[0]?.[1]).toBe(init);
  });

  it('after a token refusal asks /api/me once and repeats the write with the new token', async () => {
    fetchMock.mockImplementation(async (input, init) => {
      if (input === '/api/me') {
        // The server answers with Set-Cookie; the browser has applied it by the
        // time the response is read.
        document.cookie = 'mw_csrf=fresh; Path=/';
        return json(200, { username: 'u', accountId: 'a', csrfToken: 'fresh' });
      }
      return sentToken(init) === 'fresh' ? json(200, { ok: true }) : refusal();
    });
    const res = await apiFetch('/jmap/api', { method: 'POST', body: '{"n":1}' });
    expect(res.status).toBe(200);
    expect(fetchMock.mock.calls.map((c) => c[0])).toEqual(['/jmap/api', '/api/me', '/jmap/api']);
    expect(fetchMock.mock.calls[1]?.[1]).toEqual({ credentials: 'same-origin' });
    expect(fetchMock.mock.calls[2]?.[1]?.body).toBe('{"n":1}');
    expect(sentToken(fetchMock.mock.calls[2]?.[1])).toBe('fresh');
  });

  it('uses the csrfToken field when the cookie is not readable yet', async () => {
    fetchMock.mockImplementation(async (input, init) => {
      if (input === '/api/me') return json(200, { csrfToken: 'from-body' });
      return sentToken(init) === 'from-body' ? json(200, {}) : refusal();
    });
    const res = await apiFetch('/api/logout', { method: 'POST' });
    expect(res.status).toBe(200);
    expect(sentToken(fetchMock.mock.calls[2]?.[1])).toBe('from-body');
  });

  it('asks /admin/session for an admin route', async () => {
    fetchMock.mockImplementation(async (input, init) => {
      if (input === '/admin/session') {
        document.cookie = 'mw_csrf=adm; Path=/';
        return json(200, { username: 'root', csrfToken: 'adm' });
      }
      return sentToken(init) === 'adm' ? new Response(null, { status: 204 }) : refusal();
    });
    const res = await apiFetch('/admin/bans/203.0.113.9', { method: 'DELETE', credentials: 'same-origin' });
    expect(res.status).toBe(204);
    expect(fetchMock.mock.calls.map((c) => c[0])).toEqual([
      '/admin/bans/203.0.113.9',
      '/admin/session',
      '/admin/bans/203.0.113.9',
    ]);
  });

  it('keeps the deploy prefix on the token request', async () => {
    (globalThis as { __MW_BASE__?: unknown }).__MW_BASE__ = '/mail';
    fetchMock.mockImplementation(async (input) =>
      input === '/mail/admin/session' || input === '/mail/api/me' ? json(401, {}) : refusal(),
    );
    await apiFetch('/mail/admin/users', { method: 'POST' });
    await apiFetch('/mail/api/account/signatures', { method: 'POST' });
    expect(fetchMock.mock.calls.map((c) => c[0])).toEqual([
      '/mail/admin/users',
      '/mail/admin/session',
      '/mail/api/account/signatures',
      '/mail/api/me',
    ]);
  });

  it('returns the refusal when no token can be had (not signed in)', async () => {
    fetchMock.mockImplementation(async (input) => (input === '/api/me' ? json(401, { error: 'x' }) : refusal()));
    const res = await apiFetch('/api/logout', { method: 'POST' });
    expect(res.status).toBe(403);
    expect(await res.json()).toEqual({ error: 'missing or invalid CSRF token' });
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('retries once only: a second refusal is returned', async () => {
    fetchMock.mockImplementation(async (input) =>
      input === '/api/me' ? json(200, { csrfToken: 'still-wrong' }) : refusal(),
    );
    const res = await apiFetch('/jmap/api', { method: 'POST', body: '{}' });
    expect(res.status).toBe(403);
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });

  it('does not treat another 403 as a token refusal, and leaves its body unread', async () => {
    document.cookie = 'mw_csrf=tok-1; Path=/';
    fetchMock.mockImplementation(async () => json(403, { error: 'password change required', passwordChangeRequired: true }));
    const res = await apiFetch('/jmap/api', { method: 'POST', body: '{}' });
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(await res.json()).toEqual({ error: 'password change required', passwordChangeRequired: true });
  });

  it('returns a 403 that is not JSON, or not a real Response, without retrying', async () => {
    fetchMock.mockImplementationOnce(async () => new Response('forbidden', { status: 403 }));
    expect((await apiFetch('/api/x', { method: 'POST' })).status).toBe(403);
    // Unit tests elsewhere hand back plain objects with no `clone()`.
    const fake = { ok: false, status: 403, json: async () => ({ error: 'missing or invalid CSRF token' }) };
    fetchMock.mockImplementationOnce(async () => fake as unknown as Response);
    expect(await apiFetch('/api/x', { method: 'POST' })).toBe(fake);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('shares one token request between writes refused at the same time', async () => {
    let meCalls = 0;
    fetchMock.mockImplementation(async (input, init) => {
      if (input === '/api/me') {
        meCalls += 1;
        await new Promise((r) => setTimeout(r, 5));
        document.cookie = 'mw_csrf=fresh; Path=/';
        return json(200, { csrfToken: 'fresh' });
      }
      return sentToken(init) === 'fresh' ? json(200, {}) : refusal();
    });
    const [a, b] = await Promise.all([
      apiFetch('/jmap/api', { method: 'POST', body: '1' }),
      apiFetch('/api/sanitize', { method: 'POST', body: '2' }),
    ]);
    expect([a.status, b.status]).toEqual([200, 200]);
    expect(meCalls).toBe(1);
  });

  it('does not resend a body that can be read only once', async () => {
    fetchMock.mockImplementation(async () => refusal());
    const body = new ReadableStream<Uint8Array>();
    const res = await apiFetch('/jmap/upload/a', { method: 'POST', body });
    expect(res.status).toBe(403);
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});

// t31: in a native shell a root-relative request goes to the configured server
// with the bearer token, and a sign-in answer's token is kept.
describe('apiFetch in a native shell', () => {
  let fetchMock: ReturnType<typeof vi.fn<(...args: FetchArgs) => Promise<Response>>>;
  let stored: string | null;
  let base: string | null;
  const transport: NativeTransport = {
    base: () => base,
    token: async () => stored,
    setToken: async (token) => {
      stored = token;
    },
  };

  beforeEach(() => {
    clearCookie();
    stored = null;
    base = 'https://mail.example.org';
    fetchMock = vi.fn<(...args: FetchArgs) => Promise<Response>>();
    vi.stubGlobal('fetch', fetchMock);
    setNativeTransport(transport);
  });

  afterEach(() => {
    setNativeTransport(null);
    vi.unstubAllGlobals();
    clearCookie();
  });

  it('sends a root-relative write to the server with the bearer token and no cookies', async () => {
    stored = 'tok-n';
    // A readable mw_csrf cookie on the shell origin must not be sent along.
    document.cookie = 'mw_csrf=shell; Path=/';
    fetchMock.mockImplementation(async () => json(200, {}));
    for (const method of ['GET', 'PUT', 'PATCH', 'DELETE']) {
      await apiFetch('/api/account/appearance', { method, credentials: 'same-origin' });
    }
    expect(fetchMock).toHaveBeenCalledTimes(4);
    for (const [input, init] of fetchMock.mock.calls) {
      expect(input).toBe('https://mail.example.org/api/account/appearance');
      expect(new Headers(init?.headers).get('authorization')).toBe('Bearer tok-n');
      expect(init?.credentials).toBe('omit');
      expect(sentToken(init)).toBeNull();
    }
  });

  it('keeps the token the second-factor step answers with', async () => {
    fetchMock.mockImplementation(async () => json(200, { ok: true, token: 'tok-2fa' }));
    await apiFetch('/api/login/2fa', { method: 'POST', body: '{}' });
    expect(stored).toBe('tok-2fa');
    expect(new Headers(fetchMock.mock.calls[0]?.[1]?.headers).get('authorization')).toBeNull();
  });

  it('does not take a token from a route that is not a sign-in route', async () => {
    fetchMock.mockImplementation(async () => json(200, { token: 'not-a-session' }));
    await apiFetch('/api/keys', { method: 'POST', body: '{}' });
    expect(stored).toBeNull();
  });

  it('leaves an absolute URL alone', async () => {
    stored = 'tok-n';
    fetchMock.mockImplementation(async () => json(200, {}));
    await apiFetch('https://other.example/api/x');
    expect(fetchMock.mock.calls[0]?.[0]).toBe('https://other.example/api/x');
    expect(new Headers(fetchMock.mock.calls[0]?.[1]?.headers).get('authorization')).toBeNull();
  });

  it('fails without a request when no server address is configured', async () => {
    base = null;
    await expect(apiFetch('/api/me')).rejects.toThrow();
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
