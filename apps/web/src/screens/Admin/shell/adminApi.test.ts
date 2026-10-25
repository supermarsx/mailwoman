import { describe, it, expect, vi, afterEach } from 'vitest';
import { AdminProblemError, adminRead, adminRequest, isStale, loadEffectiveConfig, loadSettingsRegistry, parseImpact } from './adminApi.ts';
import { AdminApiError } from '../../../state/slices/admin.ts';

afterEach(() => vi.unstubAllGlobals());

function respond(status: number, body: unknown, headers: Record<string, string> = {}) {
  const fn = vi.fn(
    async (_url: string, _init?: RequestInit) =>
      new Response(body === undefined ? '' : JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json', ...headers } }),
  );
  vi.stubGlobal('fetch', fn);
  return fn;
}

describe('adminRequest', () => {
  it('sends JSON with If-Match to the console path and returns the ETag', async () => {
    const fetch = respond(200, { ok: true }, { ETag: '"v2"' });
    const r = await adminRequest('/settings/x', { method: 'PUT', body: { globalValue: 3 }, ifMatch: '"v1"', base: '' });
    expect(r).toEqual({ status: 200, data: { ok: true }, etag: '"v2"' });
    const [url, init] = fetch.mock.calls[0]!;
    expect(url).toBe('/admin/settings/x');
    const headers = new Headers(init!.headers);
    expect(headers.get('If-Match')).toBe('"v1"');
    expect(headers.get('Content-Type')).toBe('application/json');
    expect(init!.body).toBe('{"globalValue":3}');
  });

  it('reads the tag from the body when there is no ETag header', async () => {
    respond(200, { etag: 'abc', value: 1 });
    expect((await adminRequest('/x', { base: '' })).etag).toBe('abc');
  });

  it('a 422 rejects with the problem body: code, detail and the JSON pointer', async () => {
    respond(422, { code: 'out_of_range', detail: 'port must be 1-65535', field: '/smtp/port' });
    const err = await adminRequest('/x', { method: 'PUT', body: {}, base: '' }).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(AdminProblemError);
    expect(err).toBeInstanceOf(AdminApiError);
    expect(err).toMatchObject({ status: 422, code: 'out_of_range', detail: 'port must be 1-65535', field: '/smtp/port' });
  });

  it('a 412 is stale and carries who changed it, when, and the current tag', async () => {
    respond(412, { code: 'stale', updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z' }, { ETag: '"v9"' });
    const err = await adminRequest('/x', { method: 'PUT', body: {}, ifMatch: '"v1"', base: '' }).catch((e: unknown) => e);
    expect(isStale(err)).toBe(true);
    expect(err).toMatchObject({ updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z', currentEtag: '"v9"' });
  });
});

describe('adminRead', () => {
  it('a missing route is unavailable, not an error', async () => {
    respond(404, undefined);
    expect(await adminRead('/config/effective', { base: '' })).toEqual({ kind: 'unavailable' });
  });

  it('a 404 with a problem code (object not found) is an error', async () => {
    respond(404, { code: 'not_found' });
    await expect(adminRead('/x', { base: '' })).rejects.toBeInstanceOf(AdminProblemError);
  });

  it('other failures reject', async () => {
    respond(500, { error: 'boom' });
    await expect(adminRead('/x', { base: '' })).rejects.toMatchObject({ status: 500 });
  });

  it('loadEffectiveConfig and loadSettingsRegistry degrade to unavailable before their lanes land', async () => {
    respond(404, undefined);
    expect(await loadEffectiveConfig('')).toEqual({ kind: 'unavailable' });
    expect(await loadSettingsRegistry('')).toEqual({ kind: 'unavailable' });
  });

  it('loadSettingsRegistry accepts a bare list or {settings: [...]}', async () => {
    respond(200, { settings: [{ key: 'ui.language', type: 'string' }] }, { ETag: 't' });
    expect(await loadSettingsRegistry('')).toEqual({ kind: 'ok', data: [{ key: 'ui.language', type: 'string' }], etag: 't' });
    respond(200, [{ key: 'a', type: 'bool' }]);
    expect(await loadSettingsRegistry('')).toMatchObject({ kind: 'ok', data: [{ key: 'a' }] });
  });
});

describe('parseImpact', () => {
  it('normalises counts, {count, names} and bare name lists; drops junk', () => {
    expect(parseImpact({ impact: { sessionsEnded: 4, usersRefused: { count: 3, names: ['a', 'b', 7] }, domains: ['x.org'] } })).toEqual({
      sessionsEnded: { count: 4 },
      usersRefused: { count: 3, names: ['a', 'b'] },
      domains: { count: 1, names: ['x.org'] },
    });
    expect(parseImpact({ ok: true })).toBeNull();
    expect(parseImpact(null)).toBeNull();
  });
});
