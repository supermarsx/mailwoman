import { describe, expect, it, vi, beforeEach } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { LoadTimePreferences } from './LoadTimePreferences.tsx';
import * as api from '../../api/settings.ts';

const entry = (value: boolean, locked = false): api.SettingEntry => ({ value, source: locked ? 'global' : 'default', locked });

describe('LoadTimePreferences', () => {
  beforeEach(() => vi.restoreAllMocks());

  it('offers both switches, on by default, and writes the person\'s choice', async () => {
    vi.spyOn(api, 'loadSettings').mockResolvedValue(undefined);
    vi.spyOn(api, 'settings').mockReturnValue({
      'observability.clientTiming.folderLoads': entry(true),
      'observability.clientTiming.messageLoads': entry(true),
    });
    const write = vi.spyOn(api, 'writeSetting').mockResolvedValue(entry(false));
    render(() => <LoadTimePreferences />);
    const folder = await screen.findByTestId('load-time-folder') as HTMLInputElement;
    const message = screen.getByTestId('load-time-message') as HTMLInputElement;
    expect(folder.checked).toBe(true);
    expect(message.checked).toBe(true);
    fireEvent.click(message);
    await waitFor(() => expect(write).toHaveBeenCalledWith('observability.clientTiming.messageLoads', false));
    expect(write).not.toHaveBeenCalledWith('observability.clientTiming.folderLoads', expect.anything());
  });

  it('disables a switch the administrator locked and says so', async () => {
    vi.spyOn(api, 'loadSettings').mockResolvedValue(undefined);
    vi.spyOn(api, 'settings').mockReturnValue({
      'observability.clientTiming.folderLoads': entry(false, true),
      'observability.clientTiming.messageLoads': entry(true),
    });
    render(() => <LoadTimePreferences />);
    const folder = await screen.findByTestId('load-time-folder') as HTMLInputElement;
    expect(folder.checked).toBe(false);
    expect(folder.disabled).toBe(true);
    expect((screen.getByTestId('load-time-message') as HTMLInputElement).disabled).toBe(false);
  });
});
