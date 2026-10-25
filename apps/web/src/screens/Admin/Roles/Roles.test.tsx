import { describe, it, expect, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminRoles } from './index.tsx';
import { renderInShell } from '../shell/testkit.tsx';
import { parseAdminPath } from '../shell/router.ts';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import type { AdminAccountsApi, AdminRole } from '../../../api/admin/admins.ts';

const role: AdminRole = { id: 'support', name: 'Support', builtin: false, holders: 2, capabilities: ['users.read'], etag: '"v1"' };
function service(): AdminAccountsApi {
  return { accounts: vi.fn(async () => []), sessions: vi.fn(async () => []), roles: vi.fn(async () => [role]), me: vi.fn(async () => ({ username: 'root', capabilities: ['roles.manage', 'users.read', 'users.write', 'domains.read'], totpEnrolled: false })), write: vi.fn(async () => role) as AdminAccountsApi['write'] };
}
function mount(path: string, api = service()) {
  const [route, setRoute] = createSignal(parseAdminPath(path));
  return { api, ...renderInShell(() => <AdminRoles service={api} />, { route, navigate: p => setRoute(parseAdminPath(p)) }) };
}
describe('dedicated role pages', () => {
  it('keeps forms off the role list and reviews a new role before writing', async () => {
    const { api, container, navigate } = mount('/admin#/roles');
    const create = await screen.findByRole('button', { name: 'New role' });
    expect(screen.queryByRole('textbox', { name: 'Display name' })).not.toBeInTheDocument();
    fireEvent.click(create);
    expect(navigate).toHaveBeenCalledWith('/admin#/roles/new');
    fireEvent.input(screen.getByRole('textbox', { name: 'Display name' }), { target: { value: 'Mail support' } });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByText(/Users and mail/));
    fireEvent.click(screen.getByRole('checkbox', { name: /Read users.read/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(api.write).not.toHaveBeenCalled();
    await expectNoAxeViolations(container);
    fireEvent.click(screen.getByRole('button', { name: 'Create role' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/roles', 'POST', { name: 'Mail support', capabilities: ['users.read'] }, undefined));
  });
  it('loads a linked edit page and preserves its concurrency token', async () => {
    const { api } = mount('/admin#/roles/support/edit');
    const input = await screen.findByRole('textbox', { name: 'Display name' });
    fireEvent.input(input, { target: { value: 'Support team' } });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(screen.getByText(/2 accounts holding/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/roles/support', 'PUT', { name: 'Support team', capabilities: ['users.read'] }, '"v1"'));
  });
  it('keeps the review and draft on a failed save', async () => {
    const api = service(); vi.mocked(api.write).mockRejectedValue(new Error('Version conflict'));
    mount('/admin#/roles/support/edit', api);
    await screen.findByRole('textbox', { name: 'Display name' });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByRole('alert')).not.toBeEmptyDOMElement();
    expect(api.write).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
    expect(screen.getByRole('heading', { name: 'Support' })).toBeInTheDocument();
  });
});
