import { describe, it, expect, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { HostingConnections } from './index.tsx';
import { HostingActionForm } from './ActionForm.tsx';
import { AdminProblemError } from '../shell/adminApi.ts';
import { HostingManage } from './Manage.tsx';
import { emptyHostingConfig } from './ConnectionEditor.tsx';
import { renderInShell, holding } from '../shell/testkit.tsx';
import { parseAdminPath } from '../shell/router.ts';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import type { HostingApi, HostingConnection } from '../../../api/admin/hosting.ts';

const row:HostingConnection={...emptyHostingConfig(),id:'panel',name:'Mail panel',baseUrl:'https://panel.example.org',username:'operator',credential:{set:true},etag:'"rev1"',updatedAt:1};
function service():HostingApi{return {list:vi.fn(async()=>[row]),get:vi.fn(async()=>row),save:vi.fn(async(config)=>({...config,credential:{set:true},etag:'"rev2"',updatedAt:2})),remove:vi.fn(async()=>{}),perform:vi.fn(async()=>({domains:[{name:'example.org',enabled:true}],mailboxes:[],changed:false}))};}
function mount(path:string,api=service(),readOnly=false){const [route,setRoute]=createSignal(parseAdminPath(path));return {api,...renderInShell(()=><HostingConnections api={api} manage={connection=><HostingManage connection={connection} api={api}/>}/>,{route,navigate:path=>setRoute(parseAdminPath(path)),...(readOnly?{loadMe:holding(['integrations.read'])}:{})})};}
describe('dedicated hosting connection pages',()=>{
 it('keeps credential inputs off the list and navigates to a separate editor',async()=>{const {navigate,container}=mount('/admin#/integrations/hosting');await screen.findByRole('button',{name:'Mail panel'});expect(screen.queryByRole('textbox',{name:'Connection name'})).not.toBeInTheDocument();await expectNoAxeViolations(container);fireEvent.click(screen.getByRole('button',{name:'New hosting connection'}));expect(navigate).toHaveBeenCalledWith('/admin#/integrations/hosting/new');expect(screen.getByRole('textbox',{name:'Connection name'})).toBeInTheDocument();});
 it('preserves the revision and excludes response-only credential metadata when saving',async()=>{const {api}=mount('/admin#/integrations/hosting/panel/edit');const name=await screen.findByRole('textbox',{name:'Connection name'});fireEvent.input(name,{target:{value:'Renamed'}});expect(screen.queryByLabelText('Virtualmin password')).not.toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Save connection'}));await waitFor(()=>expect(api.save).toHaveBeenCalledWith({...emptyHostingConfig(),id:'panel',name:'Renamed',baseUrl:row.baseUrl,username:'operator'},{action:'keep'},'"rev1"'));});
 it('does not permit writes with read-only access',async()=>{mount('/admin#/integrations/hosting/panel/edit',service(),true);expect(await screen.findByRole('textbox',{name:'Connection name'})).toBeDisabled();expect(screen.getByRole('button',{name:'Save connection'})).toBeDisabled();expect(screen.queryByRole('button',{name:'Remove connection'})).not.toBeInTheDocument();});
 it('does not dispatch inventory requests for a disabled connection',async()=>{const {api}=mount('/admin#/integrations/hosting/panel/manage');await screen.findByText(/This connection is disabled/);expect(api.perform).not.toHaveBeenCalled();});
});

it('keeps an uncertain mutation visible without automatically retrying or clearing the draft',async()=>{
 const submit=vi.fn(async()=>{throw new AdminProblemError(502,{code:'panel-response',resultUncertain:true,detail:'The result is uncertain. Inspect the remote state before retrying.'},null);});
 const back=vi.fn();
 renderInShell(()=><HostingActionForm kind="password" connection={row} domain="example.org" account="alice" canWrite submit={submit} back={back}/>);
 const password=screen.getByLabelText('New password');fireEvent.input(password,{target:{value:'replacement-secret'}});
 fireEvent.click(screen.getByRole('button',{name:'Reset mailbox password'}));
 await screen.findByText(/The result is uncertain/);
 expect(submit).toHaveBeenCalledTimes(1);expect(submit).toHaveBeenCalledWith({action:'resetMailboxPassword',domain:'example.org',account:'alice',password:'replacement-secret'});
 expect(password).toHaveValue('replacement-secret');expect(back).not.toHaveBeenCalled();
});

it('requires the separate tunnel binding capability to edit a connection route', async () => {
 const api=service();
 renderInShell(()=><HostingConnections api={api}/>, {route:()=>parseAdminPath('/admin#/integrations/hosting/panel/edit'),loadMe:holding(['integrations.read','integrations.write'])});
 expect(await screen.findByRole('textbox',{name:'Connection name'})).not.toBeDisabled();
 expect(screen.getByRole('combobox',{name:/^Connection route$/})).toBeDisabled();
});
