import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, within } from '@solidjs/testing-library';
import { Dashboard } from './Dashboard.tsx';
import { engineDashboard, proxyDashboard } from './fixtures.ts';
import { dashboardReturning, holding, renderInShell } from '../shell/testkit.tsx';
import type { DashboardResult } from '../../../api/admin/dashboard.ts';

const ok = (data = engineDashboard()): DashboardResult => ({ kind: 'ok', data });

beforeEach(() => localStorage.clear());

describe('Dashboard — before the endpoint exists', () => {
  it('a 404 says the figures are not available yet and shows no numbers', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning({ kind: 'unavailable' }) });
    const box = await screen.findByTestId('dash-unavailable');
    expect(box).toHaveTextContent('The dashboard figures are not available yet.');
    const region = screen.getByRole('region', { name: 'Dashboard' });
    // Only the endpoint's own status code appears; no counts, sizes or times.
    expect(region.textContent!.replace('404', '')).not.toMatch(/\d/);
    expect(screen.queryByRole('figure')).toBeNull();
  });

  it('a server error says so with the status and offers a retry that reloads', async () => {
    const load = vi.fn<() => Promise<DashboardResult>>().mockResolvedValueOnce({ kind: 'error', status: 503 }).mockResolvedValueOnce(ok());
    renderInShell(() => <Dashboard />, { loadDashboard: load });
    expect(await screen.findByTestId('dash-error')).toHaveTextContent('the server answered 503');
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByTestId('tile-users')).toHaveTextContent('42');
    expect(load).toHaveBeenCalledTimes(2);
  });

  it('a network failure is not reported as a status', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning({ kind: 'error', status: null }) });
    expect(await screen.findByTestId('dash-error')).toHaveTextContent('the server did not answer');
  });
});

describe('Dashboard — with data', () => {
  it('shows the server-read counts in tiles', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    expect(await screen.findByTestId('tile-users')).toHaveTextContent('42');
    expect(screen.getByRole('figure', { name: 'Domains' })).toHaveTextContent('2 with a mail server, 1 disabled');
    expect(screen.getByRole('figure', { name: 'IP allowlists' })).toHaveTextContent('Admin on, webmail off');
    expect(screen.getByTestId('tile-volume')).toHaveTextContent('10 GB');
    expect(screen.getByTestId('tile-volume')).toHaveTextContent('of 50 GB');
  });

  it('null fields are "Not available" with the reason (proxy mode, no free-space data)', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(proxyDashboard())) });
    const volume = await screen.findByTestId('tile-volume');
    expect(volume).toHaveTextContent('Not available');
    expect(volume).toHaveTextContent('The server cannot read free space on this platform.');
    expect(volume.textContent).not.toMatch(/\d/);
    const health = screen.getByTestId('dash-health');
    expect(health).toHaveTextContent('Not available: this server runs in proxy mode and does not sync mail itself.');
    expect(health).toHaveTextContent('Not available: This server has no search index.');
  });

  it('health rows use words, with an icon and a hidden status word, not colour alone', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    const health = await screen.findByTestId('dash-health');
    expect(health).toHaveTextContent('Up to date (33 applied)');
    expect(health).toHaveTextContent('17 accounts registered, 1 with a sync error');
    expect(health).toHaveTextContent('Needs attention');
    expect(health).toHaveTextContent('Available: untrusted documents are rendered in a separate process.');
  });

  it('migrations behind are reported as applied of expected', async () => {
    const d = engineDashboard();
    d.health.migration = { applied: 31, expected: 33 };
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(d)) });
    expect(await screen.findByTestId('dash-health')).toHaveTextContent('31 of 33 applied');
  });

  it('traffic charts: one series each, from the server ring, with a per-process note', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    const req = await screen.findByTestId('spark-requests');
    expect(req).toHaveTextContent('Latest 18, highest 25');
    expect(screen.getByTestId('spark-errors')).toHaveTextContent('Latest 0, highest 1');
    expect(screen.getByTestId('spark-p95')).toHaveTextContent('Latest 22 ms, highest 31 ms');
    expect(screen.getByText(/Counted by this server process since it started/)).toBeInTheDocument();
  });

  it('recent audit lists the actions; an empty audit says so', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    const table = await screen.findByRole('table', { name: 'Recent admin actions' });
    expect(table).toHaveTextContent('user.revoke_sessions');
    expect(table).toHaveTextContent('alice@example.org');
  });

  it('warnings link to the section that fixes them; unknown codes are shown, not dropped', async () => {
    const d = engineDashboard();
    d.warnings = [
      { code: 'metrics-token-unset', count: 1 },
      { code: 'pending-migrations', count: 2 },
      { code: 'brand-new-thing', count: 3 },
    ];
    const { navigate } = renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(d)) });
    const list = await screen.findByRole('region', { name: 'Needs attention' });
    const metrics = list.querySelector('[data-warning="metrics-token-unset"]') as HTMLElement;
    expect(metrics).toHaveTextContent('The metrics endpoint has no token set.');
    fireEvent.click(within(metrics).getByRole('link', { name: 'Open the section that fixes this' }));
    expect(navigate).toHaveBeenCalledWith('/admin/console/observability');
    const mig = list.querySelector('[data-warning="pending-migrations"]') as HTMLElement;
    expect(mig).toHaveTextContent('2 database migrations have not been applied. Restart the server to apply them.');
    expect(within(mig).queryByRole('link')).toBeNull();
    expect(list.querySelector('[data-warning="brand-new-thing"]')).toHaveTextContent('Server warning brand-new-thing (3)');
  });

  it('a warning whose section the admin cannot open has no link', async () => {
    const d = engineDashboard();
    d.warnings = [{ code: 'metrics-token-unset', count: 1 }];
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(d)), loadMe: holding(['dashboard.read']) });
    const row = (await screen.findByText('The metrics endpoint has no token set.')).closest('li')!;
    expect(within(row).queryByRole('link')).toBeNull();
  });
});

describe('Dashboard — blocked accounts (DA3)', () => {
  function blocked() {
    const d = engineDashboard();
    d.blockedAccounts = {
      total: 5,
      byDomain: [
        { domain: 'old.example', accounts: 4, seenHost: 'imap.old.example' },
        { domain: 'other.example', accounts: 1, seenHost: null },
      ],
    };
    d.warnings = [{ code: 'blocked-accounts', count: 5 }];
    return d;
  }

  it('names the count and each domain, with a Configure domain action that opens the domain', async () => {
    const { navigate } = renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(blocked())) });
    const panel = await screen.findByTestId('dash-blocked');
    expect(within(panel).getByRole('heading', { name: '5 accounts blocked: no domain configuration' })).toBeInTheDocument();
    expect(panel).toHaveTextContent('old.example 4 accounts');
    expect(panel).toHaveTextContent('last used mail host imap.old.example');
    fireEvent.click(within(panel).getByRole('link', { name: 'Configure old.example' }));
    expect(navigate).toHaveBeenCalledWith('/admin/console/domains/old.example');
    expect(within(panel).getByRole('table', { name: 'Blocked accounts per domain' })).toBeInTheDocument();
  });

  it('without domains.read the action is replaced by a plain sentence', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(blocked())), loadMe: holding(['dashboard.read']) });
    const panel = await screen.findByTestId('dash-blocked');
    expect(within(panel).queryByRole('link', { name: /Configure/ })).toBeNull();
    expect(panel).toHaveTextContent('Your role cannot configure domains.');
  });

  it('no panel when nothing is blocked', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    await screen.findByTestId('tile-users');
    expect(screen.queryByTestId('dash-blocked')).toBeNull();
  });
});

describe('Dashboard — first-run checklist', () => {
  function withChecklist() {
    const d = engineDashboard();
    d.checklist = [
      { id: 'domain-exists', done: true, href: '/admin/console/domains' },
      { id: 'domain-upstream', done: false, href: '/admin/console/domains' },
      { id: 'preferred-domain', done: false, href: '/admin/console/domains', optional: true },
    ];
    return d;
  }

  it('lists items with Done/Open words, counts the open required ones, and links each open item', async () => {
    const { navigate } = renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(withChecklist())) });
    const list = await screen.findByTestId('dash-checklist');
    expect(within(list).getByRole('heading')).toHaveTextContent('Setup checklist: one item open');
    const done = list.querySelector('[data-check="domain-exists"]') as HTMLElement;
    expect(done).toHaveTextContent('Done');
    expect(within(done).queryByRole('link')).toBeNull();
    const open = list.querySelector('[data-check="domain-upstream"]') as HTMLElement;
    expect(open).toHaveTextContent('Open');
    fireEvent.click(within(open).getByRole('link', { name: 'Open' }));
    expect(navigate).toHaveBeenCalledWith('/admin/console/domains');
    expect(list.querySelector('[data-check="preferred-domain"]')).toHaveTextContent('(optional)');
  });

  it('hiding an item never marks it done, and is remembered per admin', async () => {
    const first = renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(withChecklist())), username: 'ann' });
    const list = await screen.findByTestId('dash-checklist');
    const open = list.querySelector('[data-check="domain-upstream"]') as HTMLElement;
    fireEvent.click(within(open).getByRole('button', { name: 'Hide' }));
    expect(list.querySelector('[data-check="domain-upstream"]')).toBeNull();
    expect(within(list).getByRole('heading')).toHaveTextContent('one item open');
    expect(list).toHaveTextContent('One item is hidden. Hiding does not mark it done.');
    first.unmount();

    // Same admin: still hidden. Another admin: shown.
    const again = renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(withChecklist())), username: 'ann' });
    expect((await screen.findByTestId('dash-checklist')).querySelector('[data-check="domain-upstream"]')).toBeNull();
    again.unmount();
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok(withChecklist())), username: 'bob' });
    expect((await screen.findByTestId('dash-checklist')).querySelector('[data-check="domain-upstream"]')).not.toBeNull();
  });

  it('no checklist from the server, no checklist panel', async () => {
    renderInShell(() => <Dashboard />, { loadDashboard: dashboardReturning(ok()) });
    await screen.findByTestId('tile-users');
    expect(screen.queryByTestId('dash-checklist')).toBeNull();
  });
});
