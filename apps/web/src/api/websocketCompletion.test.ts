import {afterEach,expect,it,vi} from 'vitest';
import {createWebSocketActions,websocketAction,clearWebSocketActions} from './websocketActions.ts';
import {actionContract,actionHeaders,actionRoute,validActionPath} from './websocketActionRoute.ts';
import {ActionCompletionError,ApiError} from './errors.ts';
import {recoverRequest} from './recovery.ts';
import {markAdminSessionActive} from './adminSession.ts';
import {apiFetch} from './http.ts';
class Socket {
 static all:Socket[]=[];onmessage:((event:{data:string})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;
 sent:string[]=[];closed=false;constructor(readonly endpoint:string){Socket.all.push(this);}send(value:string){this.sent.push(value);}close(){this.closed=true;}
 receive(value:unknown){this.onmessage?.({data:JSON.stringify(value)});}
}
const socketType=Socket as unknown as typeof WebSocket;
const latest=()=>Socket.all.at(-1)!;
const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
const ready=(socket=latest())=>socket.receive({type:'ready',protocol:actionContract.protocol});
const ack=(socket:Socket,index=0,extra:Record<string,unknown>={})=>socket.receive({id:JSON.parse(socket.sent[index]!).id,status:200,headers:{'content-type':'application/json'},body:'{"saved":true}',actionOutcome:'completed',...extra});
afterEach(()=>{clearWebSocketActions();Socket.all=[];document.cookie='mw_csrf=; Max-Age=0; Path=/';document.querySelector('meta[name="mw-console"]')?.remove();vi.useRealTimers();vi.unstubAllGlobals();});
it('uses exact shared actor routes and permits encoded query data without route escapes or forged headers',()=>{
 expect(actionRoute('/api/account/items?q=a%2Fb%20c',{method:'GET'},location.origin,'/ops')?.actor).toBe('webmail');
 expect(actionRoute('/ops/users?q=a%2Fb',{},location.origin,'/ops')).toMatchObject({actor:'admin',cookieEndpoint:'/ops/actions/cookies'});
 for(const path of ['/api/%2e%2e/admin','/api/a%2Fb','/api/a%252fb','/api/a%5cb','/api/a%00','/api/a%ff','/api/a%','/api/../admin'])expect(validActionPath(path)).toBe(false);
 for(const path of ['/api/me','/ops/session','/ops/actions/cookies','/api/login'])expect(actionRoute(path,{},location.origin,'/ops')).toBeUndefined();
 expect(actionRoute('/jmap/session',{},location.origin,'/ops')?.actor).toBe('webmail');
 expect(actionHeaders([['X-CSRF-Token','one'],['x-csrf-token','two']])).toBeUndefined();
 for(const header of ['cookie','authorization','host','x-forwarded-for'])expect(actionHeaders({[header]:'forged'})).toBeUndefined();
 expect(actionRoute('/api/account', {headers:{accept:'x'.repeat(actionContract.limits.headerBytes+1)}},location.origin,'/ops')).toBeUndefined();
});
it('bounds count and bytes without busy HTTP spill and rechecks authority at the actual queued send',async()=>{
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',maxInFlight:1,WebSocketImpl:socketType});
 let current=true;const first=client.send('/jmap/api',{});ready();await flush();
 const next=client.send('/jmap/api',{},()=>{if(!current)throw new ApiError(409,'Changed',{code:'actionNotDispatched'});});
 const rejected=expect(next).rejects.toMatchObject({code:'actionNotDispatched'});current=false;ack(latest());await first;await rejected;expect(latest().sent).toHaveLength(1);
 // Writes: a queued read is handed back for HTTP on close (t52w), a queued write never spills.
 const held=client.send('/jmap/api',{method:'POST'});await flush();const queued=Array.from({length:16},()=>client.send('/jmap/api',{method:'POST'}));
 await expect(client.send('/jmap/api',{})).rejects.toMatchObject({code:'actionNotDispatchedBusy'});
 const stopped=queued.map(value=>expect(value).rejects.toMatchObject({code:'actionNotDispatched'}));const unknown=expect(held).rejects.toMatchObject({code:'outcomeUnknown'});client.close();await unknown;await Promise.all(stopped);
 const bytesClient=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',maxInFlight:1,WebSocketImpl:socketType});
 const body=JSON.stringify({publicFixture:'x'.repeat(1500000)}),requests=Array.from({length:5},()=>bytesClient.send('/jmap/api',{method:'POST',body}));
 await expect(bytesClient.send('/jmap/api',{method:'POST',body})).rejects.toMatchObject({code:'actionNotDispatchedBusy'});
 const cancelled=requests.map(value=>expect(value).rejects.toMatchObject({code:'actionNotDispatched'}));bytesClient.close();await Promise.all(cancelled);
});
it('requires explicit notDispatched AND HTTP-required before any sent request can fall back',async()=>{
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType});
 const request=client.send('/jmap/api',{});ready();await flush();ack(latest(),0,{status:403,actionOutcome:'notDispatched',httpRequired:true});await expect(request).resolves.toBeUndefined();
 const refused=client.send('/jmap/api',{});ready();await flush();const notSent=expect(refused).rejects.toMatchObject({code:'actionNotDispatched'});ack(latest(),0,{status:503,actionOutcome:'notDispatched'});await notSent;
 const next=client.send('/jmap/api',{});ready();await flush();const failure=expect(next).rejects.toMatchObject({code:'outcomeUnknown'});ack(latest(),0,{status:403,actionOutcome:'unknown',httpRequired:true});await failure;
});
it('keeps webmail separate while admin serializes through cookie delivery and a fresh handshake',async()=>{
 vi.stubGlobal('WebSocket',socketType);document.cookie='mw_csrf=old; Path=/';
 let deliver!:()=>void;const delivered=new Promise<void>(resolve=>{deliver=resolve;});const fetch=vi.fn(async(_input:unknown,_options?:RequestInit)=>{await delivered;document.cookie='mw_csrf=new; Path=/';return new Response('{"delivered":true}',{headers:{'content-type':'application/json'}});});vi.stubGlobal('fetch',fetch);
 const init={method:'PATCH',headers:{'content-type':'application/json','x-csrf-token':'old'},body:'{"publicFixture":true}'};
 const first=websocketAction('/admin/settings',init,location.origin);await flush();const admin=latest();expect(admin.endpoint).toContain('/admin/actions');ready(admin);await flush();
 const second=websocketAction('/admin/settings',init,location.origin);const mail=websocketAction('/jmap/session',{},location.origin);await flush();const webmail=latest();expect(webmail.endpoint).toContain('/jmap/actions');ready(webmail);await flush();ack(webmail);await mail;
 ack(admin,0,{cookieDelivery:{token:'a'.repeat(64),expiresInMs:30000}});await flush();expect(fetch).toHaveBeenCalledTimes(1);expect(admin.sent).toHaveLength(1);expect(Socket.all).toHaveLength(2);
 expect(fetch.mock.calls[0]?.[0]).toBe(`${location.origin}/admin/actions/cookies`);deliver();expect((await first)?.status).toBe(200);await flush();const renewed=latest();expect(renewed).not.toBe(admin);expect(admin.closed).toBe(true);ready(renewed);await flush();
 expect(JSON.parse(renewed.sent[0]!).headers['x-csrf-token']).toBe('new');ack(renewed);expect((await second)?.status).toBe(200);
});
it('retains completed result on failed handoff and prevents automatic keyed recovery or queued action replay',async()=>{
 vi.stubGlobal('WebSocket',socketType);document.cookie='mw_csrf=old; Path=/';const fetch=vi.fn(async()=>new Response('{}',{status:403}));vi.stubGlobal('fetch',fetch);
 const init={method:'POST',headers:{'content-type':'application/json'},body:'{"publicFixture":true}'};
 const send=vi.fn(()=>websocketAction('/admin/settings',init,location.origin).then(value=>value!));
 const result=recoverRequest('/admin/settings',init,send,{idempotent:true,idempotencyKey:'fixture-once',timeoutMs:0});await flush();ready();await flush();
 const failure=expect(result).rejects.toBeInstanceOf(ActionCompletionError);ack(latest(),0,{cookieDelivery:{token:'b'.repeat(64),expiresInMs:30000}});await failure;expect(send).toHaveBeenCalledTimes(1);expect(fetch).toHaveBeenCalledTimes(1);
 const error=await result.catch(error=>error) as ActionCompletionError;expect(await error.actionResponse.json()).toEqual({saved:true});
 await expect(websocketAction('/admin/settings',init,location.origin)).rejects.toMatchObject({code:'actionNotDispatched'});expect(Socket.all).toHaveLength(1);
});
it('retries only the same delivery receipt after a lost ACK using newly delivered CSRF',async()=>{
 vi.stubGlobal('WebSocket',socketType);document.cookie='mw_csrf=old; Path=/';
 const fetch=vi.fn(async(_input:unknown,options?:RequestInit):Promise<Response>=>{if(fetch.mock.calls.length===1){document.cookie='mw_csrf=new; Path=/';throw new TypeError('lost control ACK');}expect(new Headers(options?.headers).get('x-csrf-token')).toBe('new');return new Response('{"delivered":true}',{headers:{'content-type':'application/json'}});});vi.stubGlobal('fetch',fetch);
 const request=websocketAction('/admin/settings',{method:'POST',headers:{'content-type':'application/json'},body:'{}'},location.origin);await flush();ready();await flush();const old=latest();ack(old,0,{cookieDelivery:{token:'c'.repeat(64),expiresInMs:30000}});expect((await request)?.status).toBe(200);
 expect(fetch).toHaveBeenCalledTimes(2);expect(fetch.mock.calls[0]?.[1]?.body).toBe(fetch.mock.calls[1]?.[1]?.body);expect(old.sent).toHaveLength(1);
});
it('rejects queued old-admin work when identity generation changes rather than adopting refreshed cookies',async()=>{
 vi.stubGlobal('WebSocket',socketType);const first=websocketAction('/admin/settings',{},location.origin);await flush();ready();await flush();
 const next=websocketAction('/admin/settings',{},location.origin);const rejected=expect(next).rejects.toMatchObject({code:'actionNotDispatched'});markAdminSessionActive();ack(latest());await first;await rejected;expect(latest().sent).toHaveLength(1);
});
it('clears unsent administrator turns immediately and never dispatches them after an old action settles',async()=>{
 vi.stubGlobal('WebSocket',socketType);const first=websocketAction('/admin/settings',{},location.origin);await flush();ready();await flush();const old=latest();
 const next=websocketAction('/admin/settings',{},location.origin);const rejected=expect(next).rejects.toMatchObject({code:'actionNotDispatched'}),unknown=expect(first).rejects.toMatchObject({code:'outcomeUnknown'});
 clearWebSocketActions();await rejected;await unknown;expect(old.sent).toHaveLength(1);expect(old.closed).toBe(true);
});
it('does not replay sent unknown outcomes even with an automatic recovery idempotency key',async()=>{
 const send=vi.fn(async()=>{throw new ApiError(409,'Unknown',{code:'outcomeUnknown'});});await expect(recoverRequest('/jmap/api',{method:'POST'},send,{idempotent:true,idempotencyKey:'once',timeoutMs:0})).rejects.toMatchObject({code:'outcomeUnknown'});expect(send).toHaveBeenCalledTimes(1);
});

it('finishes typed cookie delivery before reporting a server-confirmed completed response-transport failure',async()=>{
 vi.stubGlobal('WebSocket',socketType);const fetch=vi.fn(async()=>new Response('{"delivered":true}',{headers:{'content-type':'application/json'}}));vi.stubGlobal('fetch',fetch);
 const request=websocketAction('/admin/settings',{},location.origin);await flush();ready();await flush();
 const failure=expect(request).rejects.toMatchObject({code:'actionCompletedTransportFailed'});
 ack(latest(),0,{status:502,body:'{"code":"websocketActionCompletedTransportFailure"}',cookieDelivery:{token:'d'.repeat(64),expiresInMs:30000}});
 await failure;expect(fetch).toHaveBeenCalledTimes(1);expect(latest().sent).toHaveLength(1);
});

it('keeps a completed response during cookie completion even if the old socket closes or repeats its ACK',async()=>{
 let finish!:()=>void;const held=new Promise<void>(resolve=>{finish=resolve;});const delivery=vi.fn(async()=>held);
 const client=createWebSocketActions({endpoint:'ws://localhost/admin/actions',WebSocketImpl:socketType,maxInFlight:1,onCookieDelivery:delivery});
 const request=client.send('/admin/settings',{});ready();await flush();const socket=latest();
 const receipt={cookieDelivery:{token:'e'.repeat(64),expiresInMs:30000}};ack(socket,0,receipt);ack(socket,0,receipt);await flush();expect(delivery).toHaveBeenCalledTimes(1);
 socket.onclose?.();finish();expect((await request)?.status).toBe(200);
});

function escapedSignature():RequestInit{
 const body=JSON.stringify({value:'"\\'.repeat(300000)});
 const init={method:'PUT',headers:{'content-type':'application/json','idempotency-key':'owned-signature-write'},body};
 expect(new TextEncoder().encode(body).length).toBeLessThan(actionContract.limits.bodyBytes);
 expect(new TextEncoder().encode(JSON.stringify({id:'',path:'/admin/settings/signature',...init})).length).toBeGreaterThan(actionContract.limits.frameBytes);
 return init;
}
it('treats escaped envelope overflow as locally unsupported before queue or socket admission',async()=>{
 vi.stubGlobal('WebSocket',socketType);const init=escapedSignature();
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType,maxInFlight:1});
 // More than the queue bound must all remain known-unsent unsupported work.
 for(let i=0;i<17;i++)await expect(client.send('/jmap/api',init)).resolves.toBeUndefined();
 await expect(websocketAction('/admin/settings/signature',init,location.origin)).resolves.toBeUndefined();
 expect(Socket.all).toHaveLength(0);
 const normal=client.send('/jmap/session',{});ready();await flush();expect(latest().sent).toHaveLength(1);ack(latest());expect((await normal)?.status).toBe(200);client.close();
});
it('uses exactly one ordinary guarded HTTP send with the original escaped body after local frame overflow',async()=>{
 vi.stubGlobal('WebSocket',socketType);document.cookie='mw_csrf=owned-frame-proof; Path=/';const init=escapedSignature();
 const fetch=vi.fn(async(_input:unknown,_options?:RequestInit)=>new Response('{"saved":true}',{headers:{'content-type':'application/json'}}));vi.stubGlobal('fetch',fetch);
 const response=await apiFetch('/admin/settings/signature',init);expect(response.status).toBe(200);
 expect(Socket.all).toHaveLength(0);expect(fetch).toHaveBeenCalledTimes(1);
 expect(fetch.mock.calls[0]?.[0]).toBe('/admin/settings/signature');
 const actual=fetch.mock.calls[0]?.[1];expect(actual?.body).toBe(init.body);expect(actual?.method).toBe('PUT');
 expect(new Headers(actual?.headers).get('x-csrf-token')).toBe('owned-frame-proof');expect(new Headers(actual?.headers).get('idempotency-key')).toBe('owned-signature-write');
});
