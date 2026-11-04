import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {ReaderToolbar} from './ReaderToolbar.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {Email} from '../api/jmap-types.ts';

const state=vi.hoisted(()=>({values:{} as Record<string,{value:unknown}>}));
vi.mock('../realtime/context.ts',()=>({useRealtime:()=>({subTabs:{tabs:()=>[],activate:vi.fn()}})}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:vi.fn().mockResolvedValue({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t1',emailIds:['m1']}],notFound:[]},'thread']]})})}));
vi.mock('../api/settings.ts',()=>({settings:()=>state.values}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',supportsFromDomainSearch:()=>true,searchMessageField:vi.fn(),mailboxes:()=>[],openMessage:vi.fn(),archiveMessage:vi.fn(),trashMessage:vi.fn(),pinMessage:vi.fn(),markRead:vi.fn(),flagMessage:vi.fn(),markSpam:vi.fn(),exportMessage:vi.fn()})}));

const email={id:'m1',threadId:'t1',subject:'Hello',from:[{name:'Ann',email:'ann@example.org'}],keywords:{$seen:true},mailboxIds:{inbox:true}} as unknown as Email;
const security={value:'full-sanitized' as const,floor:null,onChange:vi.fn()};
const toolbar=()=>render(()=><ReaderToolbar email={email} onReply={vi.fn()} preparing={false} onPrint={vi.fn()} active links={[]} linksTruncated={false} linkScope="a:m1" headersScope="account:message" security={security}/>);
const present=(id:string)=>document.querySelector(`[data-reader-item="${id}"]`)!==null;

beforeEach(()=>{activateAccountPreferences('a');state.values={};});
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();});

describe('reader action row and the mail feature settings',()=>{
 it('offers reply, reply all, forward, download and print by default',()=>{
  toolbar();
  for(const id of ['reply','reply-all','forward','export','print'])expect(present(id),id).toBe(true);
  expect(screen.getByTestId('reader-reply')).toBeInTheDocument();
 });
 it('hides reply and reply all when mail.allowReply is off, leaving forward',()=>{
  state.values={'mail.allowReply':{value:false}};
  toolbar();
  expect(present('reply')).toBe(false);expect(present('reply-all')).toBe(false);
  expect(present('forward')).toBe(true);
 });
 it('hides forward when mail.allowForward is off, leaving reply',()=>{
  state.values={'mail.allowForward':{value:false}};
  toolbar();
  expect(present('forward')).toBe(false);
  expect(present('reply')).toBe(true);
 });
 it('hides print when mail.allowPrint is off and message export when mail.allowDownload is off',()=>{
  state.values={'mail.allowPrint':{value:false},'mail.allowDownload':{value:false}};
  toolbar();
  expect(present('print')).toBe(false);expect(present('export')).toBe(false);
  expect(present('reply')).toBe(true);
 });
});
