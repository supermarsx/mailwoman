import {afterEach,expect,it,vi} from 'vitest';
import {createIncomingMail,type IncomingMail} from './incomingMail.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {Client} from '../api/client.ts';import type {Email,Mailbox} from '../api/jmap-types.ts';
const monitors:ReturnType<typeof createIncomingMail>[]=[];
afterEach(()=>{for(const monitor of monitors.splice(0))monitor.stop();suspendAccountPreferences();});
const message=(id:string,box='inbox',keywords:Record<string,boolean>={}):Email=>({id,subject:id,from:[],to:[],keywords,mailboxIds:{[box]:true},receivedAt:'2026-11-12T12:00:00Z',preview:'',hasAttachment:false});
function fixture(changes:{created:string[];updated?:string[];hasMoreChanges?:boolean;newState:string}[],emails:Email[]){
 const delivered:IncomingMail[]=[];const response=(method:string,data:unknown)=>({methodResponses:[[method,data,'incoming']],sessionState:'session'});
 const jmap=vi.fn(async(request:{methodCalls:[string,Record<string,unknown>,string][]})=>{
  const [method,args]=request.methodCalls[0]!;
  if(method==='Email/get')return response(method,{accountId:'account',state:'seed',list:(args.ids as string[]).length?emails.filter(email=>(args.ids as string[]).includes(email.id)):[],notFound:[]});
  const change=changes.shift()!;return response(method,{accountId:'account',oldState:args.sinceState,updated:[],destroyed:[],hasMoreChanges:false,...change});
 });
 const boxes=[{id:'inbox',name:'Inbox',role:'inbox'},{id:'other',name:'Filed mail',role:null},{id:'spam',name:'Spam',role:'junk'}] as Mailbox[];
 const monitor=createIncomingMail({client:{jmap}as unknown as Client,accountId:()=> 'account',now:()=>Date.parse('2026-11-12T12:00:00Z'),mailboxes:()=>boxes,deliver:batch=>delivered.push(batch)});monitors.push(monitor);activateAccountPreferences('owner','notification_realm');monitor.start();return {monitor,jmap,delivered};
}
it('seeds silently and only announces created unread mail, independent of the displayed folder',async()=>{
 const f=fixture([{created:['new','spam','draft','seen','filed','historical'],newState:'next'},{created:[],updated:['old'],newState:'after-mark-unread'}],[message('old'),message('new'),message('spam','spam'),message('draft','inbox',{$draft:true}),message('seen','inbox',{$seen:true}),message('filed','other'),{...message('historical'),receivedAt:'2020-01-01T00:00:00Z'}]);
 await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledOnce());expect(f.delivered).toEqual([]);f.monitor.changed();await vi.waitFor(()=>expect(f.delivered).toHaveLength(1));expect(f.delivered[0]!.messages.map(m=>m.id)).toEqual(['new','filed']);f.monitor.changed();await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledTimes(4));expect(f.delivered).toHaveLength(1);
});
it('drains paginated changes without losing the state between pages',async()=>{
 const f=fixture([{created:['one'],newState:'page-one',hasMoreChanges:true},{created:['two'],newState:'page-two'}],[message('one'),message('two')]);await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledOnce());f.monitor.changed();await vi.waitFor(()=>expect(f.delivered).toHaveLength(2));const calls=f.jmap.mock.calls.filter(([r])=>r.methodCalls[0]![0]==='Email/changes');expect(calls.map(([r])=>r.methodCalls[0]![1].sinceState)).toEqual(['seed','page-one']);
});
it('does not deliver results that outlive logout or an account change',async()=>{
 const f=fixture([{created:['new'],newState:'next'}],[message('new')]);await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledOnce());let release!:(value:Awaited<ReturnType<typeof f.jmap>>)=>void;f.jmap.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));f.monitor.changed();await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledTimes(2));suspendAccountPreferences();release({methodResponses:[['Email/changes',{accountId:'account',created:['new'],updated:[],destroyed:[],hasMoreChanges:false,newState:'next'},'incoming']],sessionState:'s'});await Promise.resolve();expect(f.delivered).toEqual([]);
});
it('retries a failed metadata fetch from the same cursor without emitting a partial batch',async()=>{
 const f=fixture([{created:['new'],newState:'next'},{created:['new'],newState:'next'}],[message('new')]);await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledOnce());const original=f.jmap.getMockImplementation()!;let fail=true;f.jmap.mockImplementation(async request=>{if(request.methodCalls[0]![0]==='Email/get'&&(request.methodCalls[0]![1].ids as string[]).length&&fail){fail=false;throw new Error('offline');}return original(request);});f.monitor.changed();await vi.waitFor(()=>expect(f.jmap).toHaveBeenCalledTimes(3));expect(f.delivered).toEqual([]);f.monitor.changed();await vi.waitFor(()=>expect(f.delivered).toHaveLength(1));const changes=f.jmap.mock.calls.filter(([r])=>r.methodCalls[0]![0]==='Email/changes');expect(changes.map(([r])=>r.methodCalls[0]![1].sinceState)).toEqual(['seed','seed']);
});
