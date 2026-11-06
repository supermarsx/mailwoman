import {beforeEach,describe,expect,it,vi} from 'vitest';
import {forgetListCounts,listMessageCount,listQuery} from './listCount.ts';

const answer=(total:unknown,receivedAt='2026-11-01T10:00:00Z')=>({methodResponses:[['Email/query',{total,ids:['e1']},'q'],['Email/get',{list:[{id:'e1',receivedAt}]},'g']]});

describe('list message count',()=>{
 beforeEach(()=>forgetListCounts());
 it('builds the list: search from a List-Id, with or without brackets, and refuses unsearchable ids',()=>{
  expect(listQuery('<News.Example.org>')).toBe('list:news.example.org');
  expect(listQuery('a b')).toBeNull();
  expect(listQuery('a"b')).toBeNull();
  expect(listQuery('')).toBeNull();
 });
 it('asks the engine for the total and the newest date, and caches per list',async()=>{
  const jmap=vi.fn(async()=>answer(42));
  const first=await listMessageCount({jmap} as never,'acct','<news.example.org>');
  expect(first).toEqual({count:42,last:'2026-11-01T10:00:00Z'});
  const call=(jmap.mock.calls[0] as unknown as [{methodCalls:[string,Record<string,unknown>,string][]}])[0].methodCalls[0]!;
  expect(call[0]).toBe('Email/query');
  expect(call[1]['filter']).toEqual({text:'list:news.example.org'});
  expect(call[1]['calculateTotal']).toBe(true);
  await listMessageCount({jmap} as never,'acct','news.example.org');
  expect(jmap).toHaveBeenCalledOnce();
  await listMessageCount({jmap} as never,'other-account','news.example.org');
  expect(jmap).toHaveBeenCalledTimes(2);
 });
 it('expires after a minute',async()=>{
  const jmap=vi.fn(async()=>answer(1));
  await listMessageCount({jmap} as never,'a','x.y',1000);
  await listMessageCount({jmap} as never,'a','x.y',1000+61_000);
  expect(jmap).toHaveBeenCalledTimes(2);
 });
 it('answers null so the badge falls back to the sender count',async()=>{
  expect(await listMessageCount({jmap:vi.fn(async()=>{throw new Error('x');})} as never,'a','x.y')).toBeNull();
  expect(await listMessageCount({jmap:vi.fn(async()=>({methodResponses:[['error',{type:'unsupported'},'q']]}))} as never,'a','x.z')).toBeNull();
  expect(await listMessageCount({jmap:vi.fn(async()=>answer('many'))} as never,'a','x.w')).toBeNull();
 });
});
