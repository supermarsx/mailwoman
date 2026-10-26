import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Users } from './Users.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { UserSummary } from '../../state/slices/admin.ts';

const U: UserSummary = {
  accountId: 'alice@example.com',
  username: 'alice',
  domain: 'example.com',
  quota: { bytesLimit: 1000, msgLimit: 50 },
  flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
};

describe('Admin › Users', () => {
  it('lists provisioned users with quota', async () => {
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers: vi.fn(async () => [U]) }));
    expect(await screen.findByText('alice@example.com')).toBeInTheDocument();
    expect(screen.getByText('1000 / 50')).toBeInTheDocument();
  });

  it('provisions a user', async () => {
    const provisionUser = vi.fn(async () => undefined);
    renderWithAdmin(() => <Users provision />, mockAdminApi({ provisionUser }));
    const form = screen.getByRole('form', { name: 'Provision user' });
    fireEvent.input(screen.getByPlaceholderText('example.com'), { target: { value: 'example.com' } });
    const inputs = form.querySelectorAll('input');
    fireEvent.input(inputs[0]!, { target: { value: 'bob' } });
    fireEvent.submit(form);
    await Promise.resolve();
    expect(provisionUser).toHaveBeenCalledWith(
      expect.objectContaining({ username: 'bob', domain: 'example.com' }),
    );
  });

  it('toggling zero-access calls toggleZeroAccess (not setFlags)', async () => {
    const toggleZeroAccess = vi.fn(async () => undefined);
    const setFlags = vi.fn(async () => undefined);
    renderWithAdmin(
      () => <Users accountId={U.accountId} />,
      mockAdminApi({ listUsers: vi.fn(async () => [U]), toggleZeroAccess, setFlags }),
    );
    const box = await screen.findByLabelText('Zero-access for alice@example.com');
    fireEvent.change(box, { target: { checked: true } });
    await Promise.resolve();
    expect(toggleZeroAccess).toHaveBeenCalledWith('alice@example.com', true);
    expect(setFlags).not.toHaveBeenCalled();
  });

  it('says what "disabled" and "force change" do, and ties each note to its checkbox', async () => {
    renderWithAdmin(() => <Users accountId={U.accountId} />, mockAdminApi({ listUsers: vi.fn(async () => [U]) }));

    expect(await screen.findByRole('button',{name:'Block login for alice@example.com'})).toBeInTheDocument();
    expect(screen.queryByLabelText('Disable alice@example.com')).not.toBeInTheDocument();

    const force = screen.getByLabelText('Force password change for alice@example.com');
    const forceHelp = document.getElementById(force.getAttribute('aria-describedby') ?? '');
    expect(forceHelp).toHaveTextContent('held at a password-change screen');
    expect(forceHelp).toHaveTextContent('MW_PASSWD_BACKEND');
    expect(forceHelp).toHaveTextContent('until you clear this box');
  });

  it('uses the lifecycle action instead of the legacy disabled checkbox', async () => {
    const setFlags = vi.fn(async () => undefined);
    renderWithAdmin(() => <Users accountId={U.accountId} />, mockAdminApi({listUsers:vi.fn(async()=>[U]),setFlags}));
    expect(await screen.findByRole('button',{name:'Block login for alice@example.com'})).toBeInTheDocument();
    expect(screen.queryByLabelText('Disable alice@example.com')).not.toBeInTheDocument();
    expect(setFlags).not.toHaveBeenCalled();
  });

  it('revokes sessions', async () => {
    const revokeSessions = vi.fn(async () => 3);
    renderWithAdmin(() => <Users accountId={U.accountId} />, mockAdminApi({ listUsers: vi.fn(async () => [U]), revokeSessions }));
    fireEvent.click(await screen.findByRole('button', { name: 'Revoke sessions for alice@example.com' }));
    await Promise.resolve();
    expect(revokeSessions).toHaveBeenCalledWith('alice@example.com');
  });
});
