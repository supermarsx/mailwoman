import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@solidjs/testing-library';
import {InitialSyncNotice} from './InitialSyncNotice.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({jmap:vi.fn(),refresh:vi.fn()}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',refreshCurrentMailbox:state.refresh})}));
const result=(status:string)=>({methodResponses:[['Mailbox/get',{accountId:'a',mailwomanInitialSync:status},'sync']]});
afterEach(()=>{cleanup();suspendAccountPreferences();vi.useRealTimers();vi.resetAllMocks();});
describe('initial indexing notice',()=>{
 it('refreshes the selected mailbox once when indexing finishes',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockResolvedValueOnce(result('syncing')).mockResolvedValueOnce(result('ready'));
  render(()=><InitialSyncNotice/>);await vi.advanceTimersByTimeAsync(0);expect(screen.getByRole('status')).toBeInTheDocument();expect(state.refresh).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(2000);expect(screen.queryByRole('status')).not.toBeInTheDocument();expect(state.refresh).toHaveBeenCalledTimes(1);await vi.advanceTimersByTimeAsync(10000);expect(state.jmap).toHaveBeenCalledTimes(2);
 });
 it('stops polling after a terminal indexing failure',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockResolvedValue(result('failed'));render(()=><InitialSyncNotice/>);await vi.advanceTimersByTimeAsync(0);expect(screen.getByRole('status')).toHaveTextContent(/could not finish/i);await vi.advanceTimersByTimeAsync(10000);expect(state.jmap).toHaveBeenCalledTimes(1);expect(state.refresh).not.toHaveBeenCalled();
 });
 it('ignores an old authentication generation response',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');let finish:(value:unknown)=>void=()=>undefined;state.jmap.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));render(()=><InitialSyncNotice/>);await vi.advanceTimersByTimeAsync(0);suspendAccountPreferences();finish(result('syncing'));await vi.advanceTimersByTimeAsync(0);expect(screen.queryByRole('status')).not.toBeInTheDocument();expect(state.refresh).not.toHaveBeenCalled();await vi.advanceTimersByTimeAsync(10000);expect(state.jmap).toHaveBeenCalledTimes(1);
 });
});
