import {it,expect} from 'vitest';
import {screen} from '@solidjs/testing-library';
import {ObservabilityStats} from './ObservabilityStats.tsx';
import {engineDashboard} from './Dashboard/fixtures.ts';
import {dashboardReturning,renderInShell} from './shell/testkit.tsx';
it('exposes all dashboard traffic series and median latency from actual supplied buckets',async()=>{
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByRole('region',{name:'Response time, median'})).toBeInTheDocument();
 expect(screen.getByRole('region',{name:'Requests per minute'})).toBeInTheDocument();
 expect(screen.getByRole('region',{name:'Response time, 95th percentile'})).toBeInTheDocument();
 expect(screen.getByText('Pending outbox messages')).toBeInTheDocument();
});
it('does not fabricate statistics when the source is unavailable',async()=>{
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'unavailable'})});
 expect(await screen.findByText('Not available')).toHaveAttribute('role','status');
 expect(screen.queryByRole('figure')).not.toBeInTheDocument();
});
