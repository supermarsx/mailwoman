import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {OnlineDatabaseConversion} from './OnlineDatabaseConversion.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {loadOnlineConversion,startOnlineConversion,cancelOnlineConversion,type OnlineConversionReport} from '../../api/admin/onlineDatabase.ts';
import {expectNoAxeViolations,expectNoInlineStyle} from './shell/uxTestkit.ts';
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
