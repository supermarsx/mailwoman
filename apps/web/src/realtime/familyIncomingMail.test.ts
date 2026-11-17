import {afterEach,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {IncomingMail} from './incomingMail.ts';
const fixture=vi.hoisted(()=>({rows:[] as unknown[],calls:[] as unknown[],created:[] as string[],state:'seed',catalogFailures:0}));
vi.mock('../api/workspaceMail.ts',()=>({workspaceMailboxes:async()=>{if(fixture.catalogFailures-->0)throw new Error('Temporary catalog failure');return fixture.rows;},readWorkspaceJmap:async(target:{accountId:string;linkId:string},request:{methodCalls:[string,Record<string,unknown>,string][]})=>{
 fixture.calls.push({target,request});const [method,args,tag]=request.methodCalls[0]!;
 const data=method==='Mailbox/get'?{accountId:target.accountId,list:[{id:'inbox',role:'inbox',name:'Inbox'}]}:method==='Email/changes'?{accountId:target.accountId,oldState:args.sinceState,newState:fixture.state,created:[...fixture.created],updated:[],destroyed:[],hasMoreChanges:false}:{accountId:target.accountId,state:fixture.state,list:(args.ids as string[]).map(id=>({id,subject:'Arrived',keywords:{},mailboxIds:{inbox:true},receivedAt:new Date().toISOString()}))};
 return {sessionState:'s',methodResponses:[[method,data,tag]]};
}}));
import {createFamilyIncomingMail} from './familyIncomingMail.ts';
afterEach(()=>suspendAccountPreferences());
it('detects mail in an inactive linked account through its verified target and stops on logout',async()=>{
 activateAccountPreferences('primary','msa1_primary_account_test');fixture.created=[];fixture.state='seed';fixture.calls=[];fixture.catalogFailures=1;
 fixture.rows=[{accountId:'primary',linkId:null,current:true,owner:preferenceAccount(),storageAuthority:'msa1_primary_account_test',label:'Primary'},{accountId:'child',linkId:'link',current:false,owner:preferenceAccount(),storageAuthority:'msa1_child_account_test',label:'Work'}];
 const delivered:IncomingMail[]=[];let dispose!:()=>void;const monitor=createRoot(stop=>{dispose=stop;return createFamilyIncomingMail({activeLink:()=>null,deliver:batch=>delivered.push(batch)});});monitor.start();
 try{
  await vi.waitFor(()=>expect(fixture.catalogFailures).toBe(0));
  await Promise.resolve();monitor.changed(); // A catch-up tick retries a failed verified-account catalog.
  await vi.waitFor(()=>expect(fixture.calls.length).toBeGreaterThanOrEqual(3));expect(delivered).toEqual([]);
  fixture.created=['new'];fixture.state='next';monitor.changed('link','msa1_wrong_authority');await Promise.resolve();expect(delivered).toEqual([]);
  monitor.changed('link','msa1_child_account_test');await vi.waitFor(()=>expect(delivered).toHaveLength(1));expect(delivered[0]!.target?.linkId).toBe('link');expect(delivered[0]!.accountId).toBe('child');
  expect(fixture.calls.every(call=>(call as {target:{accountId:string}}).target.accountId==='child')).toBe(true);
  monitor.stop();const count=fixture.calls.length;monitor.changed();await Promise.resolve();expect(fixture.calls).toHaveLength(count);
 }finally{monitor.stop();dispose();}
});
