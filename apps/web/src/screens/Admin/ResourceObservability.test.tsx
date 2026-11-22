import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {ResourceObservability} from './ResourceObservability.tsx';
import {loadResourceSnapshot,type ResourceSnapshot} from '../../api/admin/resources.ts';
import {expectNoAxeViolations,expectNoInlineStyle} from './shell/uxTestkit.ts';
vi.mock('../../api/admin/resources.ts',async importOriginal=>({
 ...await importOriginal<typeof import('../../api/admin/resources.ts')>(),
 loadResourceSnapshot:vi.fn(),
}));
const fixture=():ResourceSnapshot=>({observedAt:1700000000,effectiveCpuCores:4,effectiveMemoryLimitBytes:4294967296,effectiveMemoryAvailableBytes:3221225472,hostMemoryTotalBytes:17179869184,hostMemoryAvailableBytes:12884901888,processResidentBytes:536870912,processCpuPercent:null,cpuSampleSeconds:null,filesystemTotalBytes:1099511627776,filesystemAvailableBytes:549755813888,cgroupDetected:false});
beforeEach(()=>{vi.mocked(loadResourceSnapshot).mockReset().mockResolvedValue(fixture());});
afterEach(()=>{vi.useRealTimers();vi.restoreAllMocks();});
it('distinguishes server entitlement, host and process and keeps CPU warmup unknown',async()=>{
 const {container}=render(()=><ResourceObservability/>);
 const process=await screen.findByRole('region',{name:'Server process'});
 expect(within(process).getByText('CPU usage').closest('tr')).toHaveTextContent('Not available');
 expect(process).toHaveTextContent('100% CPU means one logical core');
 expect(process).toHaveTextContent('CPU usage needs two observations');
 expect(screen.getByRole('region',{name:'Effective server capacity'})).toHaveTextContent('4');
 expect(screen.getByRole('region',{name:'Host memory'})).toBeVisible();
 expect(screen.getByRole('region',{name:'Data filesystem'})).toHaveTextContent('Remote database storage is not included');
 expect(screen.queryByText(/Container isolation detected/)).toBeNull();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('preserves CPU usage above one core and refreshes readings',async()=>{
 const report=fixture();report.processCpuPercent=150;report.cpuSampleSeconds=2;report.cgroupDetected=true;
 vi.mocked(loadResourceSnapshot).mockResolvedValue(report);render(()=><ResourceObservability/>);
 const process=await screen.findByRole('region',{name:'Server process'});
 expect(within(process).getByText('CPU usage').closest('tr')).toHaveTextContent('150%');
 expect(screen.getByText(/Container isolation detected/)).toBeVisible();
 fireEvent.click(screen.getByRole('button',{name:'Refresh'}));
 await waitFor(()=>expect(loadResourceSnapshot).toHaveBeenCalledTimes(2));
});
it('polls only while visible and aborts and removes listeners on unmount',async()=>{
 vi.useFakeTimers({toFake:['setInterval','clearInterval']});
 let visible=true;vi.spyOn(document,'visibilityState','get').mockImplementation(()=>visible?'visible':'hidden');
 const {unmount}=render(()=><ResourceObservability/>);
 await screen.findByRole('region',{name:'Server process'});
 await vi.advanceTimersByTimeAsync(10000);expect(loadResourceSnapshot).toHaveBeenCalledTimes(2);
 visible=false;await vi.advanceTimersByTimeAsync(30000);expect(loadResourceSnapshot).toHaveBeenCalledTimes(2);
 visible=true;document.dispatchEvent(new Event('visibilitychange'));
 await waitFor(()=>expect(loadResourceSnapshot).toHaveBeenCalledTimes(3));
 const signal=vi.mocked(loadResourceSnapshot).mock.calls.at(-1)?.[0];
 unmount();expect(signal?.aborted).toBe(true);
 await vi.advanceTimersByTimeAsync(30000);document.dispatchEvent(new Event('visibilitychange'));
 expect(loadResourceSnapshot).toHaveBeenCalledTimes(3);
});
it('does not turn unavailable memory or disk values into zero',async()=>{
 const report=fixture();report.effectiveMemoryLimitBytes=null;report.filesystemAvailableBytes=null;
 vi.mocked(loadResourceSnapshot).mockResolvedValue(report);render(()=><ResourceObservability/>);
 await screen.findByRole('region',{name:'Server process'});
 expect(screen.getByText('Memory limit').closest('tr')).toHaveTextContent('Not available');
 expect(screen.getByText('Filesystem available').closest('tr')).toHaveTextContent('Not available');
});

it('shows available CPU features separately from the selected lowercase kernel',async()=>{
 const report=fixture();report.cpuCapabilities={architecture:'x86_64',availableFeatures:['sse2','avx2','avx512f','avx512bw','aes'],selectedBackend:'avx512',avx2MinBytes:256,avx512MinBytes:512};
 vi.mocked(loadResourceSnapshot).mockResolvedValue(report);
 const {container}=render(()=><ResourceObservability/>);
 const cpu=await screen.findByRole('region',{name:'CPU capabilities'});
 expect(within(cpu).getByText('Architecture').closest('tr')).toHaveTextContent('x86_64');
 expect(within(cpu).getByText('Selected kernel for large inputs').closest('tr')).toHaveTextContent('AVX-512F + BW');
 expect(cpu).toHaveTextContent('sse2 · avx2 · avx512f · avx512bw · aes');
 expect(cpu).toHaveTextContent('Available features are not all used by the app');
 expect(cpu).toHaveTextContent('AVX2 starts at 256 bytes and AVX-512 at 512 bytes when supported');
 expect(within(cpu).queryByRole('table',{name:'Observed acceleration usage'})).toBeNull();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('reports measured calls and bytes without equating available features with observed use',async()=>{
 const report=fixture();report.cpuCapabilities={architecture:'x86_64',availableFeatures:['avx2','avx512f','avx512bw','aes'],selectedBackend:'avx512',avx2MinBytes:256,avx512MinBytes:512};
 report.cpuUsage={scalar:{calls:321,bytes:100},avx2:{calls:2,bytes:4096},avx512:{calls:0,bytes:0}};
 vi.mocked(loadResourceSnapshot).mockResolvedValue(report);
 const {container}=render(()=><ResourceObservability/>);
 const table=await screen.findByRole('table',{name:'Observed acceleration usage'});
 expect(within(table).getByRole('row',{name:/Portable/})).toHaveTextContent('321');
 expect(within(table).getByRole('row',{name:/^AVX2 /})).toHaveTextContent('4 kB');
 expect(within(within(table).getByRole('row',{name:/AVX-512F/})).getAllByRole('cell')[0]).toHaveTextContent(/^0$/);
 expect(screen.getByText(/Cumulative measured ASCII-lowercase/)).toHaveTextContent('AES, TLS and database-encryption acceleration are not measured here');
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('keeps older resource responses usable without CPU capabilities',async()=>{
 render(()=><ResourceObservability/>);await screen.findByRole('region',{name:'Server process'});
 expect(screen.queryByRole('region',{name:'CPU capabilities'})).toBeNull();
});

it('distinguishes preferred SSE2 from available SSE4.2 and shows each observed variant',async()=>{
 const report=fixture();report.cpuCapabilities={architecture:'x86_64',availableFeatures:['sse2','sse3','ssse3','sse4.1','sse4.2','avx','aes'],selectedBackend:'sse2',highestAvailableBackend:'sse42',sseMinBytes:64,avx2MinBytes:256,avx512MinBytes:512};
 report.cpuUsage={scalar:{calls:3,bytes:9},sse2:{calls:7,bytes:1024},sse3:{calls:0,bytes:0},ssse3:{calls:1,bytes:128},sse41:{calls:0,bytes:0},sse42:{calls:0,bytes:0},avx2:{calls:0,bytes:0},avx512:{calls:0,bytes:0}};
 vi.mocked(loadResourceSnapshot).mockResolvedValue(report);const {container}=render(()=><ResourceObservability/>);
 const cpu=await screen.findByRole('region',{name:'CPU capabilities'});
 expect(within(cpu).getByText('Selected kernel for large inputs').closest('tr')).toHaveTextContent('SSE2');
 expect(within(cpu).getByText('Highest available kernel').closest('tr')).toHaveTextContent('SSE4.2');
 expect(cpu).toHaveTextContent('SSE starts at 64 bytes');
 const table=within(cpu).getByRole('table',{name:'Observed acceleration usage'});
 expect(within(table).getByRole('row',{name:/^SSE2 /})).toHaveTextContent('7');
 expect(within(table).getByRole('row',{name:/^SSSE3 /})).toHaveTextContent('128');
 for(const label of ['SSE3','SSE4.1','SSE4.2'])expect(within(within(table).getByRole('row',{name:new RegExp('^'+label.replaceAll('.','\\.')+' ')})).getAllByRole('cell')[0]).toHaveTextContent(/^0$/);
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
