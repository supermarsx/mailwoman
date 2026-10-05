import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Integrations } from './Integrations.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import { AdminApiError, type ApiKeyInfo } from '../../state/slices/admin.ts';

const KEY: ApiKeyInfo = {
  id: 'k1',
  prefix: 'mwk_abc123',
  accountId: 'alice@example.com',
  scopesJson: '{"read":true}',
  createdAt: '2026-07-14T00:00:00Z',
  lastUsedAt: null,
  expiresAt: null,
  revokedAt: null,
  unattendedSendRequested: false,
  unattendedSendApproved: false,
};

// Rows as `list_api_keys` sends them (crates/mw-server/src/admin.rs:617):
// `scopesJson` is the `mw-oauth` `Scope` JSON, `unattendedSendRequested` its
// `unattended_send` member, `unattendedSendApproved` the `api_keys` column.
const SEND_SCOPE = JSON.stringify({
  read: true,
  send: true,
  delete: false,
  accounts: { subset: ['alice@example.com'] },
  folders: 'all',
  mail: true,
  pim: false,
  ip_allowlist: [],
  expires_at: null,
  rate_limit: null,
  mcp_tools: ['mail.search', 'mail.send'],
  unattended_send: true,
});
const REQUESTED: ApiKeyInfo = {
  ...KEY,
  id: 'k2',
  prefix: 'mwk_req456',
  scopesJson: SEND_SCOPE,
  unattendedSendRequested: true,
};
const APPROVED: ApiKeyInfo = { ...REQUESTED, id: 'k3', prefix: 'mwk_app789', unattendedSendApproved: true };

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

  describe('unattended send', () => {
    it('shows each key state and offers a control only where one applies', async () => {
      const revoked: ApiKeyInfo = { ...APPROVED, id: 'k4', prefix: 'mwk_rev000', revokedAt: '2026-10-01T00:00:00Z' };
      renderWithAdmin(
        () => <Integrations />,
        mockAdminApi({ listApiKeys: vi.fn(async () => [KEY, REQUESTED, APPROVED, revoked]) }),
      );
      expect(await screen.findByTestId('unattended-state-k1')).toHaveTextContent('Not requested');
      expect(screen.getByTestId('unattended-state-k2')).toHaveTextContent('Requested, not approved');
      expect(screen.getByTestId('unattended-state-k3')).toHaveTextContent('Approved');
      expect(screen.getByTestId('unattended-state-k4')).toHaveTextContent('Approved');

      // A key that did not ask cannot be approved; a revoked key has no control.
      expect(screen.queryByRole('button', { name: /unattended send for key mwk_abc123/ })).toBeNull();
      expect(screen.queryByRole('button', { name: /unattended send for key mwk_rev000/ })).toBeNull();
      expect(screen.getByRole('button', { name: 'Approve unattended send for key mwk_req456' })).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: 'Withdraw approval of unattended send for key mwk_app789' }),
      ).toBeInTheDocument();
    });

    it('approving asks first, naming the owner, the key, its scope and what approval means', async () => {
      const setApiKeyUnattendedSend = vi.fn(async () => undefined);
      const listApiKeys = vi
        .fn<() => Promise<ApiKeyInfo[]>>()
        .mockResolvedValueOnce([REQUESTED])
        .mockResolvedValue([{ ...REQUESTED, unattendedSendApproved: true }]);
      renderWithAdmin(() => <Integrations />, mockAdminApi({ listApiKeys, setApiKeyUnattendedSend }));

      fireEvent.click(await screen.findByRole('button', { name: 'Approve unattended send for key mwk_req456' }));
      const dialog = screen.getByRole('dialog');
      // Opening the dialog sends nothing.
      expect(setApiKeyUnattendedSend).not.toHaveBeenCalled();
      expect(dialog).toHaveTextContent('Approve unattended send for this key?');
      expect(dialog).toHaveTextContent('Owner: alice@example.com');
      expect(dialog).toHaveTextContent('Key: mwk_req456');
      expect(dialog).toHaveTextContent('Permissions: read, send, mail. MCP tools: mail.search, mail.send.');
      expect(dialog).toHaveTextContent(
        'Messages sent with this key through MCP are transmitted without a person releasing them.',
      );
      expect(dialog).toHaveTextContent('The approval takes effect when the server restarts.');

      fireEvent.click(screen.getByTestId('admin-unattended-confirm'));
      await waitFor(() => expect(screen.getByTestId('unattended-state-k2')).toHaveTextContent('Approved'));
      expect(setApiKeyUnattendedSend).toHaveBeenCalledTimes(1);
      expect(setApiKeyUnattendedSend).toHaveBeenCalledWith('k2', true);
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(screen.getByRole('status')).toHaveTextContent(
        'Unattended send approved for key mwk_req456. It takes effect when the server restarts.',
      );
    });

    it('cancelling sends nothing', async () => {
      const setApiKeyUnattendedSend = vi.fn(async () => undefined);
      renderWithAdmin(
        () => <Integrations />,
        mockAdminApi({ listApiKeys: vi.fn(async () => [REQUESTED]), setApiKeyUnattendedSend }),
      );
      fireEvent.click(await screen.findByRole('button', { name: 'Approve unattended send for key mwk_req456' }));
      fireEvent.click(screen.getByTestId('admin-unattended-cancel'));
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(setApiKeyUnattendedSend).not.toHaveBeenCalled();
      expect(screen.getByTestId('unattended-state-k2')).toHaveTextContent('Requested, not approved');
    });

    it('withdrawing asks first and says the key keeps sending until the restart', async () => {
      const setApiKeyUnattendedSend = vi.fn(async () => undefined);
      const listApiKeys = vi
        .fn<() => Promise<ApiKeyInfo[]>>()
        .mockResolvedValueOnce([APPROVED])
        .mockResolvedValue([{ ...APPROVED, unattendedSendApproved: false }]);
      renderWithAdmin(() => <Integrations />, mockAdminApi({ listApiKeys, setApiKeyUnattendedSend }));

      fireEvent.click(
        await screen.findByRole('button', { name: 'Withdraw approval of unattended send for key mwk_app789' }),
      );
      const dialog = screen.getByRole('dialog');
      expect(dialog).toHaveTextContent('Withdraw the approval for this key?');
      expect(dialog).toHaveTextContent(
        "Messages sent with this key through MCP wait in the owner's Outbox until a person releases them.",
      );
      expect(dialog).toHaveTextContent(
        'The withdrawal takes effect when the server restarts. Until then this key still sends without release.',
      );

      fireEvent.click(screen.getByTestId('admin-unattended-confirm'));
      await waitFor(() =>
        expect(screen.getByTestId('unattended-state-k3')).toHaveTextContent('Requested, not approved'),
      );
      expect(setApiKeyUnattendedSend).toHaveBeenCalledWith('k3', false);
      expect(screen.getByRole('status')).toHaveTextContent('Approval withdrawn for key mwk_app789.');
    });

    // The refusals of `set_key_unattended_send` (crates/mw-server/src/oauth.rs:441).
    it.each([
      [404, 'Key mwk_req456 no longer exists or was revoked. Nothing was changed.'],
      [409, 'Key mwk_req456 is revoked or its owner did not request unattended send. Nothing was approved.'],
      [500, 'Could not change the approval for key mwk_req456.'],
    ])('reports a %i from the server and claims no approval', async (status, message) => {
      const setApiKeyUnattendedSend = vi.fn(async () => {
        throw new AdminApiError(status, 'refused');
      });
      const listApiKeys = vi.fn(async () => [REQUESTED]);
      renderWithAdmin(() => <Integrations />, mockAdminApi({ listApiKeys, setApiKeyUnattendedSend }));

      fireEvent.click(await screen.findByRole('button', { name: 'Approve unattended send for key mwk_req456' }));
      fireEvent.click(screen.getByTestId('admin-unattended-confirm'));
      expect(await screen.findByRole('alert')).toHaveTextContent(message);
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(screen.queryByRole('status')).toBeNull();
      // The list was read again after the refusal.
      expect(listApiKeys).toHaveBeenCalledTimes(2);
      expect(screen.getByTestId('unattended-state-k2')).toHaveTextContent('Requested, not approved');
    });

    it('says so when a key scope cannot be read', async () => {
      renderWithAdmin(
        () => <Integrations />,
        mockAdminApi({ listApiKeys: vi.fn(async () => [{ ...REQUESTED, scopesJson: 'not json' }]) }),
      );
      fireEvent.click(await screen.findByRole('button', { name: 'Approve unattended send for key mwk_req456' }));
      expect(screen.getByRole('dialog')).toHaveTextContent('The scope of this key could not be read.');
    });
  });
});
