import {afterEach,describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Conversation} from './Conversation.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({jmap:vi.fn(),open:vi.fn()}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',mailboxes:()=>[{id:'archive',name:'Archive'}],openMessage:state.open})}));
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();});
describe('reader conversation',()=>{
 it('opens an explicitly selected member outside the displayed folder',async()=>{
  activateAccountPreferences('a');state.jmap.mockResolvedValueOnce({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t',emailIds:['x']}]},'thread']]}).mockResolvedValueOnce({methodResponses:[['Email/get',{accountId:'a',list:[{id:'x',threadId:'t',subject:'Archived message',mailboxIds:{archive:true}}]},'members']]});
  render(()=><Conversation threadId="t" messageId="current" active/>);expect(state.jmap).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'View conversation'}));
  const member=await screen.findByRole('button',{name:/Archived message/});expect(screen.getByText('Archive')).toBeInTheDocument();fireEvent.click(member);expect(state.open).toHaveBeenCalledWith('x');
 });
 it('drops a late response after replacing the authentication generation',async()=>{
  activateAccountPreferences('a');let finish:(result:unknown)=>void=()=>undefined;state.jmap.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
  render(()=><Conversation threadId="t" messageId="current" active/>);fireEvent.click(screen.getByRole('button',{name:'View conversation'}));await waitFor(()=>expect(state.jmap).toHaveBeenCalledTimes(1));
  suspendAccountPreferences();activateAccountPreferences('a');finish({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t',emailIds:['old']}]},'thread']]});await Promise.resolve();await Promise.resolve();expect(state.jmap).toHaveBeenCalledTimes(1);expect(screen.getByRole('button',{name:'View conversation'})).toHaveAttribute('aria-expanded','false');
 });
});
