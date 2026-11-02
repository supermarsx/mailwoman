import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within, cleanup } from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import { HealthPanel, statsFor, thresholdsFor, SUBMISSION_POLL_MS, SUBMISSION_FOLLOWUP_MS } from './HealthPanel.tsx';
import { engineHealth, proxyHealth } from './fixtures.ts';
import { HEALTH_COMPONENTS, type DashboardHealth, type HealthLoader } from '../../../api/admin/health.ts';

const links = (id: string | null): string | null => (id === null ? null : `/admin#/${id}`);

function mount(health: DashboardHealth, loader?: HealthLoader, open = vi.fn()) {
  const view=render(() => <HealthPanel health={health} sectionLink={links} open={open} loader={loader} />);
  return { open,...view };
}
afterEach(()=>{cleanup();vi.useRealTimers();vi.restoreAllMocks();});

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
    expect(loader).toHaveBeenCalledWith(true,{signal:expect.any(AbortSignal)});
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

function checking(refreshing=false):DashboardHealth {
  const health=engineHealth(),outbox=health.components.outbox;
  health.components.outbox={...outbox,status:refreshing?'fail':'warn',reason:refreshing?'No submission server answered a connection check.':'Submission connection check in progress.',metrics:{...outbox.metrics,submission:{configured:1,checked:refreshing?1:0,reachable:0,unreachable:refreshing?1:0,notCheckable:0,slowestHandshakeMs:refreshing?5000:null,ageSecs:refreshing?28:null,pending:!refreshing,refreshing}}};
  return health;
}
function healthy():DashboardHealth {const health=engineHealth();health.components.outbox.status='ok';health.components.outbox.reason='Nothing is stuck.';return health;}

describe('submission background check lifecycle',()=>{
  it('automatically settles a cold pending check using non-fresh reads without dashboard auto-refresh',async()=>{
    vi.useFakeTimers();
    const loader=vi.fn<HealthLoader>().mockResolvedValueOnce({kind:'ok',data:checking()}).mockResolvedValue({kind:'ok',data:healthy()});
    mount(checking(),loader);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('Warning');expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('Connection check in progress.');
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS*2);
    expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');expect(loader.mock.calls.map(call=>call[0])).toEqual([false,false]);
    await vi.advanceTimersByTimeAsync(SUBMISSION_FOLLOWUP_MS);expect(loader).toHaveBeenCalledTimes(2);
  });

  it('Check again forces one real check and retains a confirmed failure until the background result arrives',async()=>{
    vi.useFakeTimers();const prior=checking(true);(prior.components.outbox.metrics['submission'] as Record<string,unknown>)['refreshing']=false;
    const loader=vi.fn<HealthLoader>().mockResolvedValueOnce({kind:'ok',data:checking(true)}).mockResolvedValue({kind:'ok',data:healthy()});
    mount(prior,loader);fireEvent.click(screen.getByTestId('dash-health-refresh'));await vi.advanceTimersByTimeAsync(0);
    expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('Failing');expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('0 of 1 answered');expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('Connection check in progress.');
    expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('slowest 5 s');expect(screen.getByTestId('health-outbox')).toHaveTextContent('Status aggregation took 420 μs');
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS);expect(loader.mock.calls.map(call=>call[0])).toEqual([true,false]);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');
  });

  it('bounds follow-up reads and allows an explicit retry after the budget expires',async()=>{
    vi.useFakeTimers();const loader=vi.fn<HealthLoader>().mockResolvedValue({kind:'ok',data:checking()});mount(checking(),loader);
    await vi.advanceTimersByTimeAsync(SUBMISSION_FOLLOWUP_MS+SUBMISSION_POLL_MS);const attempts=loader.mock.calls.length;
    expect(attempts).toBeGreaterThan(0);expect(attempts).toBeLessThanOrEqual(SUBMISSION_FOLLOWUP_MS/SUBMISSION_POLL_MS);
    await vi.advanceTimersByTimeAsync(SUBMISSION_FOLLOWUP_MS);expect(loader).toHaveBeenCalledTimes(attempts);
    fireEvent.click(screen.getByTestId('dash-health-refresh'));await vi.advanceTimersByTimeAsync(0);expect(loader.mock.calls.at(-1)?.[0]).toBe(true);
  });

  it('pauses hidden checks, aborts hidden/unmounted requests and never overlaps a pending read',async()=>{
    vi.useFakeTimers();let visible=false;vi.spyOn(document,'visibilityState','get').mockImplementation(()=>visible?'visible':'hidden');
    const releases:((value:Awaited<ReturnType<HealthLoader>>)=>void)[]=[];
    const loader=vi.fn<HealthLoader>().mockImplementation(()=>new Promise(resolve=>{releases.push(resolve);}));const view=mount(checking(),loader);
    await vi.advanceTimersByTimeAsync(5_000);expect(loader).not.toHaveBeenCalled();visible=true;document.dispatchEvent(new Event('visibilitychange'));
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS*5);expect(loader).toHaveBeenCalledOnce();const first=loader.mock.calls[0]![1]!.signal!;
    visible=false;document.dispatchEvent(new Event('visibilitychange'));expect(first.aborted).toBe(true);releases[0]!({kind:'ok',data:healthy()});await vi.advanceTimersByTimeAsync(0);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('Warning');
    visible=true;document.dispatchEvent(new Event('visibilitychange'));await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS);expect(loader).toHaveBeenCalledTimes(2);
    const second=loader.mock.calls[1]![1]!.signal!;view.unmount();expect(second.aborted).toBe(true);releases[1]!({kind:'ok',data:healthy()});await vi.advanceTimersByTimeAsync(SUBMISSION_FOLLOWUP_MS);expect(loader).toHaveBeenCalledTimes(2);
  });

  it('stops on network errors, retains the last result and recovers through an explicit Check again',async()=>{
    vi.useFakeTimers();const loader=vi.fn<HealthLoader>().mockRejectedValueOnce(new Error('offline')).mockResolvedValue({kind:'ok',data:healthy()});mount(checking(),loader);
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS);expect(screen.getByRole('status')).toHaveTextContent('the server did not answer');expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('Warning');
    await vi.advanceTimersByTimeAsync(SUBMISSION_FOLLOWUP_MS);expect(loader).toHaveBeenCalledOnce();fireEvent.click(screen.getByTestId('dash-health-refresh'));await vi.advanceTimersByTimeAsync(0);
    expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');expect(screen.queryByRole('status')).toBeNull();
  });

  it('does not let an old follow-up response overwrite a newer dashboard snapshot',async()=>{
    vi.useFakeTimers();const [health,setHealth]=createSignal(checking());let release!:(value:Awaited<ReturnType<HealthLoader>>)=>void;
    const loader=vi.fn<HealthLoader>().mockImplementation(()=>new Promise(resolve=>{release=resolve;}));render(()=><HealthPanel health={health()} sectionLink={links} open={()=>{}} loader={loader}/>);
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS);const latest=checking(true);(latest.components.outbox.metrics['submission'] as Record<string,unknown>)['refreshing']=false;setHealth(latest);
    release({kind:'ok',data:healthy()});await vi.advanceTimersByTimeAsync(0);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('Failing');
  });

  it('aborts a hanging old request when new dashboard props arrive and permits another explicit refresh',async()=>{
    vi.useFakeTimers();const [health,setHealth]=createSignal(checking());let release!:(value:Awaited<ReturnType<HealthLoader>>)=>void;
    const loader=vi.fn<HealthLoader>().mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;})).mockResolvedValue({kind:'ok',data:healthy()});
    render(()=><HealthPanel health={health()} sectionLink={links} open={()=>{}} loader={loader}/>);
    await vi.advanceTimersByTimeAsync(SUBMISSION_POLL_MS);const signal=loader.mock.calls[0]![1]!.signal!;expect(screen.getByTestId('dash-health-refresh')).toBeDisabled();
    const latest=checking(true);(latest.components.outbox.metrics['submission'] as Record<string,unknown>)['refreshing']=false;setHealth(latest);
    expect(signal.aborted).toBe(true);expect(screen.getByTestId('dash-health-refresh')).toBeEnabled();
    fireEvent.click(screen.getByTestId('dash-health-refresh'));await vi.advanceTimersByTimeAsync(0);expect(loader.mock.calls.map(call=>call[0])).toEqual([false,true]);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');
    release({kind:'ok',data:latest});await vi.advanceTimersByTimeAsync(0);expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');expect(screen.getByTestId('dash-health-refresh')).toBeEnabled();
  });

  it('reports EWS/provider checks as unmeasured instead of a failed socket handshake',()=>{
    const health=healthy();health.components.outbox.metrics['submission']={configured:1,checked:1,reachable:0,unreachable:0,notCheckable:1,slowestHandshakeMs:null,ageSecs:2};mount(health);
    expect(screen.getByTestId('health-outbox-stats')).not.toHaveTextContent('0 of 1 answered');expect(screen.getByTestId('health-outbox-stats')).toHaveTextContent('require an authenticated provider check');expect(screen.getByTestId('health-outbox-status')).toHaveTextContent('OK');
  });
});
