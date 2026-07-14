import { describe, it, expect, vi, afterEach } from 'vitest';
import {
  listSsoProviders,
  ssoBeginPath,
  ssoMetadataPath,
  createHttpSsoAdminApi,
  SsoAdminError,
} from './client.ts';
import type { SsoBackendInput } from './types.ts';

afterEach(() => vi.unstubAllGlobals());

describe('sso paths', () => {
  it('builds the begin + metadata paths (encoding the id)', () => {
    expect(ssoBeginPath('corp-oidc')).toBe('/api/sso/corp-oidc/begin');
    expect(ssoMetadataPath('a/b')).toBe('/api/sso/a%2Fb/metadata');
  });
});

describe('listSsoProviders (public, fail-soft)', () => {
  it('returns the advertised providers on 200', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response(JSON.stringify([{ id: 'x', kind: 'oidc', displayName: 'X' }]), { status: 200 }),
      ),
    );
    expect(await listSsoProviders()).toEqual([{ id: 'x', kind: 'oidc', displayName: 'X' }]);
  });

  it('resolves to [] on a non-2xx (no SSO configured)', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 404 })));
    expect(await listSsoProviders()).toEqual([]);
  });

  it('resolves to [] when fetch throws (offline)', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => {
        throw new Error('network down');
      }),
    );
    expect(await listSsoProviders()).toEqual([]);
  });

  it('resolves to [] when the body is not an array', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('{"nope":true}', { status: 200 })));
    expect(await listSsoProviders()).toEqual([]);
  });
});

describe('createHttpSsoAdminApi', () => {
  it('GETs /admin/sso and returns the rows', async () => {
    const rows = [
      {
        id: 'corp-oidc',
        displayName: 'Acme',
        scope: 'deployment',
        enabled: true,
        config: { kind: 'oidc', issuerUrl: 'https://i', clientId: 'c', redirectUrl: 'r', scopes: ['openid'], firstLoginPolicy: 'allowlist' },
        claimMap: { email: 'email', username: 'sub', display: 'name', groups: null },
      },
    ];
    const fetchMock = vi.fn(async () => new Response(JSON.stringify(rows), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    const api = createHttpSsoAdminApi();
    expect(await api.list()).toEqual(rows);
    expect(fetchMock).toHaveBeenCalledWith('/admin/sso', { credentials: 'same-origin' });
  });

  it('POSTs the input on save', async () => {
    const fetchMock = vi.fn(async () => new Response('', { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    const input: SsoBackendInput = {
      id: 'corp-oidc',
      displayName: 'Acme',
      scope: 'deployment',
      enabled: true,
      config: { kind: 'oidc', issuerUrl: 'https://i', clientId: 'c', redirectUrl: 'r', scopes: ['openid'], firstLoginPolicy: 'allowlist' },
      claimMap: { email: 'email', username: 'sub', display: 'name', groups: null },
      secret: 's3cret',
    };
    await createHttpSsoAdminApi().save(input);
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/admin/sso');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body as string)).toEqual(input);
  });

  it('DELETEs /admin/sso/{id}', async () => {
    const fetchMock = vi.fn(async () => new Response('', { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);
    await createHttpSsoAdminApi().remove('corp-oidc');
    expect(fetchMock).toHaveBeenCalledWith('/admin/sso/corp-oidc', {
      method: 'DELETE',
      credentials: 'same-origin',
    });
  });

  it('throws SsoAdminError on a non-2xx', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 500 })));
    await expect(createHttpSsoAdminApi().list()).rejects.toBeInstanceOf(SsoAdminError);
  });
});
