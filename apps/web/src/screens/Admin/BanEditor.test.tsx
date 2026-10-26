import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { BanEditor } from './BanEditor.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import { anyDirty } from './shell/useDirtyGuard.ts';

describe('Dedicated IP ban editor', () => {
  it('saves an explicit permanent ban and clears the leave guard only after acknowledgement', async () => {
    const addBan = vi.fn(async () => undefined);
    const done = vi.fn();
    renderWithAdmin(() => <BanEditor onSaved={done} />, mockAdminApi({ addBan }));
    fireEvent.input(screen.getByLabelText('IP address'), { target: { value: '192.0.2.5' } });
    expect(anyDirty()).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Save IP ban' }));
    await waitFor(() => expect(done).toHaveBeenCalledOnce());
    expect(addBan).toHaveBeenCalledWith({ ip: '192.0.2.5', reason: '', expiresAt: null });
    expect(anyDirty()).toBe(false);
  });
  it('refuses a past expiry without dispatching and preserves the draft', async () => {
    const addBan = vi.fn();
    renderWithAdmin(() => <BanEditor />, mockAdminApi({ addBan }));
    fireEvent.input(screen.getByLabelText('IP address'), { target: { value: '192.0.2.5' } });
    fireEvent.input(screen.getByLabelText("Expiry (this device's local time)"), { target: { value: '2000-01-01T12:00' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save IP ban' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('future expiry');
    expect(addBan).not.toHaveBeenCalled(); expect(anyDirty()).toBe(true);
  });
  it('sends the local expiry as an instant and retains errors without losing edits', async () => {
    const addBan = vi.fn(async () => { throw new Error('Access refused'); });
    renderWithAdmin(() => <BanEditor />, mockAdminApi({ addBan }));
    fireEvent.input(screen.getByLabelText('IP address'), { target: { value: '192.0.2.5' } });
    fireEvent.input(screen.getByLabelText("Expiry (this device's local time)"), { target: { value: '2099-01-01T12:00' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save IP ban' }));
    await screen.findByRole('alert');
    expect(addBan).toHaveBeenCalledWith({ ip: '192.0.2.5', reason: '', expiresAt: new Date('2099-01-01T12:00').toISOString() });
    expect(screen.getByLabelText('IP address')).toHaveValue('192.0.2.5'); expect(anyDirty()).toBe(true);
  });
});
