import {it,expect,vi,beforeEach} from 'vitest';
import {screen} from '@solidjs/testing-library';
import {ObservabilityStats} from './ObservabilityStats.tsx';
import {engineDashboard} from './Dashboard/fixtures.ts';
import {dashboardReturning,renderInShell,holding} from './shell/testkit.tsx';
import {loadRequestHistory} from '../../api/admin/requestHistory.ts';
import {Show} from 'solid-js';
import {useShell} from './shell/context.ts';
vi.mock('../../api/admin/requestHistory.ts',()=>({loadRequestHistory:vi.fn(),loadNotFoundPaths:vi.fn(()=>Promise.reject(new Error('not in this test')))}));
beforeEach(()=>{vi.mocked(loadRequestHistory).mockResolvedValue({from:0,to:3600,bucketSeconds:60,series:[{t:0,requests:1,errors5xx:0,p50Ms:5,p95Ms:10,coverage:'complete'}],coverage:{observedSeconds:3600,requestedSeconds:3600,gaps:[]},policy:{defaultRangeSeconds:3600,maxRangeSeconds:2592000,retentionDays:30},scope:'server',timing:'response-ready'});});
it('exposes all dashboard traffic series and median latency from actual supplied buckets',async()=>{
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByRole('region',{name:'Response time, median'})).toBeInTheDocument();
 expect(screen.getByRole('region',{name:'Requests per bucket'})).toBeInTheDocument();
 expect(screen.getByRole('region',{name:'Response time, 95th percentile'})).toBeInTheDocument();
 expect(screen.getByText('Pending outbox messages')).toBeInTheDocument();
});
it('does not fabricate statistics when the source is unavailable',async()=>{
 vi.mocked(loadRequestHistory).mockRejectedValue(new Error('unavailable'));
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'unavailable'})});
 expect(await screen.findByText('Not available')).toHaveAttribute('role','status');
 expect(screen.queryByRole('figure')).not.toBeInTheDocument();
});
it('loads request history without requiring dashboard access',async()=>{
 const dashboard=dashboardReturning({kind:'ok',data:engineDashboard()});
 // The production shell waits for grants before rendering a protected screen.
 const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><ObservabilityStats/></Show>;};
 renderInShell(()=><Ready/>,{loadMe:holding(['observability.read']),loadDashboard:dashboard});
 expect(await screen.findByRole('region',{name:'Requests per bucket'})).toBeInTheDocument();
 expect(dashboard).not.toHaveBeenCalled();
});
