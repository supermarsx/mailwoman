import { describe, it, expect, vi } from 'vitest';
import {
  bindTrackingToken,
  mintTrackingToken,
  openCount,
  trackingPixel,
  TrackingUnavailable,
  withTrackingPixel,
} from './tracking.ts';

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

describe('open tracking client (t29-e12)', () => {
  it('mints with the subject and recipients and returns the token and image URL', async () => {
    const fetcher = vi.fn(async () => json({ token: 'tok', url: 'https://mail.example/t/tok.gif' }));
    await expect(mintTrackingToken('Hi', ['a@x.org'], fetcher)).resolves.toEqual({
      token: 'tok',
      url: 'https://mail.example/t/tok.gif',
    });
    const [url, init] = fetcher.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/track/open');
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ subject: 'Hi', to: ['a@x.org'] });
  });

  it('a refused mint carries the server reason; a lost network says so', async () => {
    await expect(mintTrackingToken('Hi', [], async () => json({ error: 'publicUrlRequired' }, 409))).rejects.toEqual(
      new TrackingUnavailable('publicUrlRequired'),
    );
    await expect(
      mintTrackingToken('Hi', [], async () => {
        throw new TypeError('Failed to fetch');
      }),
    ).rejects.toMatchObject({ reason: 'network' });
  });

  it('refuses an image URL that is not absolute http(s)', async () => {
    for (const url of ['/t/tok.gif', 'javascript:alert(1)', 'data:image/gif;base64,R0']) {
      await expect(mintTrackingToken('Hi', [], async () => json({ token: 'tok', url }))).rejects.toMatchObject({
        reason: 'malformed answer',
      });
    }
  });

  it('binds by PUT with the email id; a refusal or lost network is false', async () => {
    const ok = vi.fn(async () => new Response(null, { status: 204 }));
    await expect(bindTrackingToken('t/k', 'e1', ok)).resolves.toBe(true);
    const [url, init] = ok.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/track/open/t%2Fk');
    expect(init.method).toBe('PUT');
    expect(JSON.parse(String(init.body))).toEqual({ emailId: 'e1' });
    await expect(bindTrackingToken('t', 'e1', async () => json({ error: 'notFound' }, 404))).resolves.toBe(false);
    await expect(
      bindTrackingToken('t', 'e1', async () => {
        throw new TypeError('offline');
      }),
    ).resolves.toBe(false);
  });

  it('reads the count, and null when the message is not tracked', async () => {
    await expect(
      openCount('e1', async () => json({ count: 2, first: '2026-10-25T10:00:00.000Z', last: '2026-10-25T11:00:00.000Z' })),
    ).resolves.toEqual({ count: 2, first: '2026-10-25T10:00:00.000Z', last: '2026-10-25T11:00:00.000Z' });
    await expect(openCount('e1', async () => json({ error: 'notFound' }, 404))).resolves.toBeNull();
  });

  it('escapes the URL as an attribute', () => {
    expect(trackingPixel('https://x.example/t/a.gif?x="><script>&y')).toBe(
      '<img src="https://x.example/t/a.gif?x=&quot;&gt;&lt;script&gt;&amp;y" width="1" height="1" alt="">',
    );
  });

  it('appends the image to the draft body and mints for To, Cc and Bcc', async () => {
    const mint = vi.fn(async () => ({ token: 'tok', url: 'https://mail.example/t/tok.gif' }));
    const out = await withTrackingPixel(
      {
        from: { name: null, email: 'me@x.org' },
        to: 'A <a@x.org>',
        cc: 'b@x.org',
        bcc: 'c@x.org',
        subject: 'S',
        htmlBody: '<p>x</p>',
        draftMailboxId: 'd',
      },
      mint,
    );
    expect(mint).toHaveBeenCalledWith('S', ['a@x.org', 'b@x.org', 'c@x.org']);
    expect(out.token).toBe('tok');
    expect(out.draft.htmlBody).toBe('<p>x</p><img src="https://mail.example/t/tok.gif" width="1" height="1" alt="">');
  });
});
