import {expect,it,vi} from 'vitest';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type Mailbox} from './jmap-types.ts';
import {canEmpty,emptyAbility,expectedOutcome,prepareEmpty,runEmpty,type EmptyPreview} from './emptyFolder.ts';

const box=(id:string,role:string|null,extra:Partial<Mailbox>={}):Mailbox=>({id,name:id[0]!.toUpperCase()+id.slice(1),parentId:null,role,sortOrder:0,totalEmails:3,unreadEmails:0,...extra});
const MAILBOXES=[box('inbox','inbox'),box('work',null),box('trash','trash'),box('junk','junk'),box('saved',null,{mailwomanSearchQuery:'{}'})];
const reply=(name:string,body:Record<string,unknown>,id:string):JmapResponse=>({methodResponses:[[name,body,id]],sessionState:'s'});
const authority=(extra:{stopRequested?:()=>boolean;current?:()=>boolean}={})=>({current:()=>true,signal:new AbortController().signal,...extra});
const native=(over:Record<string,unknown>)=>reply('Mailbox/empty',{accountId:'a',mailboxId:'work',outcome:'trash',trashId:'trash',total:3,processed:0,remaining:3,...over},'empty');
const args=(jmap:ReturnType<typeof vi.fn>,call:number)=>(jmap.mock.calls[call]?.[0] as JmapRequest).methodCalls[0]!;

it('reads the account ability from the session and assumes standard JMAP without it',()=>{
 const session=(ext:unknown)=>({accounts:{a:{name:'a',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_MAIL]:{mailwomanEmptyMailbox:ext}}}}});
 expect(emptyAbility(session({moveToTrash:false,destroy:false}),'a')).toEqual({native:true,moveToTrash:false,destroy:false});
 expect(emptyAbility(session({moveToTrash:true,destroy:false}),'a')).toEqual({native:true,moveToTrash:true,destroy:false});
 expect(emptyAbility(session(undefined),'a')).toEqual({native:false,moveToTrash:true,destroy:true});
});

it('moves ordinary folders to Trash, empties Trash, Junk and Trash-less accounts for good, and hides what cannot work',()=>{
 expect(expectedOutcome(MAILBOXES[1]!,MAILBOXES)).toEqual({outcome:'trash',trashId:'trash'});
 expect(expectedOutcome(MAILBOXES[0]!,MAILBOXES).outcome).toBe('trash');
 expect(expectedOutcome(MAILBOXES[2]!,MAILBOXES).outcome).toBe('destroy');
 expect(expectedOutcome(MAILBOXES[3]!,MAILBOXES).outcome).toBe('destroy');
 const noTrash=MAILBOXES.filter(item=>item.role!=='trash');
 expect(expectedOutcome(MAILBOXES[1]!,noTrash)).toEqual({outcome:'destroy',trashId:null});
 const all={native:true,moveToTrash:true,destroy:true},pop3={native:true,moveToTrash:false,destroy:false},bridge={native:true,moveToTrash:true,destroy:false};
 expect(canEmpty(MAILBOXES[1]!,MAILBOXES,all)).toBe(true);
 expect(canEmpty(MAILBOXES[4]!,MAILBOXES,all)).toBe(false);
 expect(canEmpty(MAILBOXES[0]!,MAILBOXES,pop3)).toBe(false);
 expect(canEmpty(MAILBOXES[1]!,MAILBOXES,bridge)).toBe(true);
 expect(canEmpty(MAILBOXES[2]!,MAILBOXES,bridge)).toBe(false);
 expect(canEmpty(MAILBOXES[1]!,MAILBOXES,null)).toBe(false);
 expect(canEmpty({...MAILBOXES[1]!,myRights:{mayRemoveItems:false}} as Mailbox,MAILBOXES,all)).toBe(false);
});

it('previews on the server without mutation and adopts its count and outcome when the list was stale',async()=>{
 const jmap=vi.fn()
  .mockResolvedValueOnce(reply('error',{type:'stateMismatch',mailwomanTotal:250},'empty'))
  .mockResolvedValueOnce(reply('error',{type:'stateMismatch',mailwomanOutcome:'destroy'},'empty'))
  .mockResolvedValueOnce(native({outcome:'destroy',trashId:null,total:250,remaining:250}));
 const preview=await prepareEmpty({jmap},'a',MAILBOXES[1]!,MAILBOXES,{native:true,moveToTrash:true,destroy:true},authority());
 expect(preview).toMatchObject({mailboxId:'work',name:'Work',outcome:'destroy',total:250,native:true});
 expect(args(jmap,0)).toEqual(['Mailbox/empty',{accountId:'a',mailboxId:'work',outcome:'trash',expectedTotal:3,limit:0},'empty']);
 expect(args(jmap,1)[1]).toMatchObject({expectedTotal:250,outcome:'trash',limit:0});
 expect(args(jmap,2)[1]).toMatchObject({expectedTotal:250,outcome:'destroy',limit:0});
 jmap.mockReset().mockResolvedValueOnce(reply('error',{type:'forbidden'},'empty'));
 await expect(prepareEmpty({jmap},'a',MAILBOXES[1]!,MAILBOXES,{native:true,moveToTrash:true,destroy:true},authority())).rejects.toThrow();
});

const preview:EmptyPreview={accountId:'a',mailboxId:'work',name:'Work',role:null,outcome:'trash',trashId:'trash',total:250,native:true};

it('runs bounded batches, passing the last count each time, until the folder is empty',async()=>{
 const jmap=vi.fn()
  .mockResolvedValueOnce(native({total:250,processed:100,remaining:150}))
  .mockResolvedValueOnce(native({total:150,processed:100,remaining:50}))
  .mockResolvedValueOnce(native({total:50,processed:50,remaining:0}));
 const progress=vi.fn();
 expect(await runEmpty({jmap},preview,authority(),progress)).toEqual({processed:250,remaining:0,refused:0,uncertain:false,stopped:false});
 // The last batch is capped at the previewed count plus the small allowance (250 + 12).
 expect(jmap.mock.calls.map((_,i)=>args(jmap,i)[1])).toEqual([[250,100],[150,100],[50,62]].map(([expectedTotal,limit])=>({accountId:'a',mailboxId:'work',outcome:'trash',expectedTotal,limit})));
 expect(progress).toHaveBeenCalledTimes(3);
});

it('stops between batches on request, on a refusal, and marks an interrupted batch as uncertain',async()=>{
 let stop=false;
 const jmap=vi.fn().mockImplementationOnce(async()=>{stop=true;return native({processed:100,remaining:150});});
 expect(await runEmpty({jmap},preview,authority({stopRequested:()=>stop}))).toEqual({processed:100,remaining:150,refused:0,uncertain:false,stopped:true});
 expect(jmap).toHaveBeenCalledTimes(1);
 jmap.mockReset().mockResolvedValueOnce(native({processed:98,remaining:152,notProcessed:{x:{type:'forbidden'},y:{type:'forbidden'}}}));
 expect(await runEmpty({jmap},preview,authority())).toMatchObject({processed:98,refused:2,stopped:true});
 jmap.mockReset().mockResolvedValueOnce(reply('error',{type:'stateMismatch',mailwomanTotal:900},'empty'));
 expect(await runEmpty({jmap},preview,authority())).toMatchObject({processed:0,stopped:true,uncertain:false});
 jmap.mockReset().mockRejectedValueOnce(new Error('network'));
 expect(await runEmpty({jmap},preview,authority())).toMatchObject({processed:0,stopped:true,uncertain:true});
 expect(jmap).toHaveBeenCalledTimes(1);
});

it('does not chase mail that keeps arriving past the previewed count',async()=>{
 const small={...preview,total:3};
 // Every batch is replaced by as many new arrivals as it removed.
 const jmap=vi.fn().mockImplementation(async(request:JmapRequest)=>{const limit=(request.methodCalls[0]![1] as {limit:number}).limit;return native({processed:limit,remaining:3});});
 const result=await runEmpty({jmap},small,authority());
 expect(result.processed).toBeLessThanOrEqual(5);expect(result.stopped).toBe(true);
 expect(jmap.mock.calls.length).toBeLessThanOrEqual(2);
});

it('uses standard Email/query and Email/set against an upstream JMAP server',async()=>{
 const upstream={...preview,native:false,total:2};
 const jmap=vi.fn()
  .mockResolvedValueOnce(reply('Email/query',{accountId:'a',queryState:'q',position:0,ids:['m1','m2']},'page'))
  .mockResolvedValueOnce({methodResponses:[['Email/set',{accountId:'a',updated:{m1:null,m2:null}},'set'],['Email/query',{accountId:'a',queryState:'q',position:0,ids:[],total:0},'left']],sessionState:'s'});
 expect(await runEmpty({jmap},upstream,authority())).toEqual({processed:2,remaining:0,refused:0,uncertain:false,stopped:false});
 const set=(jmap.mock.calls[1]?.[0] as JmapRequest).methodCalls[0]!;
 expect(set[0]).toBe('Email/set');
 expect(set[1]).toEqual({accountId:'a',update:{m1:{'mailboxIds/work':null,'mailboxIds/trash':true},m2:{'mailboxIds/work':null,'mailboxIds/trash':true}}});
 const permanent={...upstream,mailboxId:'trash',outcome:'destroy' as const,trashId:null};
 jmap.mockReset()
  .mockResolvedValueOnce(reply('Email/query',{accountId:'a',queryState:'q',position:0,ids:['m1']},'page'))
  .mockResolvedValueOnce({methodResponses:[['Email/set',{accountId:'a',destroyed:['m1']},'set'],['Email/query',{accountId:'a',queryState:'q',position:0,ids:[],total:0},'left']],sessionState:'s'});
 expect((await runEmpty({jmap},permanent,authority())).processed).toBe(1);
 expect((jmap.mock.calls[1]?.[0] as JmapRequest).methodCalls[0]![1]).toEqual({accountId:'a',destroy:['m1']});
});

it('never mutates after the account changed',async()=>{
 let current=true;
 const jmap=vi.fn().mockImplementationOnce(async()=>{current=false;return native({processed:100,remaining:150});});
 const result=await runEmpty({jmap},preview,authority({current:()=>current}));
 expect(result.stopped).toBe(true);expect(jmap).toHaveBeenCalledTimes(1);
});
