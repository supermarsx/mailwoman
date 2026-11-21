import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import type {JmapRequest} from './jmap-types.ts';
import contract from './native-read-intent-contract.json';
const transport=vi.hoisted(()=>({http:vi.fn(),worker:vi.fn()}));
vi.mock('./http.ts',()=>({apiFetch:transport.http}));
vi.mock('./workerHttp.ts',()=>({workerHttp:transport.worker}));
import {createClient} from './client.ts';
import {createNetworkCore} from '../worker/networkCore.ts';
const body=():JmapRequest=>({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/get',{accountId:'owned',ids:['one','one','two'],properties:[...contract.properties]},'headers']]});
// Mock calls are arrays under noUncheckedIndexedAccess: narrow the actual receipt
// before inspecting it, rather than asserting a tuple the mock did not promise.
function requiredCall<T>(calls:readonly T[],index:number):T {
 const call=calls[index];if(call===undefined)throw new Error(`Missing expected transport call ${index}`);return call;
}
const reply=(value:unknown)=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}});
beforeEach(()=>{
 transport.http.mockReset();transport.worker.mockReset();
 transport.http.mockImplementation(async()=>reply({methodResponses:[],sessionState:'state'}));
 transport.worker.mockImplementation(async(_base:string,path:string)=>path.endsWith('/api/me')?reply({accountId:'owned',storageAuthority:'fixture-owner',csrfToken:'fixture-csrf'}):path.endsWith('/jmap/session')?reply({apiUrl:'/jmap/api'}):reply({methodResponses:[],sessionState:'state'}));
});
afterEach(()=>vi.clearAllMocks());
it('real cookie client builder marks DEFAULT36 without rewriting duplicate IDs/body',async()=>{
 const request=body();await createClient('').jmap(request);
 expect(transport.http).toHaveBeenCalledTimes(1);const init=requiredCall(transport.http.mock.calls,0)[1] as RequestInit;
 expect(new Headers(init.headers).get(contract.header)).toBe(contract.value);expect(init.body).toBe(JSON.stringify(request));
});
it('real cookie client builder leaves sequential queries and writes unmarked',async()=>{
 for(const calls of [[['Email/query',{},'query'],...body().methodCalls],[['Email/set',{accountId:'owned',destroy:['one']},'write']]]) {
  await createClient('').jmap({using:body().using,methodCalls:calls as JmapRequest['methodCalls']});
 }
 for(const call of transport.http.mock.calls)expect(new Headers((call[1] as RequestInit).headers).has(contract.header)).toBe(false);
});
it('real shared worker builder marks DEFAULT36 after fresh owner/session validation',async()=>{
 const core=createNetworkCore(),replies:unknown[]=[];core.connect('page',value=>replies.push(value));
 try {
  const request=body();await core.receive('page',{id:'read',method:'jmap',context:{base:`${location.origin}/`,linkId:null,owner:{accountId:'owned',storageAuthority:'fixture-owner'}},body:request});
  expect(replies.at(-1)).toMatchObject({id:'read',result:{sessionState:'state'}});
  const actions=transport.worker.mock.calls.filter((call)=>call[1].endsWith('/jmap/api'));expect(actions).toHaveLength(1);
  const init=requiredCall(actions,0)[2] as RequestInit;expect(new Headers(init.headers).get(contract.header)).toBe(contract.value);expect(init.body).toBe(JSON.stringify(request));
  expect(requiredCall(transport.worker.mock.calls,0)[1]).toMatch(/\/api\/me$/);expect(requiredCall(transport.worker.mock.calls,1)[1]).toMatch(/\/jmap\/session$/);
 }finally{core.close();}
});
it('real shared worker never marks sequential method calls',async()=>{
 const core=createNetworkCore(),replies:unknown[]=[];core.connect('page',value=>replies.push(value));
 try {
  const request=body();request.methodCalls.unshift(['Email/query',{},'query']);
  await core.receive('page',{id:'query',method:'jmap',context:{base:`${location.origin}/`,linkId:null},body:request});
  expect(replies.at(-1)).toMatchObject({id:'query',result:{sessionState:'state'}});
  const actions=transport.worker.mock.calls.filter((call)=>call[1].endsWith('/jmap/api'));expect(actions).toHaveLength(1);
  expect(new Headers((requiredCall(actions,0)[2] as RequestInit).headers).has(contract.header)).toBe(false);
 }finally{core.close();}
});
