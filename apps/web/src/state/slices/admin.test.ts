import { describe, it, expect, vi, afterEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createHttpAdminApi, createAdminSlice, AdminApiError, type AdminApi } from './admin.ts';

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

describe('createHttpAdminApi', () => {
  afterEach(() => vi.restoreAllMocks());

  it('session() returns null on 401 (gate)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 401 })));
    const api = createHttpAdminApi();
    expect(await api.session()).toBeNull();
  });

  it('GET domains hits /admin/domains same-origin', async () => {
    const fetchMock = vi.fn(
      async (_url: string, _init?: RequestInit): Promise<Response> =>
        jsonResponse([{ name: 'x', upstreamJson: '{}', allowlist: [], blocklist: [] }]),
    );
    vi.stubGlobal('fetch', fetchMock);
    const api = createHttpAdminApi();
    const out = await api.listDomains();
    expect(out[0]!.name).toBe('x');
    const [url, init] = fetchMock.mock.calls[0]!;
    expect(url).toBe('/admin/domains');
    expect(init?.credentials).toBe('same-origin');
  });

  it('provisionUser POSTs JSON to /admin/users', async () => {
    const fetchMock = vi.fn(
      async (_url: string, _init?: RequestInit): Promise<Response> => new Response(null, { status: 204 }),
    );
    vi.stubGlobal('fetch', fetchMock);
    const api = createHttpAdminApi();
    await api.provisionUser({ domain: 'd', username: 'u', quota: { bytesLimit: 0, msgLimit: 0 } });
    const [url, init] = fetchMock.mock.calls[0]!;
    expect(url).toBe('/admin/users');
    expect(init?.method).toBe('POST');
    expect(JSON.parse(init?.body as string)).toMatchObject({ username: 'u' });
  });

  it('throws AdminApiError on a non-2xx GET', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 500 })));
    const api = createHttpAdminApi();
    await expect(api.listBans()).rejects.toBeInstanceOf(AdminApiError);
  });

  it('exportAudit returns the raw JSONL text', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{"a":1}\n', { status: 200 })));
    const api = createHttpAdminApi();
    expect(await api.exportAudit(10)).toBe('{"a":1}\n');
  });
});

describe('createAdminSlice', () => {
  it('loadSession marks checked and stores the session', async () => {
    const api = { session: vi.fn(async () => ({ username: 'root' })) } as unknown as AdminApi;
    await createRoot(async (dispose) => {
      const slice = createAdminSlice(api);
      expect(slice.sessionChecked()).toBe(false);
      await slice.loadSession();
      expect(slice.sessionChecked()).toBe(true);
      expect(slice.session()?.username).toBe('root');
      dispose();
    });
  });

  it('logout clears the session', async () => {
    const api = { logout: vi.fn(async () => undefined), session: vi.fn() } as unknown as AdminApi;
    await createRoot(async (dispose) => {
      const slice = createAdminSlice(api);
      await slice.logout();
      expect(slice.session()).toBeNull();
      dispose();
    });
  });

  it('setSection switches the visible section', () => {
    createRoot((dispose) => {
      const slice = createAdminSlice({} as AdminApi);
      expect(slice.section()).toBe('domains');
      slice.setSection('observability');
      expect(slice.section()).toBe('observability');
      dispose();
    });
  });
});
