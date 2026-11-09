import {describe,expect,it,vi} from 'vitest';
import {prepareSenderDelete,senderDeleteRights} from './senderDelete.ts';
import {mkEmail,MAILBOXES} from './appHarness.tsx';
import type {Email,JmapRequest,JmapResponse} from '../api/jmap-types.ts';

function fixture(emails:Email[],alter?:(query:Record<string,unknown>,get:Record<string,unknown>,page:number)=>void){
  const calls:JmapRequest[]=[];
  const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
    calls.push(request);const args=request.methodCalls[0]![1],position=Number(args['position']),limit=Number(args['limit']);
    const list=emails.slice(position,position+limit),query={accountId:'acct',queryState:'stable',position,total:emails.length,ids:list.map(email=>email.id)},get={accountId:'acct',list,notFound:[]};
    alter?.(query,get,calls.length);
    return{methodResponses:[['Email/query',query,'q'],['Email/get',get,'g']],sessionState:'s'};
  });return{jmap,calls};
}

describe('whole-account sender Trash planning',()=>{
  it('queries beyond loaded rows and verifies exact sender addresses across folders',async()=>{
    const emails=Array.from({length:215},(_,i)=>mkEmail(String(i),{from:[{name:null,email:i===4?'not-ann@example.org':'ANN@example.org'}],mailboxIds:{[i%2?'archive':'inbox']:true}}));
    const client=fixture(emails),plan=await prepareSenderDelete(client,'acct','ann@example.org',MAILBOXES,()=>true);
    expect(client.calls.map(call=>call.methodCalls[0]![1]['position'])).toEqual([0,200]);
    expect(client.calls[0]!.methodCalls[0]![1]['filter']).toEqual({from:'ann@example.org',inMailboxOtherThan:['trash']});
    expect(plan.emails).toHaveLength(214);expect(plan.emails.some(email=>email.id==='4')).toBe(false);
    expect(plan.emails.at(-1)?.id).toBe('214');expect(senderDeleteRights(plan,MAILBOXES)).toBe(true);
    expect(senderDeleteRights(plan,MAILBOXES.map(box=>box.id==='archive'?{...box,myRights:{mayRemoveItems:false}}:box))).toBe(false);
  });
  it.each(['unknown-total','too-many','state-change','duplicate','missing','account-change'])('refuses %s without authorizing a partial operation',async scenario=>{
    const emails=Array.from({length:210},(_,i)=>mkEmail(String(i),{from:[{name:null,email:'ann@example.org'}]}));
    const client=fixture(emails,(query,get,page)=>{
      if(scenario==='unknown-total')delete query['total'];
      if(scenario==='too-many')query['total']=5001;
      if(scenario==='state-change'&&page===2)query['queryState']='changed';
      if(scenario==='duplicate'&&page===2)query['ids']=['0'];
      if(scenario==='missing')get['notFound']=['gone'];
    });
    await expect(prepareSenderDelete(client,'acct','ann@example.org',MAILBOXES,()=>scenario!=='account-change'||client.calls.length===0)).rejects.toThrow();
  });
  it('excludes messages already in Trash even when a server returns them as candidates',async()=>{
    const client=fixture([mkEmail('trashed',{from:[{name:null,email:'ann@example.org'}],mailboxIds:{trash:true}})]);
    expect((await prepareSenderDelete(client,'acct','ann@example.org',MAILBOXES,()=>true)).emails).toEqual([]);
  });
});
