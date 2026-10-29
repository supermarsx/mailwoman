import { afterEach, describe, expect, it, vi } from 'vitest';
import { requestImageRetry, signContent } from './remote-content.ts';

// t56: "Retry images" re-signs with retry:true so the server forgets the 60 s
// remembered failures for those URLs; an ordinary sign never sends it.
describe('signContent retry', () => {
  afterEach(() => { vi.unstubAllGlobals(); });
  function stub(): ReturnType<typeof vi.fn> {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ images: [], parts: [], files: [], refused: [] }), { status: 200, headers: { 'content-type': 'application/json' } }));
    vi.stubGlobal('fetch', fetchMock);
    return fetchMock;
  }
  const body = (mock: ReturnType<typeof vi.fn>): Record<string, unknown> => {
    const init = mock.mock.calls[0]?.[1] as RequestInit | undefined;
    return JSON.parse(String(init?.body)) as Record<string, unknown>;
  };

  it('omits retry by default', async () => {
    const mock = stub();
    await signContent('m1', { urls: ['https://img.example/a.png'] });
    expect(body(mock)).toEqual({ emailId: 'm1', urls: ['https://img.example/a.png'] });
  });

  it('sends retry:true when asked', async () => {
    const mock = stub();
    await signContent('m1', { urls: ['https://img.example/a.png'] }, 'acct', { retry: true });
    expect(body(mock)).toEqual({ emailId: 'm1', urls: ['https://img.example/a.png'], retry: true });
    const init = mock.mock.calls[0]?.[1] as RequestInit;
    expect(new Headers(init.headers).get('X-Mailwoman-Account')).toBe('acct');
  });

  it("the toast's Retry makes the next image signing of that message a retry, once (t67)", async () => {
    const mock = stub();
    requestImageRetry('m1');
    await signContent('m2', { urls: ['https://img.example/b.png'] });
    await signContent('m1', { parts: ['blob1'] }); // no images: not the reload's image signing
    await signContent('m1', { urls: ['https://img.example/a.png'] });
    await signContent('m1', { urls: ['https://img.example/a.png'] });
    const bodies = mock.mock.calls.map((call) => JSON.parse(String((call[1] as RequestInit).body)) as { retry?: boolean });
    expect(bodies.map((b) => b.retry === true)).toEqual([false, false, true, false]);
  });
});
