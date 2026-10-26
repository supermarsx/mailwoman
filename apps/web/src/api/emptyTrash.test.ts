import {expect,it,vi} from 'vitest';
import {CAP_CORE,CAP_MAIL,type JmapRequest,type JmapResponse} from './jmap-types.ts';
import {emptyTrash,prepareTrash,type TrashSnapshot} from './emptyTrash.ts';
function response(name:string,body:Record<string,unknown>,id:string):JmapResponse{return {methodResponses:[[name,body,id]],sessionState:'s'};}
const authority=()=>({current:()=>true,signal:new AbortController().signal});
const snapshot:TrashSnapshot={accountId:'a',mailboxId:'trash',ids:['one','two'],more:false};
const folder=()=>response('Mailbox/get',{accountId:'a',mailwomanFolderRoleGuard:true,list:[{id:'trash',role:'trash'}]},'box');
it('prepares a stable snapshot without mutation and rejects another folder role',async()=>{
 const jmap=vi.fn().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/query',{accountId:'a',position:0,queryState:'q',ids:['one','two']},'query'));
 expect(await prepareTrash({jmap},'a','trash',authority())).toEqual(snapshot);expect(jmap.mock.calls.map(call=>(call[0] as JmapRequest).methodCalls[0]?.[0])).toEqual(['Mailbox/get','Email/query']);
 jmap.mockResolvedValueOnce(response('Mailbox/get',{accountId:'a',list:[{id:'trash',role:'inbox'}]},'box'));await expect(prepareTrash({jmap},'a','trash',authority())).rejects.toThrow();
});
it('rechecks membership, carries ifInState and counts only confirmed destruction',async()=>{
 const jmap=vi.fn().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/get',{accountId:'a',state:'current',list:[{id:'one',mailboxIds:{trash:true}},{id:'two',mailboxIds:{inbox:true}}]},'messages')).mockResolvedValueOnce(response('Email/set',{accountId:'a',destroyed:['one']},'delete'));
 expect(await emptyTrash({jmap},snapshot,authority())).toEqual({deleted:1,skipped:1,remaining:0,uncertain:0,stopped:false});
 expect(jmap.mock.calls[2]?.[0]).toEqual({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Email/set',{accountId:'a',ifInState:'current',mailwomanRequireMailboxRole:'trash',destroy:['one']},'delete']]});
});
it('stops without retry after uncertain deletion or partial refusal',async()=>{
 const jmap=vi.fn().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/get',{accountId:'a',state:'s',list:snapshot.ids.map(id=>({id,mailboxIds:{trash:true}}))},'messages')).mockRejectedValueOnce(new Error('network uncertain'));
 expect(await emptyTrash({jmap},snapshot,authority())).toEqual({deleted:0,skipped:0,remaining:0,uncertain:2,stopped:true});expect(jmap).toHaveBeenCalledTimes(3);
 jmap.mockReset().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/get',{accountId:'a',state:'s',list:snapshot.ids.map(id=>({id,mailboxIds:{trash:true}}))},'messages')).mockResolvedValueOnce(response('Email/set',{accountId:'a',destroyed:['one'],notDestroyed:{two:{type:'forbidden'}}},'delete'));
 expect(await emptyTrash({jmap},snapshot,authority())).toEqual({deleted:1,skipped:0,remaining:1,uncertain:0,stopped:true});expect(jmap).toHaveBeenCalledTimes(3);
});
it('account replacement during read prevents any deletion',async()=>{
 let current=true;const jmap=vi.fn().mockResolvedValueOnce(folder()).mockImplementationOnce(async()=>{current=false;return response('Email/get',{accountId:'a',state:'s',list:[{id:'one',mailboxIds:{trash:true}}]},'messages');});
 expect((await emptyTrash({jmap},snapshot,{...authority(),current:()=>current})).stopped).toBe(true);expect(jmap).toHaveBeenCalledTimes(2);
});
it('refuses a state-changing paged preview and duplicate message IDs',async()=>{
 const jmap=vi.fn().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/query',{accountId:'a',position:0,queryState:'q',ids:Array.from({length:1000},(_,i)=>String(i))},'query')).mockResolvedValueOnce(response('Email/query',{accountId:'a',position:1000,queryState:'new',ids:['new']},'query'));
 await expect(prepareTrash({jmap},'a','trash',authority())).rejects.toThrow();expect(jmap).toHaveBeenCalledTimes(3);
 jmap.mockReset().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/query',{accountId:'a',position:0,queryState:'q',ids:['one','one']},'query'));await expect(prepareTrash({jmap},'a','trash',authority())).rejects.toThrow();
});

it('does not send native role extensions to an upstream proxy',async()=>{
 const jmap=vi.fn().mockResolvedValueOnce(response('Mailbox/get',{accountId:'a',list:[{id:'trash',role:'trash'}]},'box')).mockResolvedValueOnce(response('Email/get',{accountId:'a',state:'s',list:[{id:'one',mailboxIds:{trash:true}}]},'messages')).mockResolvedValueOnce(response('Email/set',{accountId:'a',destroyed:['one']},'delete'));
 await emptyTrash({jmap},{...snapshot,ids:['one']},authority());
 expect((jmap.mock.calls[2]?.[0] as JmapRequest).methodCalls[0]?.[1]).not.toHaveProperty('mailwomanRequireMailboxRole');
});

it('cancel during an in-flight deletion accounts its acknowledgement and stops future batches',async()=>{
 let stop=false;let release:(()=>void)|undefined;
 const ids=Array.from({length:101},(_,i)=>String(i));
 const jmap=vi.fn().mockResolvedValueOnce(folder()).mockResolvedValueOnce(response('Email/get',{accountId:'a',state:'s',list:ids.slice(0,100).map(id=>({id,mailboxIds:{trash:true}}))},'messages')).mockImplementationOnce(async()=>{await new Promise<void>(resolve=>{release=resolve;});return response('Email/set',{accountId:'a',destroyed:ids.slice(0,100)},'delete');});
 const running=emptyTrash({jmap},{...snapshot,ids},{...authority(),stopRequested:()=>stop});
 await vi.waitFor(()=>expect(release).toBeDefined());stop=true;release!();
 expect(await running).toEqual({deleted:100,skipped:0,remaining:1,uncertain:0,stopped:true});expect(jmap).toHaveBeenCalledTimes(3);
});
