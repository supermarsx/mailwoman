import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { AdminPasskeys } from './AdminPasskeys.tsx';
const request = vi.hoisted(() => vi.fn());
vi.mock('./shell/adminApi.ts', () => ({ adminRequest: request }));
vi.mock('../Settings/webauthn.ts', () => ({ passkeySupported: () => true, registerPasskey: vi.fn() }));
describe('administrator passkey protection', () => {
  beforeEach(() => request.mockReset());
  it('disables removal when the server protects the last usable superadmin credential', async () => {
    request.mockResolvedValue({ data: { passkeys: [{ credentialId: 'one', label: 'Only key', backupEligible: false }], recoveryRemaining: 1, removalProtected: true } });
    render(() => <AdminPasskeys />);
    expect(await screen.findByRole('button', { name: 'Remove passkey' })).toBeDisabled();
    expect(screen.getByText(/last usable sign-in credential/)).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Add administrator passkey' })).toBeEnabled());
  });
});
