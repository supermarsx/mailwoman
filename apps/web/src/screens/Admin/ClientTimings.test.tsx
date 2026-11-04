import {it,expect,vi,beforeEach} from 'vitest';
import {screen} from '@solidjs/testing-library';
import {ObservabilityStats} from './ObservabilityStats.tsx';
import {engineDashboard} from './Dashboard/fixtures.ts';
import {dashboardReturning,holding,renderInShell} from './shell/testkit.tsx';
import {loadRequestHistory} from '../../api/admin/requestHistory.ts';
import {loadClientTiming,type ClientTimingHistory,type ClientTimingStats} from '../../api/admin/clientTiming.ts';
vi.mock('../../api/admin/requestHistory.ts',()=>({loadRequestHistory:vi.fn(),loadNotFoundPaths:vi.fn(()=>Promise.reject(new Error('not in this test')))}));
vi.mock('../../api/admin/clientTiming.ts',async importOriginal=>({...await importOriginal<typeof import('../../api/admin/clientTiming.ts')>(),loadClientTiming:vi.fn()}));
const stat=(count:number,p50:number|null,p95:number|null):ClientTimingStats=>({count,sufficient:p50!==null,meanMs:p50,p50Ms:p50,p75Ms:p50,p95Ms:p95,p99Ms:p95});
function history(over:Partial<ClientTimingHistory['policy']>={}):ClientTimingHistory {
 const none={all:stat(0,null,null),devices:{},series:[]};
 return {from:0,to:7200,bucketSeconds:3600,
  metrics:{timeToUsable:{all:stat(120,1200,4800),devices:{desktop:stat(100,1100,4000),mobile:stat(20,1800,6000)},series:[{t:0,count:60,p50Ms:1100,p95Ms:4000},{t:3600,count:60,p50Ms:1300,p95Ms:5000}]},
   fcp:{all:stat(5,null,null),devices:{},series:[]},ttfb:none,dcl:none,load:none,lcp:none,folderOpen:none},
  policy:{enabled:true,sampleRatePercent:50,minSamples:20,retentionDays:30,...over},resolution:{bins:48,maxRelativeError:0.22},scope:'client',sampled:true};
}
beforeEach(()=>{
 vi.mocked(loadRequestHistory).mockResolvedValue({from:0,to:7200,bucketSeconds:3600,series:[{t:0,requests:1,errors5xx:0,p50Ms:5,p95Ms:10,coverage:'complete'}],coverage:{observedSeconds:3600,requestedSeconds:7200,gaps:[]},policy:{defaultRangeSeconds:3600,maxRangeSeconds:2592000,retentionDays:30},scope:'server',timing:'response-ready'});
});
it('shows median and p95 tiles, a chart, a device table and the sampling note',async()=>{
 vi.mocked(loadClientTiming).mockResolvedValue(history());
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByTestId('ct-timeToUsable-p50Ms')).toHaveTextContent('120 samples');
 expect(screen.getByTestId('ct-timeToUsable-p95Ms')).toBeInTheDocument();
 expect(screen.getByTestId('chart-client-usable')).toBeInTheDocument();
 expect(screen.getByTestId('ct-note')).toHaveTextContent('50% of sessions sampled');
 expect(screen.getByTestId('ct-by-device')).toHaveTextContent('Mobile');
 expect(screen.queryByTestId('ct-disabled')).not.toBeInTheDocument();
});
it('withholds a percentile below the minimum sample count and says why',async()=>{
 vi.mocked(loadClientTiming).mockResolvedValue(history());
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 const tile=await screen.findByTestId('ct-fcp-p50Ms');
 expect(tile).toHaveTextContent('Only 5 samples; at least 20 are needed.');
 expect(screen.getByTestId('ct-lcp-p50Ms')).toHaveTextContent('No samples in this period.');
});
it('says when collection is switched off and when the figures cannot be loaded',async()=>{
 vi.mocked(loadClientTiming).mockResolvedValue(history({enabled:false}));
 const view=renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByTestId('ct-disabled')).toBeInTheDocument();
 view.unmount();
 vi.mocked(loadClientTiming).mockRejectedValue(new Error('down'));
 renderInShell(()=><ObservabilityStats/>,{loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByTestId('ct-failed')).toBeInTheDocument();
});
it('does not ask for browser figures without observability.read',async()=>{
 vi.mocked(loadClientTiming).mockClear();
 renderInShell(()=><ObservabilityStats/>,{loadMe:holding(['dashboard.read']),loadDashboard:dashboardReturning({kind:'ok',data:engineDashboard()})});
 expect(await screen.findByRole('region',{name:'Requests'})).toBeInTheDocument();
 expect(loadClientTiming).not.toHaveBeenCalled();
});
