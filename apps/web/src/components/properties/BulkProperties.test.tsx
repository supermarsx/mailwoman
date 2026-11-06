import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {mkEmail} from '../appHarness.tsx';
import {AppContext} from '../../state/context.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import type {Email,JmapRequest,JmapResponse} from '../../api/jmap-types.ts';
import type {AppState} from '../../state/store.ts';
import {BulkPropertiesDialog} from './BulkProperties.tsx';
import {summarizeSelection,zipEntryName} from './bulkProps.ts';

const h=vi.hoisted(()=>({requests:[] as JmapRequest[],notDestroyed:false}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{},primaryAccounts:{},username:'me@example.org'}),
 jmap:vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  h.requests.push(request);
  const [name,args,id]=request.methodCalls[0]!;
  const ids=(args as {destroy:string[]}).destroy;
  if(h.notDestroyed)return {methodResponses:[[name,{notDestroyed:Object.fromEntries(ids.map(i=>[i,{type:'forbidden',description:'this account is under a legal hold'}]))},id]],sessionState:'s'} as JmapResponse;
  return {methodResponses:[[name,{destroyed:ids},id]],sessionState:'s'} as JmapResponse;
 }),
})}));

const json=(body:unknown)=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function policy(values:Record<string,unknown>):Promise<void> {
 const settings=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:false}]));
 await loadSettings({refresh:true,fetcher:(async()=>json({settings})) as Fetcher});
}
const emails=():Email[]=>[
 mkEmail('a',{subject:'One',from:[{name:'Alice',email:'alice@example.org'}],size:1024,receivedAt:'2026-10-01T08:00:00Z',mailboxIds:{inbox:true},keywords:{$seen:true},hasAttachment:true}),
 mkEmail('b',{subject:'Two',from:[{name:'Alice',email:'ALICE@example.org'}],size:2048,receivedAt:'2026-11-01T08:00:00Z',mailboxIds:{inbox:true},keywords:{}}),
 mkEmail('c',{subject:'Three',from:[{name:'Bob',email:'bob@example.org'}],size:1024,receivedAt:'2026-09-01T08:00:00Z',mailboxIds:{archive:true},keywords:{}}),
];
const refresh=vi.fn(async()=>undefined);
function open(rows=emails()){
 const app={accountId:()=>'acct',mailboxes:()=>[{id:'inbox',name:'Inbox'},{id:'archive',name:'Archive'}],refreshCurrentMailbox:refresh} as unknown as AppState;
 const onClose=vi.fn();
 render(()=><AppContext.Provider value={app}><BulkPropertiesDialog emails={rows} onClose={onClose}/></AppContext.Provider>);
 return {onClose};
}
beforeEach(()=>{h.requests.length=0;h.notDestroyed=false;refresh.mockClear();});
afterEach(()=>{cleanup();clearSettings();});

describe('selection summary',()=>{
 it('adds up count, size, dates, senders, folders, unread and attachments',()=>{
  const s=summarizeSelection(emails());
  expect(s.count).toBe(3);expect(s.totalBytes).toBe(4096);
  expect(s.oldest).toBe('2026-09-01T08:00:00.000Z');expect(s.newest).toBe('2026-11-01T08:00:00.000Z');
  expect(s.senders.map(x=>[x.email.toLowerCase(),x.count])).toEqual([['alice@example.org',2],['bob@example.org',1]]);
  expect(s.folders).toEqual([{id:'inbox',count:2},{id:'archive',count:1}]);
  expect(s.unread).toBe(2);expect(s.withAttachments).toBe(1);
 });
 it('copes with missing sizes, dates and senders',()=>{
  const s=summarizeSelection([{} as never,{size:Number.NaN,receivedAt:'nonsense'} as never]);
  expect(s).toMatchObject({count:2,totalBytes:0,oldest:null,newest:null,senders:[]});
 });
 it('names ZIP entries safely and in order',()=>{
  expect(zipEntryName(0,['Re: a/b','c?'].join(String.fromCharCode(92)))).toBe('0001 - Re a b c.eml');
  expect(zipEntryName(11,null)).toBe('0012 - message.eml');
 });
});

describe('bulk Properties dialog',()=>{
 it('shows the totals',async()=>{
  await policy({});
  open();
  const dialog=await screen.findByTestId('bulk-properties');
  expect(within(dialog).getByTestId('bulk-count').textContent).toBe('3');
  expect(within(dialog).getByTestId('bulk-size').textContent).toBe('4.0 KB');
  expect(within(dialog).getByTestId('bulk-senders').textContent).toContain('2 senders');
  expect(within(dialog).getByTestId('bulk-folders').textContent).toContain('Inbox');
  expect(within(dialog).getByTestId('bulk-folders').textContent).toContain('Archive');
  expect(screen.getByTestId('bulk-zip')).toBeInTheDocument();
 });
 it('deletes permanently after one confirmation, marked as the dialog delete',async()=>{
  await policy({'properties.message.permanentDelete':'allow'});
  const {onClose}=open();
  fireEvent.click(await screen.findByTestId('bulk-delete'));
  const confirm=await screen.findByRole('alertdialog');
  fireEvent.click(within(confirm).getByRole('button',{name:'Delete permanently'}));
  await waitFor(()=>expect(onClose).toHaveBeenCalled());
  const args=h.requests[0]!.methodCalls[0]![1] as Record<string,unknown>;
  expect(args['destroy']).toEqual(['a','b','c']);
  expect(args['mailwomanPermanent']).toBe(true);
  expect(refresh).toHaveBeenCalled();
 });
 it('asks twice when the administrator says confirm-twice',async()=>{
  await policy({'properties.message.permanentDelete':'confirm-twice'});
  open();
  fireEvent.click(await screen.findByTestId('bulk-delete'));
  fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Continue'}));
  await screen.findByTestId('bulk-delete-confirm2');
  expect(h.requests).toHaveLength(0);
 });
 it('turns the delete off when the administrator denies it',async()=>{
  await policy({'properties.message.permanentDelete':'deny'});
  open();
  expect(await screen.findByTestId('bulk-delete')).toBeDisabled();
 });
 it('shows the server reason when a legal hold refuses the delete',async()=>{
  await policy({});
  h.notDestroyed=true;
  const {onClose}=open();
  fireEvent.click(await screen.findByTestId('bulk-delete'));
  const confirm=await screen.findByRole('alertdialog');
  fireEvent.click(within(confirm).getByRole('button',{name:'Delete permanently'}));
  await waitFor(()=>expect(within(screen.getByRole('alertdialog')).getByText('this account is under a legal hold')).toBeInTheDocument());
  expect(onClose).not.toHaveBeenCalled();
 });
 it('hides the ZIP when downloads are off',async()=>{
  await policy({'export.message.download.enabled':false});
  open();
  await screen.findByTestId('bulk-properties');
  expect(screen.queryByTestId('bulk-zip')).toBeNull();
 });
});
