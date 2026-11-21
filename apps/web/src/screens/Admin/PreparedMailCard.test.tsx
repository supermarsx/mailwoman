import {beforeEach,expect,it,vi} from 'vitest';
import {Show} from 'solid-js';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {PreparedMailCard} from './PreparedMailCard.tsx';
import {loadPreparedMailStatus,type PreparedMailStatus} from '../../api/admin/preparedMail.ts';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {renderInShell,holding} from './shell/testkit.tsx';
import {useShell} from './shell/context.ts';
vi.mock('../../api/admin/preparedMail.ts',()=>({loadPreparedMailStatus:vi.fn()}));
vi.mock('./Settings/InlineSettingEditor.tsx',()=>({InlineSettingEditor:(props:{settingKey:string})=><div data-testid={props.settingKey}/>}));
const report:PreparedMailStatus={entries:2,accounts:1,pendingBuilds:3,invalidatedEntries:0,chargedBytes:1024,admittedJobs:4,activePreparers:2,budgetBytes:67108864,maxEntries:256,maxAccounts:128,enabled:false,idleSeconds:30,eligibleCandidates:20,readyCandidateHits:8,candidateMisses:12,completedBuilds:9,evictions:5,seedCalls:2,seededRows:3,rawResults:6,compatibilityValueResults:1,handledInvalidationHints:4,invalidationErrors:1,invalidationPollCalls:10,invalidationAckCalls:4};
beforeEach(()=>{vi.mocked(loadPreparedMailStatus).mockReset().mockResolvedValue(report);});
it('shows actual aggregate work and the real global setting without a capacity claim',async()=>{
 renderInShell(()=><PreparedMailCard settings={[{key:'performance.preparedMail'},{key:'unrelated'}] as AdminSetting[]} onChanged={()=>{}}/>);
 expect(await screen.findByText('8 / 12 / 9 / 5')).toBeVisible();
 expect(screen.getByText('4 / 16; 2 / 2; 3')).toBeVisible();
 expect(screen.getByTestId('performance.preparedMail')).toBeVisible();
 expect(screen.queryByTestId('unrelated')).toBeNull();
 expect(screen.getByText(/not sized for 10,000/)).toBeVisible();
});
it('retains observed data after failed refresh instead of manufacturing zero counts',async()=>{
 renderInShell(()=><PreparedMailCard settings={undefined} onChanged={()=>{}}/>);
 await screen.findByText('8 / 12 / 9 / 5');vi.mocked(loadPreparedMailStatus).mockRejectedValueOnce(new Error('owned status unavailable'));
 fireEvent.click(screen.getByRole('button',{name:'Refresh status'}));
 expect(await screen.findByRole('alert')).toBeVisible();expect(screen.getByText('8 / 12 / 9 / 5')).toBeVisible();
});
it('does not turn unavailable lock-consistent counts into zero',async()=>{
 vi.mocked(loadPreparedMailStatus).mockResolvedValue({...report,entries:null,accounts:null,pendingBuilds:null,budgetBytes:null,maxEntries:null,maxAccounts:null,enabled:null,idleSeconds:null});
 renderInShell(()=><PreparedMailCard settings={undefined} onChanged={()=>{}}/>);
 await screen.findByText('8 / 12 / 9 / 5');expect(screen.queryByText('0 / 256; 0 / 128')).toBeNull();
});
it('aborts an outstanding request and never requests status without permission',async()=>{
 let signal:AbortSignal|undefined;vi.mocked(loadPreparedMailStatus).mockImplementation(async value=>{signal=value;return await new Promise<PreparedMailStatus>(()=>{});});
 const page=renderInShell(()=><PreparedMailCard settings={undefined} onChanged={()=>{}}/>);await waitFor(()=>expect(signal).toBeDefined());page.unmount();expect(signal?.aborted).toBe(true);
 vi.mocked(loadPreparedMailStatus).mockClear();const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><PreparedMailCard settings={undefined} onChanged={()=>{}}/></Show>;};
 renderInShell(()=><Ready/>,{loadMe:holding([])});await screen.findByRole('heading',{name:'Prepared mail metadata cache'});expect(loadPreparedMailStatus).not.toHaveBeenCalled();
});
