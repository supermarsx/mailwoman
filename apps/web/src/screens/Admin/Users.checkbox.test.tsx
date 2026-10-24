import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Users } from './Users.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { UserSummary } from '../../state/slices/admin.ts';

// t32-h12: the flag checkboxes in the Users table keep keyboard focus across the reload
// that follows a change, and always show what the server stored.

const U: UserSummary = {
  accountId: 'alice@example.com',
  username: 'alice',
  domain: 'example.com',
  quota: { bytesLimit: 1000, msgLimit: 50 },
  flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
};

function tick(box: HTMLInputElement, checked: boolean): void {
  box.checked = checked;
  fireEvent.change(box);
}

describe('Admin › Users › flag checkboxes', () => {
  it('keeps the same checkbox, focused, after the reload that follows a change', async () => {
    let stored = U;
    const listUsers = vi.fn(async () => [stored]);
    const setFlags = vi.fn(async (_id: string, flags: UserSummary['flags']) => {
      stored = { ...stored, flags };
    });
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers, setFlags }));
    const box = (await screen.findByLabelText('Disable alice@example.com')) as HTMLInputElement;
    box.focus();
    tick(box, true);
    await waitFor(() => expect(listUsers).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(screen.getByLabelText('Disable alice@example.com')).toBe(box));
    expect(document.activeElement).toBe(box);
    expect(box.checked).toBe(true);
  });

  it('puts the box back when the change fails', async () => {
    const setFlags = vi.fn(async () => {
      throw new Error('refused');
    });
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers: vi.fn(async () => [U]), setFlags }));
    const box = (await screen.findByLabelText('Force password change for alice@example.com')) as HTMLInputElement;
    tick(box, true);
    await waitFor(() => expect(setFlags).toHaveBeenCalled());
    await waitFor(() => expect(box.checked).toBe(false));
  });

  it('shows the stored value when the server keeps the old one', async () => {
    // The server answers 200 but the reload still reports zero-access off.
    const listUsers = vi.fn(async () => [U]);
    const toggleZeroAccess = vi.fn(async () => undefined);
    renderWithAdmin(() => <Users />, mockAdminApi({ listUsers, toggleZeroAccess }));
    const box = (await screen.findByLabelText('Zero-access for alice@example.com')) as HTMLInputElement;
    tick(box, true);
    await waitFor(() => expect(listUsers).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(box.checked).toBe(false));
  });
});
