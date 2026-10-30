import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Diagnostics } from './index.tsx';
import type { Check, DiagnosticsApi, Job, Run } from './api.ts';
import { cardStatus, diffRuns, groupOf, latestChecks, previousOf, tabOf, worst } from './model.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import type { AdminRoute } from '../shell/router.ts';

const check = (id: string, status: string, extra: Partial<Check> = {}): Check => ({ id, status, durationMs: 0, durationUs: 420, summary: `${id} summary`, ...extra });
const serverRun = (id: string, startedAt: number, results: Check[]): Run => ({ id, suite: 'server', startedAt, status: results.some((c) => c.status === 'fail') ? 'fail' : 'pass', results });
const SERVER = serverRun('s1', 100, [check('database-connectivity', 'pass'), check('database-write', 'pass'), check('security-hsts', 'warn'), check('network-public-dns', 'fail')]);
const BACKEND = check('a.test:imap', 'pass', { domain: 'a.test', backendId: 'imap', protocol: 'imap', category: 'backend', detail: { protocol: 'imap', reachable: true, tls: 'implicit', tlsVersion: 'TLSv1.3', capabilityCount: 7, credentialsSent: false } });

function api(overrides: Partial<DiagnosticsApi> = {}, runs: Run[] = [SERVER]): DiagnosticsApi {
  return {
    history: vi.fn(async () => ({ kind: 'ok' as const, etag: null, data: { runs, nextCursor: null } })),
    run: vi.fn(async () => ({ run: serverRun('s2', 200, [check('database-connectivity', 'pass')]), historySaved: true, auditSaved: true })),
    catalogue: vi.fn(async () => ({ canRunBackends: true, suites: [{ id: 'server', supported: true, description: 'Database' }] })),
    ...overrides,
  };
}
function at(rest: string[]): () => AdminRoute { return () => ({ section: 'diagnostics', rest }); }
const card = (name: string): HTMLElement => screen.getByRole('row', { name });

describe('diagnostics model', () => {
  it('maps tabs, statuses and groups', () => {
    expect(tabOf(undefined)).toBe('overview');
    expect(tabOf('posture')).toBe('posture');
    expect(tabOf('nope')).toBe('overview');
    expect(['pass', 'warn', 'fail', 'error', 'skipped', 'running', undefined].map(cardStatus)).toEqual(['ok', 'warn', 'fail', 'fail', 'skipped', 'running', 'notRun']);
    expect(groupOf(check('security-new-thing', 'pass'), 'server')).toBe('posture');
    expect(groupOf(check('a.test:imap', 'pass'), 'backends')).toBe('backends');
    expect(groupOf(check('egress-private-policy', 'pass'), 'server')).toBe('connectivity');
    expect(worst(['ok', 'warn', 'notRun'])).toBe('warn');
  });
  it('takes each check from its newest run and lists what changed between runs', () => {
    const older = serverRun('old', 1, [check('database-write', 'fail'), check('gone', 'pass')]);
    const newer = serverRun('new', 2, [check('database-write', 'pass'), check('fresh', 'pass')]);
    expect(latestChecks([newer, older]).get('gone')?.run.id).toBe('old');
    expect(latestChecks([newer, older]).get('database-write')?.check.status).toBe('pass');
    expect(previousOf([newer, older], newer)?.id).toBe('old');
    expect(diffRuns(newer, older).map((c) => [c.id, c.before, c.after])).toEqual([['database-write', 'fail', 'pass'], ['fresh', undefined, 'pass'], ['gone', 'pass', undefined]]);
  });
});

describe('diagnostics screen', () => {
  it('renders a named accessible check table with distinct microsecond duration and retained details/actions',async()=>{
    const {container}=renderInShell(()=><Diagnostics api={api()}/>,{route:at(['server'])});
    const table=await screen.findByRole('table',{name:'Server'});
    expect(within(table).getAllByRole('columnheader').map(header=>header.textContent)).toEqual(['Check','Status','Result','Duration','Actions']);
    const row=within(table).getByRole('row',{name:'Database connection'});
    expect(within(row).getByRole('rowheader')).toHaveTextContent('Database connection');
    expect(row.querySelector('[data-diag-duration]')).toHaveTextContent(/420 [µμ]s/u);
    expect(within(row).getAllByText(/420 [µμ]s/u)).toHaveLength(1);
    expect(row.querySelector('details summary')).toHaveTextContent('Details');
    expect(within(row).getByRole('button',{name:'Run again'})).toBeEnabled();
    expect(screen.getByRole('region',{name:'Server'})).toHaveAttribute('tabindex','0');
    expect(screen.queryByRole('article',{name:'Database connection'})).toBeNull();
    await expectNoAxeViolations(container);
  });
  it('shows every sub-tab and opens the one in the route', async () => {
    renderInShell(() => <Diagnostics api={api()} />, { route: at(['posture']) });
    const nav = await screen.findByRole('navigation', { name: 'Diagnostics pages' });
    expect(within(nav).getAllByRole('link').map((a) => a.textContent)).toEqual(['Overview', 'Server', 'Connectivity', 'Mail backends', 'Security posture', 'Tools', 'History', 'Report']);
    expect(within(nav).getByRole('link', { name: 'Security posture' })).toHaveAttribute('aria-current', 'page');
    const hsts = await screen.findByRole('row', { name: 'HSTS' });
    expect(within(hsts).getByTestId('diag-status')).toHaveTextContent('Warning');
    // Warn and Fail carry "How to fix"; OK and Not run do not.
    expect(within(hsts).getByText('How to fix')).toBeInTheDocument();
    // A server run that did not report a check: Skipped, with the reason, no "How to fix".
    expect(within(card('CSRF protection')).getByTestId('diag-status')).toHaveTextContent('Skipped');
    expect(within(card('CSRF protection')).getByText(/did not report this check/)).toBeInTheDocument();
    // A posture claim says whether it is met rather than repeating the goal.
    expect(within(hsts).getByText('Not met')).toBeInTheDocument();
    expect(within(card('CSRF protection')).queryByText('How to fix')).not.toBeInTheDocument();
  });

  it('shows the latest results on the Overview and what needs attention', async () => {
    const { container } = renderInShell(() => <Diagnostics api={api()} />, { route: at([]) });
    const attention = await screen.findByRole('region', { name: 'Needs attention' });
    await vi.waitFor(() => expect(within(attention).getByRole('link', { name: 'HSTS' })).toBeInTheDocument());
    expect(within(attention).getByRole('link', { name: 'Public name' })).toBeInTheDocument();
    const server = container.querySelector('[data-group="server"]')!;
    expect(server).toHaveTextContent('2 OK · 0 warnings · 0 failed · 2 not run');
    await expectNoAxeViolations(container);
  });

  it('runs all suites in sequence with progress and keeps the results', async () => {
    let finish: (job: Job) => void = () => undefined;
    const running: Job = { run: { id: 'b1', suite: 'backends', startedAt: 300, status: 'running', results: [] }, total: 1, completed: 0, limited: false, historySaved: false };
    const order: string[] = [];
    const a = api({
      run: vi.fn(async () => { order.push('server'); return { run: serverRun('s2', 200, [check('database-connectivity', 'pass')]), historySaved: true, auditSaved: true }; }),
      start: vi.fn(async () => { order.push('backends'); return running; }),
      job: vi.fn(() => new Promise<Job>((resolve) => { finish = resolve; })),
    });
    renderInShell(() => <Diagnostics api={a} />, { route: at([]) });
    const button = await screen.findByRole('button', { name: 'Run all checks' });
    await vi.waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    expect(await screen.findByTestId('diag-run-all-progress')).toHaveTextContent('Step 1 of 2');
    await vi.waitFor(() => expect(screen.getByTestId('diag-run-all-progress')).toHaveTextContent('Step 2 of 2: saved backends. 0 of 1 backends checked.'), { timeout: 3000 });
    // A second click while it runs starts nothing new.
    expect(screen.getByRole('button', { name: 'Running…' })).toBeDisabled();
    await vi.waitFor(() => expect(a.job).toHaveBeenCalled(), { timeout: 3000 });
    finish({ ...running, run: { ...running.run, status: 'pass', results: [BACKEND] }, completed: 1, historySaved: true });
    expect(await screen.findByText('All checks finished. The results are saved in History.')).toBeInTheDocument();
    expect(order).toEqual(['server', 'backends']);
    expect(a.run).toHaveBeenCalledTimes(1);
    expect(a.start).toHaveBeenCalledTimes(1);
    expect(screen.queryByTestId('diag-run-all-progress')).not.toBeInTheDocument();
    expect(document.querySelector('[data-group="backends"]')).toHaveTextContent('1 OK');
  });

  it('reruns one check in place: only that card says Running and a second click joins the run', async () => {
    let answer: () => void = () => undefined;
    const run = vi.fn(() => new Promise<{ run: Run; historySaved: boolean; auditSaved: boolean }>((resolve) => {
      answer = () => resolve({ run: serverRun('s2', 200, [check('database-connectivity', 'fail'), check('database-write', 'pass')]), historySaved: true, auditSaved: true });
    }));
    renderInShell(() => <Diagnostics api={api({ run })} />, { route: at(['server']) });
    const db = await screen.findByRole('row', { name: 'Database connection' });
    await vi.waitFor(() => expect(within(db).getByTestId('diag-status')).toHaveTextContent('OK'));
    const tabsBefore = screen.getByRole('navigation', { name: 'Diagnostics pages' });
    fireEvent.click(within(db).getByRole('button', { name: 'Run again' }));
    expect(within(db).getByTestId('diag-status')).toHaveTextContent('Running');
    expect(within(card('Database writes')).getByTestId('diag-status')).toHaveTextContent('OK');
    expect(within(card('Database writes')).getByRole('button', { name: 'Run again' })).toBeDisabled();
    fireEvent.click(within(db).getByRole('button', { name: 'Running…' }));
    expect(run).toHaveBeenCalledTimes(1);
    answer();
    await vi.waitFor(() => expect(within(db).getByTestId('diag-status')).toHaveTextContent('Failed'));
    expect(within(db).getByText('How to fix')).toBeInTheDocument();
    // Same element: the card updated in place, the page was not swapped.
    expect(screen.getByRole('row', { name: 'Database connection' })).toBe(db);
    expect(screen.getByRole('navigation', { name: 'Diagnostics pages' })).toBe(tabsBefore);
  });

  it('raises a failed run as a corner toast whose Retry runs it again', async () => {
    const run = vi.fn().mockRejectedValueOnce(Object.assign(new Error('busy'), { status: 429 })).mockResolvedValue({ run: SERVER, historySaved: true, auditSaved: true });
    renderInShell(() => <Diagnostics api={api({ run })} />, { route: at(['server']) });
    const db = await screen.findByRole('row', { name: 'Database connection' });
    await vi.waitFor(() => expect(within(db).getByRole('button', { name: 'Run again' })).toBeEnabled());
    fireEvent.click(within(db).getByRole('button', { name: 'Run again' }));
    const toast = await screen.findByTestId('admin-request-error');
    fireEvent.click(within(toast).getByRole('button', { name: 'Retry' }));
    await vi.waitFor(() => expect(run).toHaveBeenCalledTimes(2));
    expect(screen.getByRole('navigation', { name: 'Diagnostics pages' })).toBeInTheDocument();
  });

  it('copies a technical value from a card', async () => {
    const writeText = vi.fn<(value: string) => Promise<void>>(async () => undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    renderInShell(() => <Diagnostics api={api()} />, { route: at(['server']) });
    const db = await screen.findByRole('row', { name: 'Database connection' });
    fireEvent.click(await within(db).findByRole('button', { name: 'Copy Check ID' }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith('database-connectivity'));
  });

  it('shows backend handshakes as cards with labelled rows and filters them', async () => {
    const backends: Run = { id: 'b0', suite: 'backends', startedAt: 90, status: 'fail', results: [BACKEND, check('b.test:smtp', 'fail', { domain: 'b.test', backendId: 'smtp', protocol: 'smtp', category: 'backend' })] };
    const domains = vi.fn(async () => [{ name: 'a.test', enabled: true, failover: true, backends: 2 }]);
    renderInShell(() => <Diagnostics api={api({ start: vi.fn(), job: vi.fn(), domains }, [SERVER, backends])} />, { route: at(['backends']) });
    const imap = await screen.findByRole('row', { name: 'a.test · imap' });
    expect(within(imap).getByText('implicit · TLSv1.3')).toBeInTheDocument();
    expect(within(imap).getByText('No')).toBeInTheDocument();
    expect(within(card('b.test · smtp')).getByText('How to fix')).toBeInTheDocument();
    expect(await screen.findByTestId('diag-failover')).toHaveTextContent('Failover on · 2 mail backends');
    fireEvent.input(screen.getByLabelText('Filter checks, domains or backends'), { target: { value: 'b.test' } });
    expect(screen.queryByRole('row', { name: 'a.test · imap' })).not.toBeInTheDocument();
    expect(card('b.test · smtp')).toBeInTheDocument();
  });

  it('runs the DNS tool for a saved domain in its card', async () => {
    const dns = vi.fn(async () => ({ domain: 'a.test', mx: { records: [{ preference: 10, host: 'mx.a.test' }], error: null }, spf: { present: true, records: ['v=spf1 -all'], error: null }, dmarc: { present: false, records: [], error: null }, srv: [] }));
    renderInShell(() => <Diagnostics api={api({ dns, domains: async () => [{ name: 'a.test', enabled: true, failover: false, backends: 1 }] })} />, { route: at(['connectivity']) });
    const tool = await screen.findByRole('row', { name: 'DNS records' });
    const go = within(tool).getByRole('button', { name: 'Run' });
    await vi.waitFor(() => expect(go).toBeEnabled());
    fireEvent.click(go);
    await vi.waitFor(() => expect(within(tool).getByTestId('diag-status')).toHaveTextContent('Warning'));
    expect(dns).toHaveBeenCalledWith('a.test');
    expect(within(tool).getByText('10 mx.a.test')).toBeInTheDocument();
  });

  it('lists saved runs in History and what changed since the run before', async () => {
    const older = serverRun('s0', 50, [check('database-connectivity', 'fail')]);
    renderInShell(() => <Diagnostics api={api({}, [SERVER, older])} />, { route: at(['history']) });
    const table = await screen.findByRole('table', { name: 'Previous runs' });
    await vi.waitFor(() => expect(table.querySelectorAll('tbody tr')).toHaveLength(2));
    expect(await screen.findByTestId('diag-changes')).toHaveTextContent('database-connectivity summary: fail → pass');
    fireEvent.click(within(table).getAllByRole('button', { name: /^View run from / })[1]!);
    expect(await screen.findByText('No earlier run of this kind to compare with.')).toBeInTheDocument();
  });

  it('copies the report with domain names replaced by default', async () => {
    const writeText = vi.fn<(value: string) => Promise<void>>(async () => undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    const named = serverRun('unsaved', 400, [check('a.test:imap', 'pass', { domain: 'secret.example', backendId: 'primary' })]);
    const report = vi.fn(async (o?: { omitDomains?: boolean }) => ({ schemaVersion: 1, redaction: { domainsOmitted: o?.omitDomains === true } }));
    renderInShell(() => <Diagnostics api={api({ report }, [named])} />, { route: at(['report']) });
    const box = await screen.findByRole('checkbox', { name: 'Replace domain names and backend IDs in the report' });
    expect(box).toBeChecked();
    fireEvent.click(screen.getByRole('button', { name: 'Copy troubleshooting report' }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(1));
    expect(report).toHaveBeenCalledWith({ omitDomains: true, selected: 'unsaved' });
    const text = writeText.mock.calls[0]![0] as string;
    expect(text).not.toContain('secret.example');
    expect(JSON.parse(text)).toMatchObject({ redaction: { domainsOmitted: true }, selectedRunOmitted: true });
    // With redaction off, an unsaved run is attached as is.
    fireEvent.click(box);
    fireEvent.click(screen.getByRole('button', { name: 'Copy troubleshooting report' }));
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(2));
    expect(JSON.parse(writeText.mock.calls[1]![0] as string)).toMatchObject({ selectedRun: { id: 'unsaved' } });
  });

  it('switches tabs without losing results', async () => {
    const [route, setRoute] = createSignal<AdminRoute>({ section: 'diagnostics', rest: ['server'] });
    const a = api();
    renderInShell(() => <Diagnostics api={a} />, { route });
    await screen.findByRole('row', { name: 'Database connection' });
    setRoute({ section: 'diagnostics', rest: ['posture'] });
    expect(await screen.findByRole('row', { name: 'HSTS' })).toBeInTheDocument();
    expect(a.history).toHaveBeenCalledTimes(1);
  });
});
