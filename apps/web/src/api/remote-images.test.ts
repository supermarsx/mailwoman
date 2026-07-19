import { describe, it, expect, vi } from 'vitest';
import {
  analyzeBlockedContent,
  coveringGrant,
  createRemoteImageApi,
  hasBlockedContent,
  imageProxyUrl,
  rewriteGrantedImages,
  scopeFor,
  senderDomain,
  type Fetcher,
  type RemoteImageGrant,
} from './remote-images.ts';

interface FetchCall {
  input: string;
  init: RequestInit | undefined;
}

/** A `Fetcher` fake that records each call and returns a canned JSON response —
 *  e6's REST grant endpoints are the whole seam, so we assert their shape. */
function fakeFetcher(status: number, body: unknown): { fetcher: Fetcher; calls: FetchCall[] } {
  const calls: FetchCall[] = [];
  const fetcher: Fetcher = vi.fn(async (input: string, init?: RequestInit): Promise<Response> => {
    calls.push({ input, init });
    return new Response(JSON.stringify(body), {
      status,
      headers: { 'content-type': 'application/json' },
    });
  });
  return { fetcher, calls };
}

describe('senderDomain', () => {
  it('extracts the domain, lower-cased', () => {
    expect(senderDomain('Alice@Example.COM')).toBe('example.com');
  });
  it('is empty for an address with no @', () => {
    expect(senderDomain('local-only')).toBe('');
  });
});

describe('scopeFor', () => {
  const ctx = { emailId: 'M1', sender: 'Bob@Spam.Example' };
  it('single uses the message id', () => {
    expect(scopeFor('single', ctx)).toEqual({ kind: 'single', value: 'M1' });
  });
  it('per-sender lower-cases the address', () => {
    expect(scopeFor('per-sender', ctx)).toEqual({ kind: 'per-sender', value: 'bob@spam.example' });
  });
  it('per-domain uses the domain', () => {
    expect(scopeFor('per-domain', ctx)).toEqual({ kind: 'per-domain', value: 'spam.example' });
  });
  it('all uses the empty account-wide value', () => {
    expect(scopeFor('all', ctx)).toEqual({ kind: 'all', value: '' });
  });
});

describe('analyzeBlockedContent', () => {
  it('is empty for null / empty / unmarked HTML', () => {
    expect(analyzeBlockedContent(null)).toEqual({ blockedHosts: [], blockedCount: 0, trackerCount: 0 });
    expect(analyzeBlockedContent('')).toEqual({ blockedHosts: [], blockedCount: 0, trackerCount: 0 });
    expect(analyzeBlockedContent('<p>hello <img src="cid:x"></p>')).toEqual({
      blockedHosts: [],
      blockedCount: 0,
      trackerCount: 0,
    });
    expect(hasBlockedContent(analyzeBlockedContent('<p>x</p>'))).toBe(false);
  });

  it('counts marked blocked resources and dedups + sorts hosts', () => {
    const html = `
      <img data-mw-blocked-host="tracker.evil">
      <img data-mw-blocked-host="cdn.example">
      <img data-mw-blocked-host="tracker.evil" data-mw-tracker>
    `;
    const r = analyzeBlockedContent(html);
    expect(r.blockedCount).toBe(3);
    expect(r.blockedHosts).toEqual(['cdn.example', 'tracker.evil']);
    expect(r.trackerCount).toBe(1);
    expect(hasBlockedContent(r)).toBe(true);
  });

  it('normalizes host case and ignores empty host markers in the host list', () => {
    const html = `<img data-mw-blocked-host="Beacon.Example"><span data-mw-blocked-host="" data-mw-tracker></span>`;
    const r = analyzeBlockedContent(html);
    // Both are blocked resources; only the non-empty host lists.
    expect(r.blockedCount).toBe(2);
    expect(r.blockedHosts).toEqual(['beacon.example']);
    expect(r.trackerCount).toBe(1);
  });
});

describe('coveringGrant', () => {
  const ctx = { emailId: 'M1', sender: 'bob@spam.example' };
  const grant = (scopeKind: RemoteImageGrant['scopeKind'], scopeValue: string): RemoteImageGrant => ({
    scopeKind,
    scopeValue,
    grantedAt: '2026-07-19T00:00:00Z',
  });

  it('returns null when nothing covers the message', () => {
    expect(coveringGrant([grant('per-sender', 'other@x.example')], ctx)).toBeNull();
    expect(coveringGrant([], ctx)).toBeNull();
  });
  it('matches an account-wide all grant', () => {
    expect(coveringGrant([grant('all', '')], ctx)?.scopeKind).toBe('all');
  });
  it('matches the message id for a single grant', () => {
    expect(coveringGrant([grant('single', 'M1')], ctx)?.scopeKind).toBe('single');
    expect(coveringGrant([grant('single', 'M2')], ctx)).toBeNull();
  });
  it('matches sender / domain case-insensitively', () => {
    expect(coveringGrant([grant('per-sender', 'BOB@spam.example')], ctx)?.scopeKind).toBe('per-sender');
    expect(coveringGrant([grant('per-domain', 'Spam.Example')], ctx)?.scopeKind).toBe('per-domain');
  });
});

describe('createRemoteImageApi (REST)', () => {
  it('grant POSTs the scope to /api/remote-images/grant', async () => {
    const { fetcher, calls } = fakeFetcher(200, { ok: true });
    await createRemoteImageApi(fetcher).grant('acct1', { kind: 'per-domain', value: 'spam.example' });
    expect(calls[0]!.input).toBe('/api/remote-images/grant');
    expect(calls[0]!.init?.method).toBe('POST');
    expect(JSON.parse(calls[0]!.init!.body as string)).toEqual({
      scopeKind: 'per-domain',
      scopeValue: 'spam.example',
    });
  });

  it('revoke POSTs the scope to /api/remote-images/revoke', async () => {
    const { fetcher, calls } = fakeFetcher(200, { ok: true });
    await createRemoteImageApi(fetcher).revoke('acct1', { kind: 'all', value: '' });
    expect(calls[0]!.input).toBe('/api/remote-images/revoke');
    expect(JSON.parse(calls[0]!.init!.body as string)).toEqual({ scopeKind: 'all', scopeValue: '' });
  });

  it('listGrants GETs /api/remote-images/grants and returns the list', async () => {
    const grants: RemoteImageGrant[] = [{ scopeKind: 'all', scopeValue: '', grantedAt: '2026-07-19T00:00:00Z' }];
    const { fetcher, calls } = fakeFetcher(200, { accountId: 'acct1', list: grants });
    const api = createRemoteImageApi(fetcher);
    expect(await api.listGrants('acct1')).toEqual(grants);
    expect(calls[0]!.input).toBe('/api/remote-images/grants');
    expect(calls[0]!.init?.method ?? 'GET').toBe('GET');
  });

  it('listGrants tolerates a missing list', async () => {
    const { fetcher } = fakeFetcher(200, { accountId: 'acct1' });
    expect(await createRemoteImageApi(fetcher).listGrants('acct1')).toEqual([]);
  });

  it('throws on a non-2xx grant response', async () => {
    const { fetcher } = fakeFetcher(403, { error: 'no' });
    await expect(
      createRemoteImageApi(fetcher).grant('acct1', { kind: 'single', value: 'M1' }),
    ).rejects.toThrow(/403/);
  });
});

describe('imageProxyUrl', () => {
  it('routes an original URL through the same-origin proxy, encoded', () => {
    expect(imageProxyUrl('https://cdn.example/a b.png?x=1&y=2')).toBe(
      '/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Fa%20b.png%3Fx%3D1%26y%3D2',
    );
  });
});

describe('rewriteGrantedImages', () => {
  const raw = '<p>hi</p><img src="https://cdn.example/logo.png"><img src="cid:inline">';
  // What the sanitizer produces from `raw`: the remote src is stripped (element
  // kept, in order), the cid: src survives, and a block marker is appended.
  const sanitized =
    '<div class="mw-email-body"><p>hi</p><img><img src="cid:inline">' +
    '<span hidden data-mw-blocked-host="cdn.example"></span></div>';

  it('returns the sanitized body BYTE-for-byte when not granted', () => {
    expect(rewriteGrantedImages(sanitized, raw, false)).toBe(sanitized);
  });

  it('routes a granted message\'s remote image through the proxy', () => {
    const out = rewriteGrantedImages(sanitized, raw, true)!;
    expect(out).toContain('src="/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Flogo.png"');
    // The cid: image is untouched; no bare remote URL is ever reintroduced.
    expect(out).toContain('src="cid:inline"');
    expect(out).not.toContain('https://cdn.example/logo.png"');
  });

  it('leaves an ungranted remote image stripped (deny-by-default)', () => {
    // With no covering grant the sanitized body is unchanged, so the stripped
    // <img> stays srcless — nothing loads.
    const out = rewriteGrantedImages(sanitized, raw, false)!;
    expect(out).not.toContain('image-proxy');
    expect(out).not.toContain('https://cdn.example');
  });

  it('fails closed (no rewrite) when the img lists cannot be aligned 1:1', () => {
    // A sanitized body with FEWER imgs than the raw (e.g. one dropped inside a
    // removed container) can't be aligned → returned unchanged, nothing loaded.
    const mismatched = '<p>hi</p><img>';
    expect(rewriteGrantedImages(mismatched, raw, true)).toBe(mismatched);
  });

  it('does not touch a cid-only or image-free body even when granted', () => {
    const cidRaw = '<img src="cid:x">';
    const cidClean = '<img src="cid:x">';
    expect(rewriteGrantedImages(cidClean, cidRaw, true)).toBe(cidClean);
    expect(rewriteGrantedImages('<p>plain</p>', '<p>plain</p>', true)).toBe('<p>plain</p>');
  });

  it('is null/empty-safe', () => {
    expect(rewriteGrantedImages(null, raw, true)).toBeNull();
    expect(rewriteGrantedImages(sanitized, null, true)).toBe(sanitized);
    expect(rewriteGrantedImages(sanitized, '', true)).toBe(sanitized);
  });
});
