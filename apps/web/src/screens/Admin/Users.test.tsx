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
    renderWithAdmin(() => <Users />, mockAdminApi({ provisionUser }));
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
      () => <Users />,
      mockAdminApi({ listUsers: vi.fn(async () => [U]), toggleZeroAccess, setFlags }),
    );
    const box = await screen.findByLabelText('Zero-access for alice@example.com');
    fireEvent.change(box, { target: { checked: true } });
    await Promise.resolve();
    expect(toggleZeroAccess).toHaveBeenCalledWith('alice@example.com', true);
    expect(setFlags).not.toHaveBeenCalled();
  });

  it('says what "disabled" and "force change" do, and ties each note to its checkbox', async () => {
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers: vi.fn(async () => [U]) }));

    const disabled = await screen.findByLabelText('Disable alice@example.com');
    const disabledHelp = document.getElementById(disabled.getAttribute('aria-describedby') ?? '');
    expect(disabledHelp).toHaveTextContent('Blocks sign-in');
    expect(disabledHelp).toHaveTextContent('API keys and tokens');
    expect(disabledHelp).toHaveTextContent('does not disable the mailbox on the mail server');

    const force = screen.getByLabelText('Force password change for alice@example.com');
    const forceHelp = document.getElementById(force.getAttribute('aria-describedby') ?? '');
    expect(forceHelp).toHaveTextContent('held at a password-change screen');
    expect(forceHelp).toHaveTextContent('MW_PASSWD_BACKEND');
    expect(forceHelp).toHaveTextContent('until you clear this box');
  });

  it('setting "disabled" sends the flag through setFlags', async () => {
    const setFlags = vi.fn(async () => undefined);
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers: vi.fn(async () => [U]), setFlags }));
    fireEvent.change(await screen.findByLabelText('Disable alice@example.com'), { target: { checked: true } });
    await Promise.resolve();
    expect(setFlags).toHaveBeenCalledWith('alice@example.com', { ...U.flags, disabled: true });
  });

  it('revokes sessions', async () => {
    const revokeSessions = vi.fn(async () => 3);
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers: vi.fn(async () => [U]), revokeSessions }));
    fireEvent.click(await screen.findByRole('button', { name: 'Revoke sessions for alice@example.com' }));
    await Promise.resolve();
    expect(revokeSessions).toHaveBeenCalledWith('alice@example.com');
  });
});
