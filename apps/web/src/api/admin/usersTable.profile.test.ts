import { beforeEach, expect, it, vi } from 'vitest';
import { assignProfile } from './usersTable.ts';
import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
import type { FeatureCatalogue } from '../../screens/Admin/Features/api.ts';

vi.mock('../../screens/Admin/shell/adminApi.ts', () => ({ adminRequest: vi.fn() }));
let data: FeatureCatalogue;
beforeEach(() => {
  vi.clearAllMocks();
  data = {
    revision: 'one', profiles: [], groups: [], memberships: {}, nativeTargets: [], catalogue: [], domains: [],
    userTargets: [{ accountId: 'po1_verified', directoryAccountId: 'raw', address: 'proxy@example.test', domain: 'example.test' }],
    assignments: [
      { scopeKind: 'user', scopeId: 'po1_verified', profileId: 'old', priority: 12 },
      { scopeKind: 'domain', scopeId: 'example.test', profileId: 'inherited', priority: 1 },
    ],
  };
  vi.mocked(adminRequest).mockImplementation(async () => ({ data, etag: '"revision"', status: 200 }));
});
it('maps a directory row to its verified subject and validates replacement before removing the old profile', async () => {
  await assignProfile('raw', 'new', 'old');
  const writes = vi.mocked(adminRequest).mock.calls.filter(([, options]) => options?.method);
  expect(writes.map(([, options]) => options?.method)).toEqual(['PUT', 'DELETE']);
  expect(writes[0]![1]?.body).toEqual({ scopeKind: 'user', scopeId: 'po1_verified', profileId: 'new', priority: 100 });
  expect(writes[1]![1]?.body).toEqual(data.assignments[0]);
});
it('clears only direct assignments on the verified subject', async () => {
  await assignProfile('raw', null, 'old');
  const writes = vi.mocked(adminRequest).mock.calls.filter(([, options]) => options?.method);
  expect(writes).toHaveLength(1);
  expect(writes[0]![1]?.method).toBe('DELETE');
  expect(writes[0]![1]?.body).toEqual(data.assignments[0]);
});
it('preserves the old assignment when the replacement is refused', async () => {
  vi.mocked(adminRequest).mockImplementation(async (_path, options) => {
    if (options?.method === 'PUT') throw new Error('refused');
    return { data, etag: '"revision"', status: 200 };
  });
  await expect(assignProfile('raw', 'invalid', 'old')).rejects.toThrow('refused');
  expect(vi.mocked(adminRequest).mock.calls.some(([, options]) => options?.method === 'DELETE')).toBe(false);
});
it('requires an explicit identity when the same upstream id exists in multiple realms', async () => {
  data.userTargets!.push({ ...data.userTargets![0]!, accountId: 'po1_other_realm' });
  await expect(assignProfile('raw', 'new', null)).rejects.toThrow();
  expect(vi.mocked(adminRequest).mock.calls).toHaveLength(1);
});
it('does not let an upstream raw id impersonate another verified policy subject', async () => {
  await expect(assignProfile('po1_verified', 'new', null)).rejects.toThrow();
  expect(vi.mocked(adminRequest).mock.calls).toHaveLength(1);
});
it('retains native account assignment through the explicit native target list', async () => {
  data.nativeTargets.push({ accountId: 'native', address: 'native@example.test', domain: 'example.test' });
  await assignProfile('native', 'new', null);
  const writes = vi.mocked(adminRequest).mock.calls.filter(([, options]) => options?.method);
  expect(writes[0]![1]?.body).toEqual({ scopeKind: 'user', scopeId: 'native', profileId: 'new', priority: 100 });
});
it('chains exact revisions without silently rebasing between profile changes', async () => {
  vi.mocked(adminRequest)
    .mockResolvedValueOnce({ data, etag: '"read"', status: 200 })
    .mockResolvedValueOnce({ data: {}, etag: '"after-put"', status: 200 })
    .mockResolvedValueOnce({ data: {}, etag: '"after-delete"', status: 200 });
  await assignProfile('raw', 'new', 'old');
  expect(vi.mocked(adminRequest).mock.calls).toHaveLength(3);
  expect(vi.mocked(adminRequest).mock.calls[1]![1]?.ifMatch).toBe('"read"');
  expect(vi.mocked(adminRequest).mock.calls[2]![1]?.ifMatch).toBe('"after-put"');
});
it('requires an explicit identity for a native and proxy account id collision', async () => {
  data.nativeTargets.push({ accountId: 'raw', address: 'native@example.test', domain: 'example.test' });
  await expect(assignProfile('raw', 'new', null)).rejects.toThrow();
  expect(vi.mocked(adminRequest).mock.calls).toHaveLength(1);
});
