import {describe,expect,it} from 'vitest';
import {folderCachePredicate} from './folderCache.ts';

describe('folderCachePredicate',()=>{
 const keep=folderCachePredicate('inbox-1',['m1','m2']);
 it('matches list requests that name the folder and no other',()=>{
  expect(keep(JSON.stringify({methodCalls:[['Email/query',{filter:{inMailbox:'inbox-1'}}]]}),'mailbox')).toBe(true);
  expect(keep(JSON.stringify({methodCalls:[['Email/query',{filter:{inMailbox:'inbox-10'}}]]}),'mailbox')).toBe(false);
 });
 it('matches only the messages known to be in the folder',()=>{
  expect(keep(JSON.stringify(['message','m1']),'message')).toBe(true);
  expect(keep(JSON.stringify(['message','other']),'message')).toBe(false);
  expect(keep('not json','message')).toBe(false);
 });
 it('also clears a durable message from a previous session in the folder',()=>{
  expect(keep(JSON.stringify(['message','previous-session']),'message',{email:{mailboxIds:{'inbox-1':true}}})).toBe(true);
  expect(keep(JSON.stringify(['message','previous-session']),'message',{email:{mailboxIds:{archive:true}}})).toBe(false);
 });
});
