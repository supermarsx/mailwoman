import { describe, it, expect, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { parseAdminPath } from '../shell/router.ts';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminAdmins } from './index.tsx';
import { AdminRoles } from '../Roles/index.tsx';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import type { AdminAccountsApi, AdminAccount, AdminRole } from '../../../api/admin/admins.ts';

const account: AdminAccount = { id: 'alice', username: 'alice', displayName: 'Alice', disabled: false, roles: [], overrides: [], totpEnrolled: false, etag: '"v1"' };
const role: AdminRole = { id: 'helpdesk', name: 'Helpdesk', builtin: true, capabilities: ['users.read'], holders: 1, etag: '"r1"' };
function service(manage = true): AdminAccountsApi {
  return { accounts: vi.fn(async () => [account]), roles: vi.fn(async () => [role]), me: vi.fn(async () => ({ username: 'root', capabilities: manage ? ['admins.manage', 'roles.manage', 'users.read'] : ['admins.read'], totpEnrolled: false })), sessions: vi.fn(async () => []), write: vi.fn(async () => ({})) as AdminAccountsApi['write'] };
}
function renderAccounts(api: AdminAccountsApi, initial = '/admin#/admins') {
  const [route, setRoute] = createSignal(parseAdminPath(initial));
  const view = renderInShell(() => <AdminAdmins service={api} />, { route, navigate: p => setRoute(parseAdminPath(p)) });
  return { ...view, route };
}
describe('admin accounts and roles', () => {
  it('assigns a role with explicit domain limits and the account version', async () => {
    const api = service(); renderAccounts(api);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Helpdesk' }));
    fireEvent.input(screen.getByRole('textbox', { name: /Helpdesk: Domains/ }), { target: { value: 'example.org, example.net' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save role assignments' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/admins/alice/roles', 'PUT', { roles: [{ roleId: 'helpdesk', domains: ['example.org', 'example.net'] }] }, '"v1"'));
  });
  it('keeps destructive actions behind confirmation and does not write on cancel', async () => {
    const api = service(); renderAccounts(api);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(api.write).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(api.write).not.toHaveBeenCalled();
  });
  it('shows readable accounts while disabling writes for read-only administrators', async () => {
    const api = service(false); const { container } = renderAccounts(api);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Create admin' })).not.toBeInTheDocument();
    await expectNoAxeViolations(container);
  });
  it('keeps account creation off the list and submits only from its dedicated page', async () => {
    const api = service(); const { route, container } = renderAccounts(api);
    await screen.findByRole('button', { name: 'Alice (alice)' });
    expect(screen.queryByRole('textbox', { name: 'Username' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Create admin' }));
    expect(route().rest).toEqual(['new']);
    expect(screen.queryByRole('button', { name: 'Alice (alice)' })).not.toBeInTheDocument();
    fireEvent.input(screen.getByRole('textbox', { name: 'Username' }), { target: { value: 'helpdesk-new' } });
    fireEvent.input(screen.getByRole('textbox', { name: 'Display name' }), { target: { value: 'Helpdesk new' } });
    fireEvent.input(screen.getByLabelText('New password (at least 12 characters)'), { target: { value: 'temporary-password' } });
    await expectNoAxeViolations(container);
    fireEvent.submit(screen.getByRole('form', { name: 'Create admin' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/admins', 'POST', { username: 'helpdesk-new', displayName: 'Helpdesk new', password: 'temporary-password', roles: [] }));
  });
  it('loads a linked edit page without showing the list or creation form', async () => {
    const api = service(); renderAccounts(api, '/admin#/admins/alice/edit');
    expect(await screen.findByRole('textbox', { name: 'Display name' })).toHaveValue('Alice');
    expect(screen.queryByRole('button', { name: 'Alice (alice)' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Create admin' })).not.toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Identity' })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Password and authenticator' })).toBeInTheDocument();
  });
  it('refuses deep-linked creation for readers and reports unknown accounts', async () => {
    const api = service(false); const { navigate } = renderAccounts(api, '/admin#/admins/new');
    await screen.findByText('You can view this account, but your role does not allow changes.');
    expect(screen.queryByRole('form')).not.toBeInTheDocument();
    await navigate('/admin#/admins/missing/edit');
    expect(await screen.findByRole('alert')).toHaveTextContent('Admin account not found.');
  });
  it('keeps built-in roles read-only and clones their permissions into a new role', async () => {
    const api = service();
    const [route, setRoute] = createSignal(parseAdminPath('/admin#/roles'));
    renderInShell(() => <AdminRoles service={api} />, { route, navigate: p => setRoute(parseAdminPath(p)) });
    fireEvent.click(await screen.findByRole('button', { name: 'Helpdesk' }));
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clone role' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Clone role' }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Display name' }), { target: { value: 'Regional support' } });
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(api.write).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Create role' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/roles', 'POST', { name: 'Regional support', capabilities: ['users.read'] }, undefined));
  });
});
