import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { DirectoryIntegration } from './DirectoryIntegration.tsx';
import type { DirectoryConfiguration, DirectoryWrite } from '../../api/admin/directory.ts';
vi.mock('../../i18n', () => ({ t: (key: string) => key, loadCatalog: async () => undefined }));
afterEach(cleanup);
const configuration = (): DirectoryConfiguration => ({ entries: [{ id: 'primary', url: 'ldaps://directory.example/', baseDn: 'dc=example', bindDn: 'cn=service', tls: 'ldaps', priority: 0, attrMap: {}, enabled: true, connectionMode: 'blocked', allowPrivate: false, password: { set: true }, revision: 'r1' }], etag: '"one"', activeRevision: '"old"', restartRequired: true, reload: 'restart', transportStatus: 'directExplicitOnly', environment: { bindDn: null, passwordSet: false, pinned: false } });
function setup(write = true, value = configuration()) { const service = { load: vi.fn<() => Promise<DirectoryConfiguration>>().mockResolvedValue(value), save: vi.fn<(entries: DirectoryWrite[], etag: string) => Promise<void>>().mockResolvedValue(undefined) }; render(() => <DirectoryIntegration canWrite={write} service={service} />); return service; }
describe('LDAP directory configuration', () => {
  it('submits connection consent and ETag while keeping secrets hidden and focus stable', async () => {
    const service = setup();
    const policy = await screen.findByLabelText('directory-connection');
    expect(screen.getByText('directory-consent')).toBeTruthy();
    expect(screen.queryByLabelText('directory-password')).toBeNull();
    fireEvent.change(policy, { target: { value: 'direct' } });
    const base = screen.getByLabelText('directory-base') as HTMLInputElement;
    base.focus(); fireEvent.input(base, { target: { value: 'dc=changed' } });
    expect(document.activeElement).toBe(base);
    fireEvent.submit(screen.getByRole('form'));
    await waitFor(() => expect(service.save).toHaveBeenCalledTimes(1));
    expect(service.save.mock.calls[0]).toEqual([[expect.objectContaining({ connectionMode: 'direct', baseDn: 'dc=changed', password: { action: 'keep' } })], '"one"']);
    expect(service.save.mock.calls[0]?.[0][0]).not.toHaveProperty('revision');
  });
  it('sends a new secret only after explicit replacement', async () => {
    const service = setup();
    fireEvent.change(await screen.findByLabelText('directory-secret-action'), { target: { value: 'set' } });
    fireEvent.input(screen.getByLabelText('directory-password'), { target: { value: 'new-secret' } });
    fireEvent.submit(screen.getByRole('form'));
    await waitFor(() => expect(service.save).toHaveBeenCalledTimes(1));
    expect(service.save.mock.calls[0]?.[0][0]?.password).toEqual({ action: 'set', value: 'new-secret' });
  });
  it('shows proxy blocking and environment pins', async () => {
    const value = configuration(); value.environment = { bindDn: 'cn=service', passwordSet: true, pinned: true }; value.transportStatus = 'proxyUnsupported';
    setup(true, value);
    expect(await screen.findByText('directory-proxy-blocked')).toBeTruthy();
    expect((screen.getByLabelText('directory-url') as HTMLInputElement).disabled).toBe(true);
    expect((screen.getByLabelText('directory-secret-action') as HTMLSelectElement).disabled).toBe(true);
  });
  it('read-only roles cannot submit', async () => {
    const service = setup(false); await screen.findByLabelText('directory-url');
    expect(screen.queryByText('directory-save')).toBeNull();
    fireEvent.submit(screen.getByRole('form')); expect(service.save).not.toHaveBeenCalled();
  });
});
