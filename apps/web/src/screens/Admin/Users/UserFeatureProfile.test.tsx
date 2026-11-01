import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { UserFeatureProfile } from './UserFeatureProfile.tsx';
import { featuresApi, type FeatureCatalogue } from '../Features/api.ts';
import { assignProfile } from '../../../api/admin/usersTable.ts';

vi.mock('../Features/api.ts', () => ({ featuresApi: { load: vi.fn() } }));
vi.mock('../../../api/admin/usersTable.ts', async importOriginal => ({ ...await importOriginal<typeof import('../../../api/admin/usersTable.ts')>(), assignProfile: vi.fn() }));

beforeEach(() => vi.clearAllMocks());
it('assigns and clears a proxy user profile without editing inherited domain assignments', async () => {
  const data: FeatureCatalogue = {
    revision: 'one', profiles: [{ id: 'work', name: 'Work', values: {}, builtin: false, updatedAt: 1 }],
    groups: [], assignments: [{ profileId: 'work', scopeKind: 'domain', scopeId: 'example.test', priority: 1 }],
    memberships: {}, nativeTargets: [], userTargets: [{ accountId: 'po1_verified', directoryAccountId: 'proxy-id', address: 'proxy@example.test', domain: 'example.test' }],
    domains: ['example.test'], catalogue: [],
  };
  vi.mocked(featuresApi.load).mockImplementation(async () => ({ data: structuredClone(data), etag: 'one' }));
  vi.mocked(assignProfile).mockImplementation(async (_account, profile) => {
    data.assignments = data.assignments.filter(a => a.scopeKind !== 'user');
    if (profile) data.assignments.push({ scopeKind: 'user', scopeId: 'po1_verified', profileId: profile, priority: 100 });
  });
  render(() => <UserFeatureProfile accountId="proxy-id" />);
  const select = await screen.findByRole('combobox', { name: 'Feature profile' });
  await waitFor(() => expect(select).toBeEnabled());
  expect(select).toHaveValue('');
  fireEvent.change(select, { target: { value: 'work' } });
  fireEvent.click(screen.getByRole('button', { name: 'Change profile' }));
  await waitFor(() => expect(assignProfile).toHaveBeenCalledWith('proxy-id', 'work', null));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Change profile' })).toBeDisabled());
  await waitFor(() => expect(select).toHaveValue('work'));
  fireEvent.change(select, { target: { value: '' } });
  fireEvent.click(screen.getByRole('button', { name: 'Change profile' }));
  await waitFor(() => expect(assignProfile).toHaveBeenCalledWith('proxy-id', null, 'work'));
  await waitFor(() => expect(select).toHaveValue(''));
  expect(data.assignments).toEqual([{ profileId: 'work', scopeKind: 'domain', scopeId: 'example.test', priority: 1 }]);
});
