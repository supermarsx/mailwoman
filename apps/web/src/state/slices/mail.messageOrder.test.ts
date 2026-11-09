import {afterEach,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {makeClient,mkEmail} from '../../components/appHarness.tsx';
import {clearSettings,loadSettings} from '../../api/settings.ts';
import {createMailSlice} from './mail.ts';
import type {JmapRequest} from '../../api/jmap-types.ts';

afterEach(()=>{clearSettings();localStorage.clear();});

it('sends the configured display timezone with calendar-day grouping',async()=>{
  const client=makeClient({emails:[mkEmail('a')]});
  await createRoot(async dispose=>{
    try{
      const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});
      await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{
        'mail.messageListOrder':{value:'subject',source:'user',locked:false},
        'mail.messageListGroup':{value:'received',source:'user',locked:false},
        'time.displayZone':{value:'America/New_York',source:'user',locked:false},
      }}),{headers:{'content-type':'application/json'}})});
      await mail.refreshMessageOrder();
      const query=vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls.filter(call=>call[0]==='Email/query')).at(-1)!;
      expect((query[1]['sort'] as unknown[]).slice(0,2)).toEqual([{property:'mailwomanGroupReceived',isAscending:false,timeZone:'America/New_York'},{property:'subject',isAscending:true}]);
    }finally{dispose();}
  });
});

it.each([
  ['newest','receivedAt',false],['oldest','receivedAt',true],
  ['sender','from',true],['sender-desc','from',false],
  ['subject','subject',true],['subject-desc','subject',false],
  ['sent-newest','sentAt',false],['sent-oldest','sentAt',true],
  ['importance','importance',false],['importance-asc','importance',true],
  ['size','size',false],['size-asc','size',true],
] as const)('%s reaches the server and follows a selected grouping comparator',async(order,property,isAscending)=>{
  const client=makeClient({emails:[mkEmail('a')]});
  await createRoot(async dispose=>{
    try{
      const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});
      const settings:Record<string,unknown>={'mail.messageListOrder':{value:order,source:'user',locked:false}};
      const load=()=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings}),{headers:{'content-type':'application/json'}})});
      const lastSort=()=>vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls.filter(call=>call[0]==='Email/query').map(call=>call[1]['sort'])).at(-1);
      await load();await mail.refreshMessageOrder();expect(lastSort()).toEqual(expect.arrayContaining([{property,isAscending}]));
      settings['mail.messageListGroup']={value:'sender',source:'user',locked:false};
      await load();await mail.refreshMessageOrder();
      expect((lastSort() as unknown[]).slice(0,2)).toEqual([{property:'mailwomanGroupFrom',isAscending:true},{property,isAscending}]);
    }finally{dispose();}
  });
});

it('refreshes the loaded search window with its exact semantic filter and new order while push refresh stays quiet',async()=>{
  const emails=Array.from({length:120},(_,index)=>mkEmail(`result-${index}`));
  const client=makeClient({emails});
  const original=client.jmap;
  client.jmap=vi.fn(async(request:JmapRequest)=>{
    const response=await original(request);
    const query=request.methodCalls.find(call=>call[0]==='Email/query');
    if(!query)return response;
    const args=query[1],position=Number(args['position']??0),limit=Number(args['limit']??50);
    const descending=(args['sort'] as {isAscending:boolean}[])[0]?.isAscending!==true;
    const ordered=descending?emails:[...emails].reverse();
    const page=ordered.slice(position,position+limit);
    for(const call of response.methodResponses){
      if(call[0]==='Email/query')Object.assign(call[1],{ids:page.map(email=>email.id),position,total:emails.length});
      if(call[0]==='Email/get')Object.assign(call[1],{list:page});
    }
    return response;
  });
  const queries=()=>vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls.filter(call=>call[0]==='Email/query').map(call=>call[1]));
  await createRoot(async dispose=>{
    try{
      const mail=createMailSlice({client,showToast:vi.fn()});
      await mail.login({username:'me@example.org',password:'p'});
      await mail.searchMessages('from:sender@example.org invoice',{semantic:true});
      const filter=structuredClone(queries().at(-1)!['filter']);
      expect(filter).toMatchObject({semantic:true});
      await mail.loadMore();
      expect(mail.messages()).toHaveLength(100);
      mail.setSelection(['result-50']);
      const before=queries().length;
      await mail.refreshCurrentMailbox();
      expect(queries()).toHaveLength(before);
      await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.messageListOrder':{value:'oldest',source:'user',locked:false}}}),{headers:{'content-type':'application/json'}})});
      await mail.refreshMessageOrder();
      expect(queries()).toHaveLength(before+1);
      expect(queries().at(-1)).toMatchObject({filter,limit:100,sort:[{property:'receivedAt',isAscending:true}]});
      expect(queries().at(-1)!['position']??0).toBe(0);
      expect(mail.messages()[0]?.id).toBe('result-119');
      expect(mail.loadedRange()).toEqual({start:0,end:100});
      expect(mail.selectedIds()).toEqual(new Set(['result-50']));
      expect(mail.searchActive()).toBe(true);
      expect(mail.search()).toBe('from:sender@example.org invoice');
      await mail.loadMore();
      expect(queries().at(-1)).toMatchObject({filter,position:100,limit:50});
      expect(mail.messages()).toHaveLength(120);
    }finally{dispose();}
  });
});

it('supersedes an in-flight append when the user changes the arrangement',async()=>{
 const emails=Array.from({length:120},(_,index)=>mkEmail(`row-${index}`));
 const client=makeClient({emails}),original=client.jmap;let release:(()=>void)|undefined;
 client.jmap=vi.fn(async(request:JmapRequest)=>{
  const response=await original(request),query=request.methodCalls.find(call=>call[0]==='Email/query');if(!query)return response;
  const args=query[1],position=Number(args['position']??0),limit=Number(args['limit']??50);
  const ascending=(args['sort'] as {isAscending:boolean}[])[0]?.isAscending===true;
  const ordered=ascending?[...emails].reverse():emails,page=ordered.slice(position,position+limit);
  for(const call of response.methodResponses){if(call[0]==='Email/query')Object.assign(call[1],{ids:page.map(email=>email.id),position,total:emails.length});if(call[0]==='Email/get')Object.assign(call[1],{list:page});}
  if(position===50)await new Promise<void>(resolve=>{release=resolve;});return response;
 });
 await createRoot(async dispose=>{try{
  const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});
  const append=mail.loadMore();await vi.waitFor(()=>expect(release).toBeTypeOf('function'));
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.messageListOrder':{value:'oldest',source:'user',locked:false}}}))});
  await mail.refreshMessageOrder();expect(mail.messages()[0]?.id).toBe('row-119');expect(mail.loadingMore()).toBe(false);
  release!();await append;expect(mail.messages()[0]?.id).toBe('row-119');expect(mail.messages()).toHaveLength(50);
 }finally{release?.();dispose();}});
});
