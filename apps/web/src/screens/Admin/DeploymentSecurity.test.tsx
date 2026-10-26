import { describe,it,expect,vi } from 'vitest';
import { screen,fireEvent,within } from '@solidjs/testing-library';
import { DeploymentSecurity,type DeploymentAssessment } from './DeploymentSecurity.tsx';
import { renderInShell,holding } from './shell/testkit.tsx';
const report:DeploymentAssessment={generatedAt:'2026-10-26T09:00:00Z',scope:'current-process-and-request',checks:[{id:'adminFactors',status:'warning',value:2,settings:[]},{id:'adminSessions',status:'info',value:{idleSeconds:1800,absoluteSeconds:43200},settings:['admin.session.idleSecs']},{id:'csrf',status:'pass',value:true,settings:[]}],limitations:['no-external-tls-or-firewall-probe','configured-worker-not-executed']};
describe('deployment assessment',()=>{
 it('distinguishes request ingress limits from response streams',async()=>{
  const snapshot={...report,checks:[{id:'httpAdmission',status:'info' as const,value:{connections:1024,requests:256,headerBytes:32768,headers:100,headerTimeoutMs:10000,bodyTimeoutMs:60000,http2Streams:32,shutdownTimeoutMs:30000},settings:[]}]};
  renderInShell(()=><DeploymentSecurity load={async()=>snapshot}/>);
  expect(await screen.findByText('HTTP admission and request limits')).toBeInTheDocument();
  expect(screen.getByText(/Response streams are not subject/)).toHaveTextContent('Fixed limits require restart');
 });

 it('describes startup connection limits without claiming an external TLS probe',async()=>{
  const snapshot={...report,checks:[{id:'connectionNegotiation',status:'info' as const,value:{tlsTimeoutMs:10000,tlsMaximum:128,proxyTimeoutMs:5000,proxyMaximum:128,requiresRestart:true},settings:[]}]};
  renderInShell(()=><DeploymentSecurity load={async()=>snapshot}/>);
  expect(await screen.findByText('Inbound connection negotiation limits')).toBeInTheDocument();
  expect(screen.getByText(/Changes require restart/)).toHaveTextContent('External proxy limits are not measured here');
 });

 it('does not offer policy editor links without settings-read permission',async()=>{
  renderInShell(()=><DeploymentSecurity load={async()=>report}/>,{loadMe:holding(['diagnostics.read'])});
  await screen.findByText('Enabled administrators without a second factor');
  expect(screen.queryByRole('link',{name:'admin.session.idleSecs'})).toBeNull();
 });
 it('shows measured values and explicit untested limits, with authoritative policy links',async()=>{
  renderInShell(()=><DeploymentSecurity load={async()=>report}/>);
  expect(await screen.findByText('Enabled administrators without a second factor')).toBeInTheDocument();
  expect(screen.getByText(/Idle limit:.*1,?800/)).toBeInTheDocument();
  expect(screen.getByText(/Public TLS certificates/)).toBeInTheDocument();
  expect(screen.getByText(/worker is not executed/)).toBeInTheDocument();
  expect(screen.getByRole('link',{name:'admin.session.idleSecs'})).toHaveAttribute('href','#/settings/admin.session.idleSecs');
 });
 it('filters checks by their visible title and status',async()=>{
  renderInShell(()=><DeploymentSecurity load={async()=>report}/>);
  await screen.findByText('Enabled administrators without a second factor');
  fireEvent.change(screen.getByLabelText('Status'),{target:{value:'warning'}});
  const table=screen.getByRole('table',{name:'Deployment security assessment'});
  expect(within(table).queryByText('Strict CSRF protection')).toBeNull();
  fireEvent.input(screen.getByLabelText('Find a check'),{target:{value:'missing'}});
  expect(screen.getByText('No checks match your filters.')).toBeInTheDocument();
 });
 it('does not invent a passing result on error and retries the real loader',async()=>{
  const load=vi.fn().mockRejectedValueOnce({status:503}).mockResolvedValue(report);
  renderInShell(()=><DeploymentSecurity load={load}/>);
  expect(await screen.findByRole('alert')).toHaveTextContent('503');
  fireEvent.click(screen.getByRole('button',{name:'Refresh assessment'}));
  expect(await screen.findByText('Enabled administrators without a second factor')).toBeInTheDocument();
  expect(load).toHaveBeenCalledTimes(2);
 });
});
