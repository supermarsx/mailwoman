import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { AdminAccountMenu } from './AdminAccountMenu.tsx';

function setup(onSignOut = vi.fn()) {
  render(() => (
    <AdminAccountMenu variant="full" placement="up" username="admin" onSignOut={onSignOut} languagePicker={<select aria-label="Language"><option>en</option></select>} />
  ));
  return { onSignOut, button: screen.getByRole('button', { name: /^Account menu for/ }) };
}

describe('AdminAccountMenu', () => {
  it('shows initials in the webmail avatar colour and is closed by default', () => {
    const { button } = setup();
    expect(button).toHaveAttribute('aria-haspopup', 'true');
    expect(button).toHaveAttribute('aria-expanded', 'false');
    const avatar = button.querySelector('.person-avatar')!;
    expect(avatar.textContent).toBe('A');
    expect(avatar.getAttribute('data-avatar-color')).not.toBeNull();
    expect(screen.queryByRole('button', { name: 'Sign out' })).toBeNull();
  });

  it('opens with the signed-in line, language picker and Sign out; Escape closes and returns focus', async () => {
    const { button } = setup();
    fireEvent.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByText(/Signed in as/)).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Language' })).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('combobox', { name: 'Language' })).toHaveFocus());
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    fireEvent.keyDown(document.activeElement!, { key: 'End' });
    expect(screen.getByRole('button', { name: 'Sign out' })).toHaveFocus();
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByRole('button', { name: 'Sign out' })).toBeNull();
    expect(button).toHaveFocus();
  });

  it('Sign out calls back and closes the menu', () => {
    const { button, onSignOut } = setup();
    fireEvent.click(button);
    fireEvent.click(screen.getByRole('button', { name: 'Sign out' }));
    expect(onSignOut).toHaveBeenCalledOnce();
    expect(screen.queryByRole('button', { name: 'Sign out' })).toBeNull();
  });
});
