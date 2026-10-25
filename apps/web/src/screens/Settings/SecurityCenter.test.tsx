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
  fireEvent.change(screen.getByLabelText('Result'), { target: { value: 'failure' } });
  await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining('result=failure'), expect.anything()));
  fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
  await waitFor(() => expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining('remembered-devices/revoke'), expect.objectContaining({ method: 'POST', body: JSON.stringify({ id: 'device-1' }) })));
});
