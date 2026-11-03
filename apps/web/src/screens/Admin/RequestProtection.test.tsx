import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {RequestProtection} from './RequestProtection.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import type {ProtectionApi,ProtectionEvent,ProtectionSubject,ProtectionStatus} from '../../api/admin/requestProtection.ts';
const row:ProtectionSubject={subjectKey:'a'.repeat(64),subjectKind:'account',updatedAt:1700000000,expiresAt:2000000000,revision:'b'.repeat(32),snapshot:{short:{normal:100,anomalous:1},long:{normal:1000,anomalous:3},shortWindowSecs:300,longWindowSecs:604800,activeUntil:2000000000,stage:1,lastAnomaly:1700000000,lastSignal:'malformed',rateCount:1,rateWindowSecs:60}};
const status:ProtectionStatus={policy:{rateMode:'off',adaptiveMode:'observe',clientRequests:600,accountRequests:300,adminRequests:120,rateWindowSecs:60},subjectCapacity:20000,eventCapacity:10000,eventRetentionSecs:2592000,propagationSecs:30,coverage:'router',preRouterHeadersObserved:false};
function api(overrides:Partial<ProtectionApi>={}):ProtectionApi{return{status:async()=>status,subjects:async()=>({subjects:[row],cursor:null}),events:async()=>({events:[],cursor:null}),clear:vi.fn(async()=>undefined),...overrides};}
describe('Adaptive request protection',()=>{
 it('shows the decision source IP separately from the last IP of an account aggregate',async()=>{
  const current={...row,snapshot:{...row.snapshot,sourceIp:'192.0.2.7'}};
  const event:ProtectionEvent={id:'ip-decision',subjectKey:row.subjectKey,subjectKind:'account',observedAt:1700000000,reason:'short-absolute',action:'throttle',enforced:true,retryAfterSecs:3,snapshot:{...row.snapshot,sourceIp:'2001:db8::42'},policy:null};
  renderInShell(()=><RequestProtection api={api({subjects:async()=>({subjects:[current,{...row,subjectKey:'d'.repeat(64)}],cursor:null}),events:async()=>({events:[event],cursor:null})})}/>);
  const subjects=await screen.findByRole('table',{name:'Current subject state'});
  const history=screen.getByRole('table',{name:'Protection decisions'});
  expect(within(subjects).getByRole('columnheader',{name:'Last source IP'})).toBeInTheDocument();
  expect(within(subjects).getByRole('cell',{name:'192.0.2.7'})).toBeInTheDocument();
  expect(within(subjects).getByRole('cell',{name:'Unknown IP'})).toBeInTheDocument();
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
  renderInShell(()=><RequestProtection api={api({subjects:async()=>({subjects:sources,cursor:null}),events:async()=>({events,cursor:null})})}/>);
  const subjects=await screen.findByRole('table',{name:'Current subject state'});
  const history=screen.getByRole('table',{name:'Protection decisions'});
  for(const table of [subjects,history]){
   expect(within(table).getByRole('columnheader',{name:'Country'})).toBeInTheDocument();
   for(const [index,label] of ['Portugal','Unknown','United States','Not applicable','Not applicable','Unknown','Unknown','Unknown'].entries()){
    const entry=table.querySelector(`[data-row-key="${String(index+1).repeat(64)}"]`)!;
    expect(within(entry as HTMLElement).getByRole('cell',{name:label})).toBeInTheDocument();
   }
  }
 });
 it.each([false,true])('explains a first-request scan-path block with its duration (enforced=%s)',async enforced=>{
  const event:ProtectionEvent={id:'first-probe',subjectKey:'c'.repeat(64),subjectKind:'client',observedAt:1700000000,reason:'probe-path',action:'block',enforced,retryAfterSecs:900,snapshot:{...row.snapshot,short:{normal:0,anomalous:1},long:{normal:0,anomalous:1},lastSignal:'probe-path',rateCount:1,stage:2},policy:{short:{seconds:300,minimum_samples:100,absolute_anomalies:50,ratio_percent:90},long:{seconds:604800,minimum_samples:1000,absolute_anomalies:500,ratio_percent:90}}};
  renderInShell(()=><RequestProtection api={api({status:async()=>({...status,policy:{...status.policy,adaptiveMode:enforced?'enforce':'observe'}}),events:async()=>({events:[event],cursor:null})})}/>);
  const decision=await screen.findByTestId('protection-decision');
  expect(decision).toHaveTextContent('Recognized scan path on the first request; temporary block for 15 min.');
  expect(decision).not.toHaveTextContent('admin-protect-decision-probe-path');
  const historyRow=decision.closest('tr')!;
  expect(historyRow).toHaveTextContent(enforced?'Temporary block · Enforced':'Temporary block · Observed only');
  fireEvent.click(within(historyRow).getByText('Evidence',{selector:'summary'}));
  expect(historyRow).toHaveTextContent('probe-path');
  expect(historyRow).toHaveTextContent(/1 unusual/);
 });
 it('shows actual modes, long-window counts and dedicated settings links',async()=>{
  const {navigate}=renderInShell(()=><RequestProtection api={api()}/>);
  expect(await screen.findByText('Request budgets: Off. Adaptive protection: Observe only.')).toBeInTheDocument();
  expect(screen.getByText('3 / 1000')).toBeInTheDocument();
  fireEvent.click(screen.getByText('Configure modes, windows, thresholds, and exemptions'));
  fireEvent.click(screen.getByRole('button',{name:/^Long anomaly window \(seconds\)$/}));
  expect(navigate).toHaveBeenCalledWith('/admin#/settings/security.protection.longWindowSecs');
 });
 it('requires confirmation and sends the exact revision; stale state stays visible',async()=>{
  const clear=vi.fn(async()=>{throw{status:412,code:'stale'};});
  renderInShell(()=><RequestProtection api={api({clear})}/>);
  fireEvent.click(await screen.findByRole('button',{name:/Clear protection state for Mailbox account/}));
  expect(clear).not.toHaveBeenCalled();
  fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button',{name:/^Clear state$/}));
  await waitFor(()=>expect(clear).toHaveBeenCalledWith(row));
  expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  expect(await screen.findByRole('alert')).toHaveTextContent(/changed|refresh|reload/i);
 });
 it('hides writes without capability and follows bounded server pages',async()=>{
  const subjects=vi.fn(async(after?:string)=>({subjects:[{...row,subjectKey:after?'c'.repeat(64):row.subjectKey}],cursor:after?null:row.subjectKey}));
  renderInShell(()=><RequestProtection api={api({subjects})}/>,{loadMe:holding(['bans.read'])});
  expect(await screen.findByText(/Mailbox account · aaaaaaaaaaaa/)).toBeInTheDocument();
  expect(screen.queryByRole('button',{name:/^Clear/})).toBeNull();
  fireEvent.click(screen.getByRole('button',{name:'Next page'}));
  expect(await screen.findByText(/Mailbox account · cccccccccccc/)).toBeInTheDocument();
  expect(subjects).toHaveBeenLastCalledWith(row.subjectKey);
 });
});
