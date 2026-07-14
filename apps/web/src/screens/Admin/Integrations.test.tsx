import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
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
  it('shows LDAP and Nextcloud as deferred (inert)', async () => {
    renderWithAdmin(() => <Integrations />);
    expect(await screen.findByText('LDAP / GAL directory')).toBeInTheDocument();
    expect(screen.getByText('Nextcloud bridge')).toBeInTheDocument();
    expect(screen.getAllByText('Deferred').length).toBeGreaterThanOrEqual(2);
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
