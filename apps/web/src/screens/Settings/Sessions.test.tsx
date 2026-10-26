import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { Sessions } from './Sessions.tsx';
import { SettingsService } from './service.ts';
it('keeps the current session safe and targets the selected device handle', async () => {
  const service = new SettingsService();
  vi.spyOn(service, 'sessions').mockResolvedValue([
    { handle: 'current', username: 'alice@example.org', createdAt: 'today', lastSeen: 'now', current: true, deviceLabel: 'This laptop' },
    { handle: 'other', username: 'alice@example.org', createdAt: 'yesterday', lastSeen: 'background poll', lastActivity: 'user interaction', current: false, deviceLabel: 'Phone', ip: '192.0.2.1', country: 'GB' },
  ]);
  const revoke = vi.spyOn(service, 'revokeSession').mockResolvedValue(undefined);
  render(() => <Sessions service={service} />);
  await screen.findByText('This laptop');
  expect(screen.getByRole('table', { name: 'Active sessions' })).toBeInTheDocument();
  expect(screen.getByText('Last active user interaction')).toBeInTheDocument();
  expect(screen.queryByText('Last active background poll')).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Sign out This laptop' })).not.toBeInTheDocument();
  fireEvent.input(screen.getByRole('searchbox'), { target: { value: '192.0.2.1' } });
  expect(screen.queryByText('This laptop')).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Sign out Phone' }));
  await waitFor(() => expect(revoke).toHaveBeenCalledWith('other'));
});
it('renders a bounded error without throwing while session loading fails', async () => {
  const service = new SettingsService();
  vi.spyOn(service, 'sessions').mockRejectedValue(new Error('private backend failure'));
  render(() => <Sessions service={service} />);
  expect(await screen.findByRole('alert')).toHaveTextContent('The session could not be changed.');
  expect(screen.queryByText('private backend failure')).not.toBeInTheDocument();
});
