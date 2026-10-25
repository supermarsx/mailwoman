import { describe, it, expect, vi } from 'vitest';
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
describe('admin accounts and roles', () => {
  it('assigns a role with explicit domain limits and the account version', async () => {
    const api = service(); renderInShell(() => <AdminAdmins service={api} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Helpdesk' }));
    fireEvent.input(screen.getByRole('textbox', { name: /Helpdesk: Domains/ }), { target: { value: 'example.org, example.net' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save role assignments' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/admins/alice/roles', 'PUT', { roles: [{ roleId: 'helpdesk', domains: ['example.org', 'example.net'] }] }, '"v1"'));
  });
  it('keeps destructive actions behind confirmation and does not write on cancel', async () => {
    const api = service(); renderInShell(() => <AdminAdmins service={api} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(api.write).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(api.write).not.toHaveBeenCalled();
  });
  it('shows readable accounts while disabling writes for read-only administrators', async () => {
    const api = service(false); const { container } = renderInShell(() => <AdminAdmins service={api} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Alice (alice)' }));
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Create admin' })).not.toBeInTheDocument();
    await expectNoAxeViolations(container);
  });
  it('keeps built-in roles read-only and clones their permissions into a new role', async () => {
    const api = service(); renderInShell(() => <AdminRoles service={api} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Helpdesk' }));
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clone role' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Clone role' }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Display name' }), { target: { value: 'Regional support' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(api.write).toHaveBeenCalledWith('/roles', 'POST', { name: 'Regional support', capabilities: ['users.read'] }, undefined));
  });
});
