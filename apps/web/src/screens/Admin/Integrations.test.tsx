import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Integrations } from './Integrations.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { ApiKeyInfo } from '../../state/slices/admin.ts';

const KEY: ApiKeyInfo = {
  id: 'k1',
  prefix: 'mwk_abc123',
  accountId: 'alice@example.com',
  scopesJson: '{"read":true}',
  createdAt: '2026-07-14T00:00:00Z',
  lastUsedAt: null,
  expiresAt: null,
  revokedAt: null,
};

describe('Admin › Integrations', () => {
  // Status strings are the server's: `IntegrationStatus::as_str` in
  // crates/mw-admin/src/provisioning.rs, sent by `get_integrations` in
  // crates/mw-server/src/admin.rs.
  it('renders the status the server reports for LDAP and Nextcloud', async () => {
    renderWithAdmin(
      () => <Integrations />,
      mockAdminApi({
        getIntegrations: vi.fn(async () => ({
          webhooks: 'active',
          apiKeyOversight: 'active',
          ldap: 'configured',
          nextcloud: 'not-configured',
        })),
      }),
    );
    await waitFor(() => expect(screen.getByTestId('integration-ldap').textContent).toBe('Configured'));
    expect(screen.getByTestId('integration-nextcloud').textContent).toBe('Not configured');
  });

  it('says "status unknown" when the server says unknown', async () => {
    renderWithAdmin(
      () => <Integrations />,
      mockAdminApi({
        getIntegrations: vi.fn(async () => ({
          webhooks: 'active',
          apiKeyOversight: 'active',
          ldap: 'unknown',
          nextcloud: 'unknown',
        })),
      }),
    );
    expect(await screen.findByText('LDAP / GAL directory')).toBeInTheDocument();
    await Promise.resolve();
    expect(screen.getByTestId('integration-ldap').textContent).toBe('Status unknown');
    expect(screen.getByTestId('integration-nextcloud').textContent).toBe('Status unknown');
  });

  it('treats a status it does not recognise as unknown, never as a known state', async () => {
    renderWithAdmin(
      () => <Integrations />,
      mockAdminApi({
        getIntegrations: vi.fn(async () => ({
          webhooks: 'active',
          apiKeyOversight: 'active',
          ldap: 'deferred',
          nextcloud: 'somethingAddedLater',
        })),
      }),
    );
    await waitFor(() => expect(screen.getByTestId('integration-ldap').getAttribute('data-status')).toBe('deferred'));
    expect(screen.getByTestId('integration-ldap').textContent).toBe('Status unknown');
    expect(screen.getByTestId('integration-nextcloud').textContent).toBe('Status unknown');
  });

  it('claims nothing when the status request fails', async () => {
    renderWithAdmin(
      () => <Integrations />,
      mockAdminApi({
        getIntegrations: vi.fn(async () => {
          throw new Error('boom');
        }),
      }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not load integrations');
    expect(screen.getByTestId('integration-ldap').textContent).toBe('Status unknown');
    expect(screen.getByTestId('integration-nextcloud').textContent).toBe('Status unknown');
  });

  it('never says "Deferred"', async () => {
    renderWithAdmin(() => <Integrations />);
    expect(await screen.findByText('Nextcloud bridge')).toBeInTheDocument();
    await Promise.resolve();
    expect(screen.queryByText('Deferred')).toBeNull();
    expect(screen.queryByText(/not yet wired/)).toBeNull();
  });

  it('lists API/MCP keys and revokes one', async () => {
    const revokeApiKey = vi.fn(async () => undefined);
    renderWithAdmin(
      () => <Integrations />,
      mockAdminApi({ listApiKeys: vi.fn(async () => [KEY]), revokeApiKey }),
    );
    expect(await screen.findByText('mwk_abc123')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Revoke key mwk_abc123' }));
    await Promise.resolve();
    expect(revokeApiKey).toHaveBeenCalledWith('k1');
  });
});
