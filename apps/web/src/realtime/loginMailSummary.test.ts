import {beforeEach,afterEach,expect,it,vi} from 'vitest';
const fixture=vi.hoisted(()=>({folders:vi.fn(),jmap:vi.fn(),targets:vi.fn(),config:vi.fn(),fetch:vi.fn(),current:true}));
vi.mock('../api/workspaceMail.ts',()=>({workspaceFolders:fixture.folders,readWorkspaceJmap:fixture.jmap,workspaceMailboxes:fixture.targets,readWorkspaceNotifications:fixture.config}));
vi.mock('../api/http.ts',()=>({apiFetch:fixture.fetch}));
vi.mock('../api/accountPreferenceLifecycle.ts',async original=>({...await original<typeof import('../api/accountPreferenceLifecycle.ts')>(),preferenceAccountIsCurrent:()=>fixture.current}));
import {resetToasts,toasts} from '../components/toast/toast.ts';
import {countSummary,summaryMayShow,summaryWindow,startLoginMailSummary} from './loginMailSummary.ts';
import type {WorkspaceMailbox} from '../api/workspaceMail.ts';
const config={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
const target={accountId:'account',label:'Mailbox'} as WorkspaceMailbox;
const check={checkedAt:1700000000,now:1700007200};
let stop:(()=>void)|undefined;
afterEach(()=>{stop?.();stop=undefined;resetToasts();});
beforeEach(()=>{vi.clearAllMocks();fixture.current=true;fixture.folders.mockResolvedValue([{id:'real-inbox',role:'inbox'},{id:'spam-not-a-name',role:'junk'}]);fixture.jmap.mockResolvedValue({methodResponses:[['Email/query',{total:7},'inbox-0'],['Email/query',{total:3},'spam-0']]});});
it('establishes the first baseline silently and caps old checks by the configured lookback',()=>{
 expect(summaryWindow({checkedAt:null,now:check.now},{})).toBeNull();
 expect(summaryMayShow({checkedAt:null,now:check.now},{},{inbox:7,spam:3},config)).toBe(false);
 expect(summaryWindow({...check,now:check.checkedAt+86400*100},{'notifications.loginSummary.lookbackDays':3})?.after).toBe(new Date((check.checkedAt+86400*97)*1000).toISOString());
});
it('counts Inbox and Spam using received-time bounds and actual folder roles without downloading bodies',async()=>{
 expect(await countSummary(target,new AbortController().signal,check,{})).toEqual({inbox:7,spam:3});
 const calls=fixture.jmap.mock.calls[0]![1].methodCalls;
 expect(calls).toHaveLength(2);
 expect(calls[0]).toEqual(['Email/query',{accountId:'account',filter:{inMailbox:'real-inbox',...summaryWindow(check,{})},limit:1,calculateTotal:true},'inbox-0']);
 expect(calls[1][1].filter.inMailbox).toBe('spam-not-a-name');
});
it('supports unread-only mode and independently excluding Spam',async()=>{
 await countSummary(target,new AbortController().signal,check,{'notifications.loginSummary.spam':false,'notifications.loginSummary.unreadOnly':true});
 const calls=fixture.jmap.mock.calls[0]![1].methodCalls;expect(calls).toHaveLength(1);expect(calls[0][1].filter.notKeyword).toBe('$seen');
});
it('never acknowledges partial totals or a failed folder check as a zero count',async()=>{
 fixture.jmap.mockResolvedValue({methodResponses:[['Email/query',{ids:[]},'inbox-0']]});
 await expect(countSummary(target,new AbortController().signal,check,{})).rejects.toThrow();
 fixture.folders.mockRejectedValue(new Error('offline'));
 await expect(countSummary(target,new AbortController().signal,check,{})).rejects.toThrow('offline');
});
it('skips brief reloads, disabled summaries and empty counts unless requested',()=>{
 expect(summaryMayShow(check,{},{inbox:7,spam:3},config)).toBe(true);
 expect(summaryMayShow({...check,now:check.checkedAt+10},{},{inbox:7,spam:3},config)).toBe(false);
 expect(summaryMayShow(check,{'notifications.loginSummary.enabled':false},{inbox:7,spam:3},config)).toBe(false);
 expect(summaryMayShow(check,{},{inbox:0,spam:0},config)).toBe(false);
 expect(summaryMayShow(check,{'notifications.loginSummary.showEmpty':true},{inbox:0,spam:0},config)).toBe(true);
});
it('respects quiet hours independently from live notification and content-rule settings',()=>{
 const quiet={...config,quietHours:{enabled:true,start:'00:00',end:'23:59',timezone:'UTC'}};
 vi.useFakeTimers();vi.setSystemTime(new Date('2026-11-23T12:00:00Z'));
 try{expect(summaryMayShow(check,{},{inbox:7,spam:3},quiet)).toBe(false);expect(summaryMayShow(check,{'notifications.loginSummary.quietHours':false},{inbox:7,spam:3},quiet)).toBe(true);}finally{vi.useRealTimers();}
});

it('acknowledges a successful snapshot before emitting the aggregate sign-in toast',async()=>{
 const owner={id:'account',generation:1,storageAuthority:'summary_authority'};
 fixture.targets.mockResolvedValue([{...target,owner,current:true,storageAuthority:owner.storageAuthority}]);fixture.config.mockResolvedValue(config);
 fixture.fetch.mockImplementation(async(_url,init)=>Response.json(init?.method==='POST'?{accepted:true}:check));
 stop=startLoginMailSummary(owner);
 await vi.waitFor(()=>expect(toasts()).toHaveLength(1));
 expect(toasts()[0]?.testId).toBe('login-mail-summary');expect(toasts()[0]?.message).toContain('7');expect(toasts()[0]?.message).toContain('3');
 const posts=fixture.fetch.mock.calls.filter(call=>call[1]?.method==='POST');expect(posts).toHaveLength(1);expect(JSON.parse(posts[0]![1].body)).toEqual({expected:check.checkedAt,checkedAt:check.now});
});
it('does not duplicate a toast when another tab already claimed the checkpoint',async()=>{
 const owner={id:'account',generation:1,storageAuthority:'summary_authority'};
 fixture.targets.mockResolvedValue([{...target,owner,current:true,storageAuthority:owner.storageAuthority}]);fixture.config.mockResolvedValue(config);
 fixture.fetch.mockImplementation(async(_url,init)=>Response.json(init?.method==='POST'?{accepted:false}:check));
 stop=startLoginMailSummary(owner);
 await vi.waitFor(()=>expect(fixture.fetch.mock.calls.some(call=>call[1]?.method==='POST')).toBe(true));
 expect(toasts()).toHaveLength(0);
});
it('preserves the baseline on failed counts and discards an account that changes during counting',async()=>{
 const owner={id:'account',generation:1,storageAuthority:'summary_authority'};
 fixture.targets.mockResolvedValue([{...target,owner,current:true,storageAuthority:owner.storageAuthority}]);fixture.config.mockResolvedValue(config);
 fixture.fetch.mockImplementation(async()=>Response.json(check));fixture.jmap.mockRejectedValue(new Error('offline'));
 stop=startLoginMailSummary(owner);await vi.waitFor(()=>expect(fixture.jmap).toHaveBeenCalled());
 expect(fixture.fetch.mock.calls.some(call=>call[1]?.method==='POST')).toBe(false);expect(toasts()).toHaveLength(0);stop();
 fixture.jmap.mockImplementation(async()=>{fixture.current=false;return {methodResponses:[['Email/query',{total:7},'inbox-0'],['Email/query',{total:3},'spam-0']]};});
 stop=startLoginMailSummary(owner);await vi.waitFor(()=>expect(fixture.jmap).toHaveBeenCalledTimes(2));
 expect(fixture.fetch.mock.calls.some(call=>call[1]?.method==='POST')).toBe(false);expect(toasts()).toHaveLength(0);
});
