import {beforeEach,describe,expect,it,vi} from 'vitest';
import {bracketed,findSentMessages,forgetSuperseded,isSuperseded} from './sentMessages.ts';

describe('sent messages for the Supersedes picker',()=>{
 it('searches the Sent folder and brackets the Message-IDs',async()=>{
  const jmap=vi.fn(async()=>({methodResponses:[['Email/query',{ids:['e1','e2']},'q'],['Email/get',{list:[
   {id:'e1',subject:'Offer',receivedAt:'2026-10-01T00:00:00Z',sentAt:'2026-10-01T00:00:05Z',messageId:['a@x.example']},
   {id:'e2',subject:null,receivedAt:'2026-09-01T00:00:00Z',messageId:null},
  ]},'g']]}));
  const found=await findSentMessages({jmap} as never,'acct','sent','offer "x"');
  expect(found).toEqual([{id:'e1',messageId:'<a@x.example>',subject:'Offer',date:'2026-10-01T00:00:05Z'}]);
  const query=(jmap.mock.calls[0] as unknown as [{methodCalls:[string,Record<string,unknown>][]}])[0].methodCalls[0]![1];
  expect(query['filter']).toEqual({inMailbox:'sent',text:'offer  x'});
  expect(bracketed('<a@b>')).toBe('<a@b>');
  expect(bracketed('a@b')).toBe('<a@b>');
 });
 it('lists the newest sent mail when the query is empty, and nothing on an error',async()=>{
  const jmap=vi.fn(async()=>({methodResponses:[['error',{type:'x'},'g']]}));
  expect(await findSentMessages({jmap} as never,'acct','sent','  ')).toEqual([]);
  const query=(jmap.mock.calls[0] as unknown as [{methodCalls:[string,Record<string,unknown>][]}])[0].methodCalls[0]![1];
  expect(query['filter']).toEqual({inMailbox:'sent'});
 });
});

describe('Superseded by a newer message',()=>{
 beforeEach(()=>forgetSuperseded());
 const answer=(total:unknown)=>({methodResponses:[['Email/query',{ids:[],total},'q']]});
 it('looks the message up by its Message-ID in the supersedes index, and caches the answer',async()=>{
  const jmap=vi.fn(async()=>answer(1));
  expect(await isSuperseded({jmap} as never,'acct','<Old@Example.org>')).toBe(true);
  expect(await isSuperseded({jmap} as never,'acct','old@example.org')).toBe(true);
  expect(jmap).toHaveBeenCalledOnce();
  const query=(jmap.mock.calls[0] as unknown as [{methodCalls:[string,Record<string,unknown>][]}])[0].methodCalls[0]![1];
  expect(query['filter']).toEqual({text:'supersedes:old@example.org'});
 });
 it('is false for no match, an odd id, or a failing search',async()=>{
  expect(await isSuperseded({jmap:vi.fn(async()=>answer(0))} as never,'acct','new@example.org')).toBe(false);
  const never=vi.fn(async()=>answer(1));
  expect(await isSuperseded({jmap:never} as never,'acct','no-at-sign')).toBe(false);
  expect(await isSuperseded({jmap:never} as never,'acct','a b@example.org')).toBe(false);
  expect(never).not.toHaveBeenCalled();
  expect(await isSuperseded({jmap:vi.fn(async()=>{throw new Error('x');})} as never,'acct','err@example.org')).toBe(false);
  expect(await isSuperseded({jmap:vi.fn(async()=>answer('many'))} as never,'acct','odd@example.org')).toBe(false);
 });
});
