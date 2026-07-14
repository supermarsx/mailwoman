import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Observability } from './Observability.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { AuditLogEntry, BanEntry } from '../../state/slices/admin.ts';

const AUDIT: AuditLogEntry = {
  id: '1',
  ts: '2026-07-14T01:02:03Z',
  actor: 'root',
  actorKind: 'admin',
  action: 'user-provisioned',
  target: 'alice@example.com',
  detailJson: '{}',
  ip: null,
};

const BAN: BanEntry = { ip: '198.51.100.9', reason: 'brute-force', bannedAt: '2026-07-14T00:00:00Z', expiresAt: null };

describe('Admin › Observability', () => {
  it('renders the audit log', async () => {
    renderWithAdmin(() => <Observability />, mockAdminApi({ listAudit: vi.fn(async () => [AUDIT]) }));
    expect(await screen.findByText('user-provisioned')).toBeInTheDocument();
    expect(screen.getByText('alice@example.com')).toBeInTheDocument();
  });

  it('exports the audit log as JSONL', async () => {
    const exportAudit = vi.fn(async () => '{"a":1}\n');
    // jsdom lacks URL.createObjectURL — stub it.
    const createObjectURL = vi.fn(() => 'blob:x');
    const revokeObjectURL = vi.fn();
    Object.assign(URL, { createObjectURL, revokeObjectURL });
    renderWithAdmin(() => <Observability />, mockAdminApi({ exportAudit }));
    fireEvent.click(await screen.findByRole('button', { name: 'Export JSONL' }));
    await Promise.resolve();
    expect(exportAudit).toHaveBeenCalled();
  });

  it('saves telemetry config', async () => {
    const setObservability = vi.fn(async () => undefined);
    renderWithAdmin(() => <Observability />, mockAdminApi({ setObservability }));
    fireEvent.click(await screen.findByLabelText('Enable Prometheus metrics endpoint'));
    fireEvent.click(screen.getByRole('button', { name: 'Save telemetry' }));
    await Promise.resolve();
    expect(setObservability).toHaveBeenCalledWith(expect.objectContaining({ metricsEnabled: true }));
  });

  it('lists bans and can unban', async () => {
    const removeBan = vi.fn(async () => undefined);
    renderWithAdmin(() => <Observability />, mockAdminApi({ listBans: vi.fn(async () => [BAN]), removeBan }));
    expect(await screen.findByText('198.51.100.9')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Unban 198.51.100.9' }));
    await Promise.resolve();
    expect(removeBan).toHaveBeenCalledWith('198.51.100.9');
  });
});
