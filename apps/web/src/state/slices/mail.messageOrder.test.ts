import {afterEach,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {makeClient,mkEmail} from '../../components/appHarness.tsx';
import {clearSettings,loadSettings} from '../../api/settings.ts';
import {createMailSlice} from './mail.ts';
import type {JmapRequest} from '../../api/jmap-types.ts';

afterEach(()=>{clearSettings();localStorage.clear();});

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
