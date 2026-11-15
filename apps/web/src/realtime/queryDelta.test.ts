import {expect,it,vi} from 'vitest';
import {captureQuery,reconcileQuery,type QuerySnapshot} from './queryDelta.ts';
import type {Client} from '../api/client.ts';
import type {Email,JmapRequest,JmapResponse} from '../api/jmap-types.ts';
const row=(id:string):Email=>({id,subject:id,keywords:{}} as Email);
const snapshot=(values=['a','b','c']):QuerySnapshot=>({using:['urn:ietf:params:jmap:mail'],accountId:'account',query:{accountId:'account',filter:{inMailbox:'inbox'},limit:50},properties:['id','subject','keywords'],queryState:'q1',emailState:'e1',ids:values,rows:values.map(row),total:values.length});
const deltas=(removed:string[],added:{id:string;index:number}[],updated:string[],total:number):JmapResponse=>({sessionState:'s',methodResponses:[['Email/queryChanges',{accountId:'account',oldQueryState:'q1',newQueryState:'q2',removed,added,total},'delta-query'],['Email/changes',{accountId:'account',oldState:'e1',newState:'e2',created:[],updated,destroyed:[],hasMoreChanges:false},'delta-emails']]});
function api(...responses:JmapResponse[]){const call=vi.fn<Client['jmap']>();for(const response of responses)call.mockResolvedValueOnce(response);return call;}
const get=(rows:Email[]):JmapResponse=>({sessionState:'s',methodResponses:[['Email/get',{accountId:'account',state:'e2',list:rows,notFound:[]},'delta-get']]});
it('updates only changed rows and inserts/removes using the new query indices',async()=>{
 const baseline=snapshot(),changed={...row('c'),keywords:{$seen:true}};
 const call=api(deltas(['b'],[{id:'new',index:0}],['c'],3),get([row('new'),changed]));
 const next=await reconcileQuery(baseline,call);
 expect(next?.ids).toEqual(['new','a','c']);expect(next?.rows[1]).toBe(baseline.rows[0]);expect(next?.rows[2]?.keywords).toEqual({$seen:true});
 expect(call.mock.calls[1]?.[0].methodCalls[0]?.[1]['ids']).toEqual(['new','c']);
 expect(call.mock.calls.flatMap(args=>args[0].methodCalls).some(([method])=>method==='Email/query')).toBe(false);
 expect(baseline.ids).toEqual(['a','b','c']);
});
it('requests a fresh baseline after cannotCalculateChanges without modifying old data',async()=>{
 const baseline=snapshot(),call=api({sessionState:'s',methodResponses:[['error',{type:'cannotCalculateChanges'},'delta-query']]});
 expect(await reconcileQuery(baseline,call)).toBeUndefined();expect(baseline.queryState).toBe('q1');
});
it('handles new mail when the cached folder was empty',async()=>{
 const call=api(deltas([],[{id:'new',index:0}],[],1),get([row('new')]));
 expect((await reconcileQuery(snapshot([]),call))?.ids).toEqual(['new']);
});
it('fetches just the missing tail when removals pull uncached rows into view',async()=>{
 const baseline={...snapshot(),query:{...snapshot().query,limit:3},total:5};
 const call=api(deltas(['b'],[],[],4),{sessionState:'s',methodResponses:[['Email/query',{accountId:'account',queryState:'q2',position:2,ids:['d']},'delta-tail']]},get([row('d')]));
 expect((await reconcileQuery(baseline,call))?.ids).toEqual(['a','c','d']);
 expect(call.mock.calls[1]?.[0].methodCalls[0]?.[1]).toMatchObject({position:2,limit:1});
});
it('rejects a disappearing added message and duplicate or invalid insertion indices',async()=>{
 expect(await reconcileQuery(snapshot(),api(deltas([],[{id:'new',index:0}],[],4),get([])))).toBeUndefined();
 expect(await reconcileQuery(snapshot(),api(deltas([],[{id:'new',index:-1}],[],4)))).toBeUndefined();
});
it('only extends a baseline with contiguous pages from the same query state',()=>{
 const request={using:[],methodCalls:[['Email/query',{accountId:'account',position:3,limit:3},'q'],['Email/get',{accountId:'account'},'g']]} as JmapRequest;
 const response={sessionState:'s',methodResponses:[['Email/query',{queryState:'q2',canCalculateChanges:true,ids:['d']},'q'],['Email/get',{state:'e2',list:[row('d')]},'g']]} as JmapResponse;
 expect(captureQuery(request,response,snapshot())).toBeUndefined();
});
