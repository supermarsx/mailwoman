import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {createInitialSync} from './initialSync.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({jmap:vi.fn(),account:'a' as string|null}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=>state.account})}));
const result=(status:string,synced?:boolean,account='a')=>({methodResponses:[['Mailbox/get',{accountId:account,mailwomanInitialSync:status,list:synced===undefined?[]:[{id:'inbox',mailwomanSynced:synced}]},'sync']]});
function Probe(props:{mailbox:()=>string|null;onIndexed:()=>void}){
 const sync=createInitialSync(props.mailbox,props.onIndexed);
 return <div><p data-testid="probe">{`${sync.status()}|${sync.pending()}|${sync.failed()}|${sync.running()}`}</p><button onClick={()=>sync.retry()}>retry</button></div>;
}
const probe=()=>screen.getByTestId('probe').textContent;
afterEach(()=>{cleanup();suspendAccountPreferences();vi.useRealTimers();vi.resetAllMocks();state.account='a';});
describe('first-sync progress',()=>{
 it('is pending until the shown mailbox is indexed, then reloads once',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');const indexed=vi.fn();
  state.jmap.mockResolvedValueOnce(result('syncing',false)).mockResolvedValueOnce(result('syncing',true)).mockResolvedValueOnce(result('ready',true));
  render(()=><Probe mailbox={()=>'inbox'} onIndexed={indexed}/>);
  expect(probe()).toMatch(/^unknown\|true/);
  await vi.advanceTimersByTimeAsync(0);expect(probe()).toBe('syncing|true|false|true');expect(indexed).not.toHaveBeenCalled();
  expect(state.jmap.mock.calls[0]![0].methodCalls[0][1]).toMatchObject({accountId:'a',ids:['inbox'],properties:['id','mailwomanSynced']});
  await vi.advanceTimersByTimeAsync(2000);expect(probe()).toBe('syncing|false|false|true');expect(indexed).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(2000);expect(probe()).toBe('ready|false|false|false');expect(indexed).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(10000);expect(state.jmap).toHaveBeenCalledTimes(3);
 });
 it('treats a server without per-mailbox progress by the account state alone',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockResolvedValueOnce(result('ready'));
  render(()=><Probe mailbox={()=>'inbox'} onIndexed={vi.fn()}/>);await vi.advanceTimersByTimeAsync(0);expect(probe()).toBe('ready|false|false|false');
 });
 it('stops at a failure and resumes with a server retry on request',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockResolvedValueOnce(result('failed',false)).mockResolvedValueOnce(result('retrying',false)).mockResolvedValueOnce(result('ready',true));
  const indexed=vi.fn();render(()=><Probe mailbox={()=>'inbox'} onIndexed={indexed}/>);await vi.advanceTimersByTimeAsync(0);
  expect(probe()).toBe('failed|false|true|false');await vi.advanceTimersByTimeAsync(10000);expect(state.jmap).toHaveBeenCalledTimes(1);
  screen.getByText('retry').click();await vi.advanceTimersByTimeAsync(0);
  expect(state.jmap.mock.calls[1]![0].methodCalls[0][1].mailwomanRetryInitialSync).toBe(true);expect(probe()).toBe('retrying|true|false|true');
  await vi.advanceTimersByTimeAsync(2000);expect(probe()).toBe('ready|false|false|false');expect(indexed).toHaveBeenCalledTimes(1);
  expect(state.jmap.mock.calls[2]![0].methodCalls[0][1].mailwomanRetryInitialSync).toBeUndefined();
 });
 it('falls back to the list state when progress cannot be read',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockRejectedValue(new Error('offline'));
  render(()=><Probe mailbox={()=>'inbox'} onIndexed={vi.fn()}/>);await vi.advanceTimersByTimeAsync(0);expect(probe()).toMatch(/^unknown\|true/);
  await vi.advanceTimersByTimeAsync(6000);expect(probe()).toBe('ready|false|false|false');expect(state.jmap).toHaveBeenCalledTimes(3);
 });
 it('drops a response from an earlier sign-in and starts over for the new account',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');let finish:(value:unknown)=>void=()=>undefined;
  state.jmap.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;})).mockResolvedValueOnce(result('ready',true,'b'));
  const indexed=vi.fn();render(()=><Probe mailbox={()=>'inbox'} onIndexed={indexed}/>);await vi.advanceTimersByTimeAsync(0);
  state.account='b';activateAccountPreferences('b');finish(result('syncing',false));await vi.advanceTimersByTimeAsync(0);
  expect(probe()).toBe('ready|false|false|false');expect(indexed).not.toHaveBeenCalled();expect(state.jmap.mock.calls.at(-1)![0].methodCalls[0][1].accountId).toBe('b');
 });
 it('needs no probe for another folder once the account is indexed',async()=>{
  vi.useFakeTimers();activateAccountPreferences('a');state.jmap.mockResolvedValueOnce(result('ready',true));const [box,setBox]=createSignal<string|null>('inbox');
  render(()=><Probe mailbox={box} onIndexed={vi.fn()}/>);await vi.advanceTimersByTimeAsync(0);setBox('archive');await vi.advanceTimersByTimeAsync(0);
  expect(probe()).toBe('ready|false|false|false');expect(state.jmap).toHaveBeenCalledTimes(1);
 });
 it('never pends without a signed-in account',async()=>{
  state.account=null;render(()=><Probe mailbox={()=>'inbox'} onIndexed={vi.fn()}/>);expect(probe()).toBe('ready|false|false|false');expect(state.jmap).not.toHaveBeenCalled();
 });
});
