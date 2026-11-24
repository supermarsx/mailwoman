import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {workerHttp} from './workerHttp.ts';
import {clearWebSocketActions,resetActionBackoff} from './websocketActions.ts';
import {ApiError} from './errors.ts';
class Socket {
 static latest:Socket;onmessage:((event:{data:string})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;sent:string[]=[];
 constructor(_endpoint:string){Socket.latest=this;}send(frame:string){this.sent.push(frame);}close(){}
}
const flush=async()=>{for(let i=0;i<12;i++)await Promise.resolve();};
beforeEach(()=>{clearWebSocketActions();vi.stubGlobal('WebSocket',Socket);});
afterEach(()=>{clearWebSocketActions();resetActionBackoff();vi.unstubAllGlobals();});
it('refuses HTTP fallback after authority changes while an unsent handshake is waiting',async()=>{
 let epoch=1;const fetch=vi.fn();vi.stubGlobal('fetch',fetch);
 const changed=new ApiError(409,'Shared session changed',{code:'actionNotDispatched'});
 const check=vi.fn(()=>{if(epoch!==1)throw changed;});
 const request=workerHttp(location.origin,'/jmap/api',{method:'POST',headers:{'content-type':'application/json'},body:'{"publicFixture":true}',allowWebSocket:true,assertAuthority:check});
 const rejected=expect(request).rejects.toBe(changed),before=check.mock.calls.length;
 expect(Socket.latest.sent).toHaveLength(0);epoch=2;Socket.latest.onerror?.();await rejected;
 expect(check).toHaveBeenCalledTimes(before+1);expect(fetch).not.toHaveBeenCalled();
});
it('preserves the original request on allowed fallback and checks direct HTTP only once',async()=>{
 // A worker without an available socket still uses the same ordinary HTTP path.
 vi.stubGlobal('WebSocket',undefined);
 const fetch=vi.fn(async()=>new Response('{"ok":true}',{headers:{'content-type':'application/json'}}));vi.stubGlobal('fetch',fetch);
 const check=vi.fn(),body='{"publicFixture":true}',headers={'content-type':'application/json','x-mailwoman-account':'owned-fixture','idempotency-key':'same-user-action'};
 await workerHttp(location.origin,'/jmap/api',{method:'POST',body,headers,allowWebSocket:true,assertAuthority:check});
 expect(check.mock.calls.length).toBeGreaterThanOrEqual(2);expect(fetch).toHaveBeenCalledTimes(1);
 expect(fetch.mock.calls[0]).toEqual([`${location.origin}/jmap/api`,{method:'POST',body,headers,credentials:'same-origin',redirect:'error',cache:'no-store'}]);
 check.mockClear();await workerHttp(location.origin,'/api/me',{assertAuthority:check});expect(check).toHaveBeenCalledTimes(1);
});
it('never fetches or replays after a sent unknown action rejection',async()=>{
 const fetch=vi.fn();vi.stubGlobal('fetch',fetch);const check=vi.fn();
 const request=workerHttp(location.origin,'/jmap/api',{method:'POST',headers:{'content-type':'application/json'},body:'{}',allowWebSocket:true,assertAuthority:check});
 Socket.latest.onmessage?.({data:JSON.stringify({type:'ready',protocol:'mailwoman-actions-v1'})});await flush();
 expect(Socket.latest.sent).toHaveLength(1);const before=check.mock.calls.length;
 const unknown=expect(request).rejects.toMatchObject({code:'outcomeUnknown'});Socket.latest.onclose?.();await unknown;
 expect(fetch).not.toHaveBeenCalled();expect(check).toHaveBeenCalledTimes(before);
});
it('passes a 304 through only to a conditional request (t52w)',async()=>{
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(null,{status:304})));
 expect((await workerHttp(location.origin,'/jmap/session',{headers:{'If-None-Match':'"s1"'}})).status).toBe(304);
 await expect(workerHttp(location.origin,'/jmap/session',{})).rejects.toMatchObject({status:304});
});
