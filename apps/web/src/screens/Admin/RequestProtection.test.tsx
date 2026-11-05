import {describe,it,expect,vi} from 'vitest';
import {createSignal} from 'solid-js';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {RequestProtection,protectionTab} from './RequestProtection.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import type {ProtectionApi,ProtectionEvent,ProtectionSubject,ProtectionStatus} from '../../api/admin/requestProtection.ts';
const row:ProtectionSubject={subjectKey:'a'.repeat(64),subjectKind:'account',updatedAt:1700000000,expiresAt:2000000000,revision:'b'.repeat(32),snapshot:{short:{normal:100,anomalous:1},long:{normal:1000,anomalous:3},shortWindowSecs:300,longWindowSecs:604800,activeUntil:2000000000,stage:1,lastAnomaly:1700000000,lastSignal:'malformed',rateCount:1,rateWindowSecs:60}};
const status:ProtectionStatus={policy:{rateMode:'off',adaptiveMode:'observe',clientRequests:600,accountRequests:300,adminRequests:120,rateWindowSecs:60},subjectCapacity:20000,eventCapacity:10000,eventRetentionSecs:2592000,propagationSecs:30,coverage:'router',preRouterHeadersObserved:false};
/** A route on one subtab of Security > Request protection; `setTab` switches it like a hash change. */
function onTab(initial:string){const [tab,setTab]=createSignal(initial);return{route:()=>({section:'security',rest:['protection',tab()]}),setTab};}
function api(overrides:Partial<ProtectionApi>={}):ProtectionApi{return{status:async()=>status,subjects:async()=>({subjects:[row],cursor:null,total:1}),events:async()=>({events:[],cursor:null,total:0}),clear:vi.fn(async()=>undefined),...overrides};}
describe('Adaptive request protection',()=>{
 it('shows the decision source IP separately from the last IP of an account aggregate',async()=>{
  const current={...row,snapshot:{...row.snapshot,sourceIp:'192.0.2.7'}};
  const event:ProtectionEvent={id:'ip-decision',subjectKey:row.subjectKey,subjectKind:'account',observedAt:1700000000,reason:'short-absolute',action:'throttle',enforced:true,retryAfterSecs:3,snapshot:{...row.snapshot,sourceIp:'2001:db8::42'},policy:null};
  const nav=onTab('state');
  renderInShell(()=><RequestProtection api={api({subjects:async()=>({subjects:[current,{...row,subjectKey:'d'.repeat(64)}],cursor:null,total:2}),events:async()=>({events:[event],cursor:null,total:1})})}/>,{route:nav.route});
  const subjects=await screen.findByRole('table',{name:'Current subject state'});
  expect(within(subjects).getByRole('columnheader',{name:'Last source IP'})).toBeInTheDocument();
  expect(within(subjects).getByRole('cell',{name:'192.0.2.7'})).toBeInTheDocument();
  expect(within(subjects).getByRole('cell',{name:'Unknown IP'})).toBeInTheDocument();
  nav.setTab('decisions');
  const history=await screen.findByRole('table',{name:'Protection decisions'});
  expect(within(history).getByRole('columnheader',{name:'Source IP'})).toBeInTheDocument();
  expect(within(history).getByRole('cell',{name:'2001:db8::42'})).toBeInTheDocument();
  expect(within(history).queryByText('192.0.2.7')).toBeNull();
 });
 it('identifies source countries in current state and history without assigning one to account aggregates',async()=>{
  const sources:ProtectionSubject[]=[
   {...row,subjectKey:'1'.repeat(64),subjectKind:'client',snapshot:{...row.snapshot,countryCode:'PT'}},
   {...row,subjectKey:'2'.repeat(64),subjectKind:'client'},
   {...row,subjectKey:'3'.repeat(64),subjectKind:'ua-search',snapshot:{...row.snapshot,countryCode:'US'}},
   {...row,subjectKey:'4'.repeat(64),subjectKind:'account',snapshot:{...row.snapshot,countryCode:'PT'}},
   {...row,subjectKey:'5'.repeat(64),subjectKind:'admin',snapshot:{...row.snapshot,countryCode:'US'}},
   {...row,subjectKey:'6'.repeat(64),subjectKind:'client',snapshot:{...row.snapshot,countryCode:'pt'}},
   {...row,subjectKey:'7'.repeat(64),subjectKind:'client',snapshot:{...row.snapshot,countryCode:'../PT'}},
   {...row,subjectKey:'8'.repeat(64),subjectKind:'ua-uncommon',snapshot:{...row.snapshot,countryCode:null}},
  ];
  const events:ProtectionEvent[]=sources.map(source=>({id:source.subjectKey,subjectKey:source.subjectKey,subjectKind:source.subjectKind,observedAt:1700000000,reason:'short-absolute',action:'throttle',enforced:true,retryAfterSecs:3,snapshot:source.snapshot,policy:null}));
  const nav=onTab('state');
  renderInShell(()=><RequestProtection api={api({subjects:async()=>({subjects:sources,cursor:null,total:sources.length}),events:async()=>({events,cursor:null,total:events.length})})}/>,{route:nav.route});
  for(const [name,tab] of [['Current subject state','state'],['Protection decisions','decisions']] as const){
   nav.setTab(tab);const table=await screen.findByRole('table',{name});
   expect(within(table).getByRole('columnheader',{name:'Country'})).toBeInTheDocument();
   for(const [index,label] of ['Portugal','Unknown','United States','Not applicable','Not applicable','Unknown','Unknown','Unknown'].entries()){
    const entry=table.querySelector(`[data-row-key="${String(index+1).repeat(64)}"]`)!;
    expect(within(entry as HTMLElement).getByRole('cell',{name:label})).toBeInTheDocument();
   }
  }
 });
 it.each([false,true])('explains a first-request scan-path block with its duration (enforced=%s)',async enforced=>{
  const event:ProtectionEvent={id:'first-probe',subjectKey:'c'.repeat(64),subjectKind:'client',observedAt:1700000000,reason:'probe-path',action:'block',enforced,retryAfterSecs:900,snapshot:{...row.snapshot,short:{normal:0,anomalous:1},long:{normal:0,anomalous:1},lastSignal:'probe-path',rateCount:1,stage:2},policy:{short:{seconds:300,minimum_samples:100,absolute_anomalies:50,ratio_percent:90},long:{seconds:604800,minimum_samples:1000,absolute_anomalies:500,ratio_percent:90}}};
  renderInShell(()=><RequestProtection api={api({status:async()=>({...status,policy:{...status.policy,adaptiveMode:enforced?'enforce':'observe'}}),events:async()=>({events:[event],cursor:null,total:1})})}/>,{route:onTab('decisions').route});
  const decision=await screen.findByTestId('protection-decision');
  expect(decision).toHaveTextContent('Recognized scan path on the first request; temporary block for 15 min.');
  expect(decision).not.toHaveTextContent('admin-protect-decision-probe-path');
  const historyRow=decision.closest('tr')!;
  expect(historyRow).toHaveTextContent(enforced?'Temporary block · Enforced':'Temporary block · Observed only');
  fireEvent.click(within(historyRow).getByText('Evidence',{selector:'summary'}));
  expect(historyRow).toHaveTextContent('probe-path');
  expect(historyRow).toHaveTextContent(/1 unusual/);
 });
 it('shows actual modes and long-window counts, with the rules edited in place rather than behind a settings link list',async()=>{
  const nav=onTab('overview');
  renderInShell(()=><RequestProtection api={api()}/>,{route:nav.route});
  expect(await screen.findByText('Request budgets: Off. Adaptive protection: Observe only.')).toBeInTheDocument();
  nav.setTab('state');
  expect(await screen.findByText('3 / 1000')).toBeInTheDocument();
  expect(screen.queryByText('Configure modes, windows, thresholds, and exemptions')).toBeNull();
 });
 it('requires confirmation and sends the exact revision; stale state stays visible',async()=>{
  const clear=vi.fn(async()=>{throw{status:412,code:'stale'};});
  renderInShell(()=><RequestProtection api={api({clear})}/>,{route:onTab('state').route});
  const action=await screen.findByRole('button',{name:/Clear protection state for Mailbox account/});
  expect(action.textContent).toBe('');
  expect(action.querySelector('svg')).not.toBeNull();
  expect(action).toHaveAttribute('title','Clear state');
  fireEvent.click(action);
  expect(clear).not.toHaveBeenCalled();
  fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button',{name:/^Clear state$/}));
  await waitFor(()=>expect(clear).toHaveBeenCalledWith(row));
  expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  expect(await screen.findByRole('alert')).toHaveTextContent(/changed|refresh|reload/i);
 });
 it('hides writes without capability and pages forward and back through server cursors, newest first',async()=>{
  const at=(key:string,updatedAt:number)=>({...row,subjectKey:key.repeat(64),updatedAt});
  const subjects=vi.fn(async(after?:string)=>after?{subjects:[at('c',1700000000)],cursor:null,total:2}:{subjects:[at('a',1700000500)],cursor:'1700000500:'+'a'.repeat(64),total:2});
  renderInShell(()=><RequestProtection api={api({subjects})}/>,{loadMe:holding(['bans.read']),route:onTab('state').route});
  expect(await screen.findByText(/Mailbox account · aaaaaaaaaaaa/)).toBeInTheDocument();
  expect(subjects).toHaveBeenLastCalledWith(undefined,100);
  expect(screen.getByRole('status')).toHaveTextContent('Page 1 of 1 · 2 in total');
  expect(screen.queryByRole('button',{name:/^Clear/})).toBeNull();
  expect(screen.getByRole('button',{name:'Previous page'})).toBeDisabled();
  fireEvent.click(screen.getByRole('button',{name:'Next page'}));
  expect(await screen.findByText(/Mailbox account · cccccccccccc/)).toBeInTheDocument();
  expect(subjects).toHaveBeenLastCalledWith('1700000500:'+'a'.repeat(64),100);
  expect(screen.getByRole('button',{name:'Next page'})).toBeDisabled();
  fireEvent.click(screen.getByRole('button',{name:'Previous page'}));
  expect(await screen.findByText(/Mailbox account · aaaaaaaaaaaa/)).toBeInTheDocument();
  expect(subjects).toHaveBeenLastCalledWith(undefined,100);
 });
 it('lets the admin pick the page size for decisions and restarts from the newest page',async()=>{
  const events=vi.fn(async(_after?:string,_limit?:number)=>({events:[],cursor:null,total:430}));
  renderInShell(()=><RequestProtection api={api({events})}/>,{route:onTab('decisions').route});
  await waitFor(()=>expect(events).toHaveBeenCalledWith(undefined,100));
  expect(await screen.findByRole('status')).toHaveTextContent('Page 1 of 5 · 430 in total');
  const picker=screen.getByRole('combobox',{name:/Protection decisions · Rows per page/});
  expect([...picker.querySelectorAll('option')].map(o=>o.textContent)).toEqual(['25','50','100','250']);
  fireEvent.change(picker,{target:{value:'250'}});
  await waitFor(()=>expect(events).toHaveBeenLastCalledWith(undefined,250));
  expect(await screen.findByText('Page 1 of 2 · 430 in total')).toBeInTheDocument();
 });
 it('gives aggregate and adaptive protection their own deep-linkable subtabs and keeps every group',async()=>{
  const nav=onTab('aggregate');
  const policy={rules:[],presets:{relaxed:{},balanced:{},strict:{}},canEdit:true,domainScopes:true} as never;
  const policyApi={policy:async()=>policy,save:vi.fn(),simulate:vi.fn()};
  renderInShell(()=><RequestProtection api={api()} policyApi={policyApi}/>,{route:nav.route});
  const tabs=await screen.findByRole('navigation',{name:'Request protection sections'});
  expect(within(tabs).getAllByRole('link').map(a=>a.textContent)).toEqual(['Overview','Aggregate','Adaptive','Object state','Decisions']);
  expect(within(tabs).getByRole('link',{name:'Aggregate'})).toHaveAttribute('aria-current','page');
  expect(within(tabs).getByRole('link',{name:'Adaptive'})).toHaveAttribute('href',expect.stringMatching(/#\/security\/protection\/adaptive$/));
  expect(await screen.findByTestId('protection-policy')).toBeInTheDocument();
  expect(screen.queryByTestId('protection-simulator')).toBeNull();
  expect(screen.queryByTestId('protection-summary')).toBeNull();
  nav.setTab('adaptive');
  expect(await screen.findByTestId('protection-simulator')).toBeInTheDocument();
  nav.setTab('overview');
  expect(await screen.findByTestId('protection-summary')).toBeInTheDocument();
  expect(screen.queryByTestId('protection-simulator')).toBeNull();
 });
 it('opens the overview for a missing or unknown subtab',()=>{
  expect(protectionTab(['protection'])).toBe('overview');
  expect(protectionTab(['protection','nope'])).toBe('overview');
  expect(protectionTab(['protection','decisions'])).toBe('decisions');
 });
});
