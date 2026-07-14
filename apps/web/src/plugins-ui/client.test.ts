import { describe, it, expect, vi, afterEach } from 'vitest';
import { listUiPlugins, callUiPluginRpc, makeRpcRequest } from './client';
import { EMPTY_REGISTRY } from './types';

afterEach(() => vi.unstubAllGlobals());

function ok(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

const registration = {
  manifest: {
    id: 'snooze',
    name: 'Snooze',
    version: '1.0.0',
    signature: null,
    extensionPoints: ['message-toolbar'],
    capabilities: ['net:host-allowlist'],
    csp: "default-src 'none'",
  },
  grants: [{ capability: 'net:host-allowlist', params: { hosts: ['api.example.com'] } }],
  enabled: true,
  approved: true,
};

describe('listUiPlugins (same-origin, fail-soft — mirrors listSsoProviders)', () => {
  it('returns the registry on 200', async () => {
    const fetchMock = vi.fn(async () => ok({ plugins: [registration], unsignedBanner: ['snooze'] }));
    vi.stubGlobal('fetch', fetchMock);
    const reg = await listUiPlugins();
    expect(fetchMock).toHaveBeenCalledWith('/api/ui-plugins', { credentials: 'same-origin' });
    expect(reg.plugins).toHaveLength(1);
    expect(reg.plugins[0]!.manifest.id).toBe('snooze');
    expect(reg.unsignedBanner).toEqual(['snooze']);
  });

  it('resolves to the EMPTY registry on a non-2xx (no plugins configured)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 404 })));
    expect(await listUiPlugins()).toEqual(EMPTY_REGISTRY);
  });

  it('resolves to the EMPTY registry when fetch throws (offline)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => {
      throw new Error('network down');
    }));
    expect(await listUiPlugins()).toEqual(EMPTY_REGISTRY);
  });

  it('resolves to the EMPTY registry when fetch is undefined (jsdom without fetch)', async () => {
    vi.stubGlobal('fetch', undefined);
    expect(await listUiPlugins()).toEqual(EMPTY_REGISTRY);
  });

  it('drops malformed registrations + unknown caps defensively', async () => {
    const bad = { plugins: [{ manifest: { name: 'no-id' } }, registration], unsignedBanner: [42, 'snooze'] };
    vi.stubGlobal('fetch', vi.fn(async () => ok(bad)));
    const reg = await listUiPlugins();
    expect(reg.plugins).toHaveLength(1); // the id-less manifest is dropped
    expect(reg.unsignedBanner).toEqual(['snooze']); // the non-string id is dropped
  });
});

describe('callUiPluginRpc (POST /api/ui-plugins/{id}/rpc)', () => {
  it('POSTs the envelope and returns the server response', async () => {
    const fetchMock = vi.fn(async () => ok({ v: 1, id: 'r1', ok: { status: 200, body: 'hi' } }));
    vi.stubGlobal('fetch', fetchMock);
    const req = makeRpcRequest('r1', 'net:host-allowlist', 'fetch', ['https://api.example.com/x']);
    const res = await callUiPluginRpc('snooze', req);
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/ui-plugins/snooze/rpc');
    expect(init.method).toBe('POST');
    expect(init.credentials).toBe('same-origin');
    expect(JSON.parse(init.body as string)).toEqual(req);
    expect('ok' in res && res.ok).toEqual({ status: 200, body: 'hi' });
  });

  it('surfaces a non-2xx as a structured internal RPC error (never throws)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 502 })));
    const res = await callUiPluginRpc('snooze', makeRpcRequest('r2', 'store:kv-scoped', 'get', ['k']));
    expect('err' in res && res.err.code).toBe('internal');
  });

  it('surfaces a transport throw as an internal RPC error', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => {
      throw new Error('boom');
    }));
    const res = await callUiPluginRpc('snooze', makeRpcRequest('r3', 'store:kv-scoped', 'get', ['k']));
    expect('err' in res && res.err.code).toBe('internal');
  });
});
