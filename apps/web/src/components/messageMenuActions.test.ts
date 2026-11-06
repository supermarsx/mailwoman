import {afterEach,describe,expect,it,vi} from 'vitest';
import type {Email} from '../api/jmap-types.ts';
import {pendingRuleDraft,takeRuleDraft} from '../state/ruleDraft.ts';
import {settingsRequest} from '../state/settingsRequest.ts';

vi.mock('../i18n',()=>({t:(id:string)=>id,loadCatalog:async()=>undefined,isolate:(v:string)=>v}));
const jmap=vi.hoisted(()=>vi.fn());
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap})}));
vi.mock('./messageQuickActions.ts',()=>({quickPrint:vi.fn()}));

import {blockSender,createFilterFrom,dialogActionsFor,messageActionRows} from './messageMenuActions.ts';

const boxes=[{id:'in',name:'Inbox',role:'inbox'},{id:'junk',name:'Junk',role:'junk'},{id:'misc',name:'Misc',role:null}];
const email=(over:Partial<Email>={}):Email=>({id:'m1',mailboxIds:{in:true},from:[{name:'A',email:'a@example.org'}],...over} as Email);
const app=(extra:Record<string,unknown>={})=>({mailboxes:()=>boxes,accountId:()=>'acct',showToast:vi.fn(),moveMessage:vi.fn(async()=>undefined),markSpam:vi.fn(async()=>undefined),...extra}) as never;

afterEach(()=>{jmap.mockReset();takeRuleDraft();});

describe('message actions no other menu offered',()=>{
 it('lists Move to, View source, Create filter and Block sender for an inbox message',()=>{
  const ids=messageActionRows(app(),email()).map(r=>r.id);
  expect(ids).toEqual(['move-to','view-source','create-filter','block-sender']);
 });
 it('adds Not spam only for a message in the junk folder, and moves it to the inbox',async()=>{
  const a=app();
  const rows=messageActionRows(a,email({mailboxIds:{junk:true}}));
  const notSpam=rows.find(r=>r.id==='not-spam')!;
  await notSpam.run();
  expect((a as unknown as {moveMessage:ReturnType<typeof vi.fn>}).moveMessage).toHaveBeenCalledWith('m1','in');
 });
 it('leaves out a sender-based action when the message has no sender',()=>{
  const ids=messageActionRows(app(),email({from:[]})).map(r=>r.id);
  expect(ids).toEqual(['move-to','view-source']);
 });
 it('Create filter queues a rule for the sender and asks for the Rules section',()=>{
  const before=settingsRequest();
  createFilterFrom(email());
  expect(pendingRuleDraft()?.conditions).toEqual([{type:'from',op:'is',value:'a@example.org'}]);
  expect(settingsRequest()?.section).toBe('rules');
  expect(settingsRequest()?.seq).not.toBe(before?.seq);
 });
 it('Block sender calls SenderControl/set block and reports whether it was applied',async()=>{
  jmap.mockResolvedValueOnce({methodResponses:[['SenderControl/set',{updated:true},'sc']]});
  expect(await blockSender(app(),email())).toBe(true);
  const sent=jmap.mock.calls[0]![0].methodCalls[0];
  expect(sent[0]).toBe('SenderControl/set');
  expect(sent[1]).toMatchObject({action:'block',address:'a@example.org',emailId:'m1'});
  jmap.mockResolvedValueOnce({methodResponses:[['SenderControl/set',{updated:false},'sc']]});
  const failing=app();
  expect(await blockSender(failing,email())).toBe(false);
  expect((failing as unknown as {showToast:ReturnType<typeof vi.fn>}).showToast).toHaveBeenCalledWith('error','mail-context-failed');
 });
 it('the Properties dialog gets the same callbacks',()=>{
  const inbox=dialogActionsFor(app(),email());
  expect(Object.keys(inbox).sort()).toEqual(['onBlockSender','onCreateFilter','onPrint','onSpam','onViewSource']);
  const junk=dialogActionsFor(app(),email({mailboxIds:{junk:true}}));
  expect(junk.onNotSpam).toBeTypeOf('function');
  expect(junk.onSpam).toBeUndefined();
 });
});
