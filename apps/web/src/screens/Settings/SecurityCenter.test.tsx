import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { SecurityCenter } from './SecurityCenter.tsx';
import { apiFetch } from '../../api/http.ts';
vi.mock('../../api/http.ts', () => ({ apiFetch: vi.fn() }));
vi.mock('./TwoFactor.tsx', () => ({ TwoFactor: () => null }));
vi.mock('./Sessions.tsx', () => ({ Sessions: () => null }));
it('filters history and revokes the selected remembered device', async () => {
  vi.mocked(apiFetch).mockImplementation(async (path) => new Response(JSON.stringify(path.includes('remembered-devices') ? { devices: [{ id: 'device-1', label: 'My laptop', expiresAt: 2000000000 }] } : { events: [] }), { status: 200 }));
  render(() => <SecurityCenter />);
  expect(await screen.findByText(/My laptop/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('tab', { name: 'Sign-in history' }));
  fireEvent.change(screen.getByLabelText('Result'), { target: { value: 'failure' } });
  await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining('result=failure'), expect.anything()));
  fireEvent.click(screen.getByRole('tab', { name: 'Remembered devices' }));
  fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
  await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining('remembered-devices/revoke'), expect.objectContaining({ method: 'POST', body: JSON.stringify({ id: 'device-1' }) })));
});

it('keeps security sections in labelled keyboard-navigable tabs', () => {
  vi.mocked(apiFetch).mockImplementation(async () => new Response(JSON.stringify({ events: [], devices: [] }), {status:200}));
  render(() => <SecurityCenter password={<p>Password settings</p>} encryption={<p>Encryption settings</p>} />);
  const signin = screen.getByRole('tab', {name:'Sign-in & passkeys'});
  expect(signin).toHaveAttribute('aria-selected', 'true');
  expect(screen.getByText('Password settings')).toBeVisible();
  expect(screen.getByText('Encryption settings')).not.toBeVisible();
  fireEvent.keyDown(signin, {key:'End'});
  expect(screen.getByRole('tab', {name:'Encryption'})).toHaveFocus();
  expect(screen.getByText('Encryption settings')).toBeVisible();
  expect(screen.getByText('Password settings')).not.toBeVisible();
});
