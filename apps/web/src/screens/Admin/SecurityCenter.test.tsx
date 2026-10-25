import { render, screen } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { AdminSecurityCenter } from './SecurityCenter.tsx';
import { adminRequest } from './shell/adminApi.ts';
vi.mock('./shell/adminApi.ts', () => ({ adminRequest: vi.fn() }));
vi.mock('./TwoFactorPolicy.tsx', () => ({ TwoFactorPolicy: () => null }));
it('shows measured factor coverage and links each policy to its scoped editor', async () => {
  vi.mocked(adminRequest).mockResolvedValue({ status: 200, etag: null, data: { knownAccounts: 10, accountsWithFactor: 4, adminsWithoutFactor: 2, staleApiKeys: 3, upstreamAllowlistConfigured: true, environmentAdminLoginEnabled: false } });
  render(() => <AdminSecurityCenter />);
  expect(await screen.findByText('Accounts with an authentication factor: 4 of 10 known accounts')).toBeInTheDocument();
  expect(screen.getByRole('link', { name: 'session.maxConcurrent' })).toHaveAttribute('href', '#/settings/session.maxConcurrent');
  expect(screen.getByText('Environment administrator sign-in is disabled')).toBeInTheDocument();
});
