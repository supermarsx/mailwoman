import {afterEach,expect,it,vi} from 'vitest';
import {createWebSocketActions,websocketAction,clearWebSocketActions} from './websocketActions.ts';
class Socket{
 static latest:Socket;onmessage:((event:{data:string})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;
 closed=false;sent:string[]=[];constructor(){Socket.latest=this;}send(value:string){this.sent.push(value);}close(){this.closed=true;}receive(value:unknown){this.onmessage?.({data:JSON.stringify(value)});}
}
afterEach(()=>{clearWebSocketActions();vi.useRealTimers();vi.unstubAllGlobals();});
const socketType=Socket as unknown as typeof WebSocket;
it('sends JSON actions with CSRF, account fences and idempotency keys and matches acknowledgements',async()=>{
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType});
 const request=client.send('/jmap/api',{method:'POST',headers:{'content-type':'application/json','X-CSRF-Token':'csrf','X-Mailwoman-Account':'a','Idempotency-Key':'move-1'},body:'{"methodCalls":[]}'});
 Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();
 const frame=JSON.parse(Socket.latest.sent[0]!);expect(frame.headers['x-csrf-token']).toBe('csrf');expect(frame.headers['idempotency-key']).toBe('move-1');
 Socket.latest.receive({id:frame.id,status:200,headers:{'content-type':'application/json'},body:'{"methodResponses":[]}'});
 expect(await (await request)!.json()).toEqual({methodResponses:[]});client.close();
});
it('falls back only before sending, never after a lost mutation acknowledgement',async()=>{
 vi.useFakeTimers();const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType,timeoutMs:100});
 const request=client.send('/jmap/api',{method:'POST',body:'{}'});Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();
 const rejected=expect(request).rejects.toMatchObject({code:'outcomeUnknown'});Socket.latest.onclose?.();await rejected;expect(Socket.latest.sent).toHaveLength(1);client.close();
 const unavailable=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType});const waiting=unavailable.send('/jmap/api',{});await vi.advanceTimersByTimeAsync(5000);await expect(waiting).resolves.toBeUndefined();expect(Socket.latest.sent).toHaveLength(0);unavailable.close();
});
it('bounds simultaneous actions even when they all wait for the same handshake',async()=>{
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType});
 const requests=Array.from({length:10},()=>client.send('/jmap/api',{method:'POST',body:'{}'}));
 Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();
 expect(Socket.latest.sent).toHaveLength(8);
 for(const frame of [...Socket.latest.sent])Socket.latest.receive({id:JSON.parse(frame).id,status:200,body:'{}'});
 await Promise.resolve();expect(Socket.latest.sent).toHaveLength(10);
 for(const frame of Socket.latest.sent.slice(8))Socket.latest.receive({id:JSON.parse(frame).id,status:200,body:'{}'});
 const responses=await Promise.all(requests);expect(responses.every(value=>value?.status===200)).toBe(true);client.close();
});

it('opens a fresh authenticated socket when the CSRF proof rotates',async()=>{
 vi.stubGlobal('WebSocket',socketType);
 const request=(proof:string)=>websocketAction('/jmap/api',{method:'POST',headers:{'content-type':'application/json','x-csrf-token':proof},body:'{}'},location.origin);
 const first=request('first');Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();
 const before=Socket.latest;before.receive({id:JSON.parse(before.sent[0]!).id,status:200,body:'{}'});await first;
 const next=request('renewed');expect(Socket.latest).not.toBe(before);Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();Socket.latest.receive({id:JSON.parse(Socket.latest.sent[0]!).id,status:200,body:'{}'});expect((await next)?.status).toBe(200);
});

it('carries account JSON reads over the action socket',async()=>{
 vi.stubGlobal('WebSocket',socketType);
 const request=websocketAction('/api/account/drafts',{headers:{'x-mailwoman-account':'a','x-mailwoman-storage-authority':'authority'}},location.origin);
 Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();
 const frame=JSON.parse(Socket.latest.sent[0]!);expect(frame.path).toBe('/api/account/drafts');expect(frame.method).toBe('GET');
 Socket.latest.receive({id:frame.id,status:200,body:'{"items":[]}'});expect(await (await request)!.json()).toEqual({items:[]});
});

it('reports established action-socket losses without treating intentional closure as failure',async()=>{
 const changes=vi.fn();const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType,onConnection:changes});
 const request=client.send('/api/account/preferences',{});Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});
 await Promise.resolve();await Promise.resolve();
 const frame=JSON.parse(Socket.latest.sent[0]!);Socket.latest.receive({id:frame.id,status:200,body:'{}'});await request;
 expect(changes.mock.calls).toEqual([[true]]);Socket.latest.onclose?.();expect(changes.mock.calls).toEqual([[true],[false]]);
 const next=client.send('/api/account/preferences',{});Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});
 await Promise.resolve();await Promise.resolve();
 Socket.latest.receive({id:JSON.parse(Socket.latest.sent[0]!).id,status:200,body:'{}'});await next;
 client.close();expect(changes.mock.calls).toEqual([[true],[false],[true]]);
});

it('drains in-flight sends before retiring a socket for a new CSRF proof',async()=>{
 vi.stubGlobal('WebSocket',socketType);
 const request=(proof:string)=>websocketAction('/jmap/api',{method:'POST',headers:{'content-type':'application/json','x-csrf-token':proof},body:'{}'},location.origin);
 const first=request('old');Socket.latest.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();
 const old=Socket.latest;expect(old.sent).toHaveLength(1);
 const second=request('new');const fresh=Socket.latest;expect(fresh).not.toBe(old);expect(old.closed).toBe(false);
 fresh.receive({type:'ready',protocol:'mailwoman-actions-v1'});await Promise.resolve();await Promise.resolve();await Promise.resolve();
 fresh.receive({id:JSON.parse(fresh.sent[0]!).id,status:200,body:'{}'});await expect(second).resolves.toHaveProperty('status',200);
 old.receive({id:JSON.parse(old.sent[0]!).id,status:200,body:'{}'});await expect(first).resolves.toHaveProperty('status',200);expect(old.closed).toBe(true);
});
it('settles a request waiting for a handshake when its socket is closed',async()=>{
 const client=createWebSocketActions({endpoint:'ws://localhost/jmap/actions',WebSocketImpl:socketType});
 const pending=client.send('/api/account/drafts',{});const rejected=expect(pending).rejects.toMatchObject({code:'actionNotDispatched'});client.close();await rejected;expect(Socket.latest.closed).toBe(true);
});
