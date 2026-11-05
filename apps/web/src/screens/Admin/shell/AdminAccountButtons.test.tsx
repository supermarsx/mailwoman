import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { AdminAccountButtons } from './AdminAccountButtons.tsx';

describe('AdminAccountButtons', () => {
  it('shows a user square and a Sign out square side by side, with no menu', () => {
    render(() => <AdminAccountButtons username="admin" onSignOut={vi.fn()} />);
    const user = screen.getByRole('button', { name: /Signed in as/ });
    const out = screen.getByRole('button', { name: 'Sign out' });
    expect(user.parentElement).toBe(out.parentElement);
    expect(user).not.toHaveAttribute('aria-haspopup');
    expect(user).not.toHaveAttribute('aria-expanded');
    const avatar = user.querySelector('.person-avatar')!;
    expect(avatar.textContent).toBe('A');
    expect(avatar.getAttribute('data-avatar-color')).not.toBeNull();
    expect(user).toHaveAttribute('title');
  });

  it('Sign out calls back directly', () => {
    const onSignOut = vi.fn();
    render(() => <AdminAccountButtons username="admin" onSignOut={onSignOut} />);
    fireEvent.click(screen.getByRole('button', { name: 'Sign out' }));
    expect(onSignOut).toHaveBeenCalledOnce();
  });

  it('the user square is a link to the account page when an href is given', () => {
    const onOpen = vi.fn((e: MouseEvent) => e.preventDefault());
    render(() => <AdminAccountButtons username="admin" onSignOut={vi.fn()} href="/admin/preferences" onOpenAccount={onOpen} />);
    const link = screen.getByRole('link', { name: /Signed in as/ });
    expect(link).toHaveAttribute('href', '/admin/preferences');
    fireEvent.click(link);
    expect(onOpen).toHaveBeenCalledOnce();
  });
});
