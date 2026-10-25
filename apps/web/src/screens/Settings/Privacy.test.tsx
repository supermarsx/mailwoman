import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { Privacy } from './Privacy.tsx';
import { clearSettings } from '../../api/settings.ts';
import { remotePolicy } from '../../api/remote-content.ts';
vi.mock('../../api/remote-content.ts', () => ({ remotePolicy: vi.fn() }));
const free = (value: unknown, locked = false) => ({ value, source: 'global', locked, userValue: value });
let writes: unknown[]; let held = false;
beforeEach(() => {
  clearSettings(); writes = []; held = false;
  vi.mocked(remotePolicy).mockResolvedValue({ mode: 'ask', trackers: true, cleanLinks: true, allowDirect: false, locked: { mode: false, trackers: false, cleanLinks: false } });
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const body = init?.method === 'PUT' ? JSON.parse(String(init.body)) as {value:unknown} : null;
    if (body) { writes.push({ url, value: body.value }); return new Response(JSON.stringify(free(body.value))); }
    return new Response(JSON.stringify({ settings: { 'privacy.remoteContent': free('ask', held), 'privacy.trackers': free(true, held), 'privacy.cleanLinks': free(true, held) } }));
  }));
});
describe('Privacy settings', () => {
  it('honors locks and does not offer direct mode without server permission', async () => {
    held = true; render(() => <Privacy />);
    const mode = await screen.findByLabelText('Remote content'); await waitFor(() => expect(mode).toBeDisabled());
    expect(screen.getByLabelText('Block tracking images and beacons')).toBeDisabled();
    expect(screen.getByLabelText('Remove tracking parameters from links')).toBeDisabled();
    expect(screen.getByRole('option', { name: "Load directly from the sender's servers" })).toBeDisabled();
    expect(writes).toEqual([]);
  });
  it('writes through the settings policy API and preserves unrelated preferences', async () => {
    render(() => <Privacy />);
    const mode = await screen.findByLabelText('Remote content'); await waitFor(() => expect(mode).toBeEnabled());
    fireEvent.change(mode, { target: { value: 'proxy' } });
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/privacy.remoteContent', value: 'proxy' }));
    fireEvent.click(screen.getByLabelText('Block tracking images and beacons'));
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/privacy.trackers', value: false }));
    expect(screen.getByLabelText('Remove tracking parameters from links')).toBeChecked();
  });
});
