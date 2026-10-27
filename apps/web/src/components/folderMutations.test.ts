import {expect,it,vi} from 'vitest';
import {mutateFolderMessages} from './folderMutations.ts';
import type {JmapRequest,JmapResponse} from '../api/jmap-types.ts';
const reply=(name:string,value:Record<string,unknown>,id:string):JmapResponse=>({sessionState:'s',methodResponses:[[name,value,id]]});
const selected=()=>reply('Email/get',{accountId:'a',state:'before',list:[{id:'one',mailboxIds:{source:true},keywords:{}},{id:'two',mailboxIds:{source:true},keywords:{}},{id:'moved',mailboxIds:{other:true},keywords:{}}]},'selected');
const input=()=>({accountId:'a',mailboxId:'source',virtual:false,ids:['one','two','moved'],operation:{kind:'read' as const,read:true},current:()=>true,signal:new AbortController().signal});
it('updates only selected owned source messages with state CAS and reports partial refusals',async()=>{
 const jmap=vi.fn(async(_request:JmapRequest)=>selected());jmap.mockResolvedValueOnce(selected()).mockResolvedValueOnce(reply('Email/set',{accountId:'a',updated:{one:null},notUpdated:{two:{type:'forbidden'}}},'update'));
 const result=await mutateFolderMessages({jmap},input());
 expect(jmap.mock.calls[1]?.[0].methodCalls[0]?.[1]).toEqual({accountId:'a',ifInState:'before',update:{one:{'keywords/$seen':true},two:{'keywords/$seen':true}}});
 expect(result).toEqual({updated:['one'],refused:['two'],skipped:['moved'],uncertain:[]});
});
it('never retries an uncertain write and preserves omitted response outcomes',async()=>{
 const jmap=vi.fn(async()=>selected());jmap.mockResolvedValueOnce(selected()).mockRejectedValueOnce(new Error('connection lost'));
 expect((await mutateFolderMessages({jmap},input())).uncertain).toEqual(['one','two']);expect(jmap).toHaveBeenCalledTimes(2);
 jmap.mockReset().mockResolvedValueOnce(selected()).mockResolvedValueOnce(reply('Email/set',{accountId:'a',updated:{one:null}},'update'));
 expect(await mutateFolderMessages({jmap},input())).toMatchObject({updated:['one'],uncertain:['two']});
});
it('does not write after the authority changes during preflight',async()=>{
 let current=true;const jmap=vi.fn(async()=>{current=false;return selected();});
 await expect(mutateFolderMessages({jmap},{...input(),current:()=>current})).rejects.toMatchObject({name:'AbortError'});expect(jmap).toHaveBeenCalledOnce();
});
it('rejects a foreign destination before reading or mutating messages',async()=>{
 const jmap=vi.fn(async()=>reply('Mailbox/get',{accountId:'foreign',list:[{id:'target'}]},'target'));
 await expect(mutateFolderMessages({jmap},{...input(),operation:{kind:'move',target:'target'}})).rejects.toThrow('Destination unavailable');expect(jmap).toHaveBeenCalledOnce();
});
it('uses the verified destination and reports a state mismatch as refused, not uncertain',async()=>{
 const jmap=vi.fn(async(_request:JmapRequest)=>selected());jmap.mockResolvedValueOnce(reply('Mailbox/get',{accountId:'a',list:[{id:'target'}]},'target')).mockResolvedValueOnce(selected()).mockResolvedValueOnce(reply('error',{type:'stateMismatch'},'update'));
 const result=await mutateFolderMessages({jmap},{...input(),operation:{kind:'move',target:'target'}});
 expect(jmap.mock.calls[2]?.[0].methodCalls[0]?.[1]).toMatchObject({ifInState:'before',update:{one:{mailboxIds:{target:true}},two:{mailboxIds:{target:true}}}});
 expect(result.refused).toEqual(['one','two']);expect(result.uncertain).toEqual([]);
});
it('allows selected saved-search results without treating the virtual ID as physical membership',async()=>{
 const jmap=vi.fn(async()=>selected());jmap.mockResolvedValueOnce(selected()).mockResolvedValueOnce(reply('Email/set',{accountId:'a',updated:{one:null,two:null,moved:null}},'update'));
 expect((await mutateFolderMessages({jmap},{...input(),mailboxId:'saved-query',virtual:true})).updated).toHaveLength(3);
});
