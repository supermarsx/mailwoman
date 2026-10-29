import {afterEach,describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Show} from 'solid-js';
import {createConversation} from './Conversation.tsx';
/** The trigger now lives in the reader action row (ReaderToolbar.tsx); this harness stands in for it. */
function Conversation(props:{threadId:string;messageId:string;active:boolean}){const c=createConversation({threadId:()=>props.threadId,messageId:()=>props.messageId,active:()=>props.active});return <><Show when={c.available()}><button type="button" aria-label="View conversation" aria-expanded={c.open()} onClick={()=>c.toggle()}/></Show>{c.dialog()}</>;}
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({jmap:vi.fn(),open:vi.fn()}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',mailboxes:()=>[{id:'archive',name:'Archive'}],openMessage:state.open})}));
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();});
describe('reader conversation',()=>{
 it('sizes the thread on open and offers the conversation only for more than one message',async()=>{
  activateAccountPreferences('a');state.jmap.mockResolvedValueOnce({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t',emailIds:['current']}]},'thread']]});
  render(()=><Conversation threadId="t" messageId="current" active/>);
  await waitFor(()=>expect(state.jmap).toHaveBeenCalledTimes(1));await Promise.resolve();
  expect(screen.queryByRole('button',{name:'View conversation'})).toBeNull();
 });

 it('opens an explicitly selected member outside the displayed folder',async()=>{
  activateAccountPreferences('a');state.jmap.mockResolvedValueOnce({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t',emailIds:['x','current']}]},'thread']]}).mockResolvedValueOnce({methodResponses:[['Email/get',{accountId:'a',list:[{id:'x',threadId:'t',subject:'Archived message',mailboxIds:{archive:true}}]},'members']]});
  render(()=><Conversation threadId="t" messageId="current" active/>);fireEvent.click(await screen.findByRole('button',{name:'View conversation'}));expect(state.jmap).toHaveBeenCalledTimes(2);
  const member=await screen.findByRole('button',{name:/Archived message/});expect(screen.getByText('Archive')).toBeInTheDocument();fireEvent.click(member);expect(state.open).toHaveBeenCalledWith('x');
 });
 it('drops a late thread size after replacing the authentication generation',async()=>{
  activateAccountPreferences('a');let finish:(result:unknown)=>void=()=>undefined;state.jmap.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
  render(()=><Conversation threadId="t" messageId="current" active/>);await waitFor(()=>expect(state.jmap).toHaveBeenCalledTimes(1));
  suspendAccountPreferences();activateAccountPreferences('a');finish({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t',emailIds:['old','older']}]},'thread']]});await Promise.resolve();await Promise.resolve();expect(screen.queryByRole('button',{name:'View conversation'})).toBeNull();
 });
});
