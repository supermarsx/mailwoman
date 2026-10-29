import { afterEach, describe, expect, it, vi } from 'vitest';
import { signContent } from './remote-content.ts';

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
});
