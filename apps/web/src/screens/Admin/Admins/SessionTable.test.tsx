import { render, screen, fireEvent } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { AdminSessionTable } from './SessionTable.tsx';
it('shows real deadlines, marks the current session and respects revoke permissions', () => {
  const revoke = vi.fn();
  render(() => <AdminSessionTable sessions={[{ id: 'opaque', adminId: 'root', createdAt: '2026-10-25', idleExpiresAt: 1800000000, expiresAt: 1800003600, current: true }]} accounts={[]} busy={false} canRevoke={false} onRevoke={revoke} />);
  expect(screen.getByRole('table', { name: 'Administrator sessions' })).toBeInTheDocument();
  expect(screen.getByText('This session')).toBeInTheDocument();
  expect(screen.getByText(new Date(1800000000 * 1000).toLocaleString())).toBeInTheDocument();
  const button = screen.getByRole('button', { name: 'Revoke session for root' });
  expect(button).toBeDisabled(); fireEvent.click(button); expect(revoke).not.toHaveBeenCalled();
});
