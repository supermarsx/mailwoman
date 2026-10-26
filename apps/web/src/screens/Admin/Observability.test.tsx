import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Observability } from './Observability.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { AuditLogEntry } from '../../state/slices/admin.ts';

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

  // The log level / OTLP DSN / metrics form stored a record nothing applied and
  // was removed (26.20, t28-e8). These fail if it comes back.
  it('has no telemetry form and does not read the stored telemetry record', async () => {
    const { api } = renderWithAdmin(() => <Observability />, mockAdminApi());
    expect(await screen.findByText('No audit entries.')).toBeInTheDocument();
    for (const label of ['Log level', 'OTLP DSN', 'Enable auth-gated Prometheus /metrics']) {
      expect(screen.queryByText(label)).toBeNull();
    }
    expect(screen.queryByLabelText('Enable Prometheus metrics endpoint')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Save telemetry' })).toBeNull();
    expect(screen.queryByRole('form', { name: 'Logging and telemetry' })).toBeNull();
    expect(api.getObservability).not.toHaveBeenCalled();
    expect(api.setObservability).not.toHaveBeenCalled();
  });

  it('names the environment variables that do set telemetry', async () => {
    renderWithAdmin(() => <Observability />);
    const region = await screen.findByRole('region', { name: 'Observability' });
    expect(region).toHaveTextContent('MW_LOG');
    expect(region).toHaveTextContent('MW_OTLP_ENDPOINT');
    expect(region).toHaveTextContent('MW_METRICS_TOKEN');
  });

  it('does not fetch security data when opening audit',async()=>{
    const api=mockAdminApi();renderWithAdmin(()=><Observability/>,api);
    await screen.findByText('No audit entries.');expect(api.listBans).not.toHaveBeenCalled();
  });

});
