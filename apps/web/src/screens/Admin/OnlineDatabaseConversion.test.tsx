import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {OnlineDatabaseConversion} from './OnlineDatabaseConversion.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {loadOnlineConversion,startOnlineConversion,cancelOnlineConversion,type OnlineConversionReport} from '../../api/admin/onlineDatabase.ts';
import {expectNoAxeViolations,expectNoInlineStyle} from './shell/uxTestkit.ts';
import {formatDateTime,formatDurationMs} from '../../i18n';
import {formatBytes,formatCount} from './Dashboard/format.ts';
vi.mock('../../api/admin/onlineDatabase.ts',()=>({loadOnlineConversion:vi.fn(),startOnlineConversion:vi.fn(),cancelOnlineConversion:vi.fn()}));
const report=(phase='copying'):OnlineConversionReport=>({id:'copy-1',phase,startedAt:1700000000,updatedAt:1700000001,progress:{tableIndex:1,tableName:'bodies',tableCount:229,copiedRows:64,copiedBytes:1048576,replayedKeys:0,pending:3,seedComplete:false,valid:true},error:null,retainedPrevious:false});
beforeEach(()=>{vi.mocked(loadOnlineConversion).mockReset().mockResolvedValue(null);vi.mocked(startOnlineConversion).mockReset().mockResolvedValue(report());vi.mocked(cancelOnlineConversion).mockReset().mockResolvedValue(true);});
afterEach(()=>{vi.useRealTimers();vi.restoreAllMocks();});
it('starts a conversion using server defaults and displays copied data and pending changes',async()=>{
 const {container}=renderInShell(()=><OnlineDatabaseConversion/>);
 const start=await screen.findByRole('button',{name:'Start conversion'});await waitFor(()=>expect(start).not.toBeDisabled());
 fireEvent.click(start);expect(await screen.findByRole('status')).toHaveTextContent('Copying data');
 expect(startOnlineConversion).toHaveBeenCalledTimes(1);expect(start).toBeDisabled();
 expect(screen.getByText('Copied rows').closest('tr')).toHaveTextContent('64');expect(screen.getByText('Pending changes').closest('tr')).toHaveTextContent('3');
 expect(screen.getByText(/Database requests may queue briefly/)).toBeVisible();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('cancels an active copy and reports the actual returned state',async()=>{
 vi.mocked(loadOnlineConversion).mockResolvedValue(report());renderInShell(()=><OnlineDatabaseConversion/>);
 const cancel=await screen.findByRole('button',{name:'Cancel conversion'});await waitFor(()=>expect(cancel).not.toBeDisabled());
 const cancelled=report('cancelled');cancelled.retainedPrevious=true;vi.mocked(loadOnlineConversion).mockResolvedValue(cancelled);
 fireEvent.click(cancel);await waitFor(()=>expect(cancelOnlineConversion).toHaveBeenCalledTimes(1));
 expect(await screen.findByRole('status')).toHaveTextContent('Cancelled');await waitFor(()=>expect(screen.queryByRole('button',{name:'Cancel conversion'})).toBeNull());
 expect(screen.getByText(/previous database generation is retained/)).toBeVisible();
});
it('respects permissions and blocks cancellation during publication',async()=>{
 vi.mocked(loadOnlineConversion).mockResolvedValue(report('publishing'));
 renderInShell(()=><OnlineDatabaseConversion/>,{loadMe:holding(['maintenance.read'])});
 expect(await screen.findByRole('status')).toHaveTextContent('Publishing the new generation');
 expect(screen.getByRole('button',{name:'Start conversion'})).toBeDisabled();expect(screen.getByRole('button',{name:'Cancel conversion'})).toBeDisabled();
});
it('polls only active visible conversions and stops after completion or unmount',async()=>{
 vi.useFakeTimers({toFake:['setInterval','clearInterval']});let visible=true;vi.spyOn(document,'visibilityState','get').mockImplementation(()=>visible?'visible':'hidden');
 vi.mocked(loadOnlineConversion).mockResolvedValue(report());const {unmount}=renderInShell(()=><OnlineDatabaseConversion/>);
 await screen.findByRole('status');await vi.advanceTimersByTimeAsync(2000);expect(loadOnlineConversion).toHaveBeenCalledTimes(2);
 visible=false;await vi.advanceTimersByTimeAsync(6000);expect(loadOnlineConversion).toHaveBeenCalledTimes(2);
 visible=true;document.dispatchEvent(new Event('visibilitychange'));await waitFor(()=>expect(loadOnlineConversion).toHaveBeenCalledTimes(3));
 vi.mocked(loadOnlineConversion).mockResolvedValue(report('completed'));await vi.advanceTimersByTimeAsync(2000);expect(loadOnlineConversion).toHaveBeenCalledTimes(4);
 await vi.advanceTimersByTimeAsync(6000);expect(loadOnlineConversion).toHaveBeenCalledTimes(4);unmount();document.dispatchEvent(new Event('visibilitychange'));await vi.advanceTimersByTimeAsync(6000);expect(loadOnlineConversion).toHaveBeenCalledTimes(4);
});
it('displays server phase-average rates, replayed keys and real start/finish times',async()=>{
 const value=report('completed');Object.assign(value,{finishedAt:1700000120,elapsedMs:120000,copyElapsedMs:4000,replayElapsedMs:2000,averageCopyBytesPerSecond:262144,averageReplayKeysPerSecond:3.5});value.progress!.replayedKeys=7;
 vi.mocked(loadOnlineConversion).mockResolvedValue(value);const {container}=renderInShell(()=><OnlineDatabaseConversion/>);
 await screen.findByRole('status');
 const row=(label:string)=>screen.getByRole('rowheader',{name:label}).closest('tr')!;
 expect(row('Started')).toHaveTextContent(formatDateTime(value.startedAt*1000));
 expect(row('Finished')).toHaveTextContent(formatDateTime(value.finishedAt!*1000));
 expect(row('Elapsed time')).toHaveTextContent(formatDurationMs(120000));
 expect(row('Average copy rate')).toHaveTextContent(`${formatBytes(262144)}/s`);
 expect(row('Average replay rate')).toHaveTextContent(`${formatCount(3.5)} keys/s`);
 expect(row('Replayed keys')).toHaveTextContent('7');
 expect(screen.getByText(/Copy rate uses baseline copied bytes/)).toHaveTextContent('Pending changes count journal events');
 expect(screen.getByText(/Copied bytes measure logical row payloads/)).toBeVisible();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('keeps terminal elapsed time frozen and never invents absent legacy measurements',async()=>{
 vi.useFakeTimers({toFake:['setInterval','clearInterval']});
 const value=report('failed');value.elapsedMs=2300;vi.mocked(loadOnlineConversion).mockResolvedValue(value);
 renderInShell(()=><OnlineDatabaseConversion/>);await screen.findByRole('status');
 const row=(label:string)=>screen.getByRole('rowheader',{name:label}).closest('tr')!;
 expect(row('Elapsed time')).toHaveTextContent(formatDurationMs(2300));
 for(const label of ['Finished','Average copy rate','Average replay rate'])expect(row(label)).toHaveTextContent('Not available');
 await vi.advanceTimersByTimeAsync(120000);expect(loadOnlineConversion).toHaveBeenCalledTimes(1);
 expect(row('Elapsed time')).toHaveTextContent(formatDurationMs(2300));
});
it('shows an active unfinished state and distinguishes measured zero from invalid or missing metrics',async()=>{
 const value=report();value.averageCopyBytesPerSecond=0;value.averageReplayKeysPerSecond=Number.NaN;value.elapsedMs=-1;
 vi.mocked(loadOnlineConversion).mockResolvedValue(value);renderInShell(()=><OnlineDatabaseConversion/>);await screen.findByRole('status');
 const row=(label:string)=>screen.getByRole('rowheader',{name:label}).closest('tr')!;
 expect(row('Finished')).toHaveTextContent('Not finished');
 expect(row('Average copy rate')).toHaveTextContent(`${formatBytes(0)}/s`);
 for(const label of ['Elapsed time','Average replay rate'])expect(row(label)).toHaveTextContent('Not available');
});
