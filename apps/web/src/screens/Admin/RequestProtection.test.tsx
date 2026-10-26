import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {RequestProtection} from './RequestProtection.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import type {ProtectionApi,ProtectionSubject,ProtectionStatus} from '../../api/admin/requestProtection.ts';
const row:ProtectionSubject={subjectKey:'a'.repeat(64),subjectKind:'account',updatedAt:1700000000,expiresAt:2000000000,revision:'b'.repeat(32),snapshot:{short:{normal:100,anomalous:1},long:{normal:1000,anomalous:3},shortWindowSecs:300,longWindowSecs:604800,activeUntil:2000000000,stage:1,lastAnomaly:1700000000,lastSignal:'malformed',rateCount:1,rateWindowSecs:60}};
const status:ProtectionStatus={policy:{rateMode:'off',adaptiveMode:'observe',clientRequests:600,accountRequests:300,adminRequests:120,rateWindowSecs:60},subjectCapacity:20000,eventCapacity:10000,eventRetentionSecs:2592000,propagationSecs:30,coverage:'router',preRouterHeadersObserved:false};
function api(overrides:Partial<ProtectionApi>={}):ProtectionApi{return{status:async()=>status,subjects:async()=>({subjects:[row],cursor:null}),events:async()=>({events:[],cursor:null}),clear:vi.fn(async()=>undefined),...overrides};}
describe('Adaptive request protection',()=>{
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
