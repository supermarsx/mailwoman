import {beforeEach,describe,expect,it,vi} from 'vitest';
// t94: a server without the route, and a refused write, never leave a body unread and never retry.
const calls:Array<{url:string;init?:RequestInit}>=[];
let next:()=>Response=()=>new Response('[]');
vi.mock('./http.ts',()=>({apiFetch:vi.fn(async(url:string,init?:RequestInit)=>{calls.push({url,...(init?{init}:{})});return next();})}));
import {FocusedSendersError,importFocusedSenders,loadFocusedSenders} from './focusedSenders.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
beforeEach(()=>{calls.length=0;suspendAccountPreferences();activateAccountPreferences('acct1','authority-0123456789abcdef');});
function tracked(status:number,body:string):{response:Response;read:()=>boolean}{
 const response=new Response(body,{status});let read=false;const text=response.text.bind(response);
 response.text=async()=>{read=true;return text();};
 return {response,read:()=>read};
}
describe('focused senders API',()=>{
 it('reads a 404 as no overrides, consuming the body, once',async()=>{
  const r=tracked(404,'not found');next=()=>r.response;
  await expect(loadFocusedSenders()).resolves.toEqual([]);
  expect(r.read()).toBe(true);expect(calls).toHaveLength(1);
 });
 it('throws a refused answer after reading its body, without retrying',async()=>{
  const r=tracked(503,'busy');next=()=>r.response;
  await expect(loadFocusedSenders()).rejects.toBeInstanceOf(FocusedSendersError);
  expect(r.read()).toBe(true);expect(calls).toHaveLength(1);
  const w=tracked(404,'');next=()=>w.response;
  await expect(importFocusedSenders({'a@x.example':'other'})).rejects.toMatchObject({status:404});
  expect(w.read()).toBe(true);expect(calls).toHaveLength(2);
  expect(new Headers(calls[1]!.init?.headers).get('x-mailwoman-storage-authority')).toBe('authority-0123456789abcdef');
 });
});
