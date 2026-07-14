import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@solidjs/testing-library';
import { AdminAssist } from './index.tsx';
import { AdminAssistApi, DEFAULT_ADMIN_ASSIST_CONFIG, type AdminAssistConfig } from './service.ts';

afterEach(() => cleanup());

function apiWith(config: AdminAssistConfig): {
  api: AdminAssistApi;
  save: ReturnType<typeof vi.fn>;
  kill: ReturnType<typeof vi.fn>;
} {
  const api = new AdminAssistApi();
  const save = vi.fn(async () => undefined);
  const kill = vi.fn(async () => undefined);
  (api as unknown as { get: () => Promise<AdminAssistConfig> }).get = async () => config;
  (api as unknown as { save: typeof save }).save = save;
  (api as unknown as { setKillSwitch: typeof kill }).setKillSwitch = kill;
  return { api, save, kill };
}

describe('admin assist governance', () => {
  it('defaults to a DENY posture (off, empty allowlist, ceilings off)', () => {
    expect(DEFAULT_ADMIN_ASSIST_CONFIG.enabled).toBe(false);
    expect(DEFAULT_ADMIN_ASSIST_CONFIG.endpointAllowlist).toEqual([]);
    expect(DEFAULT_ADMIN_ASSIST_CONFIG.dataCeilings.includeE2ee).toBe(false);
    expect(DEFAULT_ADMIN_ASSIST_CONFIG.dataCeilings.includeAttachments).toBe(false);
  });

  it('toggles the kill switch through the API', async () => {
    const { api, kill } = apiWith({ ...DEFAULT_ADMIN_ASSIST_CONFIG, enabled: false });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(screen.getByLabelText('Enable Assist tenant-wide')).toBeInTheDocument());
    fireEvent.click(screen.getByLabelText('Enable Assist tenant-wide'));
    await waitFor(() => expect(kill).toHaveBeenCalledWith(true));
  });

  it('adds and removes endpoint hosts', async () => {
    const { api, save } = apiWith({ ...DEFAULT_ADMIN_ASSIST_CONFIG, enabled: true });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save policy' })).toBeEnabled());

    fireEvent.input(screen.getByLabelText('Endpoint host'), { target: { value: 'api.openai.com' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add host' }));
    await waitFor(() => expect(screen.getByTestId('allowlist-host')).toHaveTextContent('api.openai.com'));

    fireEvent.click(screen.getByRole('button', { name: 'Save policy' }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    const saved = save.mock.calls[0]?.[0] as AdminAssistConfig;
    expect(saved.endpointAllowlist).toContain('api.openai.com');

    fireEvent.click(screen.getByRole('button', { name: 'Remove api.openai.com' }));
    await waitFor(() => expect(screen.queryByTestId('allowlist-host')).not.toBeInTheDocument());
  });

  it('locks a capability (unchecking) and persists it on save', async () => {
    const { api, save } = apiWith({ ...DEFAULT_ADMIN_ASSIST_CONFIG, enabled: true });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save policy' })).toBeEnabled());

    fireEvent.click(screen.getByLabelText('Assistant (chat)')); // allowed -> locked
    fireEvent.click(screen.getByRole('button', { name: 'Save policy' }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    const saved = save.mock.calls[0]?.[0] as AdminAssistConfig;
    expect(saved.capabilityLocks.assistant).toBe('locked');
  });

  it('allows raising the data-class ceilings (E2EE / attachments) explicitly', async () => {
    const { api, save } = apiWith({ ...DEFAULT_ADMIN_ASSIST_CONFIG, enabled: true });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save policy' })).toBeEnabled());
    fireEvent.click(screen.getByLabelText('Allow end-to-end-encrypted content to be sent'));
    fireEvent.click(screen.getByRole('button', { name: 'Save policy' }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    const saved = save.mock.calls[0]?.[0] as AdminAssistConfig;
    expect(saved.dataCeilings.includeE2ee).toBe(true);
    expect(saved.dataCeilings.includeAttachments).toBe(false);
  });
});
