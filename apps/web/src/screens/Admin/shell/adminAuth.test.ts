import { describe, it, expect, vi, afterEach } from 'vitest';
import { AdminTotpRequired, createAdminTotpVerifier, withTotpGate } from './adminAuth.ts';
import { mockAdminApi } from '../testkit.tsx';
import { AdminApiError } from '../../../state/slices/admin.ts';

afterEach(() => vi.unstubAllGlobals());

describe('withTotpGate', () => {
  it('a twofaRequired answer is raised, never returned as a session', async () => {
    const api = withTotpGate(
      mockAdminApi({ login: vi.fn(async () => ({ twofaRequired: true, pendingToken: 'pt', factors: ['totp'] }) as never) }),
    );
    const err = await api.login('a', 'b').catch((e: unknown) => e);
    expect(err).toBeInstanceOf(AdminTotpRequired);
    expect((err as AdminTotpRequired).challenge).toEqual({ pendingToken: 'pt', factors: ['totp'] });
  });

  it('a plain session passes through, and the original login is the one called', async () => {
    const inner = mockAdminApi();
    const api = withTotpGate(inner);
    expect(await api.login('root', 'pw')).toEqual({ username: 'root' });
    expect(inner.login).toHaveBeenCalledWith('root', 'pw');
  });

  it('writes reach the wrapped client (the slice installs onSessionEnded through it)', () => {
    const inner = mockAdminApi();
    const api = withTotpGate(inner);
    const hook = (): void => undefined;
    api.onSessionEnded = hook;
    expect(inner.onSessionEnded).toBe(hook);
  });
});

describe('createAdminTotpVerifier', () => {
  it('posts the pending token and code to /admin/login/totp', async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ username: 'ops' }), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    expect(await createAdminTotpVerifier('')('pt', '123456')).toEqual({ username: 'ops' });
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/admin/login/totp');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body as string)).toEqual({ pendingToken: 'pt', code: '123456' });
  });

  it('a refused code is an AdminApiError with the status', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 401 })));
    const err = await createAdminTotpVerifier('')('pt', '000000').catch((e: unknown) => e);
    expect(err).toBeInstanceOf(AdminApiError);
    expect((err as AdminApiError).status).toBe(401);
  });
});
