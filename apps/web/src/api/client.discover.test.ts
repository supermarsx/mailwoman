// `client.discover` against the shapes `POST /api/discover` really answers with
// (`crates/mw-server/src/lib.rs:2262-2306`; rate limit in
// `crates/mw-server/src/discover_ratelimit.rs:128-164`).

import { describe, it, expect, vi, afterEach } from 'vitest';
import { ApiError, NetworkError, createClient } from './client.ts';

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

describe('client.discover', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('posts the address and returns the candidate as the server serialises it', async () => {
    // `AccountCandidate` (`crates/mw-autoconfig/src/lib.rs:86-93`) plus the
    // `jmapSrv` hint the handler adds (`lib.rs:2279-2282`).
    const body = {
      imap: { host: 'imap.example.org', port: 993, tls: 'implicit' },
      pop3: null,
      smtp: { host: 'smtp.example.org', port: 587, tls: 'start-tls' },
      auth: 'password',
      source: 'thunderbird-autoconfig',
      jmapSrv: { host: 'jmap.example.org', port: 443 },
    };
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => json(body));
    vi.stubGlobal('fetch', fetchMock);

    const result = await createClient('').discover?.('ada@example.org');

    expect(result).toEqual(body);
    const [url, init] = fetchMock.mock.calls[0]!;
    expect(url).toBe('/api/discover');
    expect(init?.method).toBe('POST');
    expect(JSON.parse(String(init?.body))).toEqual({ email: 'ada@example.org' });
    // No session exists yet; the request is same-origin like every other one.
    expect(init?.credentials).toBe('same-origin');
  });

  it('prefixes the sub-path base', async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, _init?: RequestInit) => json({ source: 'jmap-srv' }));
    vi.stubGlobal('fetch', fetchMock);
    await createClient('/mail').discover?.('ada@example.org');
    expect(fetchMock.mock.calls[0]![0]).toBe('/mail/api/discover');
  });

  it.each([
    [400, { error: 'invalid email address' }],
    [404, { error: 'no configuration discovered' }],
    [429, { error: 'too many discovery requests' }],
    [502, { error: 'discovery lookup error: timeout' }],
  ])('rejects a %i with an ApiError carrying the status', async (status, body) => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => json(body, status)),
    );
    const err = await createClient('')
      .discover?.('ada@example.org')
      .catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect((err as ApiError).status).toBe(status);
  });

  it('rejects with a NetworkError when the request never reaches the server', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => {
        throw new TypeError('Failed to fetch');
      }),
    );
    await expect(createClient('').discover?.('ada@example.org')).rejects.toBeInstanceOf(NetworkError);
  });
});
