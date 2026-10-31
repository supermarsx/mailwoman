import {describe,expect,it,vi} from 'vitest';
import {cleanupCutoff,cleanupMessages,previewCleanup} from './folderMaintenance.ts';
import type {Client} from './client.ts';
import type {Mailbox} from './jmap-types.ts';
const box={id:'inbox',name:'Inbox',role:'inbox'} as Mailbox;
const trash={id:'trash',name:'Trash',role:'trash'} as Mailbox;
const authority=()=>({current:()=>true,signal:new AbortController().signal});
const reply=(method:string,id:string,data:object)=>({methodResponses:[[method,data,id]]});
describe('folder cleanup',()=>{
 it('uses calendar months and years, clamping month ends and leap days',()=>{
  expect(cleanupCutoff(1,'months',new Date('2024-03-31T12:00:00Z'))).toBe('2024-02-29T12:00:00.000Z');
  expect(cleanupCutoff(1,'years',new Date('2024-02-29T12:00:00Z'))).toBe('2023-02-28T12:00:00.000Z');
  expect(()=>cleanupCutoff(0,'weeks')).toThrow();
 });
 it('previews exact folder and cutoff without any mutations',async()=>{
  const jmap=vi.fn(async(..._args:unknown[])=>reply('Email/query','old',{accountId:'a',ids:['x']}));
  expect(await previewCleanup({jmap} as unknown as Client,'a','inbox','2024-01-01T00:00:00Z',authority())).toEqual(['x']);
  expect(jmap).toHaveBeenCalledWith(expect.objectContaining({methodCalls:[['Email/query',expect.objectContaining({filter:{inMailbox:'inbox',before:'2024-01-01T00:00:00Z'}}),'old']]}),expect.anything());
 });
 it('rechecks age and membership and preserves other folder memberships when moving to Trash',async()=>{
  const jmap=vi.fn().mockResolvedValueOnce(reply('Email/get','verify',{accountId:'a',state:'v1',list:[{id:'old',receivedAt:'2023-01-01T00:00:00Z',mailboxIds:{inbox:true,other:true}},{id:'new',receivedAt:'2025-01-01T00:00:00Z',mailboxIds:{inbox:true}},{id:'moved',receivedAt:'2023-01-01T00:00:00Z',mailboxIds:{other:true}}]})).mockResolvedValueOnce(reply('Email/set','clean',{accountId:'a',updated:{old:null}}));
  expect(await cleanupMessages({jmap} as unknown as Client,'a',box,[box,trash],['old','new','moved'],'2024-01-01T00:00:00Z',authority())).toBe(1);
  expect(jmap.mock.calls[1]?.[0].methodCalls[0]).toEqual(['Email/set',{accountId:'a',ifInState:'v1',update:{old:{'mailboxIds/inbox':null,'mailboxIds/trash':true}}},'clean']);
 });
 it('does not retry uncertain mutations and stops when authority is revoked',async()=>{
  const jmap=vi.fn().mockResolvedValueOnce(reply('Email/get','verify',{accountId:'a',state:'v1',list:[{id:'old',receivedAt:'2023-01-01T00:00:00Z',mailboxIds:{inbox:true}}]})).mockRejectedValueOnce(new Error('Network'));
  await expect(cleanupMessages({jmap} as unknown as Client,'a',box,[box,trash],['old'],'2024-01-01T00:00:00Z',authority())).rejects.toThrow('Network');expect(jmap).toHaveBeenCalledTimes(2);
  await expect(previewCleanup({jmap} as unknown as Client,'a','inbox','2024-01-01T00:00:00Z',{...authority(),current:()=>false})).rejects.toThrow('active account');expect(jmap).toHaveBeenCalledTimes(2);
 });
 it('refuses a changed confirmed destination before sending any request',async()=>{
  const jmap=vi.fn();
  await expect(cleanupMessages({jmap} as unknown as Client,'a',box,[box],['old'],'2024-01-01T00:00:00Z',authority(),{outcome:'trash',trashId:'trash'})).rejects.toThrow('destination changed');
  expect(jmap).not.toHaveBeenCalled();
 });
});
