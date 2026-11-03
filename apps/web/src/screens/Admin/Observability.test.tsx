import { beforeEach, describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, within } from '@solidjs/testing-library';
import { createSignal, Show } from 'solid-js';
import { Observability } from './Observability.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import { createAdminSlice, type AuditLogEntry } from '../../state/slices/admin.ts';
import { AdminContext } from './context.ts';
import { dashboardReturning, holding, renderInShell } from './shell/testkit.tsx';
import { useShell } from './shell/context.ts';
import type { Capability } from './shell/can.ts';
import { engineDashboard } from './Dashboard/fixtures.ts';
import { loadRequestHistory } from '../../api/admin/requestHistory.ts';

vi.mock('../../api/admin/requestHistory.ts',()=>({loadRequestHistory:vi.fn(),loadNotFoundPaths:vi.fn(async()=>{throw new Error('not available');})}));
beforeEach(()=>{
  vi.mocked(loadRequestHistory).mockClear().mockResolvedValue({from:0,to:3600,bucketSeconds:60,series:[{t:0,requests:1,errors5xx:0,p50Ms:5,p95Ms:10,coverage:'complete'}],coverage:{observedSeconds:3600,requestedSeconds:3600,gaps:[]},policy:{defaultRangeSeconds:3600,maxRangeSeconds:2592000,retentionDays:30},scope:'server',timing:'response-ready'});
});

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
  function openSection(rest:string[]=[],capabilities:Capability[]=['observability.read','audit.read','dashboard.read']){
    const api=mockAdminApi({listAudit:vi.fn(async()=>[AUDIT])});
    const admin=createAdminSlice(api);
    const dashboard=dashboardReturning({kind:'ok',data:engineDashboard()});
    const [route,setRoute]=createSignal({section:'observability',rest});
    const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><AdminContext.Provider value={admin}><Observability/></AdminContext.Provider></Show>;};
    const rendered=renderInShell(()=><Ready/>,{loadMe:holding(capabilities),loadDashboard:dashboard,route,navigate:path=>setRoute({section:'observability',rest:path.split('#/observability')[1]?.split('/').filter(Boolean)??[]})});
    return {...rendered,api,dashboard};
  }

  it('opens on Overview with key figures before charts and tables, then navigates to Audit without leaving the section',async()=>{
    const {api,navigate}=openSection();
    const overview=await screen.findByRole('region',{name:'Overview'});
    expect(within(overview).getByRole('heading',{name:'Overview'})).toBeInTheDocument();
    expect(await within(overview).findByRole('group',{name:'Traffic in this period'})).toBeInTheDocument();
    const tiles=within(overview).getByTestId('stats-tiles'),charts=within(overview).getByTestId('stats-charts'),tables=within(overview).getByTestId('stats-tables');
    expect(tiles.compareDocumentPosition(charts)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(charts.compareDocumentPosition(tables)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const tabs=screen.getByRole('navigation',{name:'Observability'});
    expect(within(tabs).getAllByRole('link').map(link=>link.textContent)).toEqual(['Overview','Audit log','Configure operational telemetry']);
    expect(within(tabs).getByRole('link',{name:'Overview'})).toHaveAttribute('aria-current','page');
    expect(within(tabs).getByRole('link',{name:'Overview'})).toHaveAttribute('href','/admin#/observability');
    expect(api.listAudit).not.toHaveBeenCalled();
    expect(api.getObservability).not.toHaveBeenCalled();
    fireEvent.click(within(tabs).getByRole('link',{name:'Audit log'}));
    expect(navigate).toHaveBeenCalledWith('/admin#/observability/audit');
    expect(await screen.findByText('user-provisioned')).toBeInTheDocument();
    expect(screen.getByRole('navigation',{name:'Observability'})).toBe(tabs);
    expect(screen.queryByTestId('statistics')).toBeNull();
  });

  it('keeps old Statistics links working as Overview',async()=>{
    openSection(['statistics']);
    expect(await screen.findByRole('region',{name:'Overview'})).toBeInTheDocument();
    const tabs=screen.getByRole('navigation',{name:'Observability'});
    expect(within(tabs).getByRole('link',{name:'Overview'})).toHaveAttribute('aria-current','page');
  });

  it('keeps Audit as the default for an audit-only administrator without requesting statistics',async()=>{
    const {dashboard}=openSection([],['audit.read']);
    expect(await screen.findByText('user-provisioned')).toBeInTheDocument();
    const tabs=screen.getByRole('navigation',{name:'Observability'});
    expect(within(tabs).getAllByRole('link').map(link=>link.textContent)).toEqual(['Audit log']);
    expect(within(tabs).getByRole('link',{name:'Audit log'})).toHaveAttribute('aria-current','page');
    expect(loadRequestHistory).not.toHaveBeenCalled();
    expect(dashboard).not.toHaveBeenCalled();
  });

  it('opens Overview for a statistics-only administrator without loading Audit',async()=>{
    const {api,dashboard}=openSection([],['observability.read']);
    expect(await screen.findByRole('region',{name:'Overview'})).toBeInTheDocument();
    expect(screen.queryByRole('link',{name:'Audit log'})).toBeNull();
    expect(api.listAudit).not.toHaveBeenCalled();
    expect(dashboard).not.toHaveBeenCalled();
  });

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
