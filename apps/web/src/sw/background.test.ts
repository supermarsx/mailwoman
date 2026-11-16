import {afterEach,expect,it,vi} from 'vitest';
import {memoryOutboxStore,type OutboxStore,type QueuedItem} from '../offline/outboxCore.ts';
import {NetworkError} from '../api/errors.ts';
import {CAP_MAIL} from '../api/jmap-types.ts';
const fixture=vi.hoisted(()=>({store:undefined as OutboxStore|undefined}));
vi.mock('../offline/encryptedOutbox.ts',()=>({encryptedOutboxStore:()=>fixture.store!}));
vi.mock('../offline/idb.ts',()=>({idbOutboxStore:()=>({})}));
import {replayBackground} from './background.ts';

const primaryAuthority='verified_primary_authority_123',linkedAuthority='verified_linked_authority_123';
const row=(patch:Partial<QueuedItem>={}):QueuedItem=>({id:'expired',type:'flag',createdAt:1,state:'queued',serverBase:'/mail',cacheBase:'https://mail.example.test/mail',linkId:'expired-link',storageAuthority:linkedAuthority,payload:{accountId:'expired',emailId:'email',keyword:'$flagged',value:true,previousValue:false},...patch});
function setup(refusal:number|'network'){
 fixture.store=memoryOutboxStore([row(),row({id:'healthy',linkId:null,storageAuthority:primaryAuthority,payload:{accountId:'primary',emailId:'email',keyword:'$flagged',value:true,previousValue:false}}),row({id:'other-server',serverBase:'/other'})]);
 const posted=vi.fn(),requests:{url:string;init:RequestInit}[]=[];
 vi.stubGlobal('registration',{scope:'https://mail.example.test/mail/'});vi.stubGlobal('clients',{matchAll:async()=>[{postMessage:posted}]});
 const response=(body:unknown,status=200)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
 vi.stubGlobal('fetch',vi.fn(async(input:string,init:RequestInit)=>{
  requests.push({url:input,init});const headers=new Headers(init.headers),path=new URL(input).pathname;
  if(headers.get('X-Mailwoman-Link')==='expired-link'){
   if(refusal==='network')throw new TypeError('Offline');return response({error:'Linked account requires attention'},refusal);
  }
  expect(headers.has('X-Mailwoman-Link')).toBe(false);
  if(path==='/mail/api/me')return response({accountId:'primary',storageAuthority:primaryAuthority,csrfToken:'fixture-csrf'});
  if(path==='/mail/jmap/session')return response({apiUrl:'/mail/jmap/api',uploadUrl:'/mail/jmap/upload/{accountId}',accounts:{primary:{}},primaryAccounts:{[CAP_MAIL]:'primary'}});
  expect(path).toBe('/mail/jmap/api');expect(headers.get('X-Mailwoman-Account')).toBe('primary');expect(headers.get('X-Mailwoman-Storage-Authority')).toBe(primaryAuthority);expect(headers.get('X-CSRF-Token')).toBe('fixture-csrf');
  const request=JSON.parse(init.body as string) as {methodCalls:[string,Record<string,unknown>,string][]},call=request.methodCalls[0]!;
  expect(call[1]['accountId']).toBe('primary');
  if(call[0]==='Email/get')return response({sessionState:'live',methodResponses:[['Email/get',{accountId:'primary',state:'verified-baseline',list:[{id:'email',keywords:{},mailboxIds:{inbox:true}}]},call[2]]]});
  expect(call[0]).toBe('Email/set');expect(call[1]['ifInState']).toBe('verified-baseline');
  return response({sessionState:'live',methodResponses:[['Email/set',{accountId:'primary',oldState:'verified-baseline',newState:'updated',created:null,updated:{email:null},destroyed:null,notCreated:null,notUpdated:null,notDestroyed:null},call[2]]]});
 }));
 return {posted,requests,store:fixture.store};
}
afterEach(()=>{vi.unstubAllGlobals();});
it.each([401,403,409])('replays a healthy primary queue after an earlier linked mailbox is refused (%s)',async status=>{
 const {store,posted,requests}=setup(status);await replayBackground();
 expect((await store.all()).map(item=>item.id)).toEqual(['expired','other-server']);
 expect(requests.filter(request=>request.url.endsWith('/jmap/api'))).toHaveLength(2);
 for(const {init} of requests){expect(init.credentials).toBe('same-origin');expect(init.redirect).toBe('error');expect(init.cache).toBe('no-store');}
 expect(posted).toHaveBeenCalledExactlyOnceWith({type:'mw-outbox-updated'});
});
it('keeps all pending work and retries on a transport outage instead of treating it as a refused link',async()=>{
 const {store,posted,requests}=setup('network');await expect(replayBackground()).rejects.toBeInstanceOf(NetworkError);
 expect((await store.all()).map(item=>item.id)).toEqual(['expired','healthy','other-server']);expect(requests).toHaveLength(1);expect(posted).not.toHaveBeenCalled();
});
