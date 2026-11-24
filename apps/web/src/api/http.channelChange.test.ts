// t52w: a read caught by an action-channel change before dispatch is sent over HTTP, never failed for a transport reason.
import {afterEach,expect,it,vi} from 'vitest';
import {apiFetch} from './http.ts';
import {clearWebSocketActions} from './websocketActions.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
class Socket{static all:Socket[]=[];onmessage:((event:{data:string})=>void)|null=null;onclose:(()=>void)|null=null;onerror:(()=>void)|null=null;sent:string[]=[];constructor(){Socket.all.push(this);}send(value:string){this.sent.push(value);}close(){}}
afterEach(()=>{clearWebSocketActions();suspendAccountPreferences();vi.unstubAllGlobals();document.cookie='mw_csrf=; Max-Age=0; Path=/';Socket.all=[];});
it('a restored session read that meets an account activation mid-handshake is answered over HTTP',async()=>{
 document.cookie='mw_csrf=restored; Path=/';
 vi.stubGlobal('WebSocket',Socket);
 const http=vi.fn(async(_url:string)=>Response.json({brandName:'Mail'}));vi.stubGlobal('fetch',http);
 const read=apiFetch('/api/public-config',{cache:'no-store'});
 await vi.waitFor(()=>expect(Socket.all.length).toBeGreaterThan(0));
 // /api/me answered: the account becomes active, which clears every action channel.
 activateAccountPreferences('acct-a','msa1_restored_authority');
 const response=await read;
 expect(response.status).toBe(200);expect(await response.json()).toEqual({brandName:'Mail'});
 expect(http).toHaveBeenCalledTimes(1);expect(Socket.all.every(socket=>socket.sent.length===0)).toBe(true);
});
