import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {createNetworkCore,type NetworkReply,type NetworkRequest} from './networkCore.ts';
import {workerHttp} from '../api/workerHttp.ts';
vi.mock('../api/workerHttp.ts',()=>({workerHttp:vi.fn()}));
const me={accountId:'a',storageAuthority:'verified_owner_123456',csrfToken:'csrf'};
const context=()=>({base:location.origin,linkId:null,owner:{accountId:'a',storageAuthority:me.storageAuthority}});
const json=(value:unknown)=>new Response(JSON.stringify(value),{headers:{'content-type':'application/json'}});
let core:ReturnType<typeof createNetworkCore>;
const left:NetworkReply[]=[],right:NetworkReply[]=[];
class Socket{
 static all:Socket[]=[];readyState=0;onopen:(()=>void)|null=null;onmessage:((event:{data:string})=>void)|null=null;onerror:(()=>void)|null=null;onclose:((event:{code:number})=>void)|null=null;
 constructor(_url:string){Socket.all.push(this);}send(){}close(){this.readyState=3;}
}
beforeEach(()=>{left.length=0;right.length=0;Socket.all=[];vi.stubGlobal('WebSocket',Socket);core=createNetworkCore();core.connect('left',value=>left.push(value));core.connect('right',value=>right.push(value));vi.mocked(workerHttp).mockImplementation(async(_base,path)=>path.endsWith('/api/me')?json(me):path.endsWith('/jmap/session')?json({apiUrl:'/jmap/api'}):json({sessionState:'s',methodResponses:[]}));});
afterEach(()=>{core.close();vi.unstubAllGlobals();vi.clearAllMocks();});
const request=(id:string,method:NetworkRequest['method'],extra:Partial<NetworkRequest>={}):NetworkRequest=>({id,method,context:context(),...extra});
it('opens one push connection for two windows and keeps it while one remains subscribed',async()=>{
 await core.receive('left',request('1','push.start'));await core.receive('right',request('2','push.start'));
 expect(Socket.all).toHaveLength(1);core.disconnect('left');expect(Socket.all[0]?.readyState).toBe(0);
 core.disconnect('right');expect(Socket.all[0]?.readyState).toBe(3);
});
it('coalesces matching JMAP reads after independently validating both callers',async()=>{
 const body={using:[],methodCalls:[['Email/get',{accountId:'a',ids:['m']},'get']]} as NonNullable<NetworkRequest['body']>;
 await Promise.all([core.receive('left',request('1','jmap',{body})),core.receive('right',request('2','jmap',{body}))]);
 expect(vi.mocked(workerHttp).mock.calls.filter(([_base,path])=>path.endsWith('/jmap/api'))).toHaveLength(1);
 expect(left.at(-1)).toMatchObject({id:'1',result:{sessionState:'s'}});expect(right.at(-1)).toMatchObject({id:'2',result:{sessionState:'s'}});
});
it('rejects a mismatched authority before issuing any JMAP request',async()=>{
 await core.receive('left',request('1','jmap',{context:{...context(),owner:{accountId:'a',storageAuthority:'different_authority_123456'}},body:{using:[],methodCalls:[]}}));
 expect(left.at(-1)).toMatchObject({error:{status:409,code:'accountChanged'}});
 expect(vi.mocked(workerHttp).mock.calls.every(([_base,path])=>path.endsWith('/api/me'))).toBe(true);
});

it('keeps identity HTTP and permits discovery/action WS only with a fresh queued authority check',async()=>{
 await core.receive('left',request('ws','jmap',{context:{...context(),preferWebSocket:true},body:{using:[],methodCalls:[['Email/get',{accountId:'a',ids:['m']},'get']]}}));
 const calls=vi.mocked(workerHttp).mock.calls;
 expect(calls.find(([_base,path])=>path.endsWith('/api/me'))?.[2]?.allowWebSocket).not.toBe(true);
 const discovery=calls.find(([_base,path])=>path.endsWith('/jmap/session'));
 const action=calls.find(([_base,path])=>path.endsWith('/jmap/api'));
 expect(discovery?.[2]?.allowWebSocket).toBe(true);expect(action?.[2]?.allowWebSocket).toBe(true);
 const queued=action?.[2]?.assertAuthority;expect(queued).toBeTypeOf('function');
 await core.receive('left',request('reset','reset'));expect(()=>queued!()).toThrow(/Shared session changed/);
});
it('does not coalesce writes or serve a read without rechecking authorization',async()=>{
 const body={using:[],methodCalls:[['Email/set',{accountId:'a',update:{m:{'keywords/$seen':true}}},'set']]} as NonNullable<NetworkRequest['body']>;
 await Promise.all([core.receive('left',request('1','jmap',{body})),core.receive('right',request('2','jmap',{body}))]);
 expect(vi.mocked(workerHttp).mock.calls.filter(([_base,path])=>path.endsWith('/jmap/api'))).toHaveLength(2);
});

it('stores account-bound view state and refuses snapshots after an authority change',async()=>{
 await core.receive('left',request('push','push.start'));await core.receive('right',request('push-right','push.start'));
 await core.receive('left',request('put','view.put',{view:{topic:'selection:inbox',sender:'left',value:['m']}}));
 expect(right.some(row=>'event' in row&&row.event==='view')).toBe(true);
 await core.receive('right',request('get','view.get',{view:{topic:'selection:inbox',sender:'right'}}));
 expect(right.at(-1)).toMatchObject({id:'get',result:['m']});
 await core.receive('right',request('bad','view.get',{context:{...context(),owner:{accountId:'a',storageAuthority:'other_authority_123456'}},view:{topic:'selection:inbox',sender:'right'}}));
 expect(right.at(-1)).toMatchObject({error:{status:409}});
});

it('does not broadcast a write invalidation while reading calendar expansions',async()=>{
 await core.receive('left',request('push','push.start'));
 await core.receive('left',request('expand','jmap',{body:{using:[],methodCalls:[['CalendarEvent/expand',{accountId:'a',start:'2026-01-01T00:00:00Z',end:'2026-02-01T00:00:00Z'},'events']]}}));
 expect(left.some(row=>'event' in row&&row.event==='invalidate')).toBe(false);
});

it('does not repopulate shared snapshots with an in-flight read older than a pushed change',async()=>{
 await core.receive('left',request('push','push.start'));
 let release:(response:Response)=>void=()=>undefined,started:()=>void=()=>undefined;
 const entered=new Promise<void>(resolve=>{started=resolve;});
 let calls=0;vi.mocked(workerHttp).mockImplementation(async(_base,path)=>{
  if(path.endsWith('/api/me'))return json(me);if(path.endsWith('/jmap/session'))return json({apiUrl:'/jmap/api'});
  calls++;if(calls===1){started();return new Promise<Response>(resolve=>{release=resolve;});}return json({sessionState:'fresh',methodResponses:[]});
 });
 const body={using:[],methodCalls:[['Email/get',{accountId:'a',ids:['m']},'get']]} as NonNullable<NetworkRequest['body']>;
 const first=core.receive('left',request('before','jmap',{body}));await entered;
 Socket.all[0]!.onmessage?.({data:JSON.stringify({'@type':'StateChange',changed:{a:{Email:'new-state'}}})});
 release(json({sessionState:'old',methodResponses:[]}));await first;
 await core.receive('left',request('after','jmap',{body}));expect(calls).toBe(2);expect(left.at(-1)).toMatchObject({result:{sessionState:'fresh'}});
});
