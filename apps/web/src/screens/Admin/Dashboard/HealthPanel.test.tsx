import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { HealthPanel, statsFor, thresholdsFor } from './HealthPanel.tsx';
import { engineHealth, proxyHealth } from './fixtures.ts';
import { HEALTH_COMPONENTS, type DashboardHealth, type HealthLoader } from '../../../api/admin/health.ts';

const links = (id: string | null): string | null => (id === null ? null : `/admin#/${id}`);

function mount(health: DashboardHealth, loader?: HealthLoader, open = vi.fn()) {
  render(() => <HealthPanel health={health} sectionLink={links} open={open} loader={loader} />);
  return { open };
}

describe('HealthPanel', () => {
  it('shows every component with a status word, the reason and its numbers', () => {
    mount(engineHealth());
    for (const id of HEALTH_COMPONENTS) {
      const row = screen.getByTestId(`health-${id}`);
      expect(within(row).getByTestId(`health-${id}-status`).textContent).toMatch(/OK|Warning|Failing|Disabled/);
      expect(within(row).getByTestId(`health-${id}-reason`).textContent).not.toBe('');
      expect(within(row).getByTestId(`health-${id}-stats`)).toBeInTheDocument();
    }
    expect(screen.getByTestId('dash-health')).not.toHaveTextContent('did not report');
    expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('2 due, 1 retrying, 0 sending');
    expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('1 of 1 answered');
    expect(screen.getByTestId('health-search-stats')).toHaveTextContent('1,200 of 1,203 stored messages');
    expect(screen.getByTestId('health-search-stats')).toHaveTextContent('2 MB');
    expect(screen.getByTestId('health-plugins-stats')).toHaveTextContent('Rspamd 1.0.0 — loaded; memory 32 MB, deadline 10 s, fuel no limit');
  });

  it('reads sub-millisecond times in μs', () => {
    mount(engineHealth());
    expect(screen.getByTestId('health-database-stats')).toHaveTextContent('Round trip400 μs');
    expect(screen.getByTestId('health-plugins-stats')).toHaveTextContent('2 μs');
    expect(screen.getByTestId('health-database')).toHaveTextContent('Probe took 420 μs');
  });

  it('a disabled component says why and how to enable it, with no numbers', () => {
    mount(proxyHealth());
    for (const id of ['sync', 'outbox', 'search', 'plugins'] as const) {
      expect(screen.getByTestId(`health-${id}-status`)).toHaveTextContent('Disabled');
      expect(screen.getByTestId(`health-${id}-enable`)).toHaveTextContent('To turn it on: Set MW_MODE=engine and restart.');
      expect(screen.queryByTestId(`health-${id}-stats`)).toBeNull();
    }
    expect(screen.getByTestId('health-database-status')).toHaveTextContent('OK');
    expect(screen.queryByTestId('dash-health-history')).toBeNull();
  });

  it('a failing probe is shown as failing with its reason', () => {
    const h = engineHealth();
    h.components.render = { status: 'fail', reason: 'The worker did not answer a ping in time.', metrics: { workerConfigured: true, inFlight: 1, failuresLastHour: 0, timeoutsLastHour: 2, busyRefusalsLastHour: 0, jailRequired: true }, thresholds: { pingWarnMs: 1000, pingTimeoutMs: 1500 } };
    mount(h);
    expect(screen.getByTestId('health-render-status')).toHaveTextContent('Failing');
    expect(screen.getByTestId('health-render-reason')).toHaveTextContent('did not answer a ping in time');
    expect(screen.getByTestId('health-render-stats')).toHaveTextContent('0 failed, 2 timed out, 0 refused while busy');
  });

  it('"Check again" reloads this panel in place with fresh=1 and keeps the old result on failure', async () => {
    const next = engineHealth();
    next.checkedAt += 60;
    next.components.sync = { ...next.components.sync, status: 'ok', reason: '17 mailbox(es) registered; none failing.' };
    const loader = vi.fn<HealthLoader>().mockResolvedValueOnce({ kind: 'ok', data: next }).mockResolvedValueOnce({ kind: 'error', status: 503 });
    mount(engineHealth(), loader);
    fireEvent.click(screen.getByTestId('dash-health-refresh'));
    await waitFor(() => expect(screen.getByTestId('health-sync-reason')).toHaveTextContent('none failing'));
    // The status word follows the new result too, not only the reason.
    expect(screen.getByTestId('health-sync-status')).toHaveTextContent('OK');
    expect(screen.getByTestId('health-sync-status')).not.toHaveTextContent('Warning');
    expect(loader).toHaveBeenCalledWith(true);
    fireEvent.click(screen.getByTestId('dash-health-refresh'));
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('status 503'));
    expect(screen.getByTestId('health-sync-reason')).toHaveTextContent('none failing');
  });

  it('links to the related pages and follows them in place', () => {
    const { open } = mount(engineHealth());
    const link = within(screen.getByTestId('health-plugins')).getByRole('link', { name: 'Plugins' });
    expect(link.getAttribute('href')).toContain('#/plugins');
    fireEvent.click(link, { button: 0 });
    expect(open).toHaveBeenCalledWith('/admin#/plugins');
    expect(within(screen.getByTestId('health-search')).getByRole('link', { name: 'Maintenance' })).toBeInTheDocument();
    expect(within(screen.getByTestId('health-outbox')).getByRole('link', { name: 'Logs' })).toBeInTheDocument();
  });

  it('draws a history chart per measure once there are two samples', () => {
    mount(engineHealth());
    const charts = screen.getByTestId('dash-health-history');
    expect(within(charts).getByRole('region', { name: 'Database round trip' })).toBeInTheDocument();
    expect(within(charts).getByRole('region', { name: 'Messages not indexed' })).toHaveTextContent('Latest 3, highest 3');
  });

  it('names the thresholds each component is judged by', () => {
    const h = engineHealth();
    expect(thresholdsFor('database', h.components.database)).toBe('Warning above 50 ms, failing above 500 ms');
    expect(thresholdsFor('search', h.components.search)).toBe('Warning above 500 messages not indexed');
    expect(statsFor('sync', proxyHealth().components.sync)).toEqual([]);
  });
});
